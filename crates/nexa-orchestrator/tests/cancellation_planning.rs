use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
use nexa_orchestrator::{
    plan_workflow_cancellation, ActiveCancellationTarget as Active, CancellationDirective as D,
    CancellationPlanningError as E, CancellationSemantics as S, CancellationTarget as T,
    InteractionWorkflow, WorkflowState,
};
use uuid::Uuid;

fn id<T>(n: u128, make: impl FnOnce(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
    make(Uuid::from_u128(n)).unwrap()
}
fn workflow() -> InteractionWorkflow {
    InteractionWorkflow::new(
        id(1, WorkflowId::new),
        id(2, SessionId::new),
        id(3, CorrelationId::new),
        id(4, TraceId::new),
    )
}
fn plan(
    w: &InteractionWorkflow,
    targets: &[Active],
) -> Result<nexa_orchestrator::WorkflowCancellationPlan, E> {
    plan_workflow_cancellation(
        w,
        w.workflow_id(),
        w.session_id(),
        w.correlation_id(),
        w.trace_id(),
        targets,
    )
}

#[test]
fn every_target_and_semantics_maps_to_exactly_one_directive() {
    let w = workflow().cancel().unwrap();
    for target in [
        T::Retrieval,
        T::TutorGeneration,
        T::Speech,
        T::Behavior,
        T::ToolExecution,
    ] {
        for (semantics, expected) in [
            (S::Cancellable, D::RequestCancellation),
            (S::NonCancellable, D::ReportNonCancellable),
        ] {
            let result = plan(&w, &[Active::new(target, semantics)]).unwrap();
            assert_eq!(result.directives().len(), 1);
            assert_eq!(
                (
                    result.directives()[0].target(),
                    result.directives()[0].directive()
                ),
                (target, expected)
            );
        }
    }
}

#[test]
fn ordering_duplicates_and_empty_sets_are_deterministic() {
    let w = workflow().cancel().unwrap();
    let forward = [
        Active::new(T::Retrieval, S::Cancellable),
        Active::new(T::Speech, S::NonCancellable),
        Active::new(T::ToolExecution, S::Cancellable),
    ];
    let reverse = [forward[2], forward[1], forward[0]];
    assert_eq!(plan(&w, &forward), plan(&w, &reverse));
    assert_eq!(plan(&w, &[]).unwrap().directives(), &[]);
    assert_eq!(
        plan(
            &w,
            &[forward[0], Active::new(T::Retrieval, S::NonCancellable)]
        ),
        Err(E::DuplicateTarget)
    );
}

#[test]
fn only_cancelled_workflows_are_accepted() {
    let mut w = workflow();
    let states = [
        WorkflowState::Created,
        WorkflowState::NormalizingInput,
        WorkflowState::PreparingContext,
        WorkflowState::SelectingPedagogy,
        WorkflowState::RetrievingKnowledge,
        WorkflowState::GeneratingTutorResponse,
    ];
    for state in states {
        assert_eq!(w.state(), state);
        assert_eq!(plan(&w, &[]), Err(E::WorkflowNotCancelled));
        if state != WorkflowState::GeneratingTutorResponse {
            w = w
                .advance(states[states.iter().position(|v| *v == state).unwrap() + 1])
                .unwrap();
        }
    }
    for state in [
        WorkflowState::ExecutingTools,
        WorkflowState::PlanningResponse,
        WorkflowState::Speaking,
        WorkflowState::WaitingForStudent,
        WorkflowState::Completed,
        WorkflowState::Failed,
    ] {
        let x = match state {
            WorkflowState::ExecutingTools => w.advance(state).unwrap(),
            WorkflowState::PlanningResponse => w.advance(state).unwrap(),
            WorkflowState::Speaking => w
                .advance(WorkflowState::PlanningResponse)
                .unwrap()
                .advance(state)
                .unwrap(),
            WorkflowState::WaitingForStudent => w
                .advance(WorkflowState::PlanningResponse)
                .unwrap()
                .advance(state)
                .unwrap(),
            WorkflowState::Completed => w
                .advance(WorkflowState::PlanningResponse)
                .unwrap()
                .advance(WorkflowState::WaitingForStudent)
                .unwrap()
                .advance(state)
                .unwrap(),
            WorkflowState::Failed => w.fail().unwrap(),
            _ => unreachable!(),
        };
        assert_eq!(plan(&x, &[]), Err(E::WorkflowNotCancelled));
    }
    assert!(plan(&w.cancel().unwrap(), &[]).is_ok());
}

#[test]
fn reassociation_is_rejected_and_identities_round_trip() {
    let w = workflow().cancel().unwrap();
    let ids = (
        w.workflow_id(),
        w.session_id(),
        w.correlation_id(),
        w.trace_id(),
    );
    assert_eq!(
        plan_workflow_cancellation(&w, id(9, WorkflowId::new), ids.1, ids.2, ids.3, &[]),
        Err(E::AssociationMismatch)
    );
    assert_eq!(
        plan_workflow_cancellation(&w, ids.0, id(9, SessionId::new), ids.2, ids.3, &[]),
        Err(E::AssociationMismatch)
    );
    assert_eq!(
        plan_workflow_cancellation(&w, ids.0, ids.1, id(9, CorrelationId::new), ids.3, &[]),
        Err(E::AssociationMismatch)
    );
    assert_eq!(
        plan_workflow_cancellation(&w, ids.0, ids.1, ids.2, id(9, TraceId::new), &[]),
        Err(E::AssociationMismatch)
    );
    let p = plan(&w, &[Active::new(T::Behavior, S::Cancellable)]).unwrap();
    let round: nexa_orchestrator::WorkflowCancellationPlan =
        serde_json::from_str(&serde_json::to_string(&p).unwrap()).unwrap();
    assert_eq!(round, p);
    assert_eq!(
        (
            p.workflow_id(),
            p.session_id(),
            p.correlation_id(),
            p.trace_id()
        ),
        ids
    );
}

fn assert_rejected<T: serde::de::DeserializeOwned>(json: &str) {
    assert!(serde_json::from_str::<T>(json).is_err(), "accepted {json}");
}

#[test]
fn all_five_targets_succeed_and_six_entries_fail_before_planning() {
    let w = workflow().cancel().unwrap();
    let all = [
        Active::new(T::Retrieval, S::Cancellable),
        Active::new(T::TutorGeneration, S::NonCancellable),
        Active::new(T::Speech, S::Cancellable),
        Active::new(T::Behavior, S::NonCancellable),
        Active::new(T::ToolExecution, S::Cancellable),
    ];
    assert_eq!(plan(&w, &all).unwrap().directives().len(), 5);
    let six = [all[0], all[1], all[2], all[3], all[4], all[0]];
    assert_eq!(plan(&w, &six), Err(E::TooManyTargets));
}

#[test]
fn every_closed_enum_has_exact_strict_wire_behavior() {
    let targets = [
        (T::Retrieval, "retrieval"),
        (T::TutorGeneration, "tutor_generation"),
        (T::Speech, "speech"),
        (T::Behavior, "behavior"),
        (T::ToolExecution, "tool_execution"),
    ];
    for (value, kind) in targets {
        let json = format!(r#"{{"version":"1.0","kind":"{kind}"}}"#);
        assert_eq!(serde_json::to_string(&value).unwrap(), json);
        assert_eq!(serde_json::from_str::<T>(&json).unwrap(), value);
    }
    for (value, kind) in [
        (S::Cancellable, "cancellable"),
        (S::NonCancellable, "non_cancellable"),
    ] {
        let json = format!(r#"{{"version":"1.0","kind":"{kind}"}}"#);
        assert_eq!(serde_json::to_string(&value).unwrap(), json);
        assert_eq!(serde_json::from_str::<S>(&json).unwrap(), value);
    }
    for (value, kind) in [
        (D::RequestCancellation, "request_cancellation"),
        (D::ReportNonCancellable, "report_non_cancellable"),
    ] {
        let json = format!(r#"{{"version":"1.0","kind":"{kind}"}}"#);
        assert_eq!(serde_json::to_string(&value).unwrap(), json);
        assert_eq!(serde_json::from_str::<D>(&json).unwrap(), value);
    }
    for (value, kind) in [
        (E::UnsupportedVersion, "unsupported_version"),
        (E::WorkflowNotCancelled, "workflow_not_cancelled"),
        (E::AssociationMismatch, "association_mismatch"),
        (E::DuplicateTarget, "duplicate_target"),
        (E::TooManyTargets, "too_many_targets"),
    ] {
        let json = format!(r#"{{"version":"1.0","kind":"{kind}"}}"#);
        assert_eq!(serde_json::to_string(&value).unwrap(), json);
        assert_eq!(serde_json::from_str::<E>(&json).unwrap(), value);
    }

    macro_rules! reject_closed {
        ($ty:ty, $kind:literal) => {{
            assert_rejected::<$ty>(concat!(r#"{"version":"2.0","kind":""#, $kind, r#""}"#));
            assert_rejected::<$ty>(concat!(
                r#"{"version":"1.0","kind":""#,
                $kind,
                r#"","extra":0}"#
            ));
            assert_rejected::<$ty>(r#"{"version":"1.0","kind":"unknown"}"#);
            assert_rejected::<$ty>(r#"{"version":1,"kind":false}"#);
        }};
    }
    reject_closed!(T, "retrieval");
    reject_closed!(S, "cancellable");
    reject_closed!(D, "request_cancellation");
    reject_closed!(E, "duplicate_target");
}

#[test]
fn active_and_directive_forms_are_strict_v1_contracts() {
    let active = Active::new(T::Speech, S::NonCancellable);
    let active_json = r#"{"version":"1.0","target":{"version":"1.0","kind":"speech"},"semantics":{"version":"1.0","kind":"non_cancellable"}}"#;
    assert_eq!(serde_json::to_string(&active).unwrap(), active_json);
    assert_eq!(serde_json::from_str::<Active>(active_json).unwrap(), active);
    assert_eq!(active.version().to_string(), "1.0");
    assert_eq!(active.target(), T::Speech);
    assert_eq!(active.semantics(), S::NonCancellable);
    for json in [
        active_json.replace(r#""1.0","target"#, r#""2.0","target"#),
        active_json.replace("}", r#","extra":0}"#),
        active_json.replace(
            r#""semantics":{"version":"1.0","kind":"non_cancellable"}"#,
            r#""semantics":false"#,
        ),
    ] {
        assert_rejected::<Active>(&json);
    }

    let w = workflow().cancel().unwrap();
    let directive = plan(&w, &[active]).unwrap().directives()[0];
    let directive_json = r#"{"version":"1.0","target":{"version":"1.0","kind":"speech"},"directive":{"version":"1.0","kind":"report_non_cancellable"}}"#;
    assert_eq!(serde_json::to_string(&directive).unwrap(), directive_json);
    assert_eq!(
        serde_json::from_str::<nexa_orchestrator::PlannedCancellationDirective>(directive_json)
            .unwrap(),
        directive
    );
    assert_eq!(directive.version().to_string(), "1.0");
    for json in [
        directive_json.replace(r#""1.0","target"#, r#""2.0","target"#),
        directive_json.replace("}", r#","extra":0}"#),
        directive_json.replace(
            r#""directive":{"version":"1.0","kind":"report_non_cancellable"}"#,
            r#""directive":[]"#,
        ),
    ] {
        assert_rejected::<nexa_orchestrator::PlannedCancellationDirective>(&json);
    }
}

#[test]
fn canonical_all_target_plan_has_exact_wire_identity_and_directives() {
    let w = workflow().cancel().unwrap();
    let targets = [
        Active::new(T::ToolExecution, S::Cancellable),
        Active::new(T::Behavior, S::NonCancellable),
        Active::new(T::Speech, S::Cancellable),
        Active::new(T::TutorGeneration, S::NonCancellable),
        Active::new(T::Retrieval, S::Cancellable),
    ];
    let value = plan(&w, &targets).unwrap();
    let json = concat!(
        r#"{"version":"1.0","workflow_id":"00000000-0000-0000-0000-000000000001","session_id":"00000000-0000-0000-0000-000000000002","correlation_id":"00000000-0000-0000-0000-000000000003","trace_id":"00000000-0000-0000-0000-000000000004","directives":["#,
        r#"{"version":"1.0","target":{"version":"1.0","kind":"retrieval"},"directive":{"version":"1.0","kind":"request_cancellation"}},"#,
        r#"{"version":"1.0","target":{"version":"1.0","kind":"tutor_generation"},"directive":{"version":"1.0","kind":"report_non_cancellable"}},"#,
        r#"{"version":"1.0","target":{"version":"1.0","kind":"speech"},"directive":{"version":"1.0","kind":"request_cancellation"}},"#,
        r#"{"version":"1.0","target":{"version":"1.0","kind":"behavior"},"directive":{"version":"1.0","kind":"report_non_cancellable"}},"#,
        r#"{"version":"1.0","target":{"version":"1.0","kind":"tool_execution"},"directive":{"version":"1.0","kind":"request_cancellation"}}]}"#
    );
    assert_eq!(serde_json::to_string(&value).unwrap(), json);
    assert_eq!(
        serde_json::from_str::<nexa_orchestrator::WorkflowCancellationPlan>(json).unwrap(),
        value
    );
    assert_eq!(value.version().to_string(), "1.0");
    assert_eq!(
        (
            value.workflow_id(),
            value.session_id(),
            value.correlation_id(),
            value.trace_id()
        ),
        (
            w.workflow_id(),
            w.session_id(),
            w.correlation_id(),
            w.trace_id()
        )
    );
}

#[test]
fn plan_wire_rejects_every_invalid_shape_and_identity() {
    type Plan = nexa_orchestrator::WorkflowCancellationPlan;
    let w = workflow().cancel().unwrap();
    let canonical = serde_json::to_value(
        plan(
            &w,
            &[
                Active::new(T::Retrieval, S::Cancellable),
                Active::new(T::Speech, S::Cancellable),
            ],
        )
        .unwrap(),
    )
    .unwrap();
    for (key, value) in [
        ("version", serde_json::json!("2.0")),
        ("workflow_id", serde_json::json!(Uuid::nil())),
        ("session_id", serde_json::json!(Uuid::nil())),
        ("correlation_id", serde_json::json!(Uuid::nil())),
        ("trace_id", serde_json::json!(Uuid::nil())),
        ("directives", serde_json::json!(false)),
    ] {
        let mut invalid = canonical.clone();
        invalid[key] = value;
        assert_rejected::<Plan>(&invalid.to_string());
    }
    let mut unknown = canonical.clone();
    unknown["extra"] = serde_json::json!(0);
    assert_rejected::<Plan>(&unknown.to_string());

    let directives = canonical["directives"].as_array().unwrap().clone();
    let mut duplicate = canonical.clone();
    duplicate["directives"] = serde_json::json!([directives[0], directives[0]]);
    assert_rejected::<Plan>(&duplicate.to_string());
    let mut reversed = canonical.clone();
    reversed["directives"] = serde_json::json!([directives[1], directives[0]]);
    assert_rejected::<Plan>(&reversed.to_string());
    let mut over_bound = canonical;
    over_bound["directives"] = serde_json::json!([
        directives[0],
        directives[1],
        directives[0],
        directives[1],
        directives[0],
        directives[1]
    ]);
    assert_rejected::<Plan>(&over_bound.to_string());
}

fn assert_diagnostics(error: E, debug: &str, display: &str) {
    assert_eq!(format!("{error:?}"), debug);
    assert_eq!(error.to_string(), display);
}

#[test]
fn every_operation_failure_has_exact_content_free_diagnostics() {
    let w = workflow().cancel().unwrap();
    assert_diagnostics(
        Active::try_new(
            nexa_domain::ProtocolVersion::new(2, 0),
            T::Speech,
            S::Cancellable,
        )
        .unwrap_err(),
        "UnsupportedVersion",
        "unsupported cancellation propagation version",
    );

    assert_diagnostics(
        plan(&workflow(), &[]).unwrap_err(),
        "WorkflowNotCancelled",
        "workflow is not cancelled",
    );
    let ids = (
        w.workflow_id(),
        w.session_id(),
        w.correlation_id(),
        w.trace_id(),
    );
    for error in [
        plan_workflow_cancellation(&w, id(9, WorkflowId::new), ids.1, ids.2, ids.3, &[])
            .unwrap_err(),
        plan_workflow_cancellation(&w, ids.0, id(9, SessionId::new), ids.2, ids.3, &[])
            .unwrap_err(),
        plan_workflow_cancellation(&w, ids.0, ids.1, id(9, CorrelationId::new), ids.3, &[])
            .unwrap_err(),
        plan_workflow_cancellation(&w, ids.0, ids.1, ids.2, id(9, TraceId::new), &[]).unwrap_err(),
    ] {
        assert_diagnostics(
            error,
            "AssociationMismatch",
            "cancellation propagation identity association mismatch",
        );
    }
    let duplicate = [Active::new(T::Speech, S::Cancellable); 2];
    assert_diagnostics(
        plan(&w, &duplicate).unwrap_err(),
        "DuplicateTarget",
        "duplicate cancellation target",
    );
    let over_bound = [Active::new(T::Speech, S::Cancellable); 6];
    assert_diagnostics(
        plan(&w, &over_bound).unwrap_err(),
        "TooManyTargets",
        "too many cancellation targets",
    );
}
