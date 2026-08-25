use nexa_domain::{
    EnvironmentInstanceId, LabSessionId, ProtocolVersion, SemanticKey, ToolExecutionId,
    ToolRequestId,
};
use nexa_labs::*;
use std::{
    future::Future,
    pin::pin,
    task::{Context, Poll, Waker},
};
use uuid::Uuid;
fn id<T>(n: u128, f: impl Fn(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
    f(Uuid::from_u128(n)).unwrap()
}
fn association() -> ToolAssociation {
    ToolAssociation {
        lab_session_id: id(1, LabSessionId::new),
        tool_request_id: id(2, ToolRequestId::new),
        tool_execution_id: id(3, ToolExecutionId::new),
        environment_instance_id: id(4, EnvironmentInstanceId::new),
        tool: SemanticKey::new("shell").unwrap(),
        operation: SemanticKey::new("inspect").unwrap(),
        request_content_digest: RequestContentDigest::new([7; 32]),
    }
}
fn sandbox(a: &ToolAssociation) -> SandboxDeclaration {
    SandboxDeclaration {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a.clone(),
        host_filesystem_access: false,
        host_network_access: false,
        privileged: false,
        root: false,
        bounds: ResourceBounds {
            cpu_millis: 1,
            memory_bytes: 1,
            storage_bytes: 1,
            process_count: 1,
            execution_time_millis: 1,
            output_bytes: 1,
        },
        network_policy: NetworkPolicy::DenyAll,
        authorized_mounts: vec![],
        authorized_capabilities: vec![],
    }
}
fn request(risk: RiskClass, decision: PolicyDecision, confirmation: bool) -> ToolAdmissionRequest {
    let a = association();
    ToolAdmissionRequest {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a.clone(),
        sandbox: sandbox(&a),
        risk_classification: RiskClassificationEvidence {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a.clone(),
            risk,
        },
        authorization: AuthorizationDecision {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a.clone(),
            risk,
            decision,
        },
        assessment: AssessmentDecision {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a.clone(),
            risk,
            decision: PolicyDecision::Allow,
        },
        confirmation: confirmation.then_some(ConfirmationEvidence {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a,
            risk,
            authorization_decision: decision,
            assessment_decision: PolicyDecision::Allow,
            confirmed: true,
        }),
        tutor_preference: TutorPreference::Prefer,
    }
}
fn admitted() -> AdmittedToolExecution {
    admit_tool_execution(&request(RiskClass::ReadOnly, PolicyDecision::Allow, false)).unwrap()
}
fn block<F: Future>(f: F) -> F::Output {
    let w = Waker::noop();
    let mut c = Context::from_waker(w);
    let mut f = pin!(f);
    loop {
        if let Poll::Ready(v) = f.as_mut().poll(&mut c) {
            return v;
        }
    }
}
#[test]
fn identities_are_non_nil_and_round_trip() {
    for value in [
        serde_json::to_string(&id(1, LabSessionId::new)).unwrap(),
        serde_json::to_string(&id(2, ToolRequestId::new)).unwrap(),
        serde_json::to_string(&id(3, ToolExecutionId::new)).unwrap(),
        serde_json::to_string(&id(4, EnvironmentInstanceId::new)).unwrap(),
    ] {
        assert!(!value.contains("00000000-0000-0000-0000-000000000000"));
    }
    assert!(LabSessionId::new(Uuid::nil()).is_err());
    let a = association();
    assert_eq!(
        serde_json::from_str::<ToolAssociation>(&serde_json::to_string(&a).unwrap()).unwrap(),
        a
    )
}
#[test]
fn wire_is_strict_and_content_free() {
    let a = association();
    let mut v = serde_json::to_value(&a).unwrap();
    v.as_object_mut()
        .unwrap()
        .insert("unknown".into(), serde_json::json!(1));
    assert!(serde_json::from_value::<ToolAssociation>(v).is_err());
    assert!(serde_json::from_str::<RiskClass>("\"future\"").is_err());
    assert!(serde_json::from_str::<RequestContentDigest>("[1,2]").is_err());
    let error = AdmissionError::Denied;
    assert_eq!(serde_json::to_string(&error).unwrap(), "\"denied\"");
    assert!(!format!("{error:?} {error}").contains("secret"));
    assert_eq!(
        format!("{:?}", a.request_content_digest),
        "RequestContentDigest(REDACTED)"
    )
}
#[test]
fn sandbox_rejects_each_unsafe_or_missing_declaration() {
    let a = association();
    assert_eq!(sandbox(&a).validate(), Ok(()));
    for mutate in [
        |s: &mut SandboxDeclaration| s.host_filesystem_access = true,
        |s: &mut SandboxDeclaration| s.host_network_access = true,
        |s: &mut SandboxDeclaration| s.privileged = true,
        |s: &mut SandboxDeclaration| s.root = true,
    ] {
        let mut s = sandbox(&a);
        mutate(&mut s);
        assert_eq!(s.validate(), Err(AdmissionError::UnrestrictedEnvironment))
    }
    for zero in 0..6 {
        let mut s = sandbox(&a);
        match zero {
            0 => s.bounds.cpu_millis = 0,
            1 => s.bounds.memory_bytes = 0,
            2 => s.bounds.storage_bytes = 0,
            3 => s.bounds.process_count = 0,
            4 => s.bounds.execution_time_millis = 0,
            _ => s.bounds.output_bytes = 0,
        };
        assert_eq!(s.validate(), Err(AdmissionError::MissingResourceBound))
    }
}
#[test]
fn authorization_and_confirmation_fail_closed() {
    assert_eq!(
        admit_tool_execution(&request(RiskClass::ReadOnly, PolicyDecision::Deny, true)),
        Err(AdmissionError::Denied)
    );
    assert_eq!(
        admit_tool_execution(&request(
            RiskClass::ReadOnly,
            PolicyDecision::ConfirmationRequired,
            false
        )),
        Err(AdmissionError::ConfirmationRequired)
    );
    for risk in [RiskClass::Destructive, RiskClass::Privileged] {
        assert_eq!(
            admit_tool_execution(&request(risk, PolicyDecision::Allow, false)),
            Err(AdmissionError::ConfirmationRequired)
        );
        assert!(admit_tool_execution(&request(risk, PolicyDecision::Allow, true)).is_ok())
    }
    let mut r = request(
        RiskClass::Mutating,
        PolicyDecision::ConfirmationRequired,
        true,
    );
    r.confirmation
        .as_mut()
        .unwrap()
        .association
        .request_content_digest = RequestContentDigest::new([9; 32]);
    assert_eq!(
        admit_tool_execution(&r),
        Err(AdmissionError::ConfirmationRequired)
    );
    let mut r = request(RiskClass::ReadOnly, PolicyDecision::Deny, true);
    r.tutor_preference = TutorPreference::Prefer;
    assert_eq!(admit_tool_execution(&r), Err(AdmissionError::Denied));
    let mut r = request(RiskClass::ReadOnly, PolicyDecision::Allow, true);
    r.assessment.decision = PolicyDecision::Deny;
    r.tutor_preference = TutorPreference::Prefer;
    assert_eq!(admit_tool_execution(&r), Err(AdmissionError::Denied))
}
#[test]
fn every_association_field_is_checked() {
    for n in 0..7 {
        let mut r = request(RiskClass::ReadOnly, PolicyDecision::Allow, false);
        match n {
            0 => r.authorization.association.lab_session_id = id(11, LabSessionId::new),
            1 => r.authorization.association.tool_request_id = id(12, ToolRequestId::new),
            2 => r.authorization.association.tool_execution_id = id(13, ToolExecutionId::new),
            3 => {
                r.authorization.association.environment_instance_id =
                    id(14, EnvironmentInstanceId::new)
            }
            4 => r.authorization.association.tool = SemanticKey::new("other").unwrap(),
            5 => r.authorization.association.operation = SemanticKey::new("other").unwrap(),
            _ => {
                r.authorization.association.request_content_digest =
                    RequestContentDigest::new([8; 32])
            }
        };
        assert_eq!(
            admit_tool_execution(&r),
            Err(AdmissionError::AssociationMismatch)
        )
    }
}
#[test]
fn cancellable_invokes_once_and_checks_ack() {
    let a = association();
    let cap = ToolCancellationCapability {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a.clone(),
        semantics: CancellationSemantics::Cancellable,
    };
    let control =
        ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Acknowledged(
            ToolCancellationAcknowledgement {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: a,
            },
        )]);
    assert_eq!(
        block(cancel_tool_execution(&cap, &admitted(), &control))
            .unwrap()
            .kind,
        ToolCancellationOutcomeKind::Accepted
    );
    assert_eq!(control.received().len(), 1);
    assert_eq!(control.remaining_outcomes(), 0);
    assert_eq!(control.active_futures(), 0);
    assert_eq!(control.dropped_futures(), 1)
}
#[test]
fn cancellation_failure_mismatch_exhaustion_and_non_cancellable_are_exact() {
    let a = association();
    let cap = ToolCancellationCapability {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a.clone(),
        semantics: CancellationSemantics::Cancellable,
    };
    for outcomes in [vec![ScriptedCancellationOutcome::DependencyFailure], vec![]] {
        let c = ScriptedToolCancellationControl::new(outcomes);
        assert_eq!(
            block(cancel_tool_execution(&cap, &admitted(), &c)),
            Err(CancellationError::DependencyFailure)
        );
        assert_eq!(c.active_futures(), 0)
    }
    let mut wrong = a.clone();
    wrong.operation = SemanticKey::new("wrong").unwrap();
    let c = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Acknowledged(
        ToolCancellationAcknowledgement {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: wrong,
        },
    )]);
    assert_eq!(
        block(cancel_tool_execution(&cap, &admitted(), &c)),
        Err(CancellationError::AcknowledgementMismatch)
    );
    let nc = ToolCancellationCapability {
        semantics: CancellationSemantics::NonCancellable,
        ..cap
    };
    let c = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::DependencyFailure]);
    assert_eq!(
        block(cancel_tool_execution(&nc, &admitted(), &c))
            .unwrap()
            .kind,
        ToolCancellationOutcomeKind::DeclaredNonCancellable
    );
    assert!(c.received().is_empty());
    assert_eq!(c.remaining_outcomes(), 1)
}
#[test]
fn dropped_pending_future_has_exact_accounting() {
    let a = association();
    let cap = ToolCancellationCapability {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a,
        semantics: CancellationSemantics::Cancellable,
    };
    let c = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Pending]);
    let admitted = admitted();
    let mut f = Box::pin(cancel_tool_execution(&cap, &admitted, &c));
    let w = Waker::noop();
    assert!(matches!(
        f.as_mut().poll(&mut Context::from_waker(w)),
        Poll::Pending
    ));
    assert_eq!(c.active_futures(), 1);
    drop(f);
    assert_eq!(c.active_futures(), 0);
    assert_eq!(c.dropped_futures(), 1)
}
#[test]
fn invalid_versions_fail_preflight_without_dependency() {
    let mut cap = ToolCancellationCapability {
        contract_version: ProtocolVersion::new(2, 0),
        association: association(),
        semantics: CancellationSemantics::Cancellable,
    };
    let c = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::DependencyFailure]);
    assert_eq!(
        block(cancel_tool_execution(&cap, &admitted(), &c)),
        Err(CancellationError::UnsupportedVersion)
    );
    assert!(c.received().is_empty());
    cap.contract_version = TOOL_EXECUTION_SECURITY_V1
}

fn mutate_association(a: &mut ToolAssociation, field: usize) {
    match field {
        0 => a.lab_session_id = id(11, LabSessionId::new),
        1 => a.tool_request_id = id(12, ToolRequestId::new),
        2 => a.tool_execution_id = id(13, ToolExecutionId::new),
        3 => a.environment_instance_id = id(14, EnvironmentInstanceId::new),
        4 => a.tool = SemanticKey::new("other-tool").unwrap(),
        5 => a.operation = SemanticKey::new("other-operation").unwrap(),
        _ => a.request_content_digest = RequestContentDigest::new([8; 32]),
    }
}

#[test]
fn v1_evidence_wire_contracts_are_strict_and_canonical() {
    let r = request(
        RiskClass::Mutating,
        PolicyDecision::ConfirmationRequired,
        true,
    );
    macro_rules! strict_v1 {
        ($ty:ty, $value:expr) => {{
            let expected: $ty = $value;
            let canonical = serde_json::to_value(&expected).unwrap();
            assert_eq!(canonical["contract_version"], serde_json::json!("1.0"));
            assert_eq!(
                serde_json::from_value::<$ty>(canonical.clone()).unwrap(),
                expected
            );
            let mut unknown = canonical.clone();
            unknown["future_field"] = serde_json::json!(true);
            assert!(serde_json::from_value::<$ty>(unknown).is_err());
            let mut version = canonical;
            version["contract_version"] = serde_json::json!("1.1");
            assert!(serde_json::from_value::<$ty>(version).is_err());
        }};
    }
    strict_v1!(RiskClassificationEvidence, r.risk_classification.clone());
    strict_v1!(AuthorizationDecision, r.authorization.clone());
    strict_v1!(AssessmentDecision, r.assessment.clone());
    strict_v1!(ConfirmationEvidence, r.confirmation.clone().unwrap());
    strict_v1!(SandboxDeclaration, r.sandbox.clone());
    strict_v1!(ToolAdmissionRequest, r.clone());
    assert_eq!(
        serde_json::to_string(&RiskClass::ReadOnly).unwrap(),
        "\"read_only\""
    );
    assert_eq!(
        serde_json::to_string(&PolicyDecision::ConfirmationRequired).unwrap(),
        "\"confirmation_required\""
    );
    assert_eq!(
        serde_json::to_string(&CancellationSemantics::NonCancellable).unwrap(),
        "\"non_cancellable\""
    );
    assert!(serde_json::from_str::<PolicyDecision>("\"ALLOW\"").is_err());
    assert!(serde_json::from_str::<CancellationSemantics>("\"future_variant\"").is_err());

    let json = serde_json::to_value(association()).unwrap();
    assert_eq!(json["request_content_digest"].as_array().unwrap().len(), 32);
    for bad in [
        serde_json::json!([]),
        serde_json::to_value(vec![0_u8; 31]).unwrap(),
        serde_json::to_value(vec![0_u8; 33]).unwrap(),
    ] {
        assert!(serde_json::from_value::<RequestContentDigest>(bad).is_err());
    }
    let mut nil = json.clone();
    nil["tool_request_id"] = serde_json::json!(Uuid::nil());
    assert!(serde_json::from_value::<ToolAssociation>(nil).is_err());
    let mut malformed = json;
    malformed["operation"] = serde_json::json!("Not Canonical");
    assert!(serde_json::from_value::<ToolAssociation>(malformed).is_err());
}

#[test]
fn all_policy_evidence_associations_and_risks_are_independently_bound() {
    for field in 0..7 {
        for evidence in 0..4 {
            let mut r = request(RiskClass::Mutating, PolicyDecision::Allow, false);
            match evidence {
                0 => mutate_association(&mut r.risk_classification.association, field),
                1 => mutate_association(&mut r.authorization.association, field),
                2 => mutate_association(&mut r.assessment.association, field),
                _ => mutate_association(&mut r.sandbox.association, field),
            }
            assert_eq!(
                admit_tool_execution(&r),
                Err(AdmissionError::AssociationMismatch)
            );
        }
        let mut r = request(RiskClass::Destructive, PolicyDecision::Allow, true);
        mutate_association(&mut r.confirmation.as_mut().unwrap().association, field);
        assert_eq!(
            admit_tool_execution(&r),
            Err(AdmissionError::ConfirmationRequired)
        );
    }
    for evidence in 0..2 {
        let mut r = request(RiskClass::Mutating, PolicyDecision::Allow, false);
        if evidence == 0 {
            r.authorization.risk = RiskClass::ReadOnly;
        } else {
            r.assessment.risk = RiskClass::ReadOnly;
        }
        assert_eq!(admit_tool_execution(&r), Err(AdmissionError::RiskMismatch));
    }
    for mismatch in 0..3 {
        let mut r = request(
            RiskClass::Destructive,
            PolicyDecision::ConfirmationRequired,
            true,
        );
        let c = r.confirmation.as_mut().unwrap();
        match mismatch {
            0 => c.risk = RiskClass::ReadOnly,
            1 => c.authorization_decision = PolicyDecision::Allow,
            _ => c.assessment_decision = PolicyDecision::Deny,
        }
        assert_eq!(
            admit_tool_execution(&r),
            Err(AdmissionError::ConfirmationRequired)
        );
    }
}

#[test]
fn network_policy_is_structural_nonempty_unique_and_canonical() {
    let a = association();
    let target = |transport: &str, endpoint: &str| NetworkTarget {
        transport: SemanticKey::new(transport).unwrap(),
        endpoint: SemanticKey::new(endpoint).unwrap(),
    };
    let mut s = sandbox(&a);
    s.network_policy = NetworkPolicy::AllowListed { targets: vec![] };
    assert_eq!(s.validate(), Err(AdmissionError::InconsistentEnvironment));
    s.network_policy = NetworkPolicy::AllowListed {
        targets: vec![target("https", "docs-api")],
    };
    assert_eq!(s.validate(), Ok(()));
    s.network_policy = NetworkPolicy::AllowListed {
        targets: vec![target("https", "docs-api"), target("https", "docs-api")],
    };
    assert_eq!(s.validate(), Err(AdmissionError::InconsistentEnvironment));
    s.network_policy = NetworkPolicy::AllowListed {
        targets: vec![target("https", "zeta"), target("https", "alpha")],
    };
    assert_eq!(s.validate(), Err(AdmissionError::InconsistentEnvironment));
    assert!(serde_json::from_str::<NetworkPolicy>(
        r#"{"allow_listed":{"targets":[{"transport":"HTTP","endpoint":"bad host"}]}}"#
    )
    .is_err());
    assert!(serde_json::from_str::<NetworkPolicy>(
        r#"{"allow_listed":{"targets":[{"transport":"https","endpoint":"docs-api"}],"future_field":true}}"#
    )
    .is_err());
    assert!(serde_json::from_str::<NetworkPolicy>(
        r#"{"deny_all":{"targets":[{"transport":"https","endpoint":"docs-api"}]}}"#
    )
    .is_err());
    assert_eq!(
        serde_json::to_string(&NetworkPolicy::DenyAll).unwrap(),
        "\"deny_all\""
    );
    let allow_listed = NetworkPolicy::AllowListed {
        targets: vec![target("https", "docs-api")],
    };
    assert_eq!(
        serde_json::from_str::<NetworkPolicy>(&serde_json::to_string(&allow_listed).unwrap())
            .unwrap(),
        allow_listed
    );
    assert!(serde_json::from_str::<NetworkPolicy>(r#"{"future_policy":{}}"#).is_err());

    let target = target("https", "docs-api");
    let mut target_json = serde_json::to_value(&target).unwrap();
    assert_eq!(
        serde_json::from_value::<NetworkTarget>(target_json.clone()).unwrap(),
        target
    );
    target_json["future_field"] = serde_json::json!(true);
    assert!(serde_json::from_value::<NetworkTarget>(target_json).is_err());

    let bounds = sandbox(&a).bounds;
    let mut bounds_json = serde_json::to_value(&bounds).unwrap();
    assert_eq!(
        serde_json::from_value::<ResourceBounds>(bounds_json.clone()).unwrap(),
        bounds
    );
    bounds_json["future_field"] = serde_json::json!(true);
    assert!(serde_json::from_value::<ResourceBounds>(bounds_json).is_err());
    let mut malformed_bounds = serde_json::to_value(&bounds).unwrap();
    malformed_bounds["process_count"] = serde_json::json!(-1);
    assert!(serde_json::from_value::<ResourceBounds>(malformed_bounds).is_err());
}

#[test]
fn admission_failures_never_touch_cancellation_work() {
    let cases = [
        request(RiskClass::ReadOnly, PolicyDecision::Deny, false),
        request(RiskClass::Destructive, PolicyDecision::Allow, false),
        {
            let mut r = request(RiskClass::Destructive, PolicyDecision::Allow, true);
            r.confirmation.as_mut().unwrap().risk = RiskClass::ReadOnly;
            r
        },
        {
            let mut r = request(RiskClass::ReadOnly, PolicyDecision::Allow, false);
            r.sandbox.association.environment_instance_id = id(99, EnvironmentInstanceId::new);
            r
        },
    ];
    for r in cases {
        let control = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Pending]);
        assert!(admit_tool_execution(&r).is_err());
        assert!(control.received().is_empty());
        assert_eq!(control.remaining_outcomes(), 1);
        assert_eq!(control.active_futures(), 0);
        assert_eq!(control.dropped_futures(), 0);
    }
}

#[test]
fn cancellation_capability_and_acknowledgement_check_every_association_field() {
    for field in 0..7 {
        let mut cap_association = association();
        mutate_association(&mut cap_association, field);
        let cap = ToolCancellationCapability {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: cap_association,
            semantics: CancellationSemantics::Cancellable,
        };
        let c = ScriptedToolCancellationControl::new([]);
        assert_eq!(
            block(cancel_tool_execution(&cap, &admitted(), &c)),
            Err(CancellationError::AssociationMismatch)
        );
        assert!(c.received().is_empty());

        let a = association();
        let cap = ToolCancellationCapability {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a.clone(),
            semantics: CancellationSemantics::Cancellable,
        };
        let mut wrong = a;
        mutate_association(&mut wrong, field);
        let c = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Acknowledged(
            ToolCancellationAcknowledgement {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: wrong,
            },
        )]);
        assert_eq!(
            block(cancel_tool_execution(&cap, &admitted(), &c)),
            Err(CancellationError::AcknowledgementMismatch)
        );
    }
}

#[test]
fn all_public_diagnostics_and_outcomes_are_request_content_free() {
    const FORBIDDEN: [&str; 3] = [
        "FORBIDDEN-PROMPT-MARKER-ALPHA",
        "FORBIDDEN-SECRET-MARKER-BRAVO",
        "FORBIDDEN-OUTPUT-MARKER-CHARLIE",
    ];
    // Request content is representable only by its opaque digest. Derive that digest from
    // command/argument/path/secret/output-like markers rather than testing an unrelated value.
    let mut digest = [0_u8; 32];
    for (index, byte) in FORBIDDEN.join(" --arg /private/path ").bytes().enumerate() {
        digest[index % digest.len()] ^= byte;
    }
    let mut a = association();
    a.request_content_digest = RequestContentDigest::new(digest);

    let admission_errors = [
        AdmissionError::UnsupportedVersion,
        AdmissionError::UnrestrictedEnvironment,
        AdmissionError::MissingResourceBound,
        AdmissionError::InconsistentEnvironment,
        AdmissionError::AssociationMismatch,
        AdmissionError::RiskMismatch,
        AdmissionError::Denied,
        AdmissionError::ConfirmationRequired,
    ];
    let cancellation_errors = [
        CancellationError::UnsupportedVersion,
        CancellationError::AssociationMismatch,
        CancellationError::DependencyFailure,
        CancellationError::AcknowledgementMismatch,
    ];
    let mut values = Vec::new();
    for error in admission_errors {
        values.push(format!("{error:?} {error}"));
        values.push(serde_json::to_string(&error).unwrap());
    }
    for error in cancellation_errors {
        values.push(format!("{error:?} {error}"));
    }
    values.push(format!(
        "{:?} {}",
        ToolCancellationDependencyError, ToolCancellationDependencyError
    ));

    let mut admission = request(RiskClass::ReadOnly, PolicyDecision::Allow, false);
    admission.association = a.clone();
    admission.sandbox.association = a.clone();
    admission.risk_classification.association = a.clone();
    admission.authorization.association = a.clone();
    admission.assessment.association = a.clone();
    let admitted = admit_tool_execution(&admission).unwrap();
    let mut denied_admission = admission.clone();
    denied_admission.authorization.decision = PolicyDecision::Deny;
    let exercised_admission_error = admit_tool_execution(&denied_admission).unwrap_err();
    assert_eq!(exercised_admission_error, AdmissionError::Denied);
    values.push(format!(
        "{exercised_admission_error:?} {exercised_admission_error}"
    ));
    let cap = ToolCancellationCapability {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a.clone(),
        semantics: CancellationSemantics::Cancellable,
    };
    let dependency =
        ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::DependencyFailure]);
    let dependency_error = block(cancel_tool_execution(&cap, &admitted, &dependency)).unwrap_err();
    values.push(format!("{dependency_error:?} {dependency_error}"));

    let mut wrong = a.clone();
    wrong.request_content_digest = RequestContentDigest::new([0; 32]);
    let mismatch =
        ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Acknowledged(
            ToolCancellationAcknowledgement {
                contract_version: TOOL_EXECUTION_SECURITY_V1,
                association: wrong,
            },
        )]);
    let mismatch_error = block(cancel_tool_execution(&cap, &admitted, &mismatch)).unwrap_err();
    values.push(format!("{mismatch_error:?} {mismatch_error}"));

    let non_cancellable = ToolCancellationCapability {
        semantics: CancellationSemantics::NonCancellable,
        ..cap.clone()
    };
    let unused = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Pending]);
    let evidence = block(cancel_tool_execution(&non_cancellable, &admitted, &unused)).unwrap();
    values.push(format!("{evidence:?}"));
    values.push(serde_json::to_string(&evidence).unwrap());

    let acknowledgement = ToolCancellationAcknowledgement {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a.clone(),
    };
    values.push(format!("{acknowledgement:?}"));
    let acknowledgement_json = serde_json::to_string(&acknowledgement).unwrap();
    assert!(acknowledgement_json.contains(&a.tool_request_id.to_string()));
    values.push(acknowledgement_json);

    let pending = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Pending]);
    let mut future = Box::pin(cancel_tool_execution(&cap, &admitted, &pending));
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    drop(future);
    assert_eq!(pending.active_futures(), 0);
    assert_eq!(pending.dropped_futures(), 1);
    values.push(format!(
        "active={} dropped={}",
        pending.active_futures(),
        pending.dropped_futures()
    ));

    for value in values {
        for marker in FORBIDDEN {
            assert!(!value.contains(marker), "leaked marker {marker}");
        }
    }
}

#[test]
fn admission_and_cancellation_envelopes_reject_unknown_fields_and_versions() {
    fn with_unknown<T: serde::Serialize>(value: &T) -> serde_json::Value {
        let mut value = serde_json::to_value(value).unwrap();
        value
            .as_object_mut()
            .unwrap()
            .insert("future_field".into(), serde_json::json!(true));
        value
    }
    fn with_version<T: serde::Serialize>(value: &T, version: &str) -> serde_json::Value {
        let mut value = serde_json::to_value(value).unwrap();
        value["contract_version"] = serde_json::json!(version);
        value
    }
    let r = request(RiskClass::ReadOnly, PolicyDecision::Allow, false);
    assert!(serde_json::from_value::<ToolAdmissionRequest>(with_unknown(&r)).is_err());
    assert!(serde_json::from_value::<ToolAdmissionRequest>(with_version(&r, "2.0")).is_err());
    assert!(
        serde_json::from_value::<RiskClassificationEvidence>(with_unknown(&r.risk_classification))
            .is_err()
    );
    assert!(
        serde_json::from_value::<RiskClassificationEvidence>(with_version(
            &r.risk_classification,
            "0.9"
        ))
        .is_err()
    );
    assert!(serde_json::from_value::<SandboxDeclaration>(with_unknown(&r.sandbox)).is_err());
    assert!(serde_json::from_value::<SandboxDeclaration>(with_version(&r.sandbox, "1.1")).is_err());

    let capability = ToolCancellationCapability {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: association(),
        semantics: CancellationSemantics::Cancellable,
    };
    let request = ToolCancellationRequest {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: association(),
    };
    let acknowledgement = ToolCancellationAcknowledgement {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: association(),
    };
    let evidence = ToolCancellationEvidence {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: association(),
        kind: ToolCancellationOutcomeKind::Accepted,
    };
    assert_eq!(
        serde_json::from_value::<ToolCancellationCapability>(
            serde_json::to_value(&capability).unwrap()
        )
        .unwrap(),
        capability
    );
    assert_eq!(
        serde_json::from_value::<ToolCancellationRequest>(serde_json::to_value(&request).unwrap())
            .unwrap(),
        request
    );
    assert_eq!(
        serde_json::from_value::<ToolCancellationAcknowledgement>(
            serde_json::to_value(&acknowledgement).unwrap()
        )
        .unwrap(),
        acknowledgement
    );
    assert_eq!(
        serde_json::from_value::<ToolCancellationEvidence>(
            serde_json::to_value(&evidence).unwrap()
        )
        .unwrap(),
        evidence
    );
    assert!(
        serde_json::from_value::<ToolCancellationCapability>(with_unknown(&capability)).is_err()
    );
    assert!(
        serde_json::from_value::<ToolCancellationCapability>(with_version(&capability, "2.0"))
            .is_err()
    );
    assert!(serde_json::from_value::<ToolCancellationRequest>(with_unknown(&request)).is_err());
    assert!(
        serde_json::from_value::<ToolCancellationRequest>(with_version(&request, "2.0")).is_err()
    );
    assert!(
        serde_json::from_value::<ToolCancellationAcknowledgement>(with_unknown(&acknowledgement))
            .is_err()
    );
    assert!(
        serde_json::from_value::<ToolCancellationAcknowledgement>(with_version(
            &acknowledgement,
            "2.0"
        ))
        .is_err()
    );
    assert!(serde_json::from_value::<ToolCancellationEvidence>(with_unknown(&evidence)).is_err());
    assert!(
        serde_json::from_value::<ToolCancellationEvidence>(with_version(&evidence, "2.0")).is_err()
    );
    assert!(serde_json::from_str::<ToolCancellationOutcomeKind>("\"future_kind\"").is_err());
    assert!(serde_json::from_str::<TutorPreference>("\"future_preference\"").is_err());
    assert!(serde_json::from_str::<AdmissionError>("\"future_error\"").is_err());
}

#[test]
fn every_admission_evidence_version_fails_closed() {
    for evidence in 0..6 {
        let mut r = request(RiskClass::Destructive, PolicyDecision::Allow, true);
        match evidence {
            0 => r.contract_version = ProtocolVersion::new(2, 0),
            1 => r.sandbox.contract_version = ProtocolVersion::new(2, 0),
            2 => r.risk_classification.contract_version = ProtocolVersion::new(2, 0),
            3 => r.authorization.contract_version = ProtocolVersion::new(2, 0),
            4 => r.assessment.contract_version = ProtocolVersion::new(2, 0),
            _ => r.confirmation.as_mut().unwrap().contract_version = ProtocolVersion::new(2, 0),
        }
        assert_eq!(
            admit_tool_execution(&r),
            Err(AdmissionError::UnsupportedVersion)
        );
    }
}
