//! Provider-neutral model-input token-counting contracts and scripted test infrastructure.

use crate::model::{ModelDescriptor, ModelInput, MAX_MODEL_INPUT_BYTES};
use nexa_domain::{ModelId, ModelProviderId, ProtocolVersion};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::VecDeque, sync::Mutex};
use thiserror::Error;

pub const MODEL_INPUT_TOKENIZATION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ModelInputTokenizationError {
    #[error("unsupported model-input tokenization version")]
    UnsupportedVersion,
    #[error("invalid model descriptor or association")]
    InvalidDescriptor,
    #[error("invalid model-input tokenization evidence")]
    InvalidEvidence,
    #[error("model-input tokenizer failed")]
    TokenizerFailure,
    #[error("scripted model-input tokenizer exhausted")]
    ScriptExhausted,
    #[error("model-input tokenizer internal failure")]
    Internal,
}

pub trait ModelInputTokenizer: Send + Sync {
    fn descriptor(&self) -> &ModelDescriptor;
    fn count_input_tokens(&self, input: &ModelInput) -> Result<u32, ModelInputTokenizationError>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelInputTokenizationEvidence {
    pub contract_version: ProtocolVersion,
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
    pub descriptor_contract_version: ProtocolVersion,
    pub input_byte_count: u32,
    pub input_sha256: String,
    pub input_token_count: u32,
    pub replay_anchor: String,
}

impl ModelInputTokenizationEvidence {
    fn new(
        descriptor: &ModelDescriptor,
        input: &ModelInput,
        input_token_count: u32,
    ) -> Result<Self, ModelInputTokenizationError> {
        let input_byte_count = u32::try_from(input.as_str().len())
            .map_err(|_| ModelInputTokenizationError::InvalidEvidence)?;
        let mut evidence = Self {
            contract_version: MODEL_INPUT_TOKENIZATION_V1,
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            descriptor_contract_version: descriptor.contract_version,
            input_byte_count,
            input_sha256: hex_hash(input.as_str().as_bytes()),
            input_token_count,
            replay_anchor: String::new(),
        };
        if input_token_count == 0 || input_byte_count == 0 {
            return Err(ModelInputTokenizationError::InvalidEvidence);
        }
        evidence.replay_anchor = evidence.compute_anchor()?;
        evidence.validate_fields()?;
        Ok(evidence)
    }

    fn compute_anchor(&self) -> Result<String, ModelInputTokenizationError> {
        #[derive(Serialize)]
        struct GovernedFields<'a> {
            contract_version: ProtocolVersion,
            provider_id: ModelProviderId,
            model_id: ModelId,
            descriptor_contract_version: ProtocolVersion,
            input_byte_count: u32,
            input_sha256: &'a str,
            input_token_count: u32,
        }
        let bytes = serde_json::to_vec(&GovernedFields {
            contract_version: self.contract_version,
            provider_id: self.provider_id,
            model_id: self.model_id,
            descriptor_contract_version: self.descriptor_contract_version,
            input_byte_count: self.input_byte_count,
            input_sha256: &self.input_sha256,
            input_token_count: self.input_token_count,
        })
        .map_err(|_| ModelInputTokenizationError::Internal)?;
        Ok(hex_hash(&bytes))
    }

    fn validate_fields(&self) -> Result<(), ModelInputTokenizationError> {
        if self.contract_version != MODEL_INPUT_TOKENIZATION_V1 {
            return Err(ModelInputTokenizationError::UnsupportedVersion);
        }
        if self.descriptor_contract_version != crate::model::MODEL_INVOCATION_V1
            || self.input_byte_count == 0
            || usize::try_from(self.input_byte_count)
                .map_or(true, |count| count > MAX_MODEL_INPUT_BYTES)
            || self.input_token_count == 0
            || !valid_hash(&self.input_sha256)
            || !valid_hash(&self.replay_anchor)
        {
            return Err(ModelInputTokenizationError::InvalidEvidence);
        }
        if self.compute_anchor()? != self.replay_anchor {
            return Err(ModelInputTokenizationError::InvalidEvidence);
        }
        Ok(())
    }

    pub fn validate_for(
        &self,
        descriptor: &ModelDescriptor,
        input: &ModelInput,
    ) -> Result<(), ModelInputTokenizationError> {
        self.validate_fields()?;
        descriptor
            .validate()
            .map_err(|_| ModelInputTokenizationError::InvalidDescriptor)?;
        let byte_count = u32::try_from(input.as_str().len())
            .map_err(|_| ModelInputTokenizationError::InvalidEvidence)?;
        if self.provider_id != descriptor.provider_id
            || self.model_id != descriptor.model_id
            || self.descriptor_contract_version != descriptor.contract_version
            || self.input_byte_count != byte_count
            || self.input_sha256 != hex_hash(input.as_str().as_bytes())
        {
            return Err(ModelInputTokenizationError::InvalidDescriptor);
        }
        Ok(())
    }
}

impl<'de> Deserialize<'de> for ModelInputTokenizationEvidence {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            provider_id: ModelProviderId,
            model_id: ModelId,
            descriptor_contract_version: ProtocolVersion,
            input_byte_count: u32,
            input_sha256: String,
            input_token_count: u32,
            replay_anchor: String,
        }
        let wire = Wire::deserialize(deserializer)?;
        let evidence = Self {
            contract_version: wire.contract_version,
            provider_id: wire.provider_id,
            model_id: wire.model_id,
            descriptor_contract_version: wire.descriptor_contract_version,
            input_byte_count: wire.input_byte_count,
            input_sha256: wire.input_sha256,
            input_token_count: wire.input_token_count,
            replay_anchor: wire.replay_anchor,
        };
        evidence
            .validate_fields()
            .map_err(serde::de::Error::custom)?;
        Ok(evidence)
    }
}

pub fn tokenize_model_input(
    contract_version: ProtocolVersion,
    descriptor: &ModelDescriptor,
    input: &ModelInput,
    tokenizer: &dyn ModelInputTokenizer,
) -> Result<ModelInputTokenizationEvidence, ModelInputTokenizationError> {
    if contract_version != MODEL_INPUT_TOKENIZATION_V1 {
        return Err(ModelInputTokenizationError::UnsupportedVersion);
    }
    descriptor
        .validate()
        .map_err(|_| ModelInputTokenizationError::InvalidDescriptor)?;
    tokenizer
        .descriptor()
        .validate()
        .map_err(|_| ModelInputTokenizationError::InvalidDescriptor)?;
    if tokenizer.descriptor() != descriptor {
        return Err(ModelInputTokenizationError::InvalidDescriptor);
    }
    let count = tokenizer.count_input_tokens(input)?;
    if count == 0 {
        return Err(ModelInputTokenizationError::InvalidEvidence);
    }
    ModelInputTokenizationEvidence::new(descriptor, input, count)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScriptedTokenizationOutcome {
    TokenCount(u32),
    Error,
}

#[derive(Debug)]
pub struct ScriptedModelInputTokenizer {
    descriptor: ModelDescriptor,
    outcomes: Mutex<VecDeque<ScriptedTokenizationOutcome>>,
}

impl ScriptedModelInputTokenizer {
    pub fn new(
        descriptor: ModelDescriptor,
        outcomes: impl IntoIterator<Item = ScriptedTokenizationOutcome>,
    ) -> Result<Self, ModelInputTokenizationError> {
        descriptor
            .validate()
            .map_err(|_| ModelInputTokenizationError::InvalidDescriptor)?;
        Ok(Self {
            descriptor,
            outcomes: Mutex::new(outcomes.into_iter().collect()),
        })
    }

    pub fn remaining(&self) -> Result<usize, ModelInputTokenizationError> {
        self.outcomes
            .lock()
            .map(|outcomes| outcomes.len())
            .map_err(|_| ModelInputTokenizationError::Internal)
    }
}

impl ModelInputTokenizer for ScriptedModelInputTokenizer {
    fn descriptor(&self) -> &ModelDescriptor {
        &self.descriptor
    }

    fn count_input_tokens(&self, _input: &ModelInput) -> Result<u32, ModelInputTokenizationError> {
        match self
            .outcomes
            .lock()
            .map_err(|_| ModelInputTokenizationError::Internal)?
            .pop_front()
        {
            Some(ScriptedTokenizationOutcome::TokenCount(count)) => Ok(count),
            Some(ScriptedTokenizationOutcome::Error) => {
                Err(ModelInputTokenizationError::TokenizerFailure)
            }
            None => Err(ModelInputTokenizationError::ScriptExhausted),
        }
    }
}

fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{ModelCapabilities, PrivacyClass};
    use serde_json::json;
    use uuid::Uuid;

    fn descriptor(provider: u128, model: u128) -> ModelDescriptor {
        ModelDescriptor::new(
            ModelProviderId::new(Uuid::from_u128(provider)).unwrap(),
            ModelId::new(Uuid::from_u128(model)).unwrap(),
            PrivacyClass::LocalOnly,
            ModelCapabilities {
                streaming: false,
                structured_output: false,
                tool_calling: false,
                vision: false,
                context_window_tokens: 4096,
                maximum_output_tokens: 512,
            },
        )
        .unwrap()
    }

    fn run(input: &str, count: u32) -> ModelInputTokenizationEvidence {
        let bound_descriptor = descriptor(1, 2);
        let tokenizer = ScriptedModelInputTokenizer::new(
            bound_descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(count)],
        )
        .unwrap();
        tokenize_model_input(
            MODEL_INPUT_TOKENIZATION_V1,
            &bound_descriptor,
            &ModelInput::new(input).unwrap(),
            &tokenizer,
        )
        .unwrap()
    }

    #[test]
    fn model_input_tokenization_counts_ascii_multibyte_and_boundaries() {
        assert_eq!(run("ascii", 1).input_token_count, 1);
        let multibyte = run("é猫", u32::MAX);
        assert_eq!(multibyte.input_byte_count, 5);
        assert_eq!(multibyte.input_token_count, u32::MAX);
    }

    #[test]
    fn model_input_tokenization_evidence_is_exact_and_deterministic() {
        let first = run("abc", 7);
        let second = run("abc", 7);
        assert_eq!(first, second);
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
        assert_eq!(first.contract_version, MODEL_INPUT_TOKENIZATION_V1);
        assert_eq!(
            first.descriptor_contract_version,
            crate::model::MODEL_INVOCATION_V1
        );
        assert_eq!(
            first.input_sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_ne!(first.input_sha256, run("abd", 7).input_sha256);
        assert_ne!(first.replay_anchor, run("abc", 8).replay_anchor);
    }

    #[test]
    fn model_input_tokenization_standalone_wire_validation_rejects_tampering() {
        let evidence = run("wire", 4);
        let value = serde_json::to_value(&evidence).unwrap();
        assert_eq!(
            serde_json::from_value::<ModelInputTokenizationEvidence>(value.clone()).unwrap(),
            evidence
        );
        let mutations = [
            ("contract_version", json!("2.0")),
            ("provider_id", json!(Uuid::from_u128(9))),
            ("model_id", json!(Uuid::from_u128(9))),
            ("descriptor_contract_version", json!("1.1")),
            ("input_byte_count", json!(5)),
            ("input_token_count", json!(5)),
            ("input_sha256", json!("a".repeat(64))),
            ("replay_anchor", json!("b".repeat(64))),
        ];
        for (field, replacement) in mutations {
            let mut changed = value.clone();
            changed[field] = replacement;
            assert!(
                serde_json::from_value::<ModelInputTokenizationEvidence>(changed).is_err(),
                "{field}"
            );
        }
        for bad_hash in ["A".repeat(64), "a".repeat(63), "g".repeat(64)] {
            let mut changed = value.clone();
            changed["input_sha256"] = json!(bad_hash);
            assert!(serde_json::from_value::<ModelInputTokenizationEvidence>(changed).is_err());
        }
        let mut unknown = value;
        unknown["prompt"] = json!("secret");
        assert!(serde_json::from_value::<ModelInputTokenizationEvidence>(unknown).is_err());
    }

    #[test]
    fn model_input_tokenization_validate_for_requires_exact_association() {
        let evidence = run("same", 2);
        let input = ModelInput::new("same").unwrap();
        assert!(evidence.validate_for(&descriptor(1, 2), &input).is_ok());
        assert!(evidence.validate_for(&descriptor(9, 2), &input).is_err());
        assert!(evidence.validate_for(&descriptor(1, 9), &input).is_err());
        assert!(evidence
            .validate_for(&descriptor(1, 2), &ModelInput::new("size").unwrap())
            .is_err());
        assert!(run("é", 2)
            .validate_for(&descriptor(1, 2), &ModelInput::new("aa").unwrap())
            .is_err());
        let mut version = descriptor(1, 2);
        version.contract_version = ProtocolVersion::new(1, 1);
        assert!(evidence.validate_for(&version, &input).is_err());
    }

    #[test]
    fn model_input_tokenization_script_is_fifo_exactly_once_and_preflight_safe() {
        let bound_descriptor = descriptor(1, 2);
        let tokenizer = ScriptedModelInputTokenizer::new(
            bound_descriptor.clone(),
            [
                ScriptedTokenizationOutcome::TokenCount(3),
                ScriptedTokenizationOutcome::Error,
                ScriptedTokenizationOutcome::TokenCount(9),
            ],
        )
        .unwrap();
        let input = ModelInput::new("private").unwrap();
        assert_eq!(
            tokenize_model_input(
                ProtocolVersion::new(2, 0),
                &bound_descriptor,
                &input,
                &tokenizer
            )
            .unwrap_err(),
            ModelInputTokenizationError::UnsupportedVersion
        );
        assert_eq!(tokenizer.remaining().unwrap(), 3);
        assert_eq!(
            tokenize_model_input(
                MODEL_INPUT_TOKENIZATION_V1,
                &descriptor(9, 2),
                &input,
                &tokenizer
            )
            .unwrap_err(),
            ModelInputTokenizationError::InvalidDescriptor
        );
        assert_eq!(tokenizer.remaining().unwrap(), 3);
        assert_eq!(
            tokenize_model_input(
                MODEL_INPUT_TOKENIZATION_V1,
                &bound_descriptor,
                &input,
                &tokenizer
            )
            .unwrap()
            .input_token_count,
            3
        );
        assert_eq!(
            tokenize_model_input(
                MODEL_INPUT_TOKENIZATION_V1,
                &bound_descriptor,
                &input,
                &tokenizer
            )
            .unwrap_err(),
            ModelInputTokenizationError::TokenizerFailure
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(
            tokenize_model_input(
                MODEL_INPUT_TOKENIZATION_V1,
                &bound_descriptor,
                &input,
                &tokenizer
            )
            .unwrap()
            .input_token_count,
            9
        );
        assert_eq!(
            tokenize_model_input(
                MODEL_INPUT_TOKENIZATION_V1,
                &bound_descriptor,
                &input,
                &tokenizer
            )
            .unwrap_err(),
            ModelInputTokenizationError::ScriptExhausted
        );
    }

    #[test]
    fn model_input_tokenization_rejects_zero_and_keeps_diagnostics_content_free() {
        let descriptor = descriptor(1, 2);
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(0)],
        )
        .unwrap();
        let sentinel = "prompt-ENDPOINT-secret-CREDENTIAL-tokenizer-private";
        let error = tokenize_model_input(
            MODEL_INPUT_TOKENIZATION_V1,
            &descriptor,
            &ModelInput::new(sentinel).unwrap(),
            &tokenizer,
        )
        .unwrap_err();
        assert_eq!(error, ModelInputTokenizationError::InvalidEvidence);
        assert!(!format!("{error:?} {error}").contains(sentinel));
        let wire = serde_json::to_string(&run(sentinel, 2)).unwrap();
        assert!(!wire.contains(sentinel));
        fn assert_send_sync<T: Send + Sync + ?Sized>() {}
        assert_send_sync::<dyn ModelInputTokenizer>();
    }
}
