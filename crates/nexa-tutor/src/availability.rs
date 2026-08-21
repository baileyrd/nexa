//! Caller-supplied, deterministic availability evidence for model selection.

use crate::{
    model::{ModelDescriptor, ModelInput},
    registry::ModelRegistry,
    selection::{
        select_model_where, ModelSelectionError, ModelSelectionRequirements, SelectedModel,
    },
};
use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub const MODEL_AVAILABILITY_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const MAX_MODEL_AVAILABILITY_ENTRIES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelAvailabilityState {
    Available,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelAvailabilityEntry {
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub state: ModelAvailabilityState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelAvailabilitySnapshot {
    pub contract_version: ProtocolVersion,
    pub entries: Vec<ModelAvailabilityEntry>,
}

impl ModelAvailabilitySnapshot {
    /// Constructs a valid snapshot, canonicalizing provider/model order.
    pub fn new(mut entries: Vec<ModelAvailabilityEntry>) -> Result<Self, ModelAvailabilityError> {
        entries.sort_by_key(|entry| (entry.provider_id, entry.model_id));
        let snapshot = Self {
            contract_version: MODEL_AVAILABILITY_V1,
            entries,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }

    pub fn validate(&self) -> Result<(), ModelAvailabilityError> {
        if self.contract_version != MODEL_AVAILABILITY_V1 {
            return Err(ModelAvailabilityError::UnsupportedAvailabilityVersion);
        }
        if self.entries.len() > MAX_MODEL_AVAILABILITY_ENTRIES {
            return Err(ModelAvailabilityError::InvalidAvailability);
        }
        for pair in self.entries.windows(2) {
            let left = (pair[0].provider_id, pair[0].model_id);
            let right = (pair[1].provider_id, pair[1].model_id);
            if left >= right {
                return Err(ModelAvailabilityError::InvalidAvailability);
            }
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for ModelAvailabilitySnapshot {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            entries: Vec<ModelAvailabilityEntry>,
        }
        let wire = Wire::deserialize(deserializer)?;
        let snapshot = Self {
            contract_version: wire.contract_version,
            entries: wire.entries,
        };
        snapshot.validate().map_err(serde::de::Error::custom)?;
        Ok(snapshot)
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ModelAvailabilityError {
    #[error("model availability data is invalid")]
    InvalidAvailability,
    #[error("model availability version is unsupported")]
    UnsupportedAvailabilityVersion,
    #[error("model availability and registry are inconsistent")]
    RegistryInconsistency,
    #[error("availability-gated model selection failed")]
    Selection(ModelSelectionError),
}

/// Selects one explicitly available model without invoking any provider.
pub fn select_available_model(
    registry: &ModelRegistry,
    input: &ModelInput,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
) -> Result<SelectedModel, ModelAvailabilityError> {
    availability.validate()?;
    let states: BTreeMap<_, _> = availability
        .entries
        .iter()
        .map(|entry| ((entry.provider_id, entry.model_id), entry.state))
        .collect();
    for entry in &availability.entries {
        let provider = registry
            .resolve(entry.provider_id, entry.model_id)
            .map_err(|_| ModelAvailabilityError::RegistryInconsistency)?;
        if provider.descriptor().provider_id != entry.provider_id
            || provider.descriptor().model_id != entry.model_id
            || !registry.inventory().contains(provider.descriptor())
        {
            return Err(ModelAvailabilityError::RegistryInconsistency);
        }
    }
    select_model_where(
        registry,
        input,
        requirements,
        |descriptor: &ModelDescriptor| {
            states.get(&(descriptor.provider_id, descriptor.model_id))
                == Some(&ModelAvailabilityState::Available)
        },
    )
    .map_err(ModelAvailabilityError::Selection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            LanguageModelProvider, ModelCapabilities, ModelDescriptor, ModelError, ModelErrorKind,
            ModelRequest, ModelResponse, PrivacyClass, RequiredCapabilities, ScriptedModelProvider,
            ScriptedOutcome,
        },
        selection::MODEL_SELECTION_V1,
    };
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use uuid::Uuid;

    fn provider_id(value: u128) -> ModelProviderId {
        ModelProviderId::new(Uuid::from_u128(value)).unwrap()
    }
    fn model_id(value: u128) -> ModelId {
        ModelId::new(Uuid::from_u128(value)).unwrap()
    }
    fn entry(provider: u128, model: u128, state: ModelAvailabilityState) -> ModelAvailabilityEntry {
        ModelAvailabilityEntry {
            provider_id: provider_id(provider),
            model_id: model_id(model),
            state,
        }
    }
    fn descriptor(
        provider: u128,
        model: u128,
        privacy: PrivacyClass,
        caps: (bool, bool, bool),
        context: u32,
        output: u32,
    ) -> ModelDescriptor {
        ModelDescriptor::new(
            provider_id(provider),
            model_id(model),
            privacy,
            ModelCapabilities {
                streaming: false,
                structured_output: caps.0,
                tool_calling: caps.1,
                vision: caps.2,
                context_window_tokens: context,
                maximum_output_tokens: output,
            },
        )
        .unwrap()
    }
    fn scripted(
        descriptor: ModelDescriptor,
    ) -> (Arc<ScriptedModelProvider>, Arc<dyn LanguageModelProvider>) {
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
    fn requirements(
        caps: (bool, bool, bool),
        output: u32,
        privacy: Vec<PrivacyClass>,
    ) -> ModelSelectionRequirements {
        ModelSelectionRequirements::new(
            RequiredCapabilities {
                structured_output: caps.0,
                tool_calling: caps.1,
                vision: caps.2,
            },
            output,
            privacy,
        )
        .unwrap()
    }

    #[test]
    fn availability_snapshot_round_trips_and_constructor_canonicalizes() {
        let snapshot = ModelAvailabilitySnapshot::new(vec![
            entry(2, 1, ModelAvailabilityState::Unavailable),
            entry(1, 2, ModelAvailabilityState::Available),
        ])
        .unwrap();
        assert_eq!(snapshot.entries[0].provider_id, provider_id(1));
        let json = serde_json::to_string(&snapshot).unwrap();
        assert_eq!(
            serde_json::from_str::<ModelAvailabilitySnapshot>(&json).unwrap(),
            snapshot
        );
        assert!(!format!("{snapshot:?}").contains("prompt"));
    }

    #[test]
    fn availability_wire_validation_fails_closed() {
        let valid =
            ModelAvailabilitySnapshot::new(vec![entry(1, 1, ModelAvailabilityState::Available)])
                .unwrap();
        let mut unsupported = valid.clone();
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        assert_eq!(
            unsupported.validate(),
            Err(ModelAvailabilityError::UnsupportedAvailabilityVersion)
        );
        assert!(serde_json::from_str::<ModelAvailabilitySnapshot>(
            &serde_json::to_string(&unsupported).unwrap()
        )
        .is_err());
        assert!(serde_json::from_str::<ModelAvailabilitySnapshot>(
            r#"{"contract_version":{"major":1,"minor":0},"entries":[],"diagnostic":"private"}"#
        )
        .is_err());
        for entries in [
            vec![
                entry(1, 1, ModelAvailabilityState::Available),
                entry(1, 1, ModelAvailabilityState::Unavailable),
            ],
            vec![
                entry(2, 1, ModelAvailabilityState::Available),
                entry(1, 1, ModelAvailabilityState::Available),
            ],
        ] {
            let invalid = ModelAvailabilitySnapshot {
                contract_version: MODEL_AVAILABILITY_V1,
                entries,
            };
            assert_eq!(
                invalid.validate(),
                Err(ModelAvailabilityError::InvalidAvailability)
            );
            assert!(serde_json::from_str::<ModelAvailabilitySnapshot>(
                &serde_json::to_string(&invalid).unwrap()
            )
            .is_err());
        }
        let over_limit = ModelAvailabilitySnapshot {
            contract_version: MODEL_AVAILABILITY_V1,
            entries: (1..=(MAX_MODEL_AVAILABILITY_ENTRIES as u128 + 1))
                .map(|id| entry(id, 1, ModelAvailabilityState::Available))
                .collect(),
        };
        assert_eq!(
            over_limit.validate(),
            Err(ModelAvailabilityError::InvalidAvailability)
        );
        assert!(serde_json::from_str::<ModelAvailabilitySnapshot>(
            &serde_json::to_string(&over_limit).unwrap()
        )
        .is_err());
    }

    #[test]
    fn empty_missing_and_unavailable_snapshots_select_nothing_without_consumption() {
        let (concrete, handle) = scripted(descriptor(
            1,
            1,
            PrivacyClass::LocalOnly,
            (false, false, false),
            20,
            10,
        ));
        let registry = ModelRegistry::try_from_providers([handle]).unwrap();
        let req = requirements((false, false, false), 1, vec![PrivacyClass::LocalOnly]);
        for snapshot in [
            ModelAvailabilitySnapshot::new(vec![]).unwrap(),
            ModelAvailabilitySnapshot::new(vec![entry(2, 2, ModelAvailabilityState::Unavailable)])
                .unwrap(),
        ] {
            let expected = if snapshot.entries.is_empty() {
                ModelAvailabilityError::Selection(ModelSelectionError::NoEligibleModel)
            } else {
                ModelAvailabilityError::RegistryInconsistency
            };
            assert_eq!(
                select_available_model(
                    &registry,
                    &ModelInput::new("private prompt").unwrap(),
                    &req,
                    &snapshot
                )
                .unwrap_err(),
                expected
            );
            assert_eq!(concrete.remaining(), 1);
        }
        let unavailable =
            ModelAvailabilitySnapshot::new(vec![entry(1, 1, ModelAvailabilityState::Unavailable)])
                .unwrap();
        assert_eq!(
            select_available_model(
                &registry,
                &ModelInput::new("x").unwrap(),
                &req,
                &unavailable
            )
            .unwrap_err(),
            ModelAvailabilityError::Selection(ModelSelectionError::NoEligibleModel)
        );
        assert_eq!(concrete.remaining(), 1);
    }

    #[test]
    fn availability_gates_but_does_not_replace_adr_0027_order_or_eligibility() {
        let (highest_script, highest) = scripted(descriptor(
            1,
            1,
            PrivacyClass::ApprovedRemote,
            (true, true, true),
            100,
            20,
        ));
        let (next_script, next) = scripted(descriptor(
            2,
            1,
            PrivacyClass::ApprovedRemote,
            (true, true, true),
            100,
            20,
        ));
        let (local_script, local) = scripted(descriptor(
            9,
            1,
            PrivacyClass::LocalOnly,
            (true, true, true),
            100,
            20,
        ));
        let registry =
            ModelRegistry::try_from_providers([Arc::clone(&local), Arc::clone(&next), highest])
                .unwrap();
        let snapshot = ModelAvailabilitySnapshot::new(vec![
            entry(1, 1, ModelAvailabilityState::Unavailable),
            entry(2, 1, ModelAvailabilityState::Available),
            entry(9, 1, ModelAvailabilityState::Available),
        ])
        .unwrap();
        let selected = select_available_model(
            &registry,
            &ModelInput::new("x").unwrap(),
            &requirements(
                (true, false, false),
                2,
                vec![PrivacyClass::ApprovedRemote, PrivacyClass::LocalOnly],
            ),
            &snapshot,
        )
        .unwrap();
        assert_eq!(selected.descriptor.provider_id, provider_id(2));
        assert!(Arc::ptr_eq(&selected.provider, &next));
        assert_eq!(highest_script.remaining(), 1);
        assert_eq!(next_script.remaining(), 1);
        assert_eq!(local_script.remaining(), 1);

        let (_, limited) = scripted(descriptor(
            9,
            1,
            PrivacyClass::LocalOnly,
            (false, false, false),
            100,
            20,
        ));
        let limited_registry = ModelRegistry::try_from_providers([limited]).unwrap();
        let limited_snapshot =
            ModelAvailabilitySnapshot::new(vec![entry(9, 1, ModelAvailabilityState::Available)])
                .unwrap();
        for (caps, output, input) in [
            ((false, true, false), 1, "x"),
            ((false, false, false), 21, "x"),
            ((false, false, false), 2, &"x".repeat(99)),
        ] {
            assert_eq!(
                select_available_model(
                    &limited_registry,
                    &ModelInput::new(input).unwrap(),
                    &requirements(caps, output, vec![PrivacyClass::LocalOnly]),
                    &limited_snapshot
                )
                .unwrap_err(),
                ModelAvailabilityError::Selection(ModelSelectionError::NoEligibleModel)
            );
        }
        assert_eq!(MODEL_SELECTION_V1, ProtocolVersion::new(1, 0));
    }

    #[test]
    fn insertion_order_is_irrelevant_and_diagnostics_are_content_free() {
        let (_, first) = scripted(descriptor(
            1,
            1,
            PrivacyClass::LocalOnly,
            (false, false, false),
            20,
            10,
        ));
        let (_, second) = scripted(descriptor(
            2,
            1,
            PrivacyClass::LocalOnly,
            (false, false, false),
            20,
            10,
        ));
        let snapshot = ModelAvailabilitySnapshot::new(vec![
            entry(2, 1, ModelAvailabilityState::Available),
            entry(1, 1, ModelAvailabilityState::Available),
        ])
        .unwrap();
        for providers in [
            [Arc::clone(&first), Arc::clone(&second)],
            [Arc::clone(&second), Arc::clone(&first)],
        ] {
            let registry = ModelRegistry::try_from_providers(providers).unwrap();
            assert_eq!(
                select_available_model(
                    &registry,
                    &ModelInput::new("secret prompt").unwrap(),
                    &requirements((false, false, false), 1, vec![PrivacyClass::LocalOnly]),
                    &snapshot
                )
                .unwrap()
                .descriptor
                .provider_id,
                provider_id(1)
            );
        }
        let diagnostics = format!(
            "{snapshot:?} {:?}",
            ModelAvailabilityError::RegistryInconsistency
        );
        assert!(!diagnostics.contains("secret prompt"));
        assert!(!diagnostics.contains("provider-private"));
    }

    struct ChangingProvider {
        registered: ModelDescriptor,
        changed: ModelDescriptor,
        calls: AtomicUsize,
        generated: AtomicUsize,
    }
    impl LanguageModelProvider for ChangingProvider {
        fn descriptor(&self) -> &ModelDescriptor {
            if self.calls.fetch_add(1, Ordering::SeqCst) < 2 {
                &self.registered
            } else {
                &self.changed
            }
        }
        fn generate(&self, _: &ModelRequest) -> Result<ModelResponse, ModelError> {
            self.generated.fetch_add(1, Ordering::SeqCst);
            Err(ModelError::new(ModelErrorKind::Internal))
        }
    }
    #[test]
    fn descriptor_registry_inconsistency_fails_without_generation() {
        let registered = descriptor(1, 1, PrivacyClass::LocalOnly, (false, false, false), 20, 10);
        let changed = descriptor(1, 2, PrivacyClass::LocalOnly, (false, false, false), 20, 10);
        let concrete = Arc::new(ChangingProvider {
            registered,
            changed,
            calls: AtomicUsize::new(0),
            generated: AtomicUsize::new(0),
        });
        let handle: Arc<dyn LanguageModelProvider> = concrete.clone();
        let registry = ModelRegistry::try_from_providers([handle]).unwrap();
        assert_eq!(
            select_available_model(
                &registry,
                &ModelInput::new("x").unwrap(),
                &requirements((false, false, false), 1, vec![PrivacyClass::LocalOnly]),
                &ModelAvailabilitySnapshot::new(vec![entry(
                    1,
                    1,
                    ModelAvailabilityState::Available
                )])
                .unwrap()
            )
            .unwrap_err(),
            ModelAvailabilityError::RegistryInconsistency
        );
        assert_eq!(concrete.generated.load(Ordering::SeqCst), 0);
    }
}
