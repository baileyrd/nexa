use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
use nexa_orchestrator::{
    plan_workflow_cancellation, propagate_workflow_cancellation,
    ActiveCancellationTarget as Active, CancellationDirective as Directive,
    CancellationPropagationError as Error, CancellationSemantics as Semantics,
    CancellationTarget as Target, InteractionWorkflow,
    ScriptedCancellationPropagationOutcome as Outcome,
    ScriptedWorkflowCancellationPropagationPort as ScriptedPort,
    WorkflowCancellationAcknowledgement as Acknowledgement,
};
use uuid::Uuid;

fn id<T>(n: u128, make: impl FnOnce(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
    make(Uuid::from_u128(n)).unwrap()
}

fn plan(targets: &[Active]) -> nexa_orchestrator::WorkflowCancellationPlan {
    let workflow = InteractionWorkflow::new(
        id(1, WorkflowId::new),
        id(2, SessionId::new),
        id(3, CorrelationId::new),
        id(4, TraceId::new),
    )
    .cancel()
    .unwrap();
    plan_workflow_cancellation(
        &workflow,
        workflow.workflow_id(),
        workflow.session_id(),
        workflow.correlation_id(),
        workflow.trace_id(),
        targets,
    )
    .unwrap()
}

fn propagate(
    port: &mut ScriptedPort,
    plan: &nexa_orchestrator::WorkflowCancellationPlan,
) -> Result<Acknowledgement, Error> {
    propagate_workflow_cancellation(
        port,
        plan,
        plan.workflow_id(),
        plan.session_id(),
        plan.correlation_id(),
        plan.trace_id(),
    )
}

#[test]
fn empty_plan_is_handed_off_once_and_acknowledged_exactly() {
    let plan = plan(&[]);
    let expected = Acknowledgement::for_plan(&plan);
    let mut port = ScriptedPort::new([Outcome::Acknowledged(expected.clone())]);
    assert_eq!(propagate(&mut port, &plan), Ok(expected));
    assert_eq!(port.received_plans(), std::slice::from_ref(&plan));
    assert_eq!(port.consumed_outcomes(), 1);
    assert_eq!(port.remaining_outcomes(), 0);
}

#[test]
fn all_targets_and_both_directives_survive_the_exact_path() {
    let targets = [
        Active::new(Target::ToolExecution, Semantics::Cancellable),
        Active::new(Target::Behavior, Semantics::NonCancellable),
        Active::new(Target::Speech, Semantics::Cancellable),
        Active::new(Target::TutorGeneration, Semantics::NonCancellable),
        Active::new(Target::Retrieval, Semantics::Cancellable),
    ];
    let plan = plan(&targets);
    let expected = Acknowledgement::for_plan(&plan);
    let mut port = ScriptedPort::new([Outcome::Acknowledged(expected.clone())]);
    let acknowledgement = propagate(&mut port, &plan).unwrap();
    assert_eq!(port.received_plans(), std::slice::from_ref(&plan));
    assert_eq!(acknowledgement, expected);
    assert_eq!(acknowledgement.directives().len(), 5);
    assert_eq!(acknowledgement.directives()[0].target(), Target::Retrieval);
    assert_eq!(
        acknowledgement.directives()[4].target(),
        Target::ToolExecution
    );
    assert!(acknowledgement
        .directives()
        .iter()
        .any(|entry| entry.directive() == Directive::RequestCancellation));
    assert!(acknowledgement
        .directives()
        .iter()
        .any(|entry| entry.directive() == Directive::ReportNonCancellable));
}

#[test]
fn every_trusted_identity_mismatch_precedes_consumption() {
    let plan = plan(&[]);
    let acknowledgement = Acknowledgement::for_plan(&plan);
    let cases = [
        (
            id(9, WorkflowId::new),
            plan.session_id(),
            plan.correlation_id(),
            plan.trace_id(),
        ),
        (
            plan.workflow_id(),
            id(9, SessionId::new),
            plan.correlation_id(),
            plan.trace_id(),
        ),
        (
            plan.workflow_id(),
            plan.session_id(),
            id(9, CorrelationId::new),
            plan.trace_id(),
        ),
        (
            plan.workflow_id(),
            plan.session_id(),
            plan.correlation_id(),
            id(9, TraceId::new),
        ),
    ];
    for (workflow_id, session_id, correlation_id, trace_id) in cases {
        let mut port = ScriptedPort::new([Outcome::Acknowledged(acknowledgement.clone())]);
        assert_eq!(
            propagate_workflow_cancellation(
                &mut port,
                &plan,
                workflow_id,
                session_id,
                correlation_id,
                trace_id
            ),
            Err(Error::AssociationMismatch)
        );
        assert_eq!(port.consumed_outcomes(), 0);
        assert!(port.received_plans().is_empty());
    }
}

#[test]
fn each_acknowledgement_identity_change_is_rejected_after_one_consumption() {
    let plan = plan(&[]);
    let json = serde_json::to_value(Acknowledgement::for_plan(&plan)).unwrap();
    for field in ["workflow_id", "session_id", "correlation_id", "trace_id"] {
        let mut changed = json.clone();
        changed[field] = serde_json::json!(Uuid::from_u128(9));
        let acknowledgement = serde_json::from_value(changed).unwrap();
        let mut port = ScriptedPort::new([Outcome::Acknowledged(acknowledgement)]);
        assert_eq!(
            propagate(&mut port, &plan),
            Err(Error::AcknowledgementMismatch)
        );
        assert_eq!(port.consumed_outcomes(), 1);
    }
}

#[test]
fn acknowledgement_directive_changes_and_strict_wire_fail_closed() {
    let plan = plan(&[
        Active::new(Target::Retrieval, Semantics::Cancellable),
        Active::new(Target::Speech, Semantics::NonCancellable),
    ]);
    let original = serde_json::to_value(Acknowledgement::for_plan(&plan)).unwrap();
    for directives in [
        serde_json::json!([]),
        serde_json::json!([original["directives"][0]]),
        {
            let mut changed = original["directives"].clone();
            changed[0]["directive"]["kind"] = serde_json::json!("report_non_cancellable");
            changed
        },
        {
            let mut changed = original["directives"].clone();
            changed[0]["target"]["kind"] = serde_json::json!("tutor_generation");
            changed
        },
    ] {
        let mut changed = original.clone();
        changed["directives"] = directives;
        let acknowledgement = serde_json::from_value(changed).unwrap();
        let mut port = ScriptedPort::new([Outcome::Acknowledged(acknowledgement)]);
        assert_eq!(
            propagate(&mut port, &plan),
            Err(Error::AcknowledgementMismatch)
        );
    }

    for changed in [
        {
            let mut value = original.clone();
            value["version"] = serde_json::json!("2.0");
            value
        },
        {
            let mut value = original.clone();
            value["unknown"] = serde_json::json!(true);
            value
        },
        {
            let mut value = original.clone();
            value["directives"] =
                serde_json::json!([original["directives"][1], original["directives"][0]]);
            value
        },
        {
            let mut value = original.clone();
            value["directives"] =
                serde_json::json!([original["directives"][0], original["directives"][0]]);
            value
        },
    ] {
        assert!(serde_json::from_value::<Acknowledgement>(changed).is_err());
    }
    let mut over_bound = original;
    over_bound["directives"] = serde_json::json!([
        over_bound["directives"][0],
        over_bound["directives"][1],
        over_bound["directives"][0],
        over_bound["directives"][1],
        over_bound["directives"][0],
        over_bound["directives"][1]
    ]);
    assert!(serde_json::from_value::<Acknowledgement>(over_bound).is_err());
    assert!(serde_json::from_str::<Acknowledgement>("null").is_err());
}

#[test]
fn scripted_outcomes_are_fifo_normalized_and_exhaust_deterministically() {
    let plan = plan(&[]);
    let acknowledgement = Acknowledgement::for_plan(&plan);
    let mut port = ScriptedPort::new([
        Outcome::DependencyFailure,
        Outcome::Acknowledged(acknowledgement.clone()),
    ]);
    assert_eq!(propagate(&mut port, &plan), Err(Error::DependencyFailure));
    assert_eq!(propagate(&mut port, &plan), Ok(acknowledgement));
    assert_eq!(propagate(&mut port, &plan), Err(Error::DependencyFailure));
    assert_eq!(port.consumed_outcomes(), 2);
    assert_eq!(port.received_plans().len(), 3);
}

#[test]
fn operation_errors_have_exact_content_free_diagnostics() {
    let cases = [
        (
            Error::UnsupportedVersion,
            "UnsupportedVersion",
            "unsupported workflow cancellation propagation version",
        ),
        (
            Error::AssociationMismatch,
            "AssociationMismatch",
            "workflow cancellation propagation identity association mismatch",
        ),
        (
            Error::InvalidPlan,
            "InvalidPlan",
            "invalid workflow cancellation propagation plan",
        ),
        (
            Error::DependencyFailure,
            "DependencyFailure",
            "workflow cancellation propagation dependency failed",
        ),
        (
            Error::AcknowledgementMismatch,
            "AcknowledgementMismatch",
            "workflow cancellation propagation acknowledgement mismatch",
        ),
    ];
    for (error, debug, display) in cases {
        assert_eq!(format!("{error:?}"), debug);
        assert_eq!(error.to_string(), display);
    }
}
