//! Static, deterministic, provider-neutral selection over an immutable registry.

use crate::{
    model::{
        LanguageModelProvider, ModelDescriptor, ModelErrorKind, ModelInput, PrivacyClass,
        RequiredCapabilities,
    },
    registry::ModelRegistry,
};
use nexa_domain::ProtocolVersion;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use thiserror::Error;

pub const MODEL_SELECTION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

/// Closed, content-free caller policy for one static selection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelSelectionRequirements {
    pub contract_version: ProtocolVersion,
    pub required_capabilities: RequiredCapabilities,
    pub maximum_output_tokens: u32,
    pub privacy_preference: Vec<PrivacyClass>,
}

impl ModelSelectionRequirements {
    pub fn new(
        required_capabilities: RequiredCapabilities,
        maximum_output_tokens: u32,
        privacy_preference: Vec<PrivacyClass>,
    ) -> Result<Self, ModelSelectionError> {
        let value = Self {
            contract_version: MODEL_SELECTION_V1,
            required_capabilities,
            maximum_output_tokens,
            privacy_preference,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> Result<(), ModelSelectionError> {
        if self.contract_version != MODEL_SELECTION_V1 {
            return Err(ModelSelectionError::UnsupportedRequirementsVersion);
        }
        if self.maximum_output_tokens == 0 || self.privacy_preference.is_empty() {
            return Err(ModelSelectionError::InvalidRequirements);
        }
        for (position, privacy) in self.privacy_preference.iter().enumerate() {
            if self.privacy_preference[..position].contains(privacy) {
                return Err(ModelSelectionError::InvalidRequirements);
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for ModelSelectionRequirements {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            required_capabilities: RequiredCapabilities,
            maximum_output_tokens: u32,
            privacy_preference: Vec<PrivacyClass>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let value = Self {
            contract_version: wire.contract_version,
            required_capabilities: wire.required_capabilities,
            maximum_output_tokens: wire.maximum_output_tokens,
            privacy_preference: wire.privacy_preference,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

/// The exact registered handle and immutable identity information selected for the host.
pub struct SelectedModel {
    pub descriptor: ModelDescriptor,
    pub provider: Arc<dyn LanguageModelProvider>,
}

impl fmt::Debug for SelectedModel {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SelectedModel")
            .field("provider_id", &self.descriptor.provider_id)
            .field("model_id", &self.descriptor.model_id)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ModelSelectionError {
    #[error("model selection requirements are invalid")]
    InvalidRequirements,
    #[error("model selection requirements version is unsupported")]
    UnsupportedRequirementsVersion,
    #[error("no registered model satisfies the selection requirements")]
    NoEligibleModel,
    #[error("model registry or descriptor is inconsistent")]
    RegistryInconsistency,
}

/// Selects exactly one model without invoking or otherwise consuming any provider.
pub fn select_model(
    registry: &ModelRegistry,
    input: &ModelInput,
    requirements: &ModelSelectionRequirements,
) -> Result<SelectedModel, ModelSelectionError> {
    select_model_where(registry, input, requirements, |_| true)
}

/// Shared ADR-0027 selection algorithm with an additional caller-owned eligibility gate.
pub(crate) fn select_model_where(
    registry: &ModelRegistry,
    input: &ModelInput,
    requirements: &ModelSelectionRequirements,
    is_available: impl Fn(&ModelDescriptor) -> bool,
) -> Result<SelectedModel, ModelSelectionError> {
    requirements.validate()?;
    let mut eligible = Vec::new();
    for descriptor in registry.inventory() {
        descriptor
            .validate()
            .map_err(|_| ModelSelectionError::RegistryInconsistency)?;
        if !is_available(descriptor) {
            continue;
        }
        let Some(privacy_rank) = requirements
            .privacy_preference
            .iter()
            .position(|privacy| privacy == &descriptor.privacy_class)
        else {
            continue;
        };
        match descriptor.validate_eligibility(
            input,
            &requirements.required_capabilities,
            requirements.maximum_output_tokens,
        ) {
            Ok(()) => eligible.push((privacy_rank, descriptor)),
            Err(error)
                if matches!(
                    error.kind,
                    ModelErrorKind::ContextTooLarge | ModelErrorKind::UnsupportedCapability
                ) => {}
            Err(_) => return Err(ModelSelectionError::RegistryInconsistency),
        }
    }
    eligible.sort_by_key(|(rank, descriptor)| (*rank, descriptor.provider_id, descriptor.model_id));
    let descriptor = eligible
        .first()
        .map(|(_, descriptor)| (*descriptor).clone())
        .ok_or(ModelSelectionError::NoEligibleModel)?;
    let provider = registry
        .resolve(descriptor.provider_id, descriptor.model_id)
        .map_err(|_| ModelSelectionError::RegistryInconsistency)?;
    if provider.descriptor() != &descriptor {
        return Err(ModelSelectionError::RegistryInconsistency);
    }
    Ok(SelectedModel {
        descriptor,
        provider,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        ModelCapabilities, ModelError, ModelRequest, ModelResponse, ScriptedModelProvider,
        ScriptedOutcome,
    };
    use nexa_domain::{ModelId, ModelProviderId};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    fn provider_id(value: u128) -> ModelProviderId {
        ModelProviderId::new(Uuid::from_u128(value)).unwrap()
    }
    fn model_id(value: u128) -> ModelId {
        ModelId::new(Uuid::from_u128(value)).unwrap()
    }
    fn descriptor(
        provider: u128,
        model: u128,
        privacy: PrivacyClass,
        capabilities: (bool, bool, bool),
        context: u32,
        output: u32,
    ) -> ModelDescriptor {
        ModelDescriptor::new(
            provider_id(provider),
            model_id(model),
            privacy,
            ModelCapabilities {
                streaming: false,
                structured_output: capabilities.0,
                tool_calling: capabilities.1,
                vision: capabilities.2,
                context_window_tokens: context,
                maximum_output_tokens: output,
            },
        )
        .unwrap()
    }
    fn provider(descriptor: ModelDescriptor) -> Arc<dyn LanguageModelProvider> {
        Arc::new(ScriptedModelProvider::new(descriptor, std::iter::empty()).unwrap())
    }

    struct InconsistentDescriptorProvider {
        registered: ModelDescriptor,
        changed: ModelDescriptor,
        descriptor_calls: AtomicUsize,
        generate_calls: AtomicUsize,
    }

    impl LanguageModelProvider for InconsistentDescriptorProvider {
        fn descriptor(&self) -> &ModelDescriptor {
            if self.descriptor_calls.fetch_add(1, Ordering::SeqCst) < 2 {
                &self.registered
            } else {
                &self.changed
            }
        }

        fn generate(&self, _request: &ModelRequest) -> Result<ModelResponse, ModelError> {
            self.generate_calls.fetch_add(1, Ordering::SeqCst);
            Err(ModelError::new(ModelErrorKind::Internal))
        }
    }
    fn requirements(
        capabilities: (bool, bool, bool),
        output: u32,
        privacy: Vec<PrivacyClass>,
    ) -> ModelSelectionRequirements {
        ModelSelectionRequirements::new(
            RequiredCapabilities {
                structured_output: capabilities.0,
                tool_calling: capabilities.1,
                vision: capabilities.2,
            },
            output,
            privacy,
        )
        .unwrap()
    }
    fn select(
        providers: Vec<Arc<dyn LanguageModelProvider>>,
        input: &str,
        requirements: &ModelSelectionRequirements,
    ) -> Result<SelectedModel, ModelSelectionError> {
        select_model(
            &ModelRegistry::try_from_providers(providers).unwrap(),
            &ModelInput::new(input).unwrap(),
            requirements,
        )
    }

    #[test]
    fn requirements_validate_versions_privacy_and_closed_wire() {
        let valid = requirements((false, false, false), 1, vec![PrivacyClass::LocalOnly]);
        assert_eq!(valid.validate(), Ok(()));
        let wire = serde_json::to_string(&valid).unwrap();
        assert_eq!(
            serde_json::from_str::<ModelSelectionRequirements>(&wire).unwrap(),
            valid
        );

        let mut unsupported = valid.clone();
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        assert_eq!(
            unsupported.validate(),
            Err(ModelSelectionError::UnsupportedRequirementsVersion)
        );
        assert!(serde_json::from_str::<ModelSelectionRequirements>(
            &serde_json::to_string(&unsupported).unwrap()
        )
        .is_err());

        let mut zero_output = valid.clone();
        zero_output.maximum_output_tokens = 0;
        assert_eq!(
            zero_output.validate(),
            Err(ModelSelectionError::InvalidRequirements)
        );
        assert!(serde_json::from_str::<ModelSelectionRequirements>(
            &serde_json::to_string(&zero_output).unwrap()
        )
        .is_err());

        for privacy in [
            vec![],
            vec![PrivacyClass::LocalOnly, PrivacyClass::LocalOnly],
        ] {
            let invalid = ModelSelectionRequirements {
                contract_version: MODEL_SELECTION_V1,
                required_capabilities: RequiredCapabilities {
                    structured_output: false,
                    tool_calling: false,
                    vision: false,
                },
                maximum_output_tokens: 1,
                privacy_preference: privacy,
            };
            assert_eq!(
                invalid.validate(),
                Err(ModelSelectionError::InvalidRequirements)
            );
            assert!(serde_json::from_str::<ModelSelectionRequirements>(
                &serde_json::to_string(&invalid).unwrap()
            )
            .is_err());
        }
        assert!(serde_json::from_str::<ModelSelectionRequirements>(
            r#"{"contract_version":{"major":1,"minor":0},"required_capabilities":{"structured_output":false,"tool_calling":false,"vision":false},"maximum_output_tokens":1,"privacy_preference":["local_only"],"prompt":"secret"}"#
        ).is_err());
    }

    #[test]
    fn empty_and_ineligible_registries_fail_closed() {
        let req = requirements((true, false, false), 2, vec![PrivacyClass::LocalOnly]);
        assert!(matches!(
            select(vec![], "x", &req),
            Err(ModelSelectionError::NoEligibleModel)
        ));
        let remote = provider(descriptor(
            1,
            1,
            PrivacyClass::ApprovedRemote,
            (true, true, true),
            20,
            10,
        ));
        assert!(matches!(
            select(vec![remote], "x", &req),
            Err(ModelSelectionError::NoEligibleModel)
        ));
    }

    #[test]
    fn capability_output_and_context_rules_filter_and_accept_exact_boundary() {
        for required in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
        ] {
            let incapable = provider(descriptor(
                1,
                1,
                PrivacyClass::LocalOnly,
                (false, false, false),
                20,
                10,
            ));
            assert!(select(
                vec![incapable],
                "x",
                &requirements(required, 1, vec![PrivacyClass::LocalOnly])
            )
            .is_err());
            let capable = provider(descriptor(1, 2, PrivacyClass::LocalOnly, required, 20, 10));
            assert_eq!(
                select(
                    vec![capable],
                    "x",
                    &requirements(required, 1, vec![PrivacyClass::LocalOnly])
                )
                .unwrap()
                .descriptor
                .model_id,
                model_id(2)
            );
        }
        let output_limited = provider(descriptor(
            1,
            1,
            PrivacyClass::LocalOnly,
            (false, false, false),
            20,
            2,
        ));
        assert_eq!(
            select(
                vec![output_limited],
                "x",
                &requirements((false, false, false), 3, vec![PrivacyClass::LocalOnly])
            )
            .unwrap_err(),
            ModelSelectionError::NoEligibleModel
        );

        let limited = provider(descriptor(
            1,
            1,
            PrivacyClass::LocalOnly,
            (false, false, false),
            5,
            2,
        ));
        assert!(select(
            vec![Arc::clone(&limited)],
            "abc",
            &requirements((false, false, false), 2, vec![PrivacyClass::LocalOnly])
        )
        .is_ok());
        assert!(select(
            vec![limited],
            "abcd",
            &requirements((false, false, false), 2, vec![PrivacyClass::LocalOnly])
        )
        .is_err());
    }

    #[test]
    fn privacy_preference_and_canonical_identities_define_total_order() {
        let local = provider(descriptor(
            9,
            9,
            PrivacyClass::LocalOnly,
            (false, false, false),
            20,
            10,
        ));
        let remote_model_low = provider(descriptor(
            1,
            1,
            PrivacyClass::ApprovedRemote,
            (false, false, false),
            20,
            10,
        ));
        let remote_low = provider(descriptor(
            1,
            2,
            PrivacyClass::ApprovedRemote,
            (false, false, false),
            20,
            10,
        ));
        let remote_high = provider(descriptor(
            2,
            1,
            PrivacyClass::ApprovedRemote,
            (false, false, false),
            20,
            10,
        ));
        let restricted = provider(descriptor(
            0x10,
            1,
            PrivacyClass::RestrictedRemote,
            (false, false, false),
            20,
            10,
        ));
        let req = requirements(
            (false, false, false),
            1,
            vec![PrivacyClass::ApprovedRemote, PrivacyClass::LocalOnly],
        );
        for providers in [
            vec![
                Arc::clone(&local),
                Arc::clone(&remote_high),
                Arc::clone(&remote_low),
                Arc::clone(&remote_model_low),
                Arc::clone(&restricted),
            ],
            vec![restricted, remote_low, remote_high, remote_model_low, local],
        ] {
            let selected = select(providers, "x", &req).unwrap();
            assert_eq!(
                (
                    selected.descriptor.provider_id,
                    selected.descriptor.model_id
                ),
                (provider_id(1), model_id(1))
            );
        }
        for privacy in [
            PrivacyClass::LocalOnly,
            PrivacyClass::ApprovedRemote,
            PrivacyClass::RestrictedRemote,
        ] {
            let only = provider(descriptor(3, 3, privacy, (false, false, false), 20, 10));
            assert!(select(
                vec![only],
                "x",
                &requirements((false, false, false), 1, vec![privacy])
            )
            .is_ok());
            let omitted = match privacy {
                PrivacyClass::LocalOnly => PrivacyClass::ApprovedRemote,
                PrivacyClass::ApprovedRemote | PrivacyClass::RestrictedRemote => {
                    PrivacyClass::LocalOnly
                }
            };
            let only = provider(descriptor(3, 3, privacy, (false, false, false), 20, 10));
            assert_eq!(
                select(
                    vec![only],
                    "x",
                    &requirements((false, false, false), 1, vec![omitted])
                )
                .unwrap_err(),
                ModelSelectionError::NoEligibleModel
            );
        }
    }

    #[test]
    fn inconsistent_descriptor_reference_fails_closed_without_generation() {
        let registered = descriptor(7, 7, PrivacyClass::LocalOnly, (false, false, false), 20, 10);
        let mut changed = registered.clone();
        changed.privacy_class = PrivacyClass::ApprovedRemote;
        let inconsistent = Arc::new(InconsistentDescriptorProvider {
            registered,
            changed,
            descriptor_calls: AtomicUsize::new(0),
            generate_calls: AtomicUsize::new(0),
        });
        let handle: Arc<dyn LanguageModelProvider> = inconsistent.clone();
        let registry = ModelRegistry::try_from_providers([handle]).unwrap();

        assert_eq!(
            select_model(
                &registry,
                &ModelInput::new("x").unwrap(),
                &requirements((false, false, false), 1, vec![PrivacyClass::LocalOnly])
            )
            .unwrap_err(),
            ModelSelectionError::RegistryInconsistency
        );
        assert_eq!(inconsistent.generate_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn selection_preserves_arc_and_does_not_consume_scripted_outcome() {
        let scripted = Arc::new(
            ScriptedModelProvider::new(
                descriptor(1, 1, PrivacyClass::LocalOnly, (false, false, false), 20, 10),
                [ScriptedOutcome::Error(ModelErrorKind::Unavailable)],
            )
            .unwrap(),
        );
        let handle: Arc<dyn LanguageModelProvider> = scripted.clone();
        let selected = select(
            vec![Arc::clone(&handle)],
            "private prompt",
            &requirements((false, false, false), 1, vec![PrivacyClass::LocalOnly]),
        )
        .unwrap();
        assert!(Arc::ptr_eq(&selected.provider, &handle));
        assert_eq!(scripted.remaining(), 1);
        let diagnostics = format!(
            "{selected:?} {} {:?}",
            ModelSelectionError::NoEligibleModel,
            ModelSelectionError::InvalidRequirements
        );
        assert!(!diagnostics.contains("private prompt"));
    }

    #[test]
    fn shared_eligibility_preserves_model_request_validation() {
        let descriptor = descriptor(1, 1, PrivacyClass::LocalOnly, (true, false, false), 5, 2);
        let request = ModelRequest {
            invocation_id: nexa_domain::ModelInvocationId::new(Uuid::from_u128(4)).unwrap(),
            provider_id: provider_id(1),
            model_id: model_id(1),
            contract_version: crate::model::MODEL_INVOCATION_V1,
            input: ModelInput::new("abc").unwrap(),
            required_capabilities: RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            maximum_output_tokens: 2,
        };
        assert_eq!(request.validate_for(&descriptor), Ok(()));
        let mut overflow = request.clone();
        overflow.input = ModelInput::new("abcd").unwrap();
        assert_eq!(
            overflow.validate_for(&descriptor).unwrap_err().kind,
            ModelErrorKind::ContextTooLarge
        );
        let mut unsupported = request;
        unsupported.required_capabilities.tool_calling = true;
        assert_eq!(
            unsupported.validate_for(&descriptor).unwrap_err().kind,
            ModelErrorKind::UnsupportedCapability
        );
    }
}
