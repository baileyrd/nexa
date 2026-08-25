//! Provider-neutral control contracts for requesting cancellation of one model invocation.

use nexa_domain::{ModelId, ModelInvocationId, ModelProviderId, ProtocolVersion};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::VecDeque;
use thiserror::Error;

/// The only supported tutor-generation cancellation-control contract version.
pub const TUTOR_GENERATION_CANCELLATION_V1: ProtocolVersion = ProtocolVersion::new(1, 0);

/// An exact, content-free request to cancel one model invocation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TutorGenerationCancellationRequest {
    pub contract_version: ProtocolVersion,
    pub invocation_id: ModelInvocationId,
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
}

impl TutorGenerationCancellationRequest {
    pub const fn new(
        invocation_id: ModelInvocationId,
        provider_id: ModelProviderId,
        model_id: ModelId,
    ) -> Self {
        Self {
            contract_version: TUTOR_GENERATION_CANCELLATION_V1,
            invocation_id,
            provider_id,
            model_id,
        }
    }

    fn validate(&self) -> Result<(), TutorGenerationCancellationError> {
        if self.contract_version != TUTOR_GENERATION_CANCELLATION_V1 {
            Err(TutorGenerationCancellationError::UnsupportedVersion)
        } else {
            Ok(())
        }
    }
}

impl<'de> Deserialize<'de> for TutorGenerationCancellationRequest {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            invocation_id: ModelInvocationId,
            provider_id: ModelProviderId,
            model_id: ModelId,
        }
        let wire = Wire::deserialize(deserializer)?;
        let request = Self {
            contract_version: wire.contract_version,
            invocation_id: wire.invocation_id,
            provider_id: wire.provider_id,
            model_id: wire.model_id,
        };
        request.validate().map_err(serde::de::Error::custom)?;
        Ok(request)
    }
}

/// Evidence that the dependency accepted the exact control request.
///
/// Acceptance does not prove that generation stopped, joined, or emitted no later output.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TutorGenerationCancellationAcknowledgement {
    pub contract_version: ProtocolVersion,
    pub invocation_id: ModelInvocationId,
    pub provider_id: ModelProviderId,
    pub model_id: ModelId,
}

impl TutorGenerationCancellationAcknowledgement {
    pub const fn for_request(request: &TutorGenerationCancellationRequest) -> Self {
        Self {
            contract_version: TUTOR_GENERATION_CANCELLATION_V1,
            invocation_id: request.invocation_id,
            provider_id: request.provider_id,
            model_id: request.model_id,
        }
    }
}

impl<'de> Deserialize<'de> for TutorGenerationCancellationAcknowledgement {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            contract_version: ProtocolVersion,
            invocation_id: ModelInvocationId,
            provider_id: ModelProviderId,
            model_id: ModelId,
        }
        let wire = Wire::deserialize(deserializer)?;
        if wire.contract_version != TUTOR_GENERATION_CANCELLATION_V1 {
            return Err(serde::de::Error::custom(
                "unsupported tutor-generation cancellation version",
            ));
        }
        Ok(Self {
            contract_version: wire.contract_version,
            invocation_id: wire.invocation_id,
            provider_id: wire.provider_id,
            model_id: wire.model_id,
        })
    }
}

/// Closed failure from a caller-supplied cancellation-control dependency.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("tutor-generation cancellation dependency failed")]
pub struct TutorGenerationCancellationDependencyError;

/// Closed, content-free host-operation failure categories.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum TutorGenerationCancellationError {
    #[error("unsupported tutor-generation cancellation version")]
    UnsupportedVersion,
    #[error("tutor-generation cancellation identity association mismatch")]
    AssociationMismatch,
    #[error("tutor-generation cancellation dependency failed")]
    DependencyFailure,
    #[error("tutor-generation cancellation acknowledgement mismatch")]
    AcknowledgementMismatch,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ErrorWire {
    contract_version: ProtocolVersion,
    kind: String,
}

impl Serialize for TutorGenerationCancellationError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let kind = match self {
            Self::UnsupportedVersion => "unsupported_version",
            Self::AssociationMismatch => "association_mismatch",
            Self::DependencyFailure => "dependency_failure",
            Self::AcknowledgementMismatch => "acknowledgement_mismatch",
        };
        ErrorWire {
            contract_version: TUTOR_GENERATION_CANCELLATION_V1,
            kind: kind.into(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for TutorGenerationCancellationError {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = ErrorWire::deserialize(deserializer)?;
        if wire.contract_version != TUTOR_GENERATION_CANCELLATION_V1 {
            return Err(serde::de::Error::custom(
                "unsupported tutor-generation cancellation version",
            ));
        }
        match wire.kind.as_str() {
            "unsupported_version" => Ok(Self::UnsupportedVersion),
            "association_mismatch" => Ok(Self::AssociationMismatch),
            "dependency_failure" => Ok(Self::DependencyFailure),
            "acknowledgement_mismatch" => Ok(Self::AcknowledgementMismatch),
            _ => Err(serde::de::Error::custom(
                "unknown tutor-generation cancellation error",
            )),
        }
    }
}

/// Synchronous provider-neutral control plane for one exact invocation.
pub trait TutorGenerationCancellationPort {
    fn request_cancellation(
        &mut self,
        request: &TutorGenerationCancellationRequest,
    ) -> Result<
        TutorGenerationCancellationAcknowledgement,
        TutorGenerationCancellationDependencyError,
    >;
}

/// Preflights exact host association, invokes the port once, and validates its acknowledgement.
pub fn request_tutor_generation_cancellation(
    port: &mut impl TutorGenerationCancellationPort,
    request: &TutorGenerationCancellationRequest,
    invocation_id: ModelInvocationId,
    provider_id: ModelProviderId,
    model_id: ModelId,
) -> Result<TutorGenerationCancellationAcknowledgement, TutorGenerationCancellationError> {
    request.validate()?;
    if (request.invocation_id, request.provider_id, request.model_id)
        != (invocation_id, provider_id, model_id)
    {
        return Err(TutorGenerationCancellationError::AssociationMismatch);
    }
    let acknowledgement = port
        .request_cancellation(request)
        .map_err(|_| TutorGenerationCancellationError::DependencyFailure)?;
    if acknowledgement.contract_version != TUTOR_GENERATION_CANCELLATION_V1
        || (
            acknowledgement.invocation_id,
            acknowledgement.provider_id,
            acknowledgement.model_id,
        ) != (invocation_id, provider_id, model_id)
    {
        return Err(TutorGenerationCancellationError::AcknowledgementMismatch);
    }
    Ok(acknowledgement)
}

/// One deterministic scripted dependency result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptedTutorGenerationCancellationOutcome {
    Acknowledged(TutorGenerationCancellationAcknowledgement),
    DependencyFailure,
}

/// Deterministic FIFO adapter exposing bounded, read-only test evidence.
#[derive(Clone, Debug)]
pub struct ScriptedTutorGenerationCancellationPort {
    outcomes: VecDeque<ScriptedTutorGenerationCancellationOutcome>,
    consumed: usize,
    received_requests: Vec<TutorGenerationCancellationRequest>,
}

impl ScriptedTutorGenerationCancellationPort {
    pub fn new(
        outcomes: impl IntoIterator<Item = ScriptedTutorGenerationCancellationOutcome>,
    ) -> Self {
        Self {
            outcomes: outcomes.into_iter().collect(),
            consumed: 0,
            received_requests: Vec::new(),
        }
    }
    pub const fn consumed_outcomes(&self) -> usize {
        self.consumed
    }
    pub fn remaining_outcomes(&self) -> usize {
        self.outcomes.len()
    }
    pub fn received_requests(&self) -> &[TutorGenerationCancellationRequest] {
        &self.received_requests
    }
}

impl TutorGenerationCancellationPort for ScriptedTutorGenerationCancellationPort {
    fn request_cancellation(
        &mut self,
        request: &TutorGenerationCancellationRequest,
    ) -> Result<
        TutorGenerationCancellationAcknowledgement,
        TutorGenerationCancellationDependencyError,
    > {
        self.received_requests.push(request.clone());
        let Some(outcome) = self.outcomes.pop_front() else {
            return Err(TutorGenerationCancellationDependencyError);
        };
        self.consumed += 1;
        match outcome {
            ScriptedTutorGenerationCancellationOutcome::Acknowledged(value) => Ok(value),
            ScriptedTutorGenerationCancellationOutcome::DependencyFailure => {
                Err(TutorGenerationCancellationDependencyError)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        LanguageModelProvider, ModelCapabilities, ModelDescriptor, ModelErrorKind,
        ScriptedModelProvider, ScriptedOutcome,
    };
    use serde_json::{json, Value};
    use uuid::Uuid;

    fn id<T>(n: u128, make: impl FnOnce(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
        make(Uuid::from_u128(n)).unwrap()
    }
    fn request(n: u128) -> TutorGenerationCancellationRequest {
        TutorGenerationCancellationRequest::new(
            id(n, ModelInvocationId::new),
            id(n + 100, ModelProviderId::new),
            id(n + 200, ModelId::new),
        )
    }
    fn call(
        port: &mut ScriptedTutorGenerationCancellationPort,
        request: &TutorGenerationCancellationRequest,
    ) -> Result<TutorGenerationCancellationAcknowledgement, TutorGenerationCancellationError> {
        request_tutor_generation_cancellation(
            port,
            request,
            request.invocation_id,
            request.provider_id,
            request.model_id,
        )
    }

    #[test]
    fn exact_v1_json_and_validating_round_trips() {
        let request = request(1);
        let acknowledgement = TutorGenerationCancellationAcknowledgement::for_request(&request);
        let exact = r#"{"contract_version":"1.0","invocation_id":"00000000-0000-0000-0000-000000000001","provider_id":"00000000-0000-0000-0000-000000000065","model_id":"00000000-0000-0000-0000-0000000000c9"}"#;
        assert_eq!(serde_json::to_string(&request).unwrap(), exact);
        assert_eq!(serde_json::to_string(&acknowledgement).unwrap(), exact);
        assert_eq!(
            serde_json::from_str::<TutorGenerationCancellationRequest>(exact).unwrap(),
            request
        );
        assert_eq!(
            serde_json::from_str::<TutorGenerationCancellationAcknowledgement>(exact).unwrap(),
            acknowledgement
        );
        for (error, error_json) in [
            (
                TutorGenerationCancellationError::UnsupportedVersion,
                r#"{"contract_version":"1.0","kind":"unsupported_version"}"#,
            ),
            (
                TutorGenerationCancellationError::AssociationMismatch,
                r#"{"contract_version":"1.0","kind":"association_mismatch"}"#,
            ),
            (
                TutorGenerationCancellationError::DependencyFailure,
                r#"{"contract_version":"1.0","kind":"dependency_failure"}"#,
            ),
            (
                TutorGenerationCancellationError::AcknowledgementMismatch,
                r#"{"contract_version":"1.0","kind":"acknowledgement_mismatch"}"#,
            ),
        ] {
            assert_eq!(serde_json::to_string(&error).unwrap(), error_json);
            assert_eq!(
                serde_json::from_str::<TutorGenerationCancellationError>(error_json).unwrap(),
                error
            );
        }
    }

    #[test]
    fn cancellation_control_does_not_invoke_or_mutate_generation() {
        let request = request(1);
        let acknowledgement = TutorGenerationCancellationAcknowledgement::for_request(&request);
        let provider = ScriptedModelProvider::new(
            ModelDescriptor::new(
                request.provider_id,
                request.model_id,
                crate::model::PrivacyClass::LocalOnly,
                ModelCapabilities {
                    streaming: false,
                    structured_output: true,
                    tool_calling: false,
                    vision: false,
                    context_window_tokens: 4_096,
                    maximum_output_tokens: 512,
                },
            )
            .unwrap(),
            [ScriptedOutcome::Error(ModelErrorKind::Internal)],
        )
        .unwrap();
        let generation_provider: &dyn LanguageModelProvider = &provider;
        let generation_state_before = provider.remaining();
        let mut cancellation_port = ScriptedTutorGenerationCancellationPort::new([
            ScriptedTutorGenerationCancellationOutcome::Acknowledged(acknowledgement.clone()),
        ]);

        assert_eq!(
            call(&mut cancellation_port, &request).unwrap(),
            acknowledgement
        );
        assert_eq!(provider.remaining(), generation_state_before);
        assert_eq!(provider.remaining(), 1);
        assert_eq!(
            generation_provider.descriptor().provider_id,
            request.provider_id
        );
    }

    #[test]
    fn wire_rejects_unknown_fields_versions_nil_malformed_ids_and_invalid_errors() {
        let original = serde_json::to_value(request(1)).unwrap();
        for field in ["invocation_id", "provider_id", "model_id"] {
            for invalid in [json!(Uuid::nil()), json!("malformed"), json!(7)] {
                let mut value = original.clone();
                value[field] = invalid;
                assert!(
                    serde_json::from_value::<TutorGenerationCancellationRequest>(value.clone())
                        .is_err()
                );
                assert!(
                    serde_json::from_value::<TutorGenerationCancellationAcknowledgement>(value)
                        .is_err()
                );
            }
        }
        for mutation in [
            json!({"contract_version":"2.0","invocation_id":"00000000-0000-0000-0000-000000000001","provider_id":"00000000-0000-0000-0000-000000000065","model_id":"00000000-0000-0000-0000-0000000000c9"}),
            json!({"contract_version":"1.0","invocation_id":"00000000-0000-0000-0000-000000000001","provider_id":"00000000-0000-0000-0000-000000000065","model_id":"00000000-0000-0000-0000-0000000000c9","extra":false}),
        ] {
            assert!(
                serde_json::from_value::<TutorGenerationCancellationRequest>(mutation.clone())
                    .is_err()
            );
            assert!(
                serde_json::from_value::<TutorGenerationCancellationAcknowledgement>(mutation)
                    .is_err()
            );
        }
        for invalid in [
            json!({"contract_version":"2.0","kind":"dependency_failure"}),
            json!({"contract_version":"1.0","kind":"provider_secret"}),
            json!({"contract_version":"1.0","kind":"dependency_failure","extra":1}),
        ] {
            assert!(serde_json::from_value::<TutorGenerationCancellationError>(invalid).is_err());
        }
    }

    #[test]
    fn every_preflight_failure_is_side_effect_free() {
        let valid = request(1);
        let ack = TutorGenerationCancellationAcknowledgement::for_request(&valid);
        for mode in 0..4 {
            let mut changed = valid.clone();
            let mut invocation = valid.invocation_id;
            let mut provider = valid.provider_id;
            let mut model = valid.model_id;
            match mode {
                0 => changed.contract_version = ProtocolVersion::new(2, 0),
                1 => invocation = id(9, ModelInvocationId::new),
                2 => provider = id(9, ModelProviderId::new),
                _ => model = id(9, ModelId::new),
            }
            let mut port = ScriptedTutorGenerationCancellationPort::new([
                ScriptedTutorGenerationCancellationOutcome::Acknowledged(ack.clone()),
            ]);
            assert!(request_tutor_generation_cancellation(
                &mut port, &changed, invocation, provider, model
            )
            .is_err());
            assert_eq!(port.consumed_outcomes(), 0);
            assert_eq!(port.remaining_outcomes(), 1);
            assert!(port.received_requests().is_empty());
        }
    }

    #[test]
    fn exact_request_fifo_failures_exhaustion_and_accounting() {
        let first = request(1);
        let second = request(2);
        let first_ack = TutorGenerationCancellationAcknowledgement::for_request(&first);
        let second_ack = TutorGenerationCancellationAcknowledgement::for_request(&second);
        let mut port = ScriptedTutorGenerationCancellationPort::new([
            ScriptedTutorGenerationCancellationOutcome::Acknowledged(first_ack.clone()),
            ScriptedTutorGenerationCancellationOutcome::DependencyFailure,
            ScriptedTutorGenerationCancellationOutcome::Acknowledged(second_ack.clone()),
        ]);
        assert_eq!(call(&mut port, &first).unwrap(), first_ack);
        assert_eq!(
            call(&mut port, &second),
            Err(TutorGenerationCancellationError::DependencyFailure)
        );
        assert_eq!(call(&mut port, &second).unwrap(), second_ack);
        assert_eq!(
            call(&mut port, &second),
            Err(TutorGenerationCancellationError::DependencyFailure)
        );
        assert_eq!(
            port.received_requests(),
            &[first, second.clone(), second.clone(), second]
        );
        assert_eq!(port.consumed_outcomes(), 3);
        assert_eq!(port.remaining_outcomes(), 0);
    }

    #[test]
    fn acknowledgement_reassociation_fails_independently_after_one_call() {
        let request = request(1);
        for mode in 0..4 {
            let mut acknowledgement =
                TutorGenerationCancellationAcknowledgement::for_request(&request);
            match mode {
                0 => acknowledgement.contract_version = ProtocolVersion::new(2, 0),
                1 => acknowledgement.invocation_id = id(9, ModelInvocationId::new),
                2 => acknowledgement.provider_id = id(9, ModelProviderId::new),
                _ => acknowledgement.model_id = id(9, ModelId::new),
            }
            let mut port = ScriptedTutorGenerationCancellationPort::new([
                ScriptedTutorGenerationCancellationOutcome::Acknowledged(acknowledgement),
            ]);
            assert_eq!(
                call(&mut port, &request),
                Err(TutorGenerationCancellationError::AcknowledgementMismatch)
            );
            assert_eq!(port.received_requests(), std::slice::from_ref(&request));
            assert_eq!(port.consumed_outcomes(), 1);
        }
    }

    #[test]
    fn diagnostics_are_closed_and_content_free() {
        for error in [
            TutorGenerationCancellationError::UnsupportedVersion,
            TutorGenerationCancellationError::AssociationMismatch,
            TutorGenerationCancellationError::DependencyFailure,
            TutorGenerationCancellationError::AcknowledgementMismatch,
        ] {
            let diagnostics = format!("{error:?} {error}");
            for sentinel in [
                "prompt-private",
                "output-private",
                "endpoint-private",
                "secret-private",
            ] {
                assert!(!diagnostics.contains(sentinel));
            }
        }
        let dependency = TutorGenerationCancellationDependencyError;
        let _: Value =
            serde_json::to_value(TutorGenerationCancellationError::DependencyFailure).unwrap();
        assert_eq!(
            format!("{dependency:?}"),
            "TutorGenerationCancellationDependencyError"
        );
        assert_eq!(
            dependency.to_string(),
            "tutor-generation cancellation dependency failed"
        );
    }
}
