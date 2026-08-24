use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
use nexa_orchestrator::{
    plan_workflow_cancellation, propagate_workflow_cancellation,
    ActiveCancellationTarget as Active, CancellationDirective as Directive,
    CancellationPropagationDependencyError as DependencyError,
    CancellationPropagationError as Error, CancellationSemantics as Semantics,
    CancellationTarget as Target, InteractionWorkflow,
    ScriptedCancellationPropagationOutcome as Outcome,
    ScriptedWorkflowCancellationPropagationPort as ScriptedPort,
    WorkflowCancellationAcknowledgement as Acknowledgement, WorkflowCancellationPlan,
};
use serde_json::{json, Value};
use uuid::Uuid;

fn id<T>(n: u128, make: impl FnOnce(Uuid) -> Result<T, nexa_domain::ValueError>) -> T {
    make(Uuid::from_u128(n)).unwrap()
}

fn plan(targets: &[Active]) -> WorkflowCancellationPlan {
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
    plan: &WorkflowCancellationPlan,
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

fn assert_error(
    result: Result<Acknowledgement, Error>,
    expected: Error,
    debug: &str,
    display: &str,
) {
    let error = result.unwrap_err();
    assert_eq!(error, expected);
    assert_eq!(format!("{error:?}"), debug);
    assert_eq!(error.to_string(), display);
}

fn canonical_json(directives: Value) -> Value {
    json!({
        "version": "1.0",
        "workflow_id": "00000000-0000-0000-0000-000000000001",
        "session_id": "00000000-0000-0000-0000-000000000002",
        "correlation_id": "00000000-0000-0000-0000-000000000003",
        "trace_id": "00000000-0000-0000-0000-000000000004",
        "directives": directives
    })
}

fn directive(target: &str, directive: &str) -> Value {
    json!({
        "version": "1.0",
        "target": {"version": "1.0", "kind": target},
        "directive": {"version": "1.0", "kind": directive}
    })
}

#[test]
fn acknowledgement_has_exact_canonical_json_and_round_trips_at_both_bounds() {
    let cases = [
        (
            vec![],
            canonical_json(json!([])),
            r#"{"version":"1.0","workflow_id":"00000000-0000-0000-0000-000000000001","session_id":"00000000-0000-0000-0000-000000000002","correlation_id":"00000000-0000-0000-0000-000000000003","trace_id":"00000000-0000-0000-0000-000000000004","directives":[]}"#,
        ),
        (
            vec![
                Active::new(Target::ToolExecution, Semantics::Cancellable),
                Active::new(Target::Behavior, Semantics::NonCancellable),
                Active::new(Target::Speech, Semantics::Cancellable),
                Active::new(Target::TutorGeneration, Semantics::NonCancellable),
                Active::new(Target::Retrieval, Semantics::Cancellable),
            ],
            canonical_json(json!([
                directive("retrieval", "request_cancellation"),
                directive("tutor_generation", "report_non_cancellable"),
                directive("speech", "request_cancellation"),
                directive("behavior", "report_non_cancellable"),
                directive("tool_execution", "request_cancellation")
            ])),
            r#"{"version":"1.0","workflow_id":"00000000-0000-0000-0000-000000000001","session_id":"00000000-0000-0000-0000-000000000002","correlation_id":"00000000-0000-0000-0000-000000000003","trace_id":"00000000-0000-0000-0000-000000000004","directives":[{"version":"1.0","target":{"version":"1.0","kind":"retrieval"},"directive":{"version":"1.0","kind":"request_cancellation"}},{"version":"1.0","target":{"version":"1.0","kind":"tutor_generation"},"directive":{"version":"1.0","kind":"report_non_cancellable"}},{"version":"1.0","target":{"version":"1.0","kind":"speech"},"directive":{"version":"1.0","kind":"request_cancellation"}},{"version":"1.0","target":{"version":"1.0","kind":"behavior"},"directive":{"version":"1.0","kind":"report_non_cancellable"}},{"version":"1.0","target":{"version":"1.0","kind":"tool_execution"},"directive":{"version":"1.0","kind":"request_cancellation"}}]}"#,
        ),
    ];
    for (targets, exact, exact_text) in cases {
        let acknowledgement = Acknowledgement::for_plan(&plan(&targets));
        assert_eq!(serde_json::to_string(&acknowledgement).unwrap(), exact_text);
        assert_eq!(serde_json::to_value(&acknowledgement).unwrap(), exact);
        assert_eq!(
            serde_json::from_value::<Acknowledgement>(exact).unwrap(),
            acknowledgement
        );
    }
}

#[test]
fn every_target_survives_the_exact_path_with_both_directive_semantics() {
    for target in [
        Target::Retrieval,
        Target::TutorGeneration,
        Target::Speech,
        Target::Behavior,
        Target::ToolExecution,
    ] {
        for (semantics, expected_directive) in [
            (Semantics::Cancellable, Directive::RequestCancellation),
            (Semantics::NonCancellable, Directive::ReportNonCancellable),
        ] {
            let plan = plan(&[Active::new(target, semantics)]);
            let expected = Acknowledgement::for_plan(&plan);
            let mut port = ScriptedPort::new([Outcome::Acknowledged(expected.clone())]);
            let acknowledgement = propagate(&mut port, &plan).unwrap();
            assert_eq!(port.received_plans(), std::slice::from_ref(&plan));
            assert_eq!(port.consumed_outcomes(), 1);
            assert_eq!(port.remaining_outcomes(), 0);
            assert_eq!(acknowledgement, expected);
            assert_eq!(acknowledgement.directives().len(), 1);
            assert_eq!(acknowledgement.directives()[0].target(), target);
            assert_eq!(
                acknowledgement.directives()[0].directive(),
                expected_directive
            );
        }
    }
}

#[test]
fn scripted_port_consumes_empty_and_all_target_plans_in_fifo_order() {
    let empty_plan = plan(&[]);
    let all_target_plan = plan(&[
        Active::new(Target::ToolExecution, Semantics::Cancellable),
        Active::new(Target::Behavior, Semantics::NonCancellable),
        Active::new(Target::Speech, Semantics::Cancellable),
        Active::new(Target::TutorGeneration, Semantics::NonCancellable),
        Active::new(Target::Retrieval, Semantics::Cancellable),
    ]);
    let empty_acknowledgement = Acknowledgement::for_plan(&empty_plan);
    let all_target_acknowledgement = Acknowledgement::for_plan(&all_target_plan);
    let mut port = ScriptedPort::new([
        Outcome::Acknowledged(empty_acknowledgement.clone()),
        Outcome::Acknowledged(all_target_acknowledgement.clone()),
    ]);

    assert_eq!(
        propagate(&mut port, &empty_plan).unwrap(),
        empty_acknowledgement
    );
    assert_eq!(port.received_plans(), std::slice::from_ref(&empty_plan));
    assert_eq!(port.consumed_outcomes(), 1);
    assert_eq!(port.remaining_outcomes(), 1);

    assert_eq!(
        propagate(&mut port, &all_target_plan).unwrap(),
        all_target_acknowledgement
    );
    assert_eq!(
        port.received_plans(),
        &[empty_plan.clone(), all_target_plan.clone()]
    );
    assert_eq!(port.consumed_outcomes(), 2);
    assert_eq!(port.remaining_outcomes(), 0);
    assert_eq!(
        all_target_plan
            .directives()
            .iter()
            .map(|directive| (directive.target(), directive.directive()))
            .collect::<Vec<_>>(),
        vec![
            (Target::Retrieval, Directive::RequestCancellation),
            (Target::TutorGeneration, Directive::ReportNonCancellable,),
            (Target::Speech, Directive::RequestCancellation),
            (Target::Behavior, Directive::ReportNonCancellable),
            (Target::ToolExecution, Directive::RequestCancellation),
        ]
    );
}

#[test]
fn acknowledgement_wire_rejects_each_nil_and_malformed_identity() {
    let original = canonical_json(json!([]));
    for field in ["workflow_id", "session_id", "correlation_id", "trace_id"] {
        for malformed in [json!(Uuid::nil()), json!("not-an-identity"), json!(7)] {
            let mut changed = original.clone();
            changed[field] = malformed;
            assert!(
                serde_json::from_value::<Acknowledgement>(changed).is_err(),
                "{field}"
            );
        }
    }
}

#[test]
fn acknowledgement_wire_rejects_malformed_nested_values_and_collection_shapes() {
    let original = canonical_json(json!([directive("retrieval", "request_cancellation")]));
    let mutations = [
        ("directives", json!(null)),
        ("directives", json!({})),
        ("directives", json!("not-a-collection")),
        ("directives.0", json!(null)),
        ("directives.0.target", json!("retrieval")),
        ("directives.0.directive", json!("request_cancellation")),
        ("directives.0.target.kind", json!(7)),
        ("directives.0.directive.kind", json!(false)),
        ("directives.0.target.version", json!({})),
        ("directives.0.directive.version", json!([])),
        ("directives.0.version", json!(1)),
    ];
    for (path, replacement) in mutations {
        let mut changed = original.clone();
        let mut cursor = &mut changed;
        for component in path.split('.') {
            cursor = if let Ok(index) = component.parse::<usize>() {
                &mut cursor[index]
            } else {
                &mut cursor[component]
            };
        }
        *cursor = replacement;
        assert!(
            serde_json::from_value::<Acknowledgement>(changed).is_err(),
            "{path}"
        );
    }
}

#[test]
fn acknowledgement_wire_rejects_unknown_versions_fields_variants_and_noncanonical_sets() {
    let one = directive("retrieval", "request_cancellation");
    let two = directive("speech", "report_non_cancellable");
    let original = canonical_json(json!([one.clone(), two.clone()]));
    let mut mutations = vec![];
    for path in [
        "version",
        "directives.0.version",
        "directives.0.target.version",
        "directives.0.directive.version",
    ] {
        let mut changed = original.clone();
        let mut cursor = &mut changed;
        for component in path.split('.') {
            cursor = if let Ok(index) = component.parse::<usize>() {
                &mut cursor[index]
            } else {
                &mut cursor[component]
            };
        }
        *cursor = json!("2.0");
        mutations.push(changed);
    }
    for path in ["directives.0.target.kind", "directives.0.directive.kind"] {
        let mut changed = original.clone();
        let mut cursor = &mut changed;
        for component in path.split('.') {
            cursor = if let Ok(index) = component.parse::<usize>() {
                &mut cursor[index]
            } else {
                &mut cursor[component]
            };
        }
        *cursor = json!("unknown_variant");
        mutations.push(changed);
    }
    for path in [
        "unknown",
        "directives.0.unknown",
        "directives.0.target.unknown",
        "directives.0.directive.unknown",
    ] {
        let mut changed = original.clone();
        let mut cursor = &mut changed;
        for component in path.split('.') {
            cursor = if let Ok(index) = component.parse::<usize>() {
                &mut cursor[index]
            } else {
                &mut cursor[component]
            };
        }
        *cursor = json!(true);
        mutations.push(changed);
    }
    mutations.push(canonical_json(json!([two.clone(), one.clone()])));
    mutations.push(canonical_json(json!([one.clone(), one.clone()])));
    mutations.push(canonical_json(json!([
        one.clone(),
        two.clone(),
        one.clone(),
        two.clone(),
        one.clone(),
        two
    ])));
    for changed in mutations {
        assert!(serde_json::from_value::<Acknowledgement>(changed).is_err());
    }
}

#[test]
fn operation_error_wire_is_exact_round_trip_and_strict() {
    let cases = [
        (Error::UnsupportedVersion, "unsupported_version"),
        (Error::AssociationMismatch, "association_mismatch"),
        (Error::InvalidPlan, "invalid_plan"),
        (Error::DependencyFailure, "dependency_failure"),
        (Error::AcknowledgementMismatch, "acknowledgement_mismatch"),
    ];
    for (error, kind) in cases {
        let exact = json!({"version": "1.0", "kind": kind});
        assert_eq!(
            serde_json::to_string(&error).unwrap(),
            format!(r#"{{"version":"1.0","kind":"{kind}"}}"#)
        );
        assert_eq!(serde_json::to_value(error).unwrap(), exact);
        assert_eq!(serde_json::from_value::<Error>(exact).unwrap(), error);
    }
    for invalid in [
        json!(null),
        json!([]),
        json!({}),
        json!({"version": "1.0"}),
        json!({"kind": "invalid_plan"}),
        json!({"version": "2.0", "kind": "invalid_plan"}),
        json!({"version": "1.0", "kind": "unknown"}),
        json!({"version": "1.0", "kind": 1}),
        json!({"version": 1, "kind": "invalid_plan"}),
        json!({"version": "1.0", "kind": "invalid_plan", "unknown": true}),
    ] {
        assert!(serde_json::from_value::<Error>(invalid).is_err());
    }
}

#[test]
fn dependency_error_is_intentionally_content_free_and_has_exact_diagnostics() {
    assert_eq!(
        format!("{:?}", DependencyError),
        "CancellationPropagationDependencyError"
    );
    assert_eq!(
        DependencyError.to_string(),
        "workflow cancellation propagation dependency failed"
    );
}

#[test]
fn every_trusted_identity_mismatch_returns_exact_operation_error_before_the_port() {
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
        assert_error(
            propagate_workflow_cancellation(
                &mut port,
                &plan,
                workflow_id,
                session_id,
                correlation_id,
                trace_id,
            ),
            Error::AssociationMismatch,
            "AssociationMismatch",
            "workflow cancellation propagation identity association mismatch",
        );
        assert_eq!(port.received_plans().len(), 0);
        assert_eq!(port.consumed_outcomes(), 0);
        assert_eq!(port.remaining_outcomes(), 1);
    }
}

#[test]
fn dependency_failure_and_exhaustion_have_exact_call_and_consumption_rules() {
    let plan = plan(&[]);
    let mut dependency = ScriptedPort::new([Outcome::DependencyFailure]);
    assert_error(
        propagate(&mut dependency, &plan),
        Error::DependencyFailure,
        "DependencyFailure",
        "workflow cancellation propagation dependency failed",
    );
    assert_eq!(dependency.received_plans(), std::slice::from_ref(&plan));
    assert_eq!(dependency.consumed_outcomes(), 1);
    assert_eq!(dependency.remaining_outcomes(), 0);

    let mut exhausted = ScriptedPort::new([]);
    assert_error(
        propagate(&mut exhausted, &plan),
        Error::DependencyFailure,
        "DependencyFailure",
        "workflow cancellation propagation dependency failed",
    );
    assert_eq!(exhausted.received_plans(), std::slice::from_ref(&plan));
    assert_eq!(exhausted.consumed_outcomes(), 0);
    assert_eq!(exhausted.remaining_outcomes(), 0);
}

#[test]
fn every_publicly_constructible_acknowledgement_mismatch_consumes_exactly_once() {
    let base = plan(&[Active::new(Target::Retrieval, Semantics::Cancellable)]);
    let base_json = serde_json::to_value(Acknowledgement::for_plan(&base)).unwrap();
    let mut acknowledgements = vec![];
    for field in ["workflow_id", "session_id", "correlation_id", "trace_id"] {
        let mut changed = base_json.clone();
        changed[field] = json!(Uuid::from_u128(9));
        acknowledgements.push(serde_json::from_value(changed).unwrap());
    }
    for targets in [
        vec![],
        vec![Active::new(Target::Retrieval, Semantics::NonCancellable)],
        vec![Active::new(Target::Speech, Semantics::Cancellable)],
        vec![
            Active::new(Target::Retrieval, Semantics::Cancellable),
            Active::new(Target::Speech, Semantics::Cancellable),
        ],
    ] {
        acknowledgements.push(Acknowledgement::for_plan(&plan(&targets)));
    }
    for acknowledgement in acknowledgements {
        let mut port = ScriptedPort::new([Outcome::Acknowledged(acknowledgement)]);
        assert_error(
            propagate(&mut port, &base),
            Error::AcknowledgementMismatch,
            "AcknowledgementMismatch",
            "workflow cancellation propagation acknowledgement mismatch",
        );
        assert_eq!(port.received_plans(), std::slice::from_ref(&base));
        assert_eq!(port.consumed_outcomes(), 1);
        assert_eq!(port.remaining_outcomes(), 0);
    }
}
