//! Single-attempt composition of provider invocation and model-output admission.

use crate::admission::{
    admit_model_output_after_preflight, validate_admission_preflight, AdmissionError,
    AdmissionResult, TrustedPlanningAuthority,
};
use crate::model::{LanguageModelProvider, ModelErrorKind, ModelRequest};
use crate::prompt::PromptCompilationResult;
use nexa_knowledge::{CitationResult, ContextPackage};
use thiserror::Error;

/// Closed failure categories for the synchronous single-attempt operation.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum InvocationAdmissionError {
    #[error("invocation-to-admission preflight failed")]
    Preflight(AdmissionError),
    #[error("model provider invocation failed: {0:?}")]
    Invocation(ModelErrorKind),
    #[error("model output admission failed")]
    Admission(AdmissionError),
}

/// Validates all host inputs, invokes the supplied provider once, and admits its exact response.
pub fn invoke_and_admit_model_output(
    provider: &dyn LanguageModelProvider,
    request: &ModelRequest,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, InvocationAdmissionError> {
    validate_admission_preflight(
        provider.descriptor(),
        request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(InvocationAdmissionError::Preflight)?;

    let response = provider
        .generate(request)
        .map_err(|error| InvocationAdmissionError::Invocation(error.kind))?;

    admit_model_output_after_preflight(
        request,
        &response,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(InvocationAdmissionError::Admission)
}
