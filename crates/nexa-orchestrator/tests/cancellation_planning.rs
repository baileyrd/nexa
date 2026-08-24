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
            assert_eq!(result.directives.len(), 1);
            assert_eq!(
                (result.directives[0].target, result.directives[0].directive),
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
    assert_eq!(plan(&w, &[]).unwrap().directives, vec![]);
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
        (p.workflow_id, p.session_id, p.correlation_id, p.trace_id),
        ids
    );
}

#[test]
fn closed_wire_and_diagnostics_are_exact() {
    for value in [
        E::UnsupportedVersion,
        E::WorkflowNotCancelled,
        E::AssociationMismatch,
        E::DuplicateTarget,
    ] {
        let json = serde_json::to_string(&value).unwrap();
        assert_eq!(serde_json::from_str::<E>(&json).unwrap(), value);
        assert!(serde_json::from_str::<E>(&json.replace("1.0", "2.0")).is_err());
    }
    assert!(serde_json::from_str::<T>(r#"{"version":"1.0","kind":"unknown"}"#).is_err());
    assert!(
        serde_json::from_str::<S>(r#"{"version":"1.0","kind":"cancellable","extra":0}"#).is_err()
    );
    assert!(
        serde_json::from_str::<D>(r#"{"version":"2.0","kind":"request_cancellation"}"#).is_err()
    );
    assert_eq!(format!("{:?}", E::DuplicateTarget), "DuplicateTarget");
    assert_eq!(
        E::DuplicateTarget.to_string(),
        "duplicate cancellation target"
    );
    assert_eq!(
        format!("{:?}", plan(&workflow(), &[]).unwrap_err()),
        "WorkflowNotCancelled"
    );
}
