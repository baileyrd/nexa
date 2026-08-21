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

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteModelAuthorizationEntry {
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub privacy_class: PrivacyClass,
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
        assert!(RemoteModelAuthorization::new("a".repeat(64), over).is_err());
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
}
