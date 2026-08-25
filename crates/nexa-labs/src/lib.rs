//! Provider-neutral Tool Execution security admission and cancellation contracts.
//!
//! These values establish structural association only. Caller-supplied policy,
//! confirmation, isolation and cancellation declarations are not proof of
//! authenticity, freshness, sandbox enforcement, or external behavior.
#![forbid(unsafe_code)]

use nexa_domain::{
    EnvironmentInstanceId, LabSessionId, ProtocolVersion, SemanticKey, ToolExecutionId,
    ToolRequestId,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use thiserror::Error;

fn deserialize_v1<'de, D: Deserializer<'de>>(d: D) -> Result<ProtocolVersion, D::Error> {
    let value = ProtocolVersion::deserialize(d)?;
    if value == TOOL_EXECUTION_SECURITY_V1 {
        Ok(value)
    } else {
        Err(serde::de::Error::custom(
            "unsupported tool execution contract version",
        ))
    }
}

pub const TOOL_EXECUTION_SECURITY_V1: ProtocolVersion = ProtocolVersion::new(1, 0);
pub type CancellationFuture<'a> = Pin<
    Box<
        dyn Future<
                Output = Result<ToolCancellationAcknowledgement, ToolCancellationDependencyError>,
            > + Send
            + 'a,
    >,
>;

/// Opaque deterministic digest of immutable request content. It never contains that content.
#[derive(Clone, Copy, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct RequestContentDigest([u8; 32]);
impl RequestContentDigest {
    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
    pub const fn bytes(self) -> [u8; 32] {
        self.0
    }
}
impl fmt::Debug for RequestContentDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RequestContentDigest(REDACTED)")
    }
}
impl<'de> Deserialize<'de> for RequestContentDigest {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = Vec::<u8>::deserialize(d)?;
        let a: [u8; 32] = v
            .try_into()
            .map_err(|_| serde::de::Error::custom("request digest must contain 32 bytes"))?;
        Ok(Self(a))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolAssociation {
    pub lab_session_id: LabSessionId,
    pub tool_request_id: ToolRequestId,
    pub tool_execution_id: ToolExecutionId,
    pub environment_instance_id: EnvironmentInstanceId,
    pub tool: SemanticKey,
    pub operation: SemanticKey,
    pub request_content_digest: RequestContentDigest,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    DenyAll,
    AllowListed { targets: Vec<NetworkTarget> },
}
/// Provider-neutral, canonical network destination. These keys declare intent;
/// they do not select or implement a networking provider.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkTarget {
    pub transport: SemanticKey,
    pub endpoint: SemanticKey,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceBounds {
    pub cpu_millis: u64,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub process_count: u32,
    pub execution_time_millis: u64,
    pub output_bytes: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SandboxDeclaration {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub host_filesystem_access: bool,
    pub host_network_access: bool,
    pub privileged: bool,
    pub root: bool,
    pub bounds: ResourceBounds,
    pub network_policy: NetworkPolicy,
    pub authorized_mounts: Vec<SemanticKey>,
    pub authorized_capabilities: Vec<SemanticKey>,
}
impl SandboxDeclaration {
    pub fn validate(&self) -> Result<(), AdmissionError> {
        if self.contract_version != TOOL_EXECUTION_SECURITY_V1 {
            return Err(AdmissionError::UnsupportedVersion);
        }
        if self.host_filesystem_access || self.host_network_access || self.privileged || self.root {
            return Err(AdmissionError::UnrestrictedEnvironment);
        }
        let b = &self.bounds;
        if b.cpu_millis == 0
            || b.memory_bytes == 0
            || b.storage_bytes == 0
            || b.process_count == 0
            || b.execution_time_millis == 0
            || b.output_bytes == 0
        {
            return Err(AdmissionError::MissingResourceBound);
        }
        if let NetworkPolicy::AllowListed { targets } = &self.network_policy {
            if targets.is_empty() || targets.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err(AdmissionError::InconsistentEnvironment);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskClass {
    ReadOnly,
    Mutating,
    Destructive,
    Privileged,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Deny,
    Allow,
    ConfirmationRequired,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TutorPreference {
    Prefer,
    Neutral,
    Avoid,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskClassificationEvidence {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub risk: RiskClass,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorizationDecision {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub risk: RiskClass,
    pub decision: PolicyDecision,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssessmentDecision {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub risk: RiskClass,
    pub decision: PolicyDecision,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmationEvidence {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub risk: RiskClass,
    pub authorization_decision: PolicyDecision,
    pub assessment_decision: PolicyDecision,
    pub confirmed: bool,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolAdmissionRequest {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub sandbox: SandboxDeclaration,
    pub risk_classification: RiskClassificationEvidence,
    pub authorization: AuthorizationDecision,
    /// Caller-supplied assessment-policy restriction; Tutor preference cannot weaken it.
    pub assessment: AssessmentDecision,
    pub confirmation: Option<ConfirmationEvidence>,
    pub tutor_preference: TutorPreference,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedToolExecution {
    association: ToolAssociation,
    risk: RiskClass,
}
impl AdmittedToolExecution {
    pub fn association(&self) -> &ToolAssociation {
        &self.association
    }
    pub const fn risk(&self) -> RiskClass {
        self.risk
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdmissionError {
    #[error("unsupported contract version")]
    UnsupportedVersion,
    #[error("environment declaration permits unrestricted access")]
    UnrestrictedEnvironment,
    #[error("required resource bound is missing")]
    MissingResourceBound,
    #[error("environment declaration is inconsistent")]
    InconsistentEnvironment,
    #[error("tool execution association mismatch")]
    AssociationMismatch,
    #[error("tool execution risk classification mismatch")]
    RiskMismatch,
    #[error("tool execution denied")]
    Denied,
    #[error("exact confirmation is required")]
    ConfirmationRequired,
}
pub fn admit_tool_execution(
    r: &ToolAdmissionRequest,
) -> Result<AdmittedToolExecution, AdmissionError> {
    if r.contract_version != TOOL_EXECUTION_SECURITY_V1
        || r.risk_classification.contract_version != TOOL_EXECUTION_SECURITY_V1
        || r.authorization.contract_version != TOOL_EXECUTION_SECURITY_V1
        || r.assessment.contract_version != TOOL_EXECUTION_SECURITY_V1
    {
        return Err(AdmissionError::UnsupportedVersion);
    }
    // Security is checked before preferences or any participant can be consulted.
    r.sandbox.validate()?;
    if r.sandbox.association != r.association
        || r.risk_classification.association != r.association
        || r.authorization.association != r.association
        || r.assessment.association != r.association
    {
        return Err(AdmissionError::AssociationMismatch);
    }
    let risk = r.risk_classification.risk;
    if r.authorization.risk != risk || r.assessment.risk != risk {
        return Err(AdmissionError::RiskMismatch);
    }
    if r.authorization.decision == PolicyDecision::Deny
        || r.assessment.decision == PolicyDecision::Deny
    {
        return Err(AdmissionError::Denied);
    }
    let required = r.authorization.decision == PolicyDecision::ConfirmationRequired
        || r.assessment.decision == PolicyDecision::ConfirmationRequired
        || matches!(risk, RiskClass::Destructive | RiskClass::Privileged);
    if required {
        let c = r
            .confirmation
            .as_ref()
            .ok_or(AdmissionError::ConfirmationRequired)?;
        if c.contract_version != TOOL_EXECUTION_SECURITY_V1 {
            return Err(AdmissionError::UnsupportedVersion);
        }
        if !c.confirmed
            || c.association != r.association
            || c.risk != risk
            || c.authorization_decision != r.authorization.decision
            || c.assessment_decision != r.assessment.decision
        {
            return Err(AdmissionError::ConfirmationRequired);
        }
    }
    Ok(AdmittedToolExecution {
        association: r.association.clone(),
        risk,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancellationSemantics {
    Cancellable,
    NonCancellable,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolCancellationCapability {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub semantics: CancellationSemantics,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolCancellationRequest {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolCancellationAcknowledgement {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolCancellationOutcomeKind {
    Accepted,
    DeclaredNonCancellable,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolCancellationEvidence {
    #[serde(deserialize_with = "deserialize_v1")]
    pub contract_version: ProtocolVersion,
    pub association: ToolAssociation,
    pub kind: ToolCancellationOutcomeKind,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ToolCancellationDependencyError;
impl fmt::Display for ToolCancellationDependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tool cancellation dependency failure")
    }
}
impl std::error::Error for ToolCancellationDependencyError {}
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum CancellationError {
    #[error("unsupported contract version")]
    UnsupportedVersion,
    #[error("tool cancellation association mismatch")]
    AssociationMismatch,
    #[error("tool cancellation dependency failure")]
    DependencyFailure,
    #[error("tool cancellation acknowledgement mismatch")]
    AcknowledgementMismatch,
}

pub trait ToolCancellationControl: Send + Sync {
    fn request_cancellation<'a>(
        &'a self,
        request: ToolCancellationRequest,
    ) -> CancellationFuture<'a>;
}
pub async fn cancel_tool_execution(
    capability: &ToolCancellationCapability,
    admitted: &AdmittedToolExecution,
    control: &dyn ToolCancellationControl,
) -> Result<ToolCancellationEvidence, CancellationError> {
    if capability.contract_version != TOOL_EXECUTION_SECURITY_V1 {
        return Err(CancellationError::UnsupportedVersion);
    }
    if capability.association != *admitted.association() {
        return Err(CancellationError::AssociationMismatch);
    }
    if capability.semantics == CancellationSemantics::NonCancellable {
        return Ok(ToolCancellationEvidence {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: capability.association.clone(),
            kind: ToolCancellationOutcomeKind::DeclaredNonCancellable,
        });
    }
    let request = ToolCancellationRequest {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: capability.association.clone(),
    };
    let ack = control
        .request_cancellation(request)
        .await
        .map_err(|_| CancellationError::DependencyFailure)?;
    if ack.contract_version != TOOL_EXECUTION_SECURITY_V1
        || ack.association != capability.association
    {
        return Err(CancellationError::AcknowledgementMismatch);
    }
    Ok(ToolCancellationEvidence {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: capability.association.clone(),
        kind: ToolCancellationOutcomeKind::Accepted,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptedCancellationOutcome {
    Acknowledged(ToolCancellationAcknowledgement),
    DependencyFailure,
    Pending,
}
#[derive(Default)]
struct ScriptState {
    outcomes: VecDeque<ScriptedCancellationOutcome>,
    received: Vec<ToolCancellationRequest>,
    active: usize,
    dropped: usize,
}
#[derive(Clone, Default)]
pub struct ScriptedToolCancellationControl {
    state: Arc<Mutex<ScriptState>>,
}
impl ScriptedToolCancellationControl {
    pub fn new(v: impl IntoIterator<Item = ScriptedCancellationOutcome>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ScriptState {
                outcomes: v.into_iter().collect(),
                ..Default::default()
            })),
        }
    }
    pub fn received(&self) -> Vec<ToolCancellationRequest> {
        self.state.lock().unwrap().received.clone()
    }
    pub fn remaining_outcomes(&self) -> usize {
        self.state.lock().unwrap().outcomes.len()
    }
    pub fn active_futures(&self) -> usize {
        self.state.lock().unwrap().active
    }
    pub fn dropped_futures(&self) -> usize {
        self.state.lock().unwrap().dropped
    }
}
struct ActiveGuard(Arc<Mutex<ScriptState>>);
impl Drop for ActiveGuard {
    fn drop(&mut self) {
        let mut s = self.0.lock().unwrap();
        s.active -= 1;
        s.dropped += 1
    }
}
impl ToolCancellationControl for ScriptedToolCancellationControl {
    fn request_cancellation<'a>(
        &'a self,
        request: ToolCancellationRequest,
    ) -> CancellationFuture<'a> {
        let state = self.state.clone();
        Box::pin(async move {
            let outcome = {
                let mut s = state.lock().unwrap();
                s.received.push(request);
                s.active += 1;
                s.outcomes.pop_front()
            };
            let _guard = ActiveGuard(state);
            match outcome {
                Some(ScriptedCancellationOutcome::Acknowledged(v)) => Ok(v),
                Some(ScriptedCancellationOutcome::Pending) => std::future::pending().await,
                Some(ScriptedCancellationOutcome::DependencyFailure) | None => {
                    Err(ToolCancellationDependencyError)
                }
            }
        })
    }
}
