//! Single-attempt composition of provider invocation and model-output admission.

use crate::admission::{
    admit_model_output_after_preflight, validate_admission_preflight, AdmissionError,
    AdmissionResult, TrustedPlanningAuthority,
};
use crate::authorization::{
    select_authorized_available_remote_model, RemoteAuthorizationError, RemoteModelAuthorization,
};
use crate::availability::{
    select_available_model, ModelAvailabilityError, ModelAvailabilitySnapshot,
};
use crate::model::{
    LanguageModelProvider, ModelErrorKind, ModelRequest, PrivacyClass, MODEL_INVOCATION_V1,
};
use crate::prompt::PromptCompilationResult;
use crate::registry::ModelRegistry;
use crate::remote_prompt::{
    select_filtered_authorized_available_remote_model, FilteredRemoteSelectionError,
    RemotePromptFilterResult,
};
use crate::selection::{
    select_model, ModelSelectionError, ModelSelectionRequirements, MODEL_SELECTION_V1,
};
use crate::tokenization::{
    tokenize_and_validate_model_request_capacity, validate_model_request_token_capacity,
    ModelInputTokenizationEvidence, ModelInputTokenizer, ModelRequestTokenCapacityError,
    TokenizeAndValidateModelRequestCapacityError,
};
use nexa_domain::{ModelInvocationId, ProtocolVersion};
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

/// Closed failure categories for token-capacity-gated single-attempt admission.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum TokenCapacityInvocationAdmissionError {
    #[error("token-capacity-gated invocation preflight failed")]
    Preflight(AdmissionError),
    #[error("model request token-capacity validation failed")]
    TokenCapacity(ModelRequestTokenCapacityError),
    #[error("model provider invocation failed: {0:?}")]
    Invocation(ModelErrorKind),
    #[error("model output admission failed")]
    Admission(AdmissionError),
}

/// Successful exact-tokenization, single-invocation, and strict-admission evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenizedInvocationAdmissionResult {
    pub tokenization_evidence: ModelInputTokenizationEvidence,
    pub admission: AdmissionResult,
}

/// Closed failures for exact tokenization followed by one invocation and admission.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum TokenizedInvocationAdmissionError {
    #[error("tokenized invocation-to-admission preflight failed")]
    Preflight(AdmissionError),
    #[error("model-input tokenization or request-capacity composition failed")]
    TokenizationCapacity(TokenizeAndValidateModelRequestCapacityError),
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

/// Closed failures for explicit local-only selection followed by exact tokenization and admission.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum SelectedTokenizedInvocationAdmissionError {
    #[error("local-only tokenized invocation composition requirements are invalid")]
    InvalidLocalOnlyRequirements,
    #[error("local-only model selection failed")]
    Selection(ModelSelectionError),
    #[error("selected model tokenized invocation or admission failed")]
    TokenizedInvocationAdmission(TokenizedInvocationAdmissionError),
}

/// Closed failure categories for availability-gated explicit local-only execution.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AvailableLocalInvocationAdmissionError {
    #[error("available local-only invocation composition requirements are invalid")]
    InvalidLocalOnlyRequirements,
    #[error("available local-only model selection failed")]
    AvailabilitySelection(ModelAvailabilityError),
    #[error("selected available model invocation or admission failed")]
    InvocationAdmission(InvocationAdmissionError),
}

/// Closed failures for availability-gated local selection followed by exact tokenization and
/// admission.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AvailableLocalTokenizedInvocationAdmissionError {
    #[error("available local-only tokenized invocation composition requirements are invalid")]
    InvalidLocalOnlyRequirements,
    #[error("available local-only model selection failed")]
    AvailabilitySelection(ModelAvailabilityError),
    #[error("selected available model tokenized invocation or admission failed")]
    TokenizedInvocationAdmission(TokenizedInvocationAdmissionError),
}

/// Closed failure categories for authorized, availability-gated remote execution.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AuthorizedAvailableRemoteInvocationAdmissionError {
    #[error("authorized available remote model selection failed")]
    AuthorizationAvailabilitySelection(RemoteAuthorizationError),
    #[error("selected authorized available remote model invocation or admission failed")]
    InvocationAdmission(InvocationAdmissionError),
}

/// Closed failures for authorized available remote selection followed by exact tokenization and
/// admission.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum AuthorizedAvailableRemoteTokenizedInvocationAdmissionError {
    #[error("authorized available remote model selection failed")]
    AuthorizationAvailabilitySelection(RemoteAuthorizationError),
    #[error("selected authorized available remote model tokenized invocation or admission failed")]
    TokenizedInvocationAdmission(TokenizedInvocationAdmissionError),
}

/// Closed failure categories for filtered, authorized, availability-gated remote execution.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum FilteredAuthorizedAvailableRemoteInvocationAdmissionError {
    #[error("filtered authorized available remote model selection failed")]
    FilteredSelection(FilteredRemoteSelectionError),
    #[error("selected filtered authorized available remote model invocation or admission failed")]
    InvocationAdmission(InvocationAdmissionError),
}

/// Closed failures for filtered authorized remote selection followed by exact tokenization and
/// admission.
#[derive(Clone, Copy, Debug, Error, Eq, PartialEq)]
pub enum FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError {
    #[error("filtered authorized available remote model selection failed")]
    FilteredSelection(FilteredRemoteSelectionError),
    #[error("selected filtered authorized available remote model tokenized invocation or admission failed")]
    TokenizedInvocationAdmission(TokenizedInvocationAdmissionError),
}

fn validate_explicit_local_requirements(
    requirements: &ModelSelectionRequirements,
) -> Result<(), ()> {
    if requirements.contract_version != MODEL_SELECTION_V1
        || requirements.maximum_output_tokens == 0
        || !requirements.required_capabilities.structured_output
        || requirements.privacy_preference.as_slice() != [PrivacyClass::LocalOnly]
    {
        return Err(());
    }
    Ok(())
}

fn request_for_selected(
    invocation_id: ModelInvocationId,
    descriptor: &crate::model::ModelDescriptor,
    requirements: &ModelSelectionRequirements,
    compilation: &PromptCompilationResult,
) -> ModelRequest {
    ModelRequest {
        invocation_id,
        provider_id: descriptor.provider_id,
        model_id: descriptor.model_id,
        contract_version: MODEL_INVOCATION_V1,
        input: compilation.model_input.clone(),
        required_capabilities: requirements.required_capabilities.clone(),
        maximum_output_tokens: requirements.maximum_output_tokens,
    }
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

/// Runs admission preflight and exact token-capacity validation before one invocation.
#[allow(clippy::too_many_arguments)]
pub fn invoke_and_admit_model_output_with_token_capacity(
    provider: &dyn LanguageModelProvider,
    request: &ModelRequest,
    tokenization_evidence: &ModelInputTokenizationEvidence,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, TokenCapacityInvocationAdmissionError> {
    validate_admission_preflight(
        provider.descriptor(),
        request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(TokenCapacityInvocationAdmissionError::Preflight)?;

    validate_model_request_token_capacity(provider.descriptor(), request, tokenization_evidence)
        .map_err(TokenCapacityInvocationAdmissionError::TokenCapacity)?;

    let response = provider
        .generate(request)
        .map_err(|error| TokenCapacityInvocationAdmissionError::Invocation(error.kind))?;

    admit_model_output_after_preflight(
        request,
        &response,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(TokenCapacityInvocationAdmissionError::Admission)
}

/// Performs complete admission preflight, exact tokenization, one invocation, and admission.
#[allow(clippy::too_many_arguments)]
pub fn tokenize_invoke_and_admit_model_output_with_token_capacity(
    tokenization_contract_version: ProtocolVersion,
    tokenizer: &dyn ModelInputTokenizer,
    provider: &dyn LanguageModelProvider,
    request: &ModelRequest,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<TokenizedInvocationAdmissionResult, TokenizedInvocationAdmissionError> {
    validate_admission_preflight(
        provider.descriptor(),
        request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(TokenizedInvocationAdmissionError::Preflight)?;

    let tokenization_evidence = tokenize_and_validate_model_request_capacity(
        tokenization_contract_version,
        provider.descriptor(),
        request,
        tokenizer,
    )
    .map_err(TokenizedInvocationAdmissionError::TokenizationCapacity)?;

    let response = provider
        .generate(request)
        .map_err(|error| TokenizedInvocationAdmissionError::Invocation(error.kind))?;
    let admission = admit_model_output_after_preflight(
        request,
        &response,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(TokenizedInvocationAdmissionError::Admission)?;

    Ok(TokenizedInvocationAdmissionResult {
        tokenization_evidence,
        admission,
    })
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
    validate_explicit_local_requirements(requirements)
        .map_err(|()| SelectedInvocationAdmissionError::InvalidLocalOnlyRequirements)?;

    let selected = select_model(registry, &compilation.model_input, requirements)
        .map_err(SelectedInvocationAdmissionError::Selection)?;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

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

/// Selects one explicitly requested local model, tokenizes its exact input, invokes it once, and
/// admits its exact response.
#[allow(clippy::too_many_arguments)]
pub fn select_local_model_tokenize_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    tokenization_contract_version: ProtocolVersion,
    tokenizer: &dyn ModelInputTokenizer,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<TokenizedInvocationAdmissionResult, SelectedTokenizedInvocationAdmissionError> {
    validate_explicit_local_requirements(requirements)
        .map_err(|()| SelectedTokenizedInvocationAdmissionError::InvalidLocalOnlyRequirements)?;

    let selected = select_model(registry, &compilation.model_input, requirements)
        .map_err(SelectedTokenizedInvocationAdmissionError::Selection)?;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

    tokenize_invoke_and_admit_model_output_with_token_capacity(
        tokenization_contract_version,
        tokenizer,
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(SelectedTokenizedInvocationAdmissionError::TokenizedInvocationAdmission)
}

/// Selects one caller-marked-available local model, invokes it once, and admits its response.
#[allow(clippy::too_many_arguments)]
pub fn select_available_local_model_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, AvailableLocalInvocationAdmissionError> {
    validate_explicit_local_requirements(requirements)
        .map_err(|()| AvailableLocalInvocationAdmissionError::InvalidLocalOnlyRequirements)?;

    let selected = select_available_model(
        registry,
        &compilation.model_input,
        requirements,
        availability,
    )
    .map_err(AvailableLocalInvocationAdmissionError::AvailabilitySelection)?;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

    invoke_and_admit_model_output(
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(AvailableLocalInvocationAdmissionError::InvocationAdmission)
}

/// Selects one caller-marked-available local model, tokenizes its exact input, invokes it once,
/// and admits its exact response.
#[allow(clippy::too_many_arguments)]
pub fn select_available_local_model_tokenize_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    tokenization_contract_version: ProtocolVersion,
    tokenizer: &dyn ModelInputTokenizer,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<TokenizedInvocationAdmissionResult, AvailableLocalTokenizedInvocationAdmissionError> {
    validate_explicit_local_requirements(requirements).map_err(|()| {
        AvailableLocalTokenizedInvocationAdmissionError::InvalidLocalOnlyRequirements
    })?;

    let selected = select_available_model(
        registry,
        &compilation.model_input,
        requirements,
        availability,
    )
    .map_err(AvailableLocalTokenizedInvocationAdmissionError::AvailabilitySelection)?;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

    tokenize_invoke_and_admit_model_output_with_token_capacity(
        tokenization_contract_version,
        tokenizer,
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(AvailableLocalTokenizedInvocationAdmissionError::TokenizedInvocationAdmission)
}

/// Selects one explicitly authorized, caller-marked-available remote model, invokes it once, and
/// admits its exact response.
#[allow(clippy::too_many_arguments)]
pub fn select_authorized_available_remote_model_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    authorization: &RemoteModelAuthorization,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, AuthorizedAvailableRemoteInvocationAdmissionError> {
    let selected = select_authorized_available_remote_model(
        registry,
        requirements,
        availability,
        authorization,
        compilation,
    )
    .map_err(
        AuthorizedAvailableRemoteInvocationAdmissionError::AuthorizationAvailabilitySelection,
    )?;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

    invoke_and_admit_model_output(
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(AuthorizedAvailableRemoteInvocationAdmissionError::InvocationAdmission)
}

/// Selects one explicitly authorized, caller-marked-available remote model, tokenizes its exact
/// input, invokes it once, and admits its exact response.
#[allow(clippy::too_many_arguments)]
pub fn select_authorized_available_remote_model_tokenize_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    authorization: &RemoteModelAuthorization,
    tokenization_contract_version: ProtocolVersion,
    tokenizer: &dyn ModelInputTokenizer,
    compilation: &PromptCompilationResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<
    TokenizedInvocationAdmissionResult,
    AuthorizedAvailableRemoteTokenizedInvocationAdmissionError,
> {
    let selected = select_authorized_available_remote_model(
        registry,
        requirements,
        availability,
        authorization,
        compilation,
    )
    .map_err(
        AuthorizedAvailableRemoteTokenizedInvocationAdmissionError::AuthorizationAvailabilitySelection,
    )?;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

    tokenize_invoke_and_admit_model_output_with_token_capacity(
        tokenization_contract_version,
        tokenizer,
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(
        AuthorizedAvailableRemoteTokenizedInvocationAdmissionError::TokenizedInvocationAdmission,
    )
}

/// Selects through ADR-0034, invokes the selected provider once, and strictly admits its response.
#[allow(clippy::too_many_arguments)]
pub fn select_filtered_authorized_available_remote_model_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    authorization: &RemoteModelAuthorization,
    filtered_result: &RemotePromptFilterResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<AdmissionResult, FilteredAuthorizedAvailableRemoteInvocationAdmissionError> {
    let selected = select_filtered_authorized_available_remote_model(
        registry,
        requirements,
        availability,
        authorization,
        filtered_result,
    )
    .map_err(FilteredAuthorizedAvailableRemoteInvocationAdmissionError::FilteredSelection)?;
    let compilation = &filtered_result.filtered_compilation;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

    invoke_and_admit_model_output(
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(FilteredAuthorizedAvailableRemoteInvocationAdmissionError::InvocationAdmission)
}

/// Selects through ADR-0034, tokenizes the exact filtered input, invokes the selected provider
/// once, and strictly admits its response.
#[allow(clippy::too_many_arguments)]
pub fn select_filtered_authorized_available_remote_model_tokenize_invoke_and_admit(
    registry: &ModelRegistry,
    invocation_id: ModelInvocationId,
    requirements: &ModelSelectionRequirements,
    availability: &ModelAvailabilitySnapshot,
    authorization: &RemoteModelAuthorization,
    tokenization_contract_version: ProtocolVersion,
    tokenizer: &dyn ModelInputTokenizer,
    filtered_result: &RemotePromptFilterResult,
    authority: &TrustedPlanningAuthority,
    context: &ContextPackage,
    citations: &CitationResult,
) -> Result<
    TokenizedInvocationAdmissionResult,
    FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError,
> {
    let selected = select_filtered_authorized_available_remote_model(
        registry,
        requirements,
        availability,
        authorization,
        filtered_result,
    )
    .map_err(
        FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError::FilteredSelection,
    )?;
    let compilation = &filtered_result.filtered_compilation;
    let request = request_for_selected(
        invocation_id,
        &selected.descriptor,
        requirements,
        compilation,
    );

    tokenize_invoke_and_admit_model_output_with_token_capacity(
        tokenization_contract_version,
        tokenizer,
        selected.provider.as_ref(),
        &request,
        compilation,
        authority,
        context,
        citations,
    )
    .map_err(
        FilteredAuthorizedAvailableRemoteTokenizedInvocationAdmissionError::TokenizedInvocationAdmission,
    )
}
