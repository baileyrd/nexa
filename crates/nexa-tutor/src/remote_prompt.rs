//! Caller-directed, whole-layer disclosure filtering for remote prompt compilation.
//!
//! This module makes no disclosure decision itself. It neither examines layer content nor
//! invokes, selects, authorizes, or resolves a provider.

use crate::{
    authorization::{
        select_authorized_available_remote_model, RemoteAuthorizationError,
        RemoteModelAuthorization,
    },
    availability::ModelAvailabilitySnapshot,
    model::PrivacyClass,
    prompt::{
        compile_prompt, PromptCompilationRequest, PromptCompilationResult, PromptError,
        PromptLayerKind, CANONICAL_LAYER_ORDER,
    },
    registry::ModelRegistry,
    selection::{ModelSelectionRequirements, SelectedModel},
};
use nexa_domain::ProtocolVersion;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fmt};
use thiserror::Error;

pub const REMOTE_PROMPT_FILTER_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const REMOTE_PROMPT_DISCLOSURE_POLICY_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RemotePromptLayerDisposition {
    Include,
    Omit,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemotePromptLayerRule {
    pub kind: PromptLayerKind,
    pub disposition: RemotePromptLayerDisposition,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemotePromptDisclosurePolicy {
    pub contract_version: ProtocolVersion,
    pub policy_version: ProtocolVersion,
    pub target_privacy_class: PrivacyClass,
    pub rules: Vec<RemotePromptLayerRule>,
}

impl RemotePromptDisclosurePolicy {
    pub fn new(
        target_privacy_class: PrivacyClass,
        rules: Vec<RemotePromptLayerRule>,
    ) -> Result<Self, RemotePromptFilterError> {
        let mut value = Self {
            contract_version: REMOTE_PROMPT_FILTER_V1,
            policy_version: REMOTE_PROMPT_DISCLOSURE_POLICY_V1,
            target_privacy_class,
            rules,
        };
        value
            .rules
            .sort_by_key(|rule| canonical_position(rule.kind));
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), RemotePromptFilterError> {
        if self.contract_version != REMOTE_PROMPT_FILTER_V1
            || self.policy_version != REMOTE_PROMPT_DISCLOSURE_POLICY_V1
        {
            return Err(RemotePromptFilterError::UnsupportedVersion);
        }
        if !matches!(
            self.target_privacy_class,
            PrivacyClass::ApprovedRemote | PrivacyClass::RestrictedRemote
        ) {
            return Err(RemotePromptFilterError::InvalidPrivacyClass);
        }
        if self.rules.len() != CANONICAL_LAYER_ORDER.len() {
            return Err(RemotePromptFilterError::InvalidRules);
        }
        for (rule, expected) in self.rules.iter().zip(CANONICAL_LAYER_ORDER) {
            if rule.kind != expected {
                return Err(RemotePromptFilterError::InvalidRules);
            }
            if expected.is_required() && rule.disposition == RemotePromptLayerDisposition::Omit {
                return Err(RemotePromptFilterError::MandatoryLayerOmitted);
            }
        }
        Ok(())
    }

    fn anchor(&self) -> Result<String, RemotePromptFilterError> {
        let bytes = serde_json::to_vec(self).map_err(|_| RemotePromptFilterError::Replay)?;
        Ok(sha256(&bytes))
    }
}

impl<'de> Deserialize<'de> for RemotePromptDisclosurePolicy {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            policy_version: ProtocolVersion,
            target_privacy_class: PrivacyClass,
            rules: Vec<RemotePromptLayerRule>,
        }
        let wire = Wire::deserialize(d)?;
        let value = Self {
            contract_version: wire.contract_version,
            policy_version: wire.policy_version,
            target_privacy_class: wire.target_privacy_class,
            rules: wire.rules,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemotePromptFilterEvidence {
    pub contract_version: ProtocolVersion,
    pub policy_version: ProtocolVersion,
    pub target_privacy_class: PrivacyClass,
    pub policy_replay_anchor: String,
    /// Binds independently retained source compilation evidence; it cannot reconstruct omitted
    /// content bytes by itself.
    pub source_compilation_replay_anchor: String,
    /// Canonical, content-free inventory of the layers that were present in the source.
    pub source_present_layer_kinds: Vec<PromptLayerKind>,
    pub filtered_compilation_replay_anchor: String,
    pub included_layer_kinds: Vec<PromptLayerKind>,
    pub omitted_layer_kinds: Vec<PromptLayerKind>,
    pub filter_replay_anchor: String,
}

impl RemotePromptFilterEvidence {
    pub fn validate(&self) -> Result<(), RemotePromptFilterError> {
        if self.contract_version != REMOTE_PROMPT_FILTER_V1
            || self.policy_version != REMOTE_PROMPT_DISCLOSURE_POLICY_V1
        {
            return Err(RemotePromptFilterError::UnsupportedVersion);
        }
        if !matches!(
            self.target_privacy_class,
            PrivacyClass::ApprovedRemote | PrivacyClass::RestrictedRemote
        ) {
            return Err(RemotePromptFilterError::InvalidPrivacyClass);
        }
        for hash in [
            &self.policy_replay_anchor,
            &self.source_compilation_replay_anchor,
            &self.filtered_compilation_replay_anchor,
            &self.filter_replay_anchor,
        ] {
            if !valid_hash(hash) {
                return Err(RemotePromptFilterError::Replay);
            }
        }
        validate_kinds(&self.included_layer_kinds)?;
        validate_kinds(&self.omitted_layer_kinds)?;
        validate_kinds(&self.source_present_layer_kinds)?;
        if CANONICAL_LAYER_ORDER
            .iter()
            .filter(|kind| kind.is_required())
            .any(|kind| !self.source_present_layer_kinds.contains(kind))
            || self
                .omitted_layer_kinds
                .iter()
                .any(|kind| kind.is_required())
            || self
                .included_layer_kinds
                .iter()
                .any(|kind| self.omitted_layer_kinds.contains(kind))
        {
            return Err(RemotePromptFilterError::InvalidEvidence);
        }
        let partition: Vec<_> = CANONICAL_LAYER_ORDER
            .into_iter()
            .filter(|kind| {
                self.included_layer_kinds.contains(kind) || self.omitted_layer_kinds.contains(kind)
            })
            .collect();
        if partition != self.source_present_layer_kinds
            || self.source_present_layer_kinds.iter().any(|kind| {
                self.included_layer_kinds.contains(kind) == self.omitted_layer_kinds.contains(kind)
            })
        {
            return Err(RemotePromptFilterError::InvalidEvidence);
        }
        if evidence_anchor(self)? != self.filter_replay_anchor {
            return Err(RemotePromptFilterError::Replay);
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for RemotePromptFilterEvidence {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            policy_version: ProtocolVersion,
            target_privacy_class: PrivacyClass,
            policy_replay_anchor: String,
            source_compilation_replay_anchor: String,
            source_present_layer_kinds: Vec<PromptLayerKind>,
            filtered_compilation_replay_anchor: String,
            included_layer_kinds: Vec<PromptLayerKind>,
            omitted_layer_kinds: Vec<PromptLayerKind>,
            filter_replay_anchor: String,
        }
        let w = Wire::deserialize(d)?;
        let value = Self {
            contract_version: w.contract_version,
            policy_version: w.policy_version,
            target_privacy_class: w.target_privacy_class,
            policy_replay_anchor: w.policy_replay_anchor,
            source_compilation_replay_anchor: w.source_compilation_replay_anchor,
            source_present_layer_kinds: w.source_present_layer_kinds,
            filtered_compilation_replay_anchor: w.filtered_compilation_replay_anchor,
            included_layer_kinds: w.included_layer_kinds,
            omitted_layer_kinds: w.omitted_layer_kinds,
            filter_replay_anchor: w.filter_replay_anchor,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemotePromptFilterResult {
    pub policy: RemotePromptDisclosurePolicy,
    pub filtered_compilation: PromptCompilationResult,
    pub evidence: RemotePromptFilterEvidence,
}

impl fmt::Debug for RemotePromptFilterResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RemotePromptFilterResult")
            .field("policy", &self.policy)
            .field("filtered_compilation", &self.filtered_compilation)
            .field("evidence", &self.evidence)
            .finish()
    }
}

impl RemotePromptFilterResult {
    pub fn validate(&self) -> Result<(), RemotePromptFilterError> {
        self.policy.validate()?;
        self.filtered_compilation.validate()?;
        self.evidence.validate()?;
        if self.policy.anchor()? != self.evidence.policy_replay_anchor
            || self.policy.policy_version != self.evidence.policy_version
            || self.policy.target_privacy_class != self.evidence.target_privacy_class
            || self.filtered_compilation.replay_anchor
                != self.evidence.filtered_compilation_replay_anchor
        {
            return Err(RemotePromptFilterError::Association);
        }
        let manifest: Vec<_> = self
            .filtered_compilation
            .manifest
            .iter()
            .map(|entry| entry.kind)
            .collect();
        if manifest != self.evidence.included_layer_kinds {
            return Err(RemotePromptFilterError::Association);
        }
        for rule in &self.policy.rules {
            let present = self
                .evidence
                .source_present_layer_kinds
                .contains(&rule.kind);
            let included = self.evidence.included_layer_kinds.contains(&rule.kind);
            let omitted = self.evidence.omitted_layer_kinds.contains(&rule.kind);
            if (present
                && ((rule.disposition == RemotePromptLayerDisposition::Include && !included)
                    || (rule.disposition == RemotePromptLayerDisposition::Omit && !omitted)))
                || (!present && (included || omitted))
            {
                return Err(RemotePromptFilterError::Association);
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for RemotePromptFilterResult {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            policy: RemotePromptDisclosurePolicy,
            filtered_compilation: PromptCompilationResult,
            evidence: RemotePromptFilterEvidence,
        }
        let w = Wire::deserialize(d)?;
        let value = Self {
            policy: w.policy,
            filtered_compilation: w.filtered_compilation,
            evidence: w.evidence,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum RemotePromptFilterError {
    #[error("remote prompt filter has an unsupported version")]
    UnsupportedVersion,
    #[error("remote prompt filter has an invalid privacy class")]
    InvalidPrivacyClass,
    #[error("remote prompt filter has invalid disclosure rules")]
    InvalidRules,
    #[error("remote prompt filter cannot omit a mandatory layer")]
    MandatoryLayerOmitted,
    #[error("remote prompt filter has invalid evidence")]
    InvalidEvidence,
    #[error("remote prompt filter replay evidence is invalid")]
    Replay,
    #[error("remote prompt filter evidence is not associated")]
    Association,
    #[error("source or filtered prompt compilation is invalid")]
    Prompt(#[from] PromptError),
}

/// Closed, content-free failures for ADR-0034's non-invoking composition.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum FilteredRemoteSelectionError {
    #[error("filtered remote selection requirements do not match the disclosure policy")]
    FilterPrivacyRequirements,
    #[error("remote prompt filter evidence is invalid or not associated")]
    FilterEvidence,
    #[error("authorized available remote selection failed")]
    AuthorizedSelection(#[from] RemoteAuthorizationError),
}

/// Selects from the exact validated filtered compilation without invoking a provider.
pub fn select_filtered_authorized_available_remote_model(
    registry: &ModelRegistry,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    authorization: &RemoteModelAuthorization,
    filtered_result: &RemotePromptFilterResult,
) -> Result<SelectedModel, FilteredRemoteSelectionError> {
    filtered_result
        .validate()
        .map_err(|_| FilteredRemoteSelectionError::FilterEvidence)?;
    if requirements.validate().is_err()
        || requirements.privacy_preference.as_slice()
            != [filtered_result.policy.target_privacy_class]
    {
        return Err(FilteredRemoteSelectionError::FilterPrivacyRequirements);
    }
    select_authorized_available_remote_model(
        registry,
        requirements,
        availability,
        authorization,
        &filtered_result.filtered_compilation,
    )
    .map_err(FilteredRemoteSelectionError::AuthorizedSelection)
}

pub fn filter_and_compile_remote_prompt(
    source: &PromptCompilationRequest,
    policy: &RemotePromptDisclosurePolicy,
) -> Result<RemotePromptFilterResult, RemotePromptFilterError> {
    policy.validate()?;
    let source_compilation = compile_prompt(source)?;
    let rules: BTreeMap<_, _> = policy
        .rules
        .iter()
        .map(|r| (r.kind, r.disposition))
        .collect();
    let mut filtered = source.clone();
    filtered
        .layers
        .retain(|layer| rules[&layer.kind] == RemotePromptLayerDisposition::Include);
    let filtered_compilation = compile_prompt(&filtered)?;
    let present: Vec<_> = source_compilation
        .manifest
        .iter()
        .map(|entry| entry.kind)
        .collect();
    let included_layer_kinds: Vec<_> = present
        .iter()
        .copied()
        .filter(|kind| rules[kind] == RemotePromptLayerDisposition::Include)
        .collect();
    let omitted_layer_kinds: Vec<_> = present
        .iter()
        .copied()
        .filter(|kind| rules[kind] == RemotePromptLayerDisposition::Omit)
        .collect();
    let mut evidence = RemotePromptFilterEvidence {
        contract_version: REMOTE_PROMPT_FILTER_V1,
        policy_version: policy.policy_version,
        target_privacy_class: policy.target_privacy_class,
        policy_replay_anchor: policy.anchor()?,
        source_compilation_replay_anchor: source_compilation.replay_anchor,
        source_present_layer_kinds: present,
        filtered_compilation_replay_anchor: filtered_compilation.replay_anchor.clone(),
        included_layer_kinds,
        omitted_layer_kinds,
        filter_replay_anchor: String::new(),
    };
    evidence.filter_replay_anchor = evidence_anchor(&evidence)?;
    let result = RemotePromptFilterResult {
        policy: policy.clone(),
        filtered_compilation,
        evidence,
    };
    result.validate()?;
    Ok(result)
}

fn canonical_position(kind: PromptLayerKind) -> usize {
    CANONICAL_LAYER_ORDER
        .iter()
        .position(|x| *x == kind)
        .expect("closed kind")
}
fn validate_kinds(kinds: &[PromptLayerKind]) -> Result<(), RemotePromptFilterError> {
    if kinds
        .windows(2)
        .any(|pair| canonical_position(pair[0]) >= canonical_position(pair[1]))
    {
        Err(RemotePromptFilterError::InvalidEvidence)
    } else {
        Ok(())
    }
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn evidence_anchor(e: &RemotePromptFilterEvidence) -> Result<String, RemotePromptFilterError> {
    #[derive(Serialize)]
    struct Governed<'a> {
        contract_version: ProtocolVersion,
        policy_version: ProtocolVersion,
        target_privacy_class: PrivacyClass,
        policy_replay_anchor: &'a str,
        source_compilation_replay_anchor: &'a str,
        source_present_layer_kinds: &'a [PromptLayerKind],
        filtered_compilation_replay_anchor: &'a str,
        included_layer_kinds: &'a [PromptLayerKind],
        omitted_layer_kinds: &'a [PromptLayerKind],
    }
    let governed = Governed {
        contract_version: e.contract_version,
        policy_version: e.policy_version,
        target_privacy_class: e.target_privacy_class,
        policy_replay_anchor: &e.policy_replay_anchor,
        source_compilation_replay_anchor: &e.source_compilation_replay_anchor,
        source_present_layer_kinds: &e.source_present_layer_kinds,
        filtered_compilation_replay_anchor: &e.filtered_compilation_replay_anchor,
        included_layer_kinds: &e.included_layer_kinds,
        omitted_layer_kinds: &e.omitted_layer_kinds,
    };
    serde_json::to_vec(&governed)
        .map(|v| sha256(&v))
        .map_err(|_| RemotePromptFilterError::Replay)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        authorization::{
            select_authorized_available_remote_model, RemoteModelAuthorization,
            RemoteModelAuthorizationEntry,
        },
        availability::{ModelAvailabilityEntry, ModelAvailabilitySnapshot, ModelAvailabilityState},
        model::{
            LanguageModelProvider, ModelCapabilities, ModelDescriptor, ModelErrorKind,
            RequiredCapabilities, ScriptedModelProvider, ScriptedOutcome,
        },
        prompt::{
            LayerClassification, PromptContent, PromptLayer, PromptLimits, PROMPT_COMPILATION_V1,
        },
        registry::ModelRegistry,
        selection::ModelSelectionRequirements,
    };
    use nexa_domain::{ModelId, ModelProviderId};
    use serde_json::json;
    use std::sync::Arc;
    use uuid::Uuid;

    fn layer(kind: PromptLayerKind) -> PromptLayer {
        PromptLayer {
            kind,
            classification: kind.classification(),
            content: PromptContent::new(format!(
                "exact-{kind:?}-BEGIN PRIVATE KEY-https://endpoint-credential-provider-private-learner-conversation-knowledge-tool-{{instructions}}"
            ))
            .unwrap(),
        }
    }
    fn source(all: bool) -> PromptCompilationRequest {
        PromptCompilationRequest {
            contract_version: PROMPT_COMPILATION_V1,
            prompt_package_version: ProtocolVersion::new(1, 0),
            context_builder_version: ProtocolVersion::new(1, 0),
            output_schema_version: ProtocolVersion::new(1, 0),
            limits: PromptLimits {
                maximum_layer_bytes: 4096,
                maximum_compiled_bytes: 65_536,
            },
            layers: CANONICAL_LAYER_ORDER
                .into_iter()
                .filter(|k| all || k.is_required())
                .rev()
                .map(layer)
                .collect(),
        }
    }
    fn rules(disposition: RemotePromptLayerDisposition) -> Vec<RemotePromptLayerRule> {
        CANONICAL_LAYER_ORDER
            .into_iter()
            .map(|kind| RemotePromptLayerRule {
                kind,
                disposition: if kind.is_required() {
                    RemotePromptLayerDisposition::Include
                } else {
                    disposition
                },
            })
            .rev()
            .collect()
    }
    fn policy(
        privacy: PrivacyClass,
        disposition: RemotePromptLayerDisposition,
    ) -> RemotePromptDisclosurePolicy {
        RemotePromptDisclosurePolicy::new(privacy, rules(disposition)).unwrap()
    }

    #[test]
    fn remote_prompt_filter_policy_round_trips_and_constructor_canonicalizes() {
        for privacy in [PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote] {
            let p = policy(privacy, RemotePromptLayerDisposition::Omit);
            assert_eq!(
                p.rules.iter().map(|r| r.kind).collect::<Vec<_>>(),
                CANONICAL_LAYER_ORDER
            );
            assert_eq!(
                serde_json::from_str::<RemotePromptDisclosurePolicy>(
                    &serde_json::to_string(&p).unwrap()
                )
                .unwrap(),
                p
            );
        }
    }

    #[test]
    fn remote_prompt_filter_policy_decode_fails_closed() {
        let p = policy(
            PrivacyClass::ApprovedRemote,
            RemotePromptLayerDisposition::Include,
        );
        let mut value = serde_json::to_value(&p).unwrap();
        for mutation in 0..7 {
            let mut bad = value.clone();
            match mutation {
                0 => bad["extra"] = json!(true),
                1 => bad["contract_version"] = json!("2.0"),
                2 => bad["target_privacy_class"] = json!("local_only"),
                3 => {
                    bad["rules"].as_array_mut().unwrap().pop();
                }
                4 => {
                    let rules = bad["rules"].as_array_mut().unwrap();
                    rules[1] = rules[0].clone();
                }
                5 => bad["rules"].as_array_mut().unwrap().swap(0, 1),
                _ => bad["rules"][0]["disposition"] = json!("sometimes"),
            }
            assert!(serde_json::from_value::<RemotePromptDisclosurePolicy>(bad).is_err());
        }
        for kind in CANONICAL_LAYER_ORDER
            .into_iter()
            .filter(|kind| kind.is_required())
        {
            let mut bad = serde_json::to_value(&p).unwrap();
            let index = canonical_position(kind);
            bad["rules"][index]["disposition"] = json!("omit");
            assert!(serde_json::from_value::<RemotePromptDisclosurePolicy>(bad).is_err());
        }
        value["policy_version"] = json!("1.1");
        assert!(serde_json::from_value::<RemotePromptDisclosurePolicy>(value).is_err());
    }

    #[test]
    fn remote_prompt_filter_all_included_equals_direct_compilation() {
        let source = source(true);
        let direct = compile_prompt(&source).unwrap();
        let result = filter_and_compile_remote_prompt(
            &source,
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Include,
            ),
        )
        .unwrap();
        assert_eq!(result.filtered_compilation, direct);
        assert_eq!(result.evidence.included_layer_kinds, CANONICAL_LAYER_ORDER);
        assert!(result.evidence.omitted_layer_kinds.is_empty());
        assert_eq!(
            serde_json::from_str::<RemotePromptFilterResult>(
                &serde_json::to_string(&result).unwrap()
            )
            .unwrap(),
            result
        );
    }

    #[test]
    fn remote_prompt_filter_optional_layers_are_whole_and_order_independent() {
        for omitted in CANONICAL_LAYER_ORDER
            .into_iter()
            .filter(|kind| !kind.is_required())
        {
            let mut rs = rules(RemotePromptLayerDisposition::Include);
            rs.iter_mut()
                .find(|r| r.kind == omitted)
                .unwrap()
                .disposition = RemotePromptLayerDisposition::Omit;
            let original = source(true);
            let result = filter_and_compile_remote_prompt(
                &original,
                &RemotePromptDisclosurePolicy::new(PrivacyClass::RestrictedRemote, rs).unwrap(),
            )
            .unwrap();
            let mut manual = original.clone();
            manual.layers.retain(|layer| layer.kind != omitted);
            assert_eq!(
                result.filtered_compilation,
                compile_prompt(&manual).unwrap()
            );
            assert_eq!(result.evidence.omitted_layer_kinds, vec![omitted]);
            assert!(!result
                .filtered_compilation
                .manifest
                .iter()
                .any(|entry| entry.kind == omitted));
            assert!(!result
                .filtered_compilation
                .model_input
                .as_str()
                .contains(&format!("exact-{omitted:?}")));
            for entry in &result.filtered_compilation.manifest {
                assert!(result
                    .filtered_compilation
                    .model_input
                    .as_str()
                    .contains(&format!("exact-{:?}", entry.kind)));
            }
        }
        let a = filter_and_compile_remote_prompt(
            &source(true),
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        let mut ordered = source(true);
        ordered.layers.reverse();
        let b = filter_and_compile_remote_prompt(
            &ordered,
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn remote_prompt_filter_absent_optional_is_not_reported_as_omitted() {
        let result = filter_and_compile_remote_prompt(
            &source(false),
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        assert!(result.evidence.omitted_layer_kinds.is_empty());
        assert_eq!(result.evidence.included_layer_kinds.len(), 6);
    }

    #[test]
    fn remote_prompt_filter_source_validation_is_unchanged_and_content_free() {
        let p = policy(
            PrivacyClass::ApprovedRemote,
            RemotePromptLayerDisposition::Include,
        );
        let mut bad = source(false);
        bad.layers[0].classification = LayerClassification::GovernedEvidence;
        let error = filter_and_compile_remote_prompt(&bad, &p).unwrap_err();
        let diagnostic = format!("{error:?} {error}");
        assert!(!diagnostic.contains("BEGIN PRIVATE KEY"));
        assert!(matches!(
            error,
            RemotePromptFilterError::Prompt(PromptError::InvalidClassification)
        ));
        let mut duplicate = source(false);
        duplicate.layers.push(duplicate.layers[0].clone());
        assert!(filter_and_compile_remote_prompt(&duplicate, &p).is_err());
        let mut missing = source(false);
        missing.layers.pop();
        assert!(filter_and_compile_remote_prompt(&missing, &p).is_err());
        let mut version = source(false);
        version.contract_version = ProtocolVersion::new(2, 0);
        assert!(filter_and_compile_remote_prompt(&version, &p).is_err());
        let mut limits = source(false);
        limits.limits.maximum_layer_bytes = 0;
        assert!(filter_and_compile_remote_prompt(&limits, &p).is_err());

        let mut exact_layer = source(false);
        let maximum = compile_prompt(&exact_layer)
            .unwrap()
            .manifest
            .iter()
            .map(|entry| entry.content_bytes)
            .max()
            .unwrap();
        exact_layer.limits.maximum_layer_bytes = maximum;
        assert!(filter_and_compile_remote_prompt(&exact_layer, &p).is_ok());
        exact_layer.limits.maximum_layer_bytes = maximum - 1;
        assert!(matches!(
            filter_and_compile_remote_prompt(&exact_layer, &p),
            Err(RemotePromptFilterError::Prompt(PromptError::SizeLimit))
        ));

        let mut exact_compiled = source(false);
        exact_compiled.limits.maximum_layer_bytes = compile_prompt(&exact_compiled)
            .unwrap()
            .manifest
            .iter()
            .map(|entry| entry.content_bytes)
            .max()
            .unwrap();
        let approximate = compile_prompt(&exact_compiled).unwrap().compiled_bytes;
        let exact = (exact_compiled.limits.maximum_layer_bytes..=approximate)
            .find(|candidate| {
                exact_compiled.limits.maximum_compiled_bytes = *candidate;
                compile_prompt(&exact_compiled).is_ok()
            })
            .unwrap();
        exact_compiled.limits.maximum_compiled_bytes = exact;
        assert!(filter_and_compile_remote_prompt(&exact_compiled, &p).is_ok());
        exact_compiled.limits.maximum_compiled_bytes = exact - 1;
        assert!(matches!(
            filter_and_compile_remote_prompt(&exact_compiled, &p),
            Err(RemotePromptFilterError::Prompt(PromptError::SizeLimit))
        ));

        let mut oversized = source(false);
        oversized.limits.maximum_compiled_bytes = (crate::model::MAX_MODEL_INPUT_BYTES + 1) as u32;
        assert!(filter_and_compile_remote_prompt(&oversized, &p).is_err());
    }

    #[test]
    fn remote_prompt_filter_anchors_change_with_governed_inputs() {
        let approved = filter_and_compile_remote_prompt(
            &source(true),
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Include,
            ),
        )
        .unwrap();
        let restricted = filter_and_compile_remote_prompt(
            &source(true),
            &policy(
                PrivacyClass::RestrictedRemote,
                RemotePromptLayerDisposition::Include,
            ),
        )
        .unwrap();
        let omitted = filter_and_compile_remote_prompt(
            &source(true),
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        assert_ne!(
            approved.evidence.policy_replay_anchor,
            restricted.evidence.policy_replay_anchor
        );
        assert_ne!(
            approved.evidence.filter_replay_anchor,
            restricted.evidence.filter_replay_anchor
        );
        assert_ne!(
            approved.evidence.filtered_compilation_replay_anchor,
            omitted.evidence.filtered_compilation_replay_anchor
        );
        assert_eq!(
            approved.evidence.source_compilation_replay_anchor,
            omitted.evidence.source_compilation_replay_anchor
        );
    }

    #[test]
    fn remote_prompt_filter_standalone_tampering_and_reassociation_fail() {
        let result = filter_and_compile_remote_prompt(
            &source(true),
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        let mut evidence = serde_json::to_value(&result.evidence).unwrap();
        for field in [
            "policy_replay_anchor",
            "source_compilation_replay_anchor",
            "filtered_compilation_replay_anchor",
            "filter_replay_anchor",
        ] {
            let mut bad = evidence.clone();
            bad[field] = json!("ABC");
            assert!(serde_json::from_value::<RemotePromptFilterEvidence>(bad).is_err());
        }
        let mut final_mismatch = result.evidence.clone();
        final_mismatch.filter_replay_anchor = "0".repeat(64);
        assert!(final_mismatch.validate().is_err());
        evidence["included_layer_kinds"]
            .as_array_mut()
            .unwrap()
            .swap(0, 1);
        assert!(serde_json::from_value::<RemotePromptFilterEvidence>(evidence).is_err());
        let mut reassociated = result.clone();
        reassociated.policy = policy(
            PrivacyClass::RestrictedRemote,
            RemotePromptLayerDisposition::Omit,
        );
        assert!(reassociated.validate().is_err());
        let debug = format!("{result:?}");
        for sentinel in [
            "BEGIN PRIVATE KEY",
            "endpoint",
            "credential",
            "provider-private",
            "learner",
            "conversation",
            "knowledge",
            "tool",
            "instructions",
        ] {
            assert!(!debug.contains(sentinel));
        }
    }

    #[test]
    fn remote_prompt_filter_evidence_and_result_wire_fail_closed() {
        let result = filter_and_compile_remote_prompt(
            &source(true),
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        let evidence = serde_json::to_value(&result.evidence).unwrap();
        for (field, value) in [
            ("contract_version", json!("2.0")),
            ("policy_version", json!("2.0")),
            ("target_privacy_class", json!("local_only")),
        ] {
            let mut bad = evidence.clone();
            bad[field] = value;
            assert!(serde_json::from_value::<RemotePromptFilterEvidence>(bad).is_err());
        }
        let mut extra = evidence.clone();
        extra["unknown"] = json!(true);
        assert!(serde_json::from_value::<RemotePromptFilterEvidence>(extra).is_err());

        for mutation in 0..9 {
            let mut bad = result.evidence.clone();
            match mutation {
                0 => bad.source_present_layer_kinds.swap(0, 1),
                1 => bad
                    .source_present_layer_kinds
                    .push(PromptLayerKind::OutputContract),
                2 => {
                    bad.source_present_layer_kinds.pop();
                }
                3 => bad.included_layer_kinds.push(bad.included_layer_kinds[0]),
                4 => bad.omitted_layer_kinds.swap(0, 1),
                5 => bad.omitted_layer_kinds.push(bad.included_layer_kinds[0]),
                6 => bad
                    .omitted_layer_kinds
                    .push(PromptLayerKind::PlatformContract),
                7 => {
                    bad.included_layer_kinds.pop();
                }
                _ => {
                    bad.omitted_layer_kinds.pop();
                }
            }
            bad.filter_replay_anchor = evidence_anchor(&bad).unwrap();
            assert!(bad.validate().is_err(), "mutation {mutation}");
        }

        let mut wire = serde_json::to_value(&result).unwrap();
        wire["unknown"] = json!(true);
        assert!(serde_json::from_value::<RemotePromptFilterResult>(wire).is_err());
        let mut nested = serde_json::to_value(&result).unwrap();
        nested["filtered_compilation"]["unknown"] = json!(true);
        assert!(serde_json::from_value::<RemotePromptFilterResult>(nested).is_err());
        let mut invalid_compilation = result.clone();
        invalid_compilation.filtered_compilation.replay_anchor = "0".repeat(64);
        assert!(invalid_compilation.validate().is_err());

        let mut disagreement = result.clone();
        disagreement.evidence.filtered_compilation_replay_anchor = "0".repeat(64);
        disagreement.evidence.filter_replay_anchor =
            evidence_anchor(&disagreement.evidence).unwrap();
        assert!(disagreement.validate().is_err());
        let mut policy_disagreement = result.clone();
        policy_disagreement.evidence.policy_replay_anchor = "0".repeat(64);
        policy_disagreement.evidence.filter_replay_anchor =
            evidence_anchor(&policy_disagreement.evidence).unwrap();
        assert!(policy_disagreement.validate().is_err());
        let mut manifest_disagreement = result.clone();
        manifest_disagreement.evidence.included_layer_kinds.pop();
        manifest_disagreement
            .evidence
            .source_present_layer_kinds
            .pop();
        manifest_disagreement.evidence.filter_replay_anchor =
            evidence_anchor(&manifest_disagreement.evidence).unwrap();
        assert!(manifest_disagreement.validate().is_err());
    }

    #[test]
    fn remote_prompt_filter_coordinated_partition_reclassification_fails() {
        let mut result = filter_and_compile_remote_prompt(
            &source(true),
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        let moved = result.evidence.omitted_layer_kinds.remove(0);
        result.evidence.included_layer_kinds.push(moved);
        result
            .evidence
            .included_layer_kinds
            .sort_by_key(|kind| canonical_position(*kind));
        result.evidence.filter_replay_anchor = evidence_anchor(&result.evidence).unwrap();
        assert_eq!(result.evidence.validate(), Ok(()));
        assert!(result.validate().is_err());
    }

    #[test]
    fn remote_prompt_filter_anchors_bind_source_content_inventory_policy_and_filtered_result() {
        let base_source = source(true);
        let include_policy = policy(
            PrivacyClass::ApprovedRemote,
            RemotePromptLayerDisposition::Include,
        );
        let base = filter_and_compile_remote_prompt(&base_source, &include_policy).unwrap();
        let mut changed_content = base_source.clone();
        changed_content.layers[0].content = PromptContent::new("changed-source-sentinel").unwrap();
        let content = filter_and_compile_remote_prompt(&changed_content, &include_policy).unwrap();
        let inventory = filter_and_compile_remote_prompt(&source(false), &include_policy).unwrap();
        let disposition = filter_and_compile_remote_prompt(
            &base_source,
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        let privacy = filter_and_compile_remote_prompt(
            &base_source,
            &policy(
                PrivacyClass::RestrictedRemote,
                RemotePromptLayerDisposition::Include,
            ),
        )
        .unwrap();
        assert_ne!(
            base.evidence.source_compilation_replay_anchor,
            content.evidence.source_compilation_replay_anchor
        );
        assert_ne!(
            base.evidence.source_present_layer_kinds,
            inventory.evidence.source_present_layer_kinds
        );
        assert_ne!(
            base.evidence.policy_replay_anchor,
            disposition.evidence.policy_replay_anchor
        );
        assert_ne!(
            base.evidence.policy_replay_anchor,
            privacy.evidence.policy_replay_anchor
        );
        assert_ne!(
            base.evidence.filtered_compilation_replay_anchor,
            content.evidence.filtered_compilation_replay_anchor
        );
        for other in [&content, &inventory, &disposition, &privacy] {
            assert_ne!(
                base.evidence.filter_replay_anchor,
                other.evidence.filter_replay_anchor
            );
        }
        assert_eq!(
            base,
            filter_and_compile_remote_prompt(&base_source, &include_policy).unwrap()
        );
    }

    #[test]
    fn filtered_authorized_remote_selection_uses_exact_anchor_and_preserves_provider() {
        let source_request = source(true);
        let unfiltered = compile_prompt(&source_request).unwrap();
        let filtered = filter_and_compile_remote_prompt(
            &source_request,
            &policy(
                PrivacyClass::ApprovedRemote,
                RemotePromptLayerDisposition::Omit,
            ),
        )
        .unwrap();
        let provider_id = ModelProviderId::new(Uuid::from_u128(41)).unwrap();
        let model_id = ModelId::new(Uuid::from_u128(42)).unwrap();
        let descriptor = ModelDescriptor::new(
            provider_id,
            model_id,
            PrivacyClass::ApprovedRemote,
            ModelCapabilities {
                streaming: false,
                structured_output: true,
                tool_calling: false,
                vision: false,
                context_window_tokens: 100_000,
                maximum_output_tokens: 128,
            },
        )
        .unwrap();
        let scripted = Arc::new(
            ScriptedModelProvider::new(
                descriptor,
                [
                    ScriptedOutcome::Error(ModelErrorKind::Unavailable),
                    ScriptedOutcome::Error(ModelErrorKind::Internal),
                ],
            )
            .unwrap(),
        );
        let registry =
            ModelRegistry::try_from_providers([scripted.clone() as Arc<dyn LanguageModelProvider>])
                .unwrap();
        let requirements = ModelSelectionRequirements::new(
            RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            1,
            vec![PrivacyClass::ApprovedRemote],
        )
        .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id,
            model_id,
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let entry = RemoteModelAuthorizationEntry {
            provider_id,
            model_id,
            privacy_class: PrivacyClass::ApprovedRemote,
        };
        let filtered_authorization = RemoteModelAuthorization::new(
            filtered.filtered_compilation.replay_anchor.clone(),
            vec![entry],
        )
        .unwrap();
        let selected = select_filtered_authorized_available_remote_model(
            &registry,
            &requirements,
            &availability,
            &filtered_authorization,
            &filtered,
        )
        .unwrap();
        assert!(Arc::ptr_eq(
            &selected.provider,
            &(scripted.clone() as Arc<dyn LanguageModelProvider>)
        ));
        assert!(select_authorized_available_remote_model(
            &registry,
            &requirements,
            &availability,
            &filtered_authorization,
            &unfiltered
        )
        .is_err());
        let source_authorization =
            RemoteModelAuthorization::new(unfiltered.replay_anchor.clone(), vec![entry]).unwrap();
        assert!(select_authorized_available_remote_model(
            &registry,
            &requirements,
            &availability,
            &source_authorization,
            &filtered.filtered_compilation
        )
        .is_err());
        assert_eq!(scripted.remaining(), 2);
    }

    #[test]
    fn filtered_authorized_remote_selection_rejects_privacy_and_filter_mismatch_without_consumption(
    ) {
        for privacy in [PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote] {
            let filtered = filter_and_compile_remote_prompt(
                &source(true),
                &policy(privacy, RemotePromptLayerDisposition::Omit),
            )
            .unwrap();
            let provider_id = ModelProviderId::new(Uuid::from_u128(51)).unwrap();
            let model_id = ModelId::new(Uuid::from_u128(52)).unwrap();
            let descriptor = ModelDescriptor::new(
                provider_id,
                model_id,
                privacy,
                ModelCapabilities {
                    streaming: false,
                    structured_output: true,
                    tool_calling: false,
                    vision: false,
                    context_window_tokens: 100_000,
                    maximum_output_tokens: 128,
                },
            )
            .unwrap();
            let scripted = Arc::new(
                ScriptedModelProvider::new(
                    descriptor,
                    [ScriptedOutcome::Error(ModelErrorKind::Internal)],
                )
                .unwrap(),
            );
            let registry = ModelRegistry::try_from_providers([
                scripted.clone() as Arc<dyn LanguageModelProvider>
            ])
            .unwrap();
            let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
                provider_id,
                model_id,
                state: ModelAvailabilityState::Available,
            }])
            .unwrap();
            let authorization = RemoteModelAuthorization::new(
                filtered.filtered_compilation.replay_anchor.clone(),
                vec![RemoteModelAuthorizationEntry {
                    provider_id,
                    model_id,
                    privacy_class: privacy,
                }],
            )
            .unwrap();
            let caps = RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            };
            for preferences in [
                vec![],
                vec![PrivacyClass::LocalOnly],
                vec![privacy, PrivacyClass::LocalOnly],
                vec![PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote],
                vec![if privacy == PrivacyClass::ApprovedRemote {
                    PrivacyClass::RestrictedRemote
                } else {
                    PrivacyClass::ApprovedRemote
                }],
            ] {
                let requirements = ModelSelectionRequirements {
                    contract_version: crate::selection::MODEL_SELECTION_V1,
                    required_capabilities: caps.clone(),
                    maximum_output_tokens: 1,
                    privacy_preference: preferences,
                };
                assert_eq!(
                    select_filtered_authorized_available_remote_model(
                        &registry,
                        &requirements,
                        &availability,
                        &authorization,
                        &filtered
                    )
                    .unwrap_err(),
                    FilteredRemoteSelectionError::FilterPrivacyRequirements
                );
            }
            let mut tampered = filtered.clone();
            tampered.evidence.policy_replay_anchor = "0".repeat(64);
            assert_eq!(
                select_filtered_authorized_available_remote_model(
                    &registry,
                    &ModelSelectionRequirements::new(caps, 1, vec![privacy]).unwrap(),
                    &availability,
                    &authorization,
                    &tampered
                )
                .unwrap_err(),
                FilteredRemoteSelectionError::FilterEvidence
            );
            assert_eq!(scripted.remaining(), 1);
        }
    }
}
