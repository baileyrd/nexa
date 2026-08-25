use nexa_domain::{
    EnvironmentInstanceId, LabSessionId, ProtocolVersion, SemanticKey, ToolExecutionId,
    ToolRequestId,
};
use nexa_labs::*;
use std::{
    future::Future,
    pin::pin,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
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
        environment_instance_id: a.environment_instance_id,
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
        risk,
        authorization: AuthorizationDecision {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a.clone(),
            decision,
        },
        assessment_decision: PolicyDecision::Allow,
        confirmation: confirmation.then_some(ConfirmationEvidence {
            contract_version: TOOL_EXECUTION_SECURITY_V1,
            association: a,
            confirmed: true,
        }),
        tutor_preference: TutorPreference::Prefer,
    }
}
fn admitted() -> AdmittedToolExecution {
    admit_tool_execution(&request(RiskClass::ReadOnly, PolicyDecision::Allow, false)).unwrap()
}
fn block<F: Future>(f: F) -> F::Output {
    struct W;
    impl Wake for W {
        fn wake(self: Arc<Self>) {}
    }
    let w = Waker::from(Arc::new(W));
    let mut c = Context::from_waker(&w);
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
    r.assessment_decision = PolicyDecision::Deny;
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
    struct W;
    impl Wake for W {
        fn wake(self: Arc<Self>) {}
    }
    let a = association();
    let cap = ToolCancellationCapability {
        contract_version: TOOL_EXECUTION_SECURITY_V1,
        association: a,
        semantics: CancellationSemantics::Cancellable,
    };
    let c = ScriptedToolCancellationControl::new([ScriptedCancellationOutcome::Pending]);
    let admitted = admitted();
    let mut f = Box::pin(cancel_tool_execution(&cap, &admitted, &c));
    let w = Waker::from(Arc::new(W));
    assert!(matches!(
        f.as_mut().poll(&mut Context::from_waker(&w)),
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
