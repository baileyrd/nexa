//! Immutable, provider-neutral model registration and exact resolution.

use crate::model::{LanguageModelProvider, ModelDescriptor, ModelErrorKind};
use nexa_domain::{ModelId, ModelProviderId};
use std::{collections::BTreeMap, fmt, sync::Arc};
use thiserror::Error;

type RegistryKey = (ModelProviderId, ModelId);

/// Closed, content-free failures for model registry operations.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ModelRegistryError {
    #[error("model registry descriptor is invalid")]
    InvalidDescriptor,
    #[error("model registry descriptor version is unsupported")]
    UnsupportedDescriptorVersion,
    #[error("model registry contains a duplicate exact registration")]
    DuplicateRegistration,
    #[error("model registry has no exact registration")]
    MissingRegistration,
}

/// An immutable collection of validated provider/model associations.
///
/// Inventory order is the canonical ordering of `ModelProviderId`, followed by
/// `ModelId`. Construction, inventory, and resolution never invoke providers.
pub struct ModelRegistry {
    providers: BTreeMap<RegistryKey, Arc<dyn LanguageModelProvider>>,
    inventory: Vec<ModelDescriptor>,
}

impl ModelRegistry {
    /// Atomically validates and registers all supplied shared providers.
    pub fn try_from_providers(
        providers: impl IntoIterator<Item = Arc<dyn LanguageModelProvider>>,
    ) -> Result<Self, ModelRegistryError> {
        let mut registered = BTreeMap::new();
        for provider in providers {
            let descriptor = provider.descriptor();
            descriptor.validate().map_err(|error| match error.kind {
                ModelErrorKind::UnsupportedVersion => {
                    ModelRegistryError::UnsupportedDescriptorVersion
                }
                _ => ModelRegistryError::InvalidDescriptor,
            })?;
            let key = (descriptor.provider_id, descriptor.model_id);
            if registered.insert(key, provider).is_some() {
                return Err(ModelRegistryError::DuplicateRegistration);
            }
        }
        let inventory = registered
            .values()
            .map(|provider| provider.descriptor().clone())
            .collect();
        Ok(Self {
            providers: registered,
            inventory,
        })
    }

    /// Returns validated descriptors in canonical provider-then-model order.
    pub fn inventory(&self) -> &[ModelDescriptor] {
        &self.inventory
    }

    /// Resolves only the exact provider/model association supplied by the caller.
    pub fn resolve(
        &self,
        provider_id: ModelProviderId,
        model_id: ModelId,
    ) -> Result<Arc<dyn LanguageModelProvider>, ModelRegistryError> {
        self.providers
            .get(&(provider_id, model_id))
            .cloned()
            .ok_or(ModelRegistryError::MissingRegistration)
    }
}

impl fmt::Debug for ModelRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelRegistry")
            .field("registration_count", &self.providers.len())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        ModelCapabilities, ModelError, ModelRequest, ModelResponse, PrivacyClass,
        ScriptedModelProvider, ScriptedOutcome, MODEL_INVOCATION_V1,
    };
    use nexa_domain::ProtocolVersion;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    fn provider_id(value: u128) -> ModelProviderId {
        ModelProviderId::new(Uuid::from_u128(value)).unwrap()
    }

    fn model_id(value: u128) -> ModelId {
        ModelId::new(Uuid::from_u128(value)).unwrap()
    }

    fn descriptor(provider: u128, model: u128) -> ModelDescriptor {
        ModelDescriptor::new(
            provider_id(provider),
            model_id(model),
            PrivacyClass::LocalOnly,
            ModelCapabilities {
                streaming: false,
                structured_output: true,
                tool_calling: false,
                vision: false,
                context_window_tokens: 128,
                maximum_output_tokens: 32,
            },
        )
        .unwrap()
    }

    struct CountingProvider {
        descriptor: ModelDescriptor,
        calls: AtomicUsize,
        private_secret: &'static str,
    }

    impl CountingProvider {
        fn new(descriptor: ModelDescriptor) -> Self {
            Self {
                descriptor,
                calls: AtomicUsize::new(0),
                private_secret: "distinctive-private-provider-state",
            }
        }
    }

    impl LanguageModelProvider for CountingProvider {
        fn descriptor(&self) -> &ModelDescriptor {
            &self.descriptor
        }

        fn generate(&self, _request: &ModelRequest) -> Result<ModelResponse, ModelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let _ = self.private_secret;
            Err(ModelError::new(ModelErrorKind::Internal))
        }
    }

    fn shared(provider: u128, model: u128) -> Arc<dyn LanguageModelProvider> {
        Arc::new(CountingProvider::new(descriptor(provider, model)))
    }

    #[test]
    fn empty_registry_has_empty_inventory_and_exact_lookup_is_missing() {
        let registry = ModelRegistry::try_from_providers(std::iter::empty()).unwrap();
        assert!(registry.inventory().is_empty());
        assert!(matches!(
            registry.resolve(provider_id(1), model_id(1)),
            Err(ModelRegistryError::MissingRegistration)
        ));
    }

    #[test]
    fn exact_pairs_coexist_and_only_exact_pairs_resolve() {
        let one = shared(1, 10);
        let two = shared(1, 20);
        let three = shared(2, 10);
        let registry = ModelRegistry::try_from_providers([
            Arc::clone(&one),
            Arc::clone(&two),
            Arc::clone(&three),
        ])
        .unwrap();

        assert!(Arc::ptr_eq(
            &registry.resolve(provider_id(1), model_id(10)).unwrap(),
            &one
        ));
        assert!(registry.resolve(provider_id(2), model_id(20)).is_err());
        assert!(registry.resolve(provider_id(1), model_id(30)).is_err());
        assert_eq!(registry.inventory().len(), 3);
    }

    #[test]
    fn inventory_is_canonical_and_independent_of_input_order() {
        let ascending =
            ModelRegistry::try_from_providers([shared(1, 10), shared(1, 20), shared(2, 10)])
                .unwrap();
        let shuffled =
            ModelRegistry::try_from_providers([shared(2, 10), shared(1, 20), shared(1, 10)])
                .unwrap();
        assert_eq!(ascending.inventory(), shuffled.inventory());
        assert_eq!(ascending.inventory()[0], descriptor(1, 10));
        assert_eq!(ascending.inventory()[1], descriptor(1, 20));
        assert_eq!(ascending.inventory()[2], descriptor(2, 10));
    }

    #[test]
    fn invalid_and_unsupported_descriptors_fail_closed() {
        let mut invalid = descriptor(1, 1);
        invalid.capabilities.maximum_output_tokens = 0;
        assert_eq!(
            ModelRegistry::try_from_providers([Arc::new(CountingProvider::new(invalid)) as Arc<_>])
                .unwrap_err(),
            ModelRegistryError::InvalidDescriptor
        );

        let mut unsupported = descriptor(1, 1);
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        assert_ne!(unsupported.contract_version, MODEL_INVOCATION_V1);
        assert_eq!(
            ModelRegistry::try_from_providers([
                Arc::new(CountingProvider::new(unsupported)) as Arc<_>
            ])
            .unwrap_err(),
            ModelRegistryError::UnsupportedDescriptorVersion
        );
    }

    #[test]
    fn duplicate_exact_registration_is_rejected_atomically() {
        assert_eq!(
            ModelRegistry::try_from_providers([shared(1, 1), shared(1, 1)]).unwrap_err(),
            ModelRegistryError::DuplicateRegistration
        );
    }

    #[test]
    fn registry_operations_do_not_invoke_or_consume_providers() {
        let scripted = Arc::new(
            ScriptedModelProvider::new(
                descriptor(1, 1),
                [ScriptedOutcome::Error(ModelErrorKind::Unavailable)],
            )
            .unwrap(),
        );
        let handle: Arc<dyn LanguageModelProvider> = scripted.clone();
        let registry = ModelRegistry::try_from_providers([Arc::clone(&handle)]).unwrap();
        assert_eq!(scripted.remaining(), 1);
        let resolved = registry.resolve(provider_id(1), model_id(1)).unwrap();
        assert!(Arc::ptr_eq(&resolved, &handle));
        assert_eq!(scripted.remaining(), 1);
        assert_eq!(registry.inventory().len(), 1);
        assert_eq!(scripted.remaining(), 1);
    }

    #[test]
    fn registry_is_send_sync_and_diagnostics_are_content_free() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ModelRegistry>();
        assert_send_sync::<Arc<dyn LanguageModelProvider>>();

        let registry = ModelRegistry::try_from_providers([shared(1, 1)]).unwrap();
        let diagnostics = format!(
            "{registry:?} {} {:?}",
            ModelRegistryError::MissingRegistration,
            ModelRegistryError::InvalidDescriptor
        );
        assert!(!diagnostics.contains("distinctive-private-provider-state"));
        assert!(!diagnostics.contains("distinctive-provider-output-secret"));
        assert_eq!(
            format!("{registry:?}"),
            "ModelRegistry { registration_count: 1 }"
        );
    }
}
