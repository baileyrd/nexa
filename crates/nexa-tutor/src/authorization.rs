//! Caller-supplied remote authorization evidence and non-invoking selection.

use crate::{
    availability::{
        availability_states_for_registry, ModelAvailabilityError, ModelAvailabilitySnapshot,
        ModelAvailabilityState,
    },
    model::PrivacyClass,
    prompt::PromptCompilationResult,
    registry::ModelRegistry,
    selection::{
        select_model_where, ModelSelectionError, ModelSelectionRequirements, SelectedModel,
    },
};
use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub const REMOTE_AUTHORIZATION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const REMOTE_AUTHORIZATION_POLICY_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const MAX_REMOTE_AUTHORIZATION_ENTRIES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteModelAuthorizationEntry {
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub privacy_class: PrivacyClass,
}

impl<'de> Deserialize<'de> for RemoteModelAuthorizationEntry {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            provider_id: ModelProviderId,
            model_id: ModelId,
            privacy_class: PrivacyClass,
        }

        let wire = Wire::deserialize(deserializer)?;
        if !matches!(
            wire.privacy_class,
            PrivacyClass::ApprovedRemote | PrivacyClass::RestrictedRemote
        ) {
            return Err(serde::de::Error::custom(
                RemoteAuthorizationError::InvalidAuthorizationEvidence,
            ));
        }
        Ok(Self {
            provider_id: wire.provider_id,
            model_id: wire.model_id,
            privacy_class: wire.privacy_class,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteModelAuthorization {
    pub contract_version: ProtocolVersion,
    pub policy_version: ProtocolVersion,
    pub prompt_compilation_replay_anchor: String,
    pub entries: Vec<RemoteModelAuthorizationEntry>,
}

impl RemoteModelAuthorization {
    pub fn new(
        prompt_compilation_replay_anchor: impl Into<String>,
        mut entries: Vec<RemoteModelAuthorizationEntry>,
    ) -> Result<Self, RemoteAuthorizationError> {
        if entries.len() > MAX_REMOTE_AUTHORIZATION_ENTRIES {
            return Err(RemoteAuthorizationError::InvalidAuthorizationEvidence);
        }
        entries.sort_by_key(|entry| (entry.provider_id, entry.model_id));
        let value = Self {
            contract_version: REMOTE_AUTHORIZATION_V1,
            policy_version: REMOTE_AUTHORIZATION_POLICY_V1,
            prompt_compilation_replay_anchor: prompt_compilation_replay_anchor.into(),
            entries,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), RemoteAuthorizationError> {
        if self.contract_version != REMOTE_AUTHORIZATION_V1
            || self.policy_version != REMOTE_AUTHORIZATION_POLICY_V1
        {
            return Err(RemoteAuthorizationError::UnsupportedAuthorizationVersion);
        }
        if !valid_anchor(&self.prompt_compilation_replay_anchor)
            || self.entries.len() > MAX_REMOTE_AUTHORIZATION_ENTRIES
            || self.entries.iter().any(|entry| {
                !matches!(
                    entry.privacy_class,
                    PrivacyClass::ApprovedRemote | PrivacyClass::RestrictedRemote
                )
            })
            || self.entries.windows(2).any(|pair| {
                (pair[0].provider_id, pair[0].model_id) >= (pair[1].provider_id, pair[1].model_id)
            })
        {
            return Err(RemoteAuthorizationError::InvalidAuthorizationEvidence);
        }
        Ok(())
    }
}

fn valid_anchor(value: &str) -> bool {
    value.len() == 64
        && value
            .as_bytes()
            .iter()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(byte))
}

impl<'de> Deserialize<'de> for RemoteModelAuthorization {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct BoundedEntries(Vec<RemoteModelAuthorizationEntry>);
        impl<'de> Deserialize<'de> for BoundedEntries {
            fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
                struct Visitor;
                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = BoundedEntries;
                    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "a bounded remote authorization entry list")
                    }
                    fn visit_seq<A: serde::de::SeqAccess<'de>>(
                        self,
                        mut sequence: A,
                    ) -> Result<Self::Value, A::Error> {
                        let mut entries = Vec::new();
                        while let Some(entry) = sequence.next_element()? {
                            if entries.len() == MAX_REMOTE_AUTHORIZATION_ENTRIES {
                                return Err(serde::de::Error::custom(
                                    RemoteAuthorizationError::InvalidAuthorizationEvidence,
                                ));
                            }
                            entries.push(entry);
                        }
                        Ok(BoundedEntries(entries))
                    }
                }
                d.deserialize_seq(Visitor)
            }
        }
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            policy_version: ProtocolVersion,
            prompt_compilation_replay_anchor: String,
            entries: BoundedEntries,
        }
        let wire = Wire::deserialize(deserializer)?;
        let value = Self {
            contract_version: wire.contract_version,
            policy_version: wire.policy_version,
            prompt_compilation_replay_anchor: wire.prompt_compilation_replay_anchor,
            entries: wire.entries.0,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum RemoteAuthorizationError {
    #[error("explicit remote selection requirements are invalid")]
    InvalidRemoteRequirements,
    #[error("remote authorization version is unsupported")]
    UnsupportedAuthorizationVersion,
    #[error("remote authorization evidence is invalid")]
    InvalidAuthorizationEvidence,
    #[error("remote authorization is not associated with the prompt compilation")]
    PromptCompilationAssociation,
    #[error("remote authorization and model registry are inconsistent")]
    AuthorizationRegistryInconsistency,
    #[error("availability-gated remote selection failed")]
    AvailabilitySelection(ModelAvailabilityError),
    #[error("authorized remote model selection failed")]
    Selection(ModelSelectionError),
}

pub fn select_authorized_available_remote_model(
    registry: &ModelRegistry,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    authorization: &RemoteModelAuthorization,
    compilation: &PromptCompilationResult,
) -> Result<SelectedModel, RemoteAuthorizationError> {
    requirements
        .validate()
        .map_err(|_| RemoteAuthorizationError::InvalidRemoteRequirements)?;
    if requirements.privacy_preference.is_empty()
        || requirements.privacy_preference.iter().any(|privacy| {
            !matches!(
                privacy,
                PrivacyClass::ApprovedRemote | PrivacyClass::RestrictedRemote
            )
        })
    {
        return Err(RemoteAuthorizationError::InvalidRemoteRequirements);
    }
    compilation
        .validate()
        .map_err(|_| RemoteAuthorizationError::PromptCompilationAssociation)?;
    authorization.validate()?;
    if authorization.prompt_compilation_replay_anchor != compilation.replay_anchor {
        return Err(RemoteAuthorizationError::PromptCompilationAssociation);
    }
    for entry in &authorization.entries {
        let provider = registry
            .resolve(entry.provider_id, entry.model_id)
            .map_err(|_| RemoteAuthorizationError::AuthorizationRegistryInconsistency)?;
        if provider.descriptor().provider_id != entry.provider_id
            || provider.descriptor().model_id != entry.model_id
            || provider.descriptor().privacy_class != entry.privacy_class
            || !registry.inventory().contains(provider.descriptor())
        {
            return Err(RemoteAuthorizationError::AuthorizationRegistryInconsistency);
        }
    }
    let states = availability_states_for_registry(registry, availability)
        .map_err(RemoteAuthorizationError::AvailabilitySelection)?;
    select_model_where(
        registry,
        &compilation.model_input,
        requirements,
        |descriptor| {
            authorization.entries.iter().any(|entry| {
                entry.provider_id == descriptor.provider_id
                    && entry.model_id == descriptor.model_id
                    && entry.privacy_class == descriptor.privacy_class
            }) && states.get(&(descriptor.provider_id, descriptor.model_id))
                == Some(&ModelAvailabilityState::Available)
        },
    )
    .map_err(RemoteAuthorizationError::Selection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        availability::{ModelAvailabilityEntry, ModelAvailabilityState},
        model::{
            LanguageModelProvider, ModelCapabilities, ModelDescriptor, ModelErrorKind,
            PrivacyClass, RequiredCapabilities, ScriptedModelProvider, ScriptedOutcome,
        },
        prompt::{
            compile_prompt, PromptCompilationRequest, PromptContent, PromptLayer, PromptLayerKind,
            PromptLimits, PROMPT_COMPILATION_V1,
        },
    };
    use std::sync::Arc;
    use uuid::Uuid;

    fn pid(value: u128) -> ModelProviderId {
        ModelProviderId::new(Uuid::from_u128(value)).unwrap()
    }
    fn mid(value: u128) -> ModelId {
        ModelId::new(Uuid::from_u128(value)).unwrap()
    }
    fn auth_entry(
        provider: u128,
        model: u128,
        privacy_class: PrivacyClass,
    ) -> RemoteModelAuthorizationEntry {
        RemoteModelAuthorizationEntry {
            provider_id: pid(provider),
            model_id: mid(model),
            privacy_class,
        }
    }
    fn compilation(student: &str) -> PromptCompilationResult {
        let kinds = [
            PromptLayerKind::PlatformContract,
            PromptLayerKind::NexaIdentity,
            PromptLayerKind::Policy,
            PromptLayerKind::Pedagogy,
            PromptLayerKind::StudentInput,
            PromptLayerKind::OutputContract,
        ];
        compile_prompt(&PromptCompilationRequest {
            contract_version: PROMPT_COMPILATION_V1,
            prompt_package_version: ProtocolVersion::new(1, 0),
            context_builder_version: ProtocolVersion::new(1, 0),
            output_schema_version: ProtocolVersion::new(1, 0),
            limits: PromptLimits {
                maximum_layer_bytes: 1024,
                maximum_compiled_bytes: 8192,
            },
            layers: kinds
                .into_iter()
                .map(|kind| PromptLayer {
                    kind,
                    classification: kind.classification(),
                    content: PromptContent::new(if kind == PromptLayerKind::StudentInput {
                        student
                    } else {
                        "x"
                    })
                    .unwrap(),
                })
                .collect(),
        })
        .unwrap()
    }
    fn provider(
        provider: u128,
        model: u128,
        privacy_class: PrivacyClass,
        context: u32,
        output: u32,
    ) -> (Arc<ScriptedModelProvider>, Arc<dyn LanguageModelProvider>) {
        let descriptor = ModelDescriptor::new(
            pid(provider),
            mid(model),
            privacy_class,
            ModelCapabilities {
                streaming: false,
                structured_output: true,
                tool_calling: false,
                vision: false,
                context_window_tokens: context,
                maximum_output_tokens: output,
            },
        )
        .unwrap();
        let concrete = Arc::new(
            ScriptedModelProvider::new(
                descriptor,
                [ScriptedOutcome::Error(ModelErrorKind::Unavailable)],
            )
            .unwrap(),
        );
        let handle: Arc<dyn LanguageModelProvider> = concrete.clone();
        (concrete, handle)
    }
    fn provider_with_capabilities(
        provider: u128,
        model: u128,
        privacy_class: PrivacyClass,
        capabilities: ModelCapabilities,
        outcomes: usize,
    ) -> (Arc<ScriptedModelProvider>, Arc<dyn LanguageModelProvider>) {
        let descriptor =
            ModelDescriptor::new(pid(provider), mid(model), privacy_class, capabilities).unwrap();
        let concrete = Arc::new(
            ScriptedModelProvider::new(
                descriptor,
                (0..outcomes).map(|_| ScriptedOutcome::Error(ModelErrorKind::Unavailable)),
            )
            .unwrap(),
        );
        let handle: Arc<dyn LanguageModelProvider> = concrete.clone();
        (concrete, handle)
    }
    fn requirements(privacy: Vec<PrivacyClass>, output: u32) -> ModelSelectionRequirements {
        ModelSelectionRequirements::new(
            RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            output,
            privacy,
        )
        .unwrap()
    }

    #[test]
    fn authorized_remote_selection_constructor_wire_and_intrinsic_validation() {
        let c = compilation("sentinel-prompt");
        let value = RemoteModelAuthorization::new(
            c.replay_anchor.clone(),
            vec![
                auth_entry(2, 1, PrivacyClass::RestrictedRemote),
                auth_entry(1, 2, PrivacyClass::ApprovedRemote),
            ],
        )
        .unwrap();
        assert_eq!(value.entries[0].provider_id, pid(1));
        let wire = serde_json::to_string(&value).unwrap();
        assert_eq!(
            serde_json::from_str::<RemoteModelAuthorization>(&wire).unwrap(),
            value
        );
        let mut unknown: serde_json::Value = serde_json::from_str(&wire).unwrap();
        unknown["secret"] = serde_json::json!("credential-sentinel");
        assert!(serde_json::from_value::<RemoteModelAuthorization>(unknown).is_err());
        for anchor in ["a".repeat(63), "A".repeat(64), "g".repeat(64)] {
            assert_eq!(
                RemoteModelAuthorization::new(anchor, vec![]),
                Err(RemoteAuthorizationError::InvalidAuthorizationEvidence)
            );
        }
        assert!(RemoteModelAuthorization::new(
            c.replay_anchor,
            vec![auth_entry(1, 1, PrivacyClass::LocalOnly)]
        )
        .is_err());
    }

    #[test]
    fn authorized_remote_selection_entry_standalone_wire_validation() {
        for privacy_class in [PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote] {
            let entry = auth_entry(1, 2, privacy_class);
            let wire = serde_json::to_value(entry).unwrap();
            assert_eq!(
                serde_json::from_value::<RemoteModelAuthorizationEntry>(wire).unwrap(),
                entry
            );
        }
        let local = serde_json::to_value(auth_entry(1, 2, PrivacyClass::LocalOnly)).unwrap();
        assert!(serde_json::from_value::<RemoteModelAuthorizationEntry>(local).is_err());
        let mut unknown =
            serde_json::to_value(auth_entry(1, 2, PrivacyClass::ApprovedRemote)).unwrap();
        unknown["credential-sentinel"] = serde_json::json!(true);
        assert!(serde_json::from_value::<RemoteModelAuthorizationEntry>(unknown).is_err());
        for identity in ["provider_id", "model_id"] {
            let mut nil =
                serde_json::to_value(auth_entry(1, 2, PrivacyClass::ApprovedRemote)).unwrap();
            nil[identity] = serde_json::json!(Uuid::nil());
            assert!(serde_json::from_value::<RemoteModelAuthorizationEntry>(nil).is_err());
        }
    }

    #[test]
    fn authorized_remote_selection_versions_bounds_duplicates_and_noncanonical_wire() {
        let c = compilation("x");
        let mut valid = RemoteModelAuthorization::new(c.replay_anchor, vec![]).unwrap();
        valid.contract_version = ProtocolVersion::new(2, 0);
        assert_eq!(
            valid.validate(),
            Err(RemoteAuthorizationError::UnsupportedAuthorizationVersion)
        );
        valid.contract_version = REMOTE_AUTHORIZATION_V1;
        valid.policy_version = ProtocolVersion::new(2, 0);
        assert_eq!(
            valid.validate(),
            Err(RemoteAuthorizationError::UnsupportedAuthorizationVersion)
        );
        let entries: Vec<_> = (1..=MAX_REMOTE_AUTHORIZATION_ENTRIES as u128)
            .map(|v| auth_entry(1, v, PrivacyClass::ApprovedRemote))
            .collect();
        assert!(RemoteModelAuthorization::new("a".repeat(64), entries.clone()).is_ok());
        let mut over = entries;
        over.push(auth_entry(2, 1, PrivacyClass::ApprovedRemote));
        assert!(RemoteModelAuthorization::new("a".repeat(64), over.clone()).is_err());
        let over_wire = serde_json::json!({"contract_version":{"major":1,"minor":0},"policy_version":{"major":1,"minor":0},"prompt_compilation_replay_anchor":"a".repeat(64),"entries":over});
        assert!(serde_json::from_value::<RemoteModelAuthorization>(over_wire).is_err());
        for entries in [
            vec![
                auth_entry(2, 1, PrivacyClass::ApprovedRemote),
                auth_entry(1, 1, PrivacyClass::ApprovedRemote),
            ],
            vec![auth_entry(1, 1, PrivacyClass::ApprovedRemote); 2],
        ] {
            let wire = serde_json::json!({"contract_version":{"major":1,"minor":0},"policy_version":{"major":1,"minor":0},"prompt_compilation_replay_anchor":"a".repeat(64),"entries":entries});
            assert!(serde_json::from_value::<RemoteModelAuthorization>(wire).is_err());
        }
    }

    #[test]
    fn authorized_remote_selection_intersects_authorization_availability_and_static_eligibility_without_invocation(
    ) {
        let c = compilation("remote-sensitive-sentinel");
        let needed = c.compiled_bytes + 8;
        let (approved, approved_handle) = provider(2, 1, PrivacyClass::ApprovedRemote, needed, 8);
        let (restricted, restricted_handle) =
            provider(1, 1, PrivacyClass::RestrictedRemote, needed, 8);
        let (local, local_handle) = provider(3, 1, PrivacyClass::LocalOnly, needed, 8);
        let registry = ModelRegistry::try_from_providers([
            approved_handle.clone(),
            restricted_handle.clone(),
            local_handle,
        ])
        .unwrap();
        let authorization = RemoteModelAuthorization::new(
            c.replay_anchor.clone(),
            vec![
                auth_entry(2, 1, PrivacyClass::ApprovedRemote),
                auth_entry(1, 1, PrivacyClass::RestrictedRemote),
            ],
        )
        .unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![
            ModelAvailabilityEntry {
                provider_id: pid(2),
                model_id: mid(1),
                state: ModelAvailabilityState::Available,
            },
            ModelAvailabilityEntry {
                provider_id: pid(1),
                model_id: mid(1),
                state: ModelAvailabilityState::Available,
            },
        ])
        .unwrap();
        let selected = select_authorized_available_remote_model(
            &registry,
            &requirements(
                vec![PrivacyClass::ApprovedRemote, PrivacyClass::RestrictedRemote],
                8,
            ),
            &availability,
            &authorization,
            &c,
        )
        .unwrap();
        assert!(Arc::ptr_eq(&selected.provider, &approved_handle));
        assert_eq!(
            (
                approved.remaining(),
                restricted.remaining(),
                local.remaining()
            ),
            (1, 1, 1)
        );
        let selected = select_authorized_available_remote_model(
            &registry,
            &requirements(vec![PrivacyClass::RestrictedRemote], 8),
            &availability,
            &authorization,
            &c,
        )
        .unwrap();
        assert!(Arc::ptr_eq(&selected.provider, &restricted_handle));
        assert_eq!(
            (
                approved.remaining(),
                restricted.remaining(),
                local.remaining()
            ),
            (1, 1, 1)
        );
    }

    #[test]
    fn authorized_remote_selection_denials_association_and_content_free_errors() {
        let c = compilation("distinctive-prompt-learner-knowledge-credential-endpoint");
        let (remote, handle) =
            provider(1, 1, PrivacyClass::ApprovedRemote, c.compiled_bytes + 4, 4);
        let registry = ModelRegistry::try_from_providers([handle]).unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: pid(1),
            model_id: mid(1),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let empty = RemoteModelAuthorization::new(c.replay_anchor.clone(), vec![]).unwrap();
        let error = select_authorized_available_remote_model(
            &registry,
            &requirements(vec![PrivacyClass::ApprovedRemote], 4),
            &availability,
            &empty,
            &c,
        )
        .unwrap_err();
        assert_eq!(
            error,
            RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel)
        );
        let wrong = RemoteModelAuthorization::new(
            "a".repeat(64),
            vec![auth_entry(1, 1, PrivacyClass::ApprovedRemote)],
        )
        .unwrap();
        let error = select_authorized_available_remote_model(
            &registry,
            &requirements(vec![PrivacyClass::ApprovedRemote], 4),
            &availability,
            &wrong,
            &c,
        )
        .unwrap_err();
        assert_eq!(
            error,
            RemoteAuthorizationError::PromptCompilationAssociation
        );
        for privacy in [
            vec![PrivacyClass::LocalOnly],
            vec![PrivacyClass::LocalOnly, PrivacyClass::ApprovedRemote],
        ] {
            assert_eq!(
                select_authorized_available_remote_model(
                    &registry,
                    &requirements(privacy, 4),
                    &availability,
                    &empty,
                    &c
                )
                .unwrap_err(),
                RemoteAuthorizationError::InvalidRemoteRequirements
            );
        }
        let diagnostics = format!("{error:?} {error}");
        for sentinel in [
            "distinctive-prompt",
            "learner",
            "knowledge",
            "credential",
            "endpoint",
        ] {
            assert!(!diagnostics.contains(sentinel));
        }
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn authorized_remote_selection_rejects_intrinsically_tampered_compilation() {
        let c = compilation("learner-secret-knowledge-secret-provider-private-secret");
        let (remote, handle) =
            provider(1, 1, PrivacyClass::ApprovedRemote, c.compiled_bytes + 4, 4);
        let registry = ModelRegistry::try_from_providers([handle]).unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: pid(1),
            model_id: mid(1),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let authorization = RemoteModelAuthorization::new(
            c.replay_anchor.clone(),
            vec![auth_entry(1, 1, PrivacyClass::ApprovedRemote)],
        )
        .unwrap();
        let mut tampered = Vec::new();
        let mut manifest = c.clone();
        manifest.manifest[0].content_bytes += 1;
        tampered.push(manifest);
        let mut count = c.clone();
        count.compiled_bytes += 1;
        tampered.push(count);
        let mut replay = c.clone();
        replay.replay_anchor = "a".repeat(64);
        tampered.push(replay);
        let mut reassociated = c.clone();
        reassociated.model_input = compilation("different-input").model_input;
        tampered.push(reassociated);
        for evidence in tampered {
            let error = select_authorized_available_remote_model(
                &registry,
                &requirements(vec![PrivacyClass::ApprovedRemote], 4),
                &availability,
                &authorization,
                &evidence,
            )
            .unwrap_err();
            assert_eq!(
                error,
                RemoteAuthorizationError::PromptCompilationAssociation
            );
            let diagnostics = format!("{error:?} {error}");
            for sentinel in [
                "learner-secret",
                "knowledge-secret",
                "provider-private-secret",
            ] {
                assert!(!diagnostics.contains(sentinel));
            }
        }
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn authorized_remote_selection_validates_requirements_at_exact_boundary() {
        let c = compilation("x");
        let (remote, handle) =
            provider(1, 1, PrivacyClass::ApprovedRemote, c.compiled_bytes + 4, 4);
        let registry = ModelRegistry::try_from_providers([handle]).unwrap();
        let availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: pid(1),
            model_id: mid(1),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        let authorization = RemoteModelAuthorization::new(c.replay_anchor.clone(), vec![]).unwrap();
        let base = requirements(vec![PrivacyClass::ApprovedRemote], 4);
        let mut invalid = Vec::new();
        let mut empty = base.clone();
        empty.privacy_preference.clear();
        invalid.push(empty);
        let mut duplicate = base.clone();
        duplicate
            .privacy_preference
            .push(PrivacyClass::ApprovedRemote);
        invalid.push(duplicate);
        let mut version = base.clone();
        version.contract_version = ProtocolVersion::new(2, 0);
        invalid.push(version);
        let mut local = base.clone();
        local.privacy_preference = vec![PrivacyClass::LocalOnly];
        invalid.push(local);
        let mut mixed = base;
        mixed.privacy_preference = vec![PrivacyClass::ApprovedRemote, PrivacyClass::LocalOnly];
        invalid.push(mixed);
        for requirements in invalid {
            assert_eq!(
                select_authorized_available_remote_model(
                    &registry,
                    &requirements,
                    &availability,
                    &authorization,
                    &c
                )
                .unwrap_err(),
                RemoteAuthorizationError::InvalidRemoteRequirements
            );
        }
        assert_eq!(remote.remaining(), 1);
    }

    #[test]
    fn authorized_remote_selection_registry_agreement_and_exact_intersection() {
        let c = compilation("x");
        let needed = c.compiled_bytes + 4;
        let (one, one_handle) = provider(1, 1, PrivacyClass::ApprovedRemote, needed, 4);
        let (two, two_handle) = provider(2, 1, PrivacyClass::ApprovedRemote, needed, 4);
        let registry = ModelRegistry::try_from_providers([two_handle, one_handle.clone()]).unwrap();
        let available_one = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: pid(1),
            model_id: mid(1),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        for entry in [
            auth_entry(9, 9, PrivacyClass::ApprovedRemote),
            auth_entry(1, 1, PrivacyClass::RestrictedRemote),
        ] {
            let authorization =
                RemoteModelAuthorization::new(c.replay_anchor.clone(), vec![entry]).unwrap();
            assert_eq!(
                select_authorized_available_remote_model(
                    &registry,
                    &requirements(vec![PrivacyClass::ApprovedRemote], 4),
                    &available_one,
                    &authorization,
                    &c
                )
                .unwrap_err(),
                RemoteAuthorizationError::AuthorizationRegistryInconsistency
            );
        }
        let authorize_one = RemoteModelAuthorization::new(
            c.replay_anchor.clone(),
            vec![auth_entry(1, 1, PrivacyClass::ApprovedRemote)],
        )
        .unwrap();
        let available_two = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: pid(2),
            model_id: mid(1),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        assert_eq!(
            select_authorized_available_remote_model(
                &registry,
                &requirements(vec![PrivacyClass::ApprovedRemote], 4),
                &available_two,
                &authorize_one,
                &c
            )
            .unwrap_err(),
            RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel)
        );
        let unavailable_one = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: pid(1),
            model_id: mid(1),
            state: ModelAvailabilityState::Unavailable,
        }])
        .unwrap();
        for availability in [&available_two, &unavailable_one] {
            assert_eq!(
                select_authorized_available_remote_model(
                    &registry,
                    &requirements(vec![PrivacyClass::ApprovedRemote], 4),
                    availability,
                    &authorize_one,
                    &c
                )
                .unwrap_err(),
                RemoteAuthorizationError::Selection(ModelSelectionError::NoEligibleModel)
            );
        }
        let unknown_availability = ModelAvailabilitySnapshot::new(vec![ModelAvailabilityEntry {
            provider_id: pid(99),
            model_id: mid(99),
            state: ModelAvailabilityState::Available,
        }])
        .unwrap();
        assert_eq!(
            select_authorized_available_remote_model(
                &registry,
                &requirements(vec![PrivacyClass::ApprovedRemote], 4),
                &unknown_availability,
                &authorize_one,
                &c
            )
            .unwrap_err(),
            RemoteAuthorizationError::AvailabilitySelection(
                ModelAvailabilityError::RegistryInconsistency
            )
        );
        assert_eq!((one.remaining(), two.remaining()), (1, 1));
        assert!(Arc::ptr_eq(
            &one_handle,
            &registry.resolve(pid(1), mid(1)).unwrap()
        ));
    }

    #[test]
    fn authorized_remote_selection_preserves_order_capabilities_capacity_and_all_outcomes() {
        let c = compilation("x");
        let input = c.compiled_bytes;
        let caps = |structured_output,
                    tool_calling,
                    vision,
                    context_window_tokens,
                    maximum_output_tokens| ModelCapabilities {
            streaming: false,
            structured_output,
            tool_calling,
            vision,
            context_window_tokens,
            maximum_output_tokens,
        };
        let (approved_high, approved_high_handle) = provider_with_capabilities(
            4,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, true, true, input + 8, 8),
            3,
        );
        let (approved_low, approved_low_handle) = provider_with_capabilities(
            2,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, true, true, input + 8, 8),
            3,
        );
        let (restricted, restricted_handle) = provider_with_capabilities(
            1,
            1,
            PrivacyClass::RestrictedRemote,
            caps(true, true, true, input + 8, 8),
            3,
        );
        let (missing_tools, missing_tools_handle) = provider_with_capabilities(
            5,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, false, true, input + 8, 8),
            3,
        );
        let (missing_vision, missing_vision_handle) = provider_with_capabilities(
            6,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, true, false, input + 8, 8),
            3,
        );
        let (missing_structured, missing_structured_handle) = provider_with_capabilities(
            7,
            1,
            PrivacyClass::ApprovedRemote,
            caps(false, true, true, input + 8, 8),
            3,
        );
        let (short_output, short_output_handle) = provider_with_capabilities(
            8,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, true, true, input + 8, 7),
            3,
        );
        let (one_over, one_over_handle) = provider_with_capabilities(
            9,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, true, true, input + 7, 8),
            3,
        );
        let (unauthorized, unauthorized_handle) = provider_with_capabilities(
            10,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, true, true, input + 8, 8),
            3,
        );
        let (unavailable, unavailable_handle) = provider_with_capabilities(
            11,
            1,
            PrivacyClass::ApprovedRemote,
            caps(true, true, true, input + 8, 8),
            3,
        );
        let (local, local_handle) = provider_with_capabilities(
            12,
            1,
            PrivacyClass::LocalOnly,
            caps(true, true, true, input + 8, 8),
            3,
        );
        // Deliberately reverse identity order to prove registry insertion order is irrelevant.
        let registry = ModelRegistry::try_from_providers([
            local_handle,
            unavailable_handle,
            unauthorized_handle,
            one_over_handle,
            short_output_handle,
            missing_structured_handle,
            missing_vision_handle,
            missing_tools_handle,
            approved_high_handle,
            restricted_handle.clone(),
            approved_low_handle.clone(),
        ])
        .unwrap();
        let authorized_ids = [1_u128, 2, 4, 5, 6, 7, 8, 9, 11];
        let authorization = RemoteModelAuthorization::new(
            c.replay_anchor.clone(),
            authorized_ids
                .into_iter()
                .map(|id| {
                    auth_entry(
                        id,
                        1,
                        if id == 1 {
                            PrivacyClass::RestrictedRemote
                        } else {
                            PrivacyClass::ApprovedRemote
                        },
                    )
                })
                .collect(),
        )
        .unwrap();
        let available_ids = [1_u128, 2, 4, 5, 6, 7, 8, 9, 10];
        let availability = ModelAvailabilitySnapshot::new(
            available_ids
                .into_iter()
                .map(|id| ModelAvailabilityEntry {
                    provider_id: pid(id),
                    model_id: mid(1),
                    state: ModelAvailabilityState::Available,
                })
                .chain([ModelAvailabilityEntry {
                    provider_id: pid(11),
                    model_id: mid(1),
                    state: ModelAvailabilityState::Unavailable,
                }])
                .collect(),
        )
        .unwrap();
        let required = RequiredCapabilities {
            structured_output: true,
            tool_calling: true,
            vision: true,
        };
        let choose = |privacy_preference| {
            let requirements =
                ModelSelectionRequirements::new(required.clone(), 8, privacy_preference).unwrap();
            select_authorized_available_remote_model(
                &registry,
                &requirements,
                &availability,
                &authorization,
                &c,
            )
            .unwrap()
        };
        let selected = choose(vec![
            PrivacyClass::RestrictedRemote,
            PrivacyClass::ApprovedRemote,
        ]);
        assert!(Arc::ptr_eq(&selected.provider, &restricted_handle));
        let selected = choose(vec![
            PrivacyClass::ApprovedRemote,
            PrivacyClass::RestrictedRemote,
        ]);
        assert!(Arc::ptr_eq(&selected.provider, &approved_low_handle));
        for remaining in [
            approved_high.remaining(),
            approved_low.remaining(),
            restricted.remaining(),
            missing_tools.remaining(),
            missing_vision.remaining(),
            missing_structured.remaining(),
            short_output.remaining(),
            one_over.remaining(),
            unauthorized.remaining(),
            unavailable.remaining(),
            local.remaining(),
        ] {
            assert_eq!(remaining, 3);
        }
    }
}
