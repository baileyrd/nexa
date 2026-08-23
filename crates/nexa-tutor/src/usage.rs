//! Non-invoking reconciliation of optional reported usage with exact input evidence.

use crate::{
    model::{ModelDescriptor, ModelErrorKind, ModelRequest, ModelResponse},
    tokenization::{ModelInputTokenizationError, ModelInputTokenizationEvidence},
};
use thiserror::Error;

/// Closed, content-free failures from reported input-usage reconciliation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Error)]
pub enum ModelResponseReportedUsageValidationError {
    #[error("model request validation failed")]
    Request(ModelErrorKind),
    #[error("model response validation failed")]
    Response(ModelErrorKind),
    #[error("model-input tokenization evidence validation failed")]
    TokenizationEvidence(ModelInputTokenizationError),
    #[error("reported input-token count does not match tokenization evidence")]
    InputTokenCountMismatch,
}

/// Validates exact contract associations before comparing optional reported input usage.
///
/// Equality establishes agreement between the two supplied reports only. It does not establish
/// tokenizer correctness, provider truth, authenticity, freshness, billing accuracy, or output
/// token correctness.
pub fn validate_model_response_reported_usage(
    descriptor: &ModelDescriptor,
    request: &ModelRequest,
    response: &ModelResponse,
    tokenization_evidence: &ModelInputTokenizationEvidence,
) -> Result<(), ModelResponseReportedUsageValidationError> {
    request
        .validate_for(descriptor)
        .map_err(|error| ModelResponseReportedUsageValidationError::Request(error.kind))?;
    response
        .validate_for(request)
        .map_err(|error| ModelResponseReportedUsageValidationError::Response(error.kind))?;
    tokenization_evidence
        .validate_for(descriptor, &request.input)
        .map_err(ModelResponseReportedUsageValidationError::TokenizationEvidence)?;

    if response
        .reported_usage
        .as_ref()
        .is_some_and(|usage| usage.input_tokens != tokenization_evidence.input_token_count)
    {
        return Err(ModelResponseReportedUsageValidationError::InputTokenCountMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        model::{
            FinishReason, ModelCapabilities, ModelInput, ModelUsage, PrivacyClass, RawModelOutput,
            RequiredCapabilities, ScriptedModelProvider, ScriptedOutcome, MODEL_INVOCATION_V1,
        },
        tokenization::{
            tokenize_model_input, ScriptedModelInputTokenizer, ScriptedTokenizationOutcome,
            MODEL_INPUT_TOKENIZATION_V1,
        },
    };
    use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion};
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
                context_window_tokens: 100,
                maximum_output_tokens: 20,
            },
        )
        .unwrap()
    }

    fn contracts() -> (
        ModelDescriptor,
        ModelRequest,
        ModelResponse,
        ModelInputTokenizationEvidence,
    ) {
        let descriptor = descriptor(1, 2);
        let request = ModelRequest {
            invocation_id: ModelInvocationId::new(Uuid::from_u128(3)).unwrap(),
            provider_id: descriptor.provider_id,
            model_id: descriptor.model_id,
            contract_version: MODEL_INVOCATION_V1,
            input: ModelInput::new(
                "learner-context credential endpoint tokenizer-provider sentinel",
            )
            .unwrap(),
            required_capabilities: RequiredCapabilities {
                structured_output: false,
                tool_calling: false,
                vision: false,
            },
            maximum_output_tokens: 20,
        };
        let response = ModelResponse {
            invocation_id: request.invocation_id,
            provider_id: request.provider_id,
            model_id: request.model_id,
            contract_version: MODEL_INVOCATION_V1,
            output: RawModelOutput::new("private response output usage sentinel").unwrap(),
            finish_reason: FinishReason::Complete,
            reported_usage: Some(ModelUsage {
                input_tokens: 7,
                output_tokens: 20,
            }),
        };
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        let evidence = tokenize_model_input(
            MODEL_INPUT_TOKENIZATION_V1,
            &descriptor,
            &request.input,
            &tokenizer,
        )
        .unwrap();
        (descriptor, request, response, evidence)
    }

    #[test]
    fn request_failures_are_exact_and_take_precedence() {
        let (descriptor, request, response, evidence) = contracts();
        let mut invalid_descriptor = descriptor.clone();
        invalid_descriptor.capabilities.context_window_tokens = 0;
        assert_eq!(
            validate_model_response_reported_usage(
                &invalid_descriptor,
                &request,
                &response,
                &evidence
            ),
            Err(ModelResponseReportedUsageValidationError::Request(
                ModelErrorKind::InvalidContract
            ))
        );
        let cases = [
            (ModelErrorKind::UnsupportedVersion, {
                let mut v = request.clone();
                v.contract_version = ProtocolVersion::new(2, 0);
                v
            }),
            (ModelErrorKind::UnsupportedCapability, {
                let mut v = request.clone();
                v.required_capabilities.vision = true;
                v
            }),
        ];
        for (kind, invalid) in cases {
            let mut invalid_response = response.clone();
            invalid_response.contract_version = ProtocolVersion::new(2, 0);
            let mut invalid_evidence = evidence.clone();
            invalid_evidence.replay_anchor = "0".repeat(64);
            assert_eq!(
                validate_model_response_reported_usage(
                    &descriptor,
                    &invalid,
                    &invalid_response,
                    &invalid_evidence
                ),
                Err(ModelResponseReportedUsageValidationError::Request(kind))
            );
        }
    }

    #[test]
    fn response_failures_are_exact_and_precede_evidence() {
        let (descriptor, request, response, evidence) = contracts();
        let cases = [
            (ModelErrorKind::UnsupportedVersion, {
                let mut v = response.clone();
                v.contract_version = ProtocolVersion::new(2, 0);
                v
            }),
            (ModelErrorKind::IdentityMismatch, {
                let mut v = response.clone();
                v.invocation_id = ModelInvocationId::new(Uuid::from_u128(99)).unwrap();
                v
            }),
            (ModelErrorKind::IdentityMismatch, {
                let mut v = response.clone();
                v.provider_id = ModelProviderId::new(Uuid::from_u128(99)).unwrap();
                v
            }),
            (ModelErrorKind::IdentityMismatch, {
                let mut v = response.clone();
                v.model_id = ModelId::new(Uuid::from_u128(99)).unwrap();
                v
            }),
            (ModelErrorKind::InvalidResponse, {
                let mut v = response.clone();
                v.reported_usage.as_mut().unwrap().output_tokens = 21;
                v
            }),
        ];
        for (kind, invalid) in cases {
            let mut invalid_evidence = evidence.clone();
            invalid_evidence.replay_anchor = "0".repeat(64);
            assert_eq!(
                validate_model_response_reported_usage(
                    &descriptor,
                    &request,
                    &invalid,
                    &invalid_evidence
                ),
                Err(ModelResponseReportedUsageValidationError::Response(kind))
            );
        }
    }

    #[test]
    fn evidence_failures_preserve_exact_categories() {
        let (descriptor, request, response, evidence) = contracts();
        let mut unsupported = evidence.clone();
        unsupported.contract_version = ProtocolVersion::new(2, 0);
        let mut malformed = evidence.clone();
        malformed.replay_anchor = "tampered private credential".into();
        let other_descriptor = self::descriptor(9, 10);
        let other_tokenizer = ScriptedModelInputTokenizer::new(
            other_descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(7)],
        )
        .unwrap();
        let descriptor_reassociated = tokenize_model_input(
            MODEL_INPUT_TOKENIZATION_V1,
            &other_descriptor,
            &request.input,
            &other_tokenizer,
        )
        .unwrap();
        let other_input =
            ModelInput::new("other same-length private learner sentinel text!!!!!!!!!!!!").unwrap();
        let mut input_request = request.clone();
        input_request.input = other_input;
        for (expected, supplied_request, supplied_evidence) in [
            (
                ModelInputTokenizationError::UnsupportedVersion,
                &request,
                &unsupported,
            ),
            (
                ModelInputTokenizationError::InvalidEvidence,
                &request,
                &malformed,
            ),
            (
                ModelInputTokenizationError::InvalidDescriptor,
                &request,
                &descriptor_reassociated,
            ),
            (
                ModelInputTokenizationError::InvalidDescriptor,
                &input_request,
                &evidence,
            ),
        ] {
            assert_eq!(
                validate_model_response_reported_usage(
                    &descriptor,
                    supplied_request,
                    &response,
                    supplied_evidence
                ),
                Err(ModelResponseReportedUsageValidationError::TokenizationEvidence(expected))
            );
        }
    }

    #[test]
    fn optional_and_equal_usage_succeed_but_lower_or_higher_counts_fail() {
        let (descriptor, request, response, evidence) = contracts();
        let mut absent = response.clone();
        absent.reported_usage = None;
        assert_eq!(
            validate_model_response_reported_usage(&descriptor, &request, &absent, &evidence),
            Ok(())
        );
        assert_eq!(
            validate_model_response_reported_usage(&descriptor, &request, &response, &evidence),
            Ok(())
        );
        for count in [6, 8] {
            let mut mismatch = response.clone();
            mismatch.reported_usage.as_mut().unwrap().input_tokens = count;
            assert_eq!(
                validate_model_response_reported_usage(&descriptor, &request, &mismatch, &evidence),
                Err(ModelResponseReportedUsageValidationError::InputTokenCountMismatch)
            );
        }
    }

    #[test]
    fn validation_consumes_no_scripted_dependency_and_errors_leak_no_content() {
        let (descriptor, request, response, evidence) = contracts();
        let tokenizer = ScriptedModelInputTokenizer::new(
            descriptor.clone(),
            [ScriptedTokenizationOutcome::TokenCount(99)],
        )
        .unwrap();
        let provider = ScriptedModelProvider::new(
            descriptor.clone(),
            [ScriptedOutcome::Response(response.clone())],
        )
        .unwrap();
        assert_eq!(
            validate_model_response_reported_usage(&descriptor, &request, &response, &evidence),
            Ok(())
        );
        assert_eq!(tokenizer.remaining().unwrap(), 1);
        assert_eq!(provider.remaining(), 1);
        let errors = [
            ModelResponseReportedUsageValidationError::Request(ModelErrorKind::Internal),
            ModelResponseReportedUsageValidationError::Response(ModelErrorKind::InvalidResponse),
            ModelResponseReportedUsageValidationError::TokenizationEvidence(
                ModelInputTokenizationError::TokenizerFailure,
            ),
            ModelResponseReportedUsageValidationError::InputTokenCountMismatch,
        ];
        for error in errors {
            let rendered = format!("{error:?} {error}");
            for sentinel in [
                "learner-context",
                "credential",
                "endpoint",
                "tokenizer-provider",
                "private response",
                "usage sentinel",
            ] {
                assert!(!rendered.contains(sentinel));
            }
        }
    }
}
