//! Provider-neutral model invocation contracts and a deterministic scripted adapter.

use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion};
use serde::{Deserialize, Serialize};
use std::{collections::VecDeque, fmt, sync::Mutex};
use thiserror::Error;

pub const MODEL_INVOCATION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub const MAX_MODEL_INPUT_BYTES: usize = 262_144;
pub const MAX_MODEL_OUTPUT_BYTES: usize = 131_072;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyClass {
    LocalOnly,
    ApprovedRemote,
    RestrictedRemote,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelCapabilities {
    pub streaming: bool,
    pub structured_output: bool,
    pub tool_calling: bool,
    pub vision: bool,
    pub context_window_tokens: u32,
    pub maximum_output_tokens: u32,
}

impl ModelCapabilities {
    fn validate(&self) -> Result<(), ModelError> {
        if self.streaming
            || self.context_window_tokens == 0
            || self.maximum_output_tokens == 0
            || self.maximum_output_tokens > self.context_window_tokens
        {
            Err(ModelError::new(ModelErrorKind::InvalidContract))
        } else {
            Ok(())
        }
    }
}

impl<'de> Deserialize<'de> for ModelCapabilities {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            streaming: bool,
            structured_output: bool,
            tool_calling: bool,
            vision: bool,
            context_window_tokens: u32,
            maximum_output_tokens: u32,
        }
        let wire = Wire::deserialize(deserializer)?;
        let value = Self {
            streaming: wire.streaming,
            structured_output: wire.structured_output,
            tool_calling: wire.tool_calling,
            vision: wire.vision,
            context_window_tokens: wire.context_window_tokens,
            maximum_output_tokens: wire.maximum_output_tokens,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCapabilities {
    pub structured_output: bool,
    pub tool_calling: bool,
    pub vision: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDescriptor {
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub contract_version: ProtocolVersion,
    pub privacy_class: PrivacyClass,
    pub capabilities: ModelCapabilities,
}

impl ModelDescriptor {
    pub fn new(
        provider_id: ModelProviderId,
        model_id: ModelId,
        privacy_class: PrivacyClass,
        capabilities: ModelCapabilities,
    ) -> Result<Self, ModelError> {
        let value = Self {
            provider_id,
            model_id,
            contract_version: MODEL_INVOCATION_V1,
            privacy_class,
            capabilities,
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), ModelError> {
        if self.contract_version != MODEL_INVOCATION_V1 {
            return Err(ModelError::new(ModelErrorKind::UnsupportedVersion));
        }
        self.capabilities.validate()
    }
}

impl<'de> Deserialize<'de> for ModelDescriptor {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            provider_id: ModelProviderId,
            model_id: ModelId,
            contract_version: ProtocolVersion,
            privacy_class: PrivacyClass,
            capabilities: ModelCapabilities,
        }
        let wire = Wire::deserialize(deserializer)?;
        let value = Self {
            provider_id: wire.provider_id,
            model_id: wire.model_id,
            contract_version: wire.contract_version,
            privacy_class: wire.privacy_class,
            capabilities: wire.capabilities,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ModelInput(String);

impl ModelInput {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_MODEL_INPUT_BYTES {
            Err(ModelError::new(ModelErrorKind::InvalidContract))
        } else {
            Ok(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ModelInput {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl fmt::Debug for ModelInput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ModelInput([REDACTED])")
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(transparent)]
pub struct RawModelOutput(String);

impl RawModelOutput {
    pub fn new(value: impl Into<String>) -> Result<Self, ModelError> {
        let value = value.into();
        if value.is_empty() || value.len() > MAX_MODEL_OUTPUT_BYTES {
            Err(ModelError::new(ModelErrorKind::InvalidResponse))
        } else {
            Ok(Self(value))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RawModelOutput {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

impl fmt::Debug for RawModelOutput {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RawModelOutput([REDACTED])")
    }
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelRequest {
    pub invocation_id: ModelInvocationId,
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub contract_version: ProtocolVersion,
    pub input: ModelInput,
    pub required_capabilities: RequiredCapabilities,
    pub maximum_output_tokens: u32,
}

impl ModelRequest {
    pub fn validate_for(&self, descriptor: &ModelDescriptor) -> Result<(), ModelError> {
        descriptor.validate()?;
        if self.contract_version != MODEL_INVOCATION_V1 {
            return Err(ModelError::new(ModelErrorKind::UnsupportedVersion));
        }
        if self.provider_id != descriptor.provider_id || self.model_id != descriptor.model_id {
            return Err(ModelError::new(ModelErrorKind::IdentityMismatch));
        }
        if self.maximum_output_tokens == 0
            || self.maximum_output_tokens > descriptor.capabilities.maximum_output_tokens
        {
            return Err(ModelError::new(ModelErrorKind::ContextTooLarge));
        }
        // V1 deliberately treats each UTF-8 input byte as one provider-neutral context unit.
        // This conservative evidence is checkable without selecting a provider tokenizer.
        let input_units = u32::try_from(self.input.as_str().len())
            .map_err(|_| ModelError::new(ModelErrorKind::ContextTooLarge))?;
        if input_units
            .checked_add(self.maximum_output_tokens)
            .is_none_or(|total| total > descriptor.capabilities.context_window_tokens)
        {
            return Err(ModelError::new(ModelErrorKind::ContextTooLarge));
        }
        let required = &self.required_capabilities;
        let available = &descriptor.capabilities;
        if (required.structured_output && !available.structured_output)
            || (required.tool_calling && !available.tool_calling)
            || (required.vision && !available.vision)
        {
            return Err(ModelError::new(ModelErrorKind::UnsupportedCapability));
        }
        Ok(())
    }
}

impl fmt::Debug for ModelRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelRequest")
            .field("invocation_id", &self.invocation_id)
            .field("provider_id", &self.provider_id)
            .field("model_id", &self.model_id)
            .field("input", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelRequestWire {
    invocation_id: ModelInvocationId,
    provider_id: ModelProviderId,
    model_id: ModelId,
    contract_version: ProtocolVersion,
    input: ModelInput,
    required_capabilities: RequiredCapabilities,
    maximum_output_tokens: u32,
}

impl<'de> Deserialize<'de> for ModelRequest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ModelRequestWire::deserialize(deserializer)?;
        if wire.contract_version != MODEL_INVOCATION_V1 || wire.maximum_output_tokens == 0 {
            return Err(serde::de::Error::custom("invalid model request contract"));
        }
        Ok(Self {
            invocation_id: wire.invocation_id,
            provider_id: wire.provider_id,
            model_id: wire.model_id,
            contract_version: wire.contract_version,
            input: wire.input,
            required_capabilities: wire.required_capabilities,
            maximum_output_tokens: wire.maximum_output_tokens,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Complete,
    OutputLimit,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

#[derive(Clone, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelResponse {
    pub invocation_id: ModelInvocationId,
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub contract_version: ProtocolVersion,
    pub output: RawModelOutput,
    pub finish_reason: FinishReason,
    pub reported_usage: Option<ModelUsage>,
}

impl ModelResponse {
    pub fn validate_for(&self, request: &ModelRequest) -> Result<(), ModelError> {
        if self.contract_version != MODEL_INVOCATION_V1 {
            return Err(ModelError::new(ModelErrorKind::UnsupportedVersion));
        }
        if self.invocation_id != request.invocation_id
            || self.provider_id != request.provider_id
            || self.model_id != request.model_id
        {
            return Err(ModelError::new(ModelErrorKind::IdentityMismatch));
        }
        if self
            .reported_usage
            .as_ref()
            .is_some_and(|usage| usage.output_tokens > request.maximum_output_tokens)
        {
            return Err(ModelError::new(ModelErrorKind::InvalidResponse));
        }
        Ok(())
    }
}

impl fmt::Debug for ModelResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ModelResponse")
            .field("invocation_id", &self.invocation_id)
            .field("provider_id", &self.provider_id)
            .field("model_id", &self.model_id)
            .field("output", &"[REDACTED]")
            .finish_non_exhaustive()
    }
}

impl<'de> Deserialize<'de> for ModelResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            invocation_id: ModelInvocationId,
            provider_id: ModelProviderId,
            model_id: ModelId,
            contract_version: ProtocolVersion,
            output: RawModelOutput,
            finish_reason: FinishReason,
            reported_usage: Option<ModelUsage>,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.contract_version != MODEL_INVOCATION_V1 {
            return Err(serde::de::Error::custom(
                "unsupported model response version",
            ));
        }
        Ok(Self {
            invocation_id: wire.invocation_id,
            provider_id: wire.provider_id,
            model_id: wire.model_id,
            contract_version: wire.contract_version,
            output: wire.output,
            finish_reason: wire.finish_reason,
            reported_usage: wire.reported_usage,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelErrorKind {
    Timeout,
    Unavailable,
    RateLimited,
    ContextTooLarge,
    InvalidResponse,
    UnsupportedCapability,
    Cancelled,
    IdentityMismatch,
    UnsupportedVersion,
    InvalidContract,
    ScriptExhausted,
    Internal,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
#[error("model invocation failed: {kind:?}")]
pub struct ModelError {
    pub kind: ModelErrorKind,
}

impl ModelError {
    pub const fn new(kind: ModelErrorKind) -> Self {
        Self { kind }
    }
}

pub trait LanguageModelProvider: Send + Sync {
    fn descriptor(&self) -> &ModelDescriptor;
    fn generate(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptedOutcome {
    Response(ModelResponse),
    Error(ModelErrorKind),
}

#[derive(Debug)]
pub struct ScriptedModelProvider {
    descriptor: ModelDescriptor,
    outcomes: Mutex<VecDeque<ScriptedOutcome>>,
}

impl ScriptedModelProvider {
    pub fn new(
        descriptor: ModelDescriptor,
        outcomes: impl IntoIterator<Item = ScriptedOutcome>,
    ) -> Result<Self, ModelError> {
        descriptor.validate()?;
        Ok(Self {
            descriptor,
            outcomes: Mutex::new(outcomes.into_iter().collect()),
        })
    }

    pub fn remaining(&self) -> usize {
        self.outcomes.lock().map_or(0, |outcomes| outcomes.len())
    }
}

impl LanguageModelProvider for ScriptedModelProvider {
    fn descriptor(&self) -> &ModelDescriptor {
        &self.descriptor
    }

    fn generate(&self, request: &ModelRequest) -> Result<ModelResponse, ModelError> {
        request.validate_for(&self.descriptor)?;
        let outcome = self
            .outcomes
            .lock()
            .map_err(|_| ModelError::new(ModelErrorKind::Internal))?
            .pop_front();
        match outcome {
            Some(ScriptedOutcome::Response(response)) => {
                response.validate_for(request)?;
                Ok(response)
            }
            Some(ScriptedOutcome::Error(kind)) => Err(ModelError::new(kind)),
            None => Err(ModelError::new(ModelErrorKind::ScriptExhausted)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn provider_id(value: u128) -> ModelProviderId {
        ModelProviderId::new(Uuid::from_u128(value)).unwrap()
    }
    fn model_id(value: u128) -> ModelId {
        ModelId::new(Uuid::from_u128(value)).unwrap()
    }
    fn invocation_id(value: u128) -> ModelInvocationId {
        ModelInvocationId::new(Uuid::from_u128(value)).unwrap()
    }
    fn descriptor() -> ModelDescriptor {
        ModelDescriptor::new(
            provider_id(1),
            model_id(2),
            PrivacyClass::LocalOnly,
            ModelCapabilities {
                streaming: false,
                structured_output: true,
                tool_calling: false,
                vision: false,
                context_window_tokens: 4096,
                maximum_output_tokens: 512,
            },
        )
        .unwrap()
    }
    fn request(id: u128) -> ModelRequest {
        ModelRequest {
            invocation_id: invocation_id(id),
            provider_id: provider_id(1),
            model_id: model_id(2),
            contract_version: MODEL_INVOCATION_V1,
            input: ModelInput::new("private learner prompt").unwrap(),
            required_capabilities: RequiredCapabilities {
                structured_output: true,
                tool_calling: false,
                vision: false,
            },
            maximum_output_tokens: 100,
        }
    }
    fn response(id: u128, text: &str) -> ModelResponse {
        ModelResponse {
            invocation_id: invocation_id(id),
            provider_id: provider_id(1),
            model_id: model_id(2),
            contract_version: MODEL_INVOCATION_V1,
            output: RawModelOutput::new(text).unwrap(),
            finish_reason: FinishReason::Complete,
            reported_usage: Some(ModelUsage {
                input_tokens: 4,
                output_tokens: 3,
            }),
        }
    }

    #[test]
    fn scripted_provider_is_fifo_and_deterministic() {
        let provider = ScriptedModelProvider::new(
            descriptor(),
            [
                ScriptedOutcome::Response(response(10, "first secret output")),
                ScriptedOutcome::Error(ModelErrorKind::Unavailable),
            ],
        )
        .unwrap();
        assert_eq!(
            provider.generate(&request(10)).unwrap().output.as_str(),
            "first secret output"
        );
        assert_eq!(
            provider.generate(&request(11)).unwrap_err().kind,
            ModelErrorKind::Unavailable
        );
        assert_eq!(
            provider.generate(&request(12)).unwrap_err().kind,
            ModelErrorKind::ScriptExhausted
        );
    }

    #[test]
    fn invalid_request_does_not_consume_script() {
        let mut invalid = request(10);
        invalid.model_id = model_id(99);
        let provider = ScriptedModelProvider::new(
            descriptor(),
            [ScriptedOutcome::Response(response(10, "output"))],
        )
        .unwrap();
        assert_eq!(
            provider.generate(&invalid).unwrap_err().kind,
            ModelErrorKind::IdentityMismatch
        );
        assert_eq!(provider.remaining(), 1);
        assert!(provider.generate(&request(10)).is_ok());
    }

    #[test]
    fn content_is_redacted_and_wire_rejects_unknown_fields() {
        let request = request(10);
        let response = response(10, "private output");
        assert!(!format!("{request:?}").contains("private learner prompt"));
        assert!(!format!("{response:?}").contains("private output"));

        let mut value = serde_json::to_value(&request).unwrap();
        value["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<ModelRequest>(value).is_err());
        let round_trip: ModelResponse =
            serde_json::from_str(&serde_json::to_string(&response).unwrap()).unwrap();
        assert_eq!(round_trip, response);
    }

    #[test]
    fn unsupported_capability_fails_closed() {
        let mut request = request(10);
        request.required_capabilities.tool_calling = true;
        assert_eq!(
            request.validate_for(&descriptor()).unwrap_err().kind,
            ModelErrorKind::UnsupportedCapability
        );
    }

    #[test]
    fn capabilities_wire_enforces_all_v1_intrinsic_invariants() {
        let valid = serde_json::json!({
            "streaming": false,
            "structured_output": false,
            "tool_calling": false,
            "vision": false,
            "context_window_tokens": 8,
            "maximum_output_tokens": 4
        });
        assert!(serde_json::from_value::<ModelCapabilities>(valid.clone()).is_ok());

        for invalid in [
            serde_json::json!({ "streaming": true }),
            serde_json::json!({ "context_window_tokens": 0 }),
            serde_json::json!({ "maximum_output_tokens": 0 }),
            serde_json::json!({ "context_window_tokens": 3, "maximum_output_tokens": 4 }),
        ] {
            let mut wire = valid.clone();
            for (key, value) in invalid.as_object().unwrap() {
                wire[key] = value.clone();
            }
            assert!(serde_json::from_value::<ModelCapabilities>(wire).is_err());
        }
    }

    #[test]
    fn over_context_request_does_not_consume_scripted_work() {
        let mut small_descriptor = descriptor();
        small_descriptor.capabilities.context_window_tokens = 20;
        small_descriptor.capabilities.maximum_output_tokens = 10;
        let provider = ScriptedModelProvider::new(
            small_descriptor,
            [ScriptedOutcome::Response(response(10, "output"))],
        )
        .unwrap();
        let mut over_context = request(10);
        over_context.input = ModelInput::new("eleven bytes").unwrap();
        over_context.maximum_output_tokens = 10;

        assert_eq!(
            provider.generate(&over_context).unwrap_err().kind,
            ModelErrorKind::ContextTooLarge
        );
        assert_eq!(provider.remaining(), 1);
    }

    #[test]
    fn provider_port_supports_shared_send_sync_trait_objects() {
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn LanguageModelProvider>();

        let provider: std::sync::Arc<dyn LanguageModelProvider> = std::sync::Arc::new(
            ScriptedModelProvider::new(descriptor(), std::iter::empty()).unwrap(),
        );
        let _shared = std::sync::Arc::clone(&provider);
    }

    #[test]
    fn identifiers_reject_nil_and_round_trip() {
        assert!(ModelProviderId::new(Uuid::nil()).is_err());
        let encoded = serde_json::to_string(&model_id(5)).unwrap();
        assert_eq!(
            serde_json::from_str::<ModelId>(&encoded).unwrap(),
            model_id(5)
        );
    }
}
