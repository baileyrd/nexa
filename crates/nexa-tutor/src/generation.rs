//! Single-attempt composition of provider invocation and model-output admission.

use crate::admission::{
    admit_model_output_after_preflight, validate_admission_preflight, AdmissionError,
    AdmissionResult, TrustedPlanningAuthority,
};
use crate::model::{
    LanguageModelProvider, ModelErrorKind, ModelRequest, PrivacyClass, MODEL_INVOCATION_V1,
};
use crate::prompt::PromptCompilationResult;
use crate::registry::ModelRegistry;
use crate::selection::{
    select_model, ModelSelectionError, ModelSelectionRequirements, MODEL_SELECTION_V1,
};
use nexa_domain::ModelInvocationId;
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

/// Closed failure categories for explicit local-only selection and single-attempt admission.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SelectedInvocationAdmissionError {
    #[error("local-only invocation composition requirements are invalid")]
    InvalidLocalOnlyRequirements,
    #[error("local-only model selection failed")]
    Selection(ModelSelectionError),
    #[error("selected model invocation or admission failed")]
    InvocationAdmission(InvocationAdmissionError),
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

/// Selects one explicitly requested local model, invokes it once, and admits its exact response.
pub fn select_local_model_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, SelectedInvocationAdmissionError> {
    if requirements.contract_version != MODEL_SELECTION_V1
        || requirements.maximum_output_tokens == 0
        || !requirements.required_capabilities.structured_output
        || requirements.privacy_preference.as_slice() != [PrivacyClass::LocalOnly]
    {
        return Err(SelectedInvocationAdmissionError::InvalidLocalOnlyRequirements);
    }

    let selected = select_model(registry, &compilation.model_input, requirements)
        .map_err(SelectedInvocationAdmissionError::Selection)?;
    let request = ModelRequest {
        invocation_id,
        provider_id: selected.descriptor.provider_id,
        model_id: selected.descriptor.model_id,
        contract_version: MODEL_INVOCATION_V1,
        input: compilation.model_input.clone(),
        required_capabilities: requirements.required_capabilities.clone(),
        maximum_output_tokens: requirements.maximum_output_tokens,
    };

    invoke_and_admit_model_output(
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(SelectedInvocationAdmissionError::InvocationAdmission)
}
