use nexa_domain::{CorrelationId, SessionId, TraceId, WorkflowId};
use nexa_orchestrator::{
    InteractionWorkflow, RuntimeSessionState as S, SessionTransitionError, WorkflowLifecycleError,
    WorkflowState as W,
};
use uuid::Uuid;

const S_STATES: [S; 9] = [
    S::Created,
    S::Initializing,
    S::Ready,
    S::Active,
    S::Paused,
    S::Degraded,
    S::Ending,
    S::Completed,
    S::Failed,
];
const W_STATES: [W; 13] = [
    W::Created,
    W::NormalizingInput,
    W::PreparingContext,
    W::SelectingPedagogy,
    W::RetrievingKnowledge,
    W::GeneratingTutorResponse,
    W::ExecutingTools,
    W::PlanningResponse,
    W::Speaking,
    W::WaitingForStudent,
    W::Completed,
    W::Cancelled,
    W::Failed,
];
const S_WIRE: [(S, &str); 9] = [
    (S::Created, "created"),
    (S::Initializing, "initializing"),
    (S::Ready, "ready"),
    (S::Active, "active"),
    (S::Paused, "paused"),
    (S::Degraded, "degraded"),
    (S::Ending, "ending"),
    (S::Completed, "completed"),
    (S::Failed, "failed"),
];
const W_WIRE: [(W, &str); 13] = [
    (W::Created, "created"),
    (W::NormalizingInput, "normalizing_input"),
    (W::PreparingContext, "preparing_context"),
    (W::SelectingPedagogy, "selecting_pedagogy"),
    (W::RetrievingKnowledge, "retrieving_knowledge"),
    (W::GeneratingTutorResponse, "generating_tutor_response"),
    (W::ExecutingTools, "executing_tools"),
    (W::PlanningResponse, "planning_response"),
    (W::Speaking, "speaking"),
    (W::WaitingForStudent, "waiting_for_student"),
    (W::Completed, "completed"),
    (W::Cancelled, "cancelled"),
    (W::Failed, "failed"),
];
const SESSION_LEGAL: [(S, S); 11] = [
    (S::Created, S::Initializing),
    (S::Initializing, S::Ready),
    (S::Ready, S::Active),
    (S::Active, S::Paused),
    (S::Paused, S::Active),
    (S::Active, S::Degraded),
    (S::Degraded, S::Active),
    (S::Active, S::Ending),
    (S::Paused, S::Ending),
    (S::Degraded, S::Ending),
    (S::Ending, S::Completed),
];
const WORKFLOW_LEGAL: [(W, W); 12] = [
    (W::Created, W::NormalizingInput),
    (W::NormalizingInput, W::PreparingContext),
    (W::PreparingContext, W::SelectingPedagogy),
    (W::SelectingPedagogy, W::RetrievingKnowledge),
    (W::RetrievingKnowledge, W::GeneratingTutorResponse),
    (W::GeneratingTutorResponse, W::ExecutingTools),
    (W::GeneratingTutorResponse, W::PlanningResponse),
    (W::ExecutingTools, W::PlanningResponse),
    (W::PlanningResponse, W::Speaking),
    (W::PlanningResponse, W::WaitingForStudent),
    (W::Speaking, W::WaitingForStudent),
    (W::WaitingForStudent, W::Completed),
];

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
fn identities(w: InteractionWorkflow) -> (WorkflowId, SessionId, CorrelationId, TraceId) {
    (
        w.workflow_id(),
        w.session_id(),
        w.correlation_id(),
        w.trace_id(),
    )
}
fn at(target: W) -> InteractionWorkflow {
    if target == W::Created {
        return workflow();
    }
    let mut w = workflow();
    for (_, next) in WORKFLOW_LEGAL[..5].iter() {
        w = w.advance(*next).unwrap();
        if *next == target {
            return w;
        }
    }
    match target {
        W::ExecutingTools => w.advance(W::ExecutingTools).unwrap(),
        W::PlanningResponse => w.advance(W::PlanningResponse).unwrap(),
        W::Speaking => w
            .advance(W::PlanningResponse)
            .unwrap()
            .advance(W::Speaking)
            .unwrap(),
        W::WaitingForStudent => w
            .advance(W::PlanningResponse)
            .unwrap()
            .advance(W::WaitingForStudent)
            .unwrap(),
        W::Completed => w
            .advance(W::PlanningResponse)
            .unwrap()
            .advance(W::WaitingForStudent)
            .unwrap()
            .advance(W::Completed)
            .unwrap(),
        W::Cancelled => w.cancel().unwrap(),
        W::Failed => w.fail().unwrap(),
        _ => unreachable!(),
    }
}

#[test]
fn session_transition_matrix_is_exhaustive() {
    for from in S_STATES {
        for to in S_STATES {
            let expected = SESSION_LEGAL.contains(&(from, to))
                || (!matches!(from, S::Completed | S::Failed) && to == S::Failed);
            assert_eq!(
                from.transition_to(to).ok(),
                expected.then_some(to),
                "{from:?}->{to:?}"
            );
        }
    }
}
#[test]
fn every_nonterminal_session_can_fail_and_terminals_are_immutable() {
    for state in S_STATES {
        if !matches!(state, S::Completed | S::Failed) {
            assert_eq!(state.transition_to(S::Failed), Ok(S::Failed));
        }
    }
    for terminal in [S::Completed, S::Failed] {
        for next in S_STATES {
            assert_eq!(terminal.transition_to(next), Err(SessionTransitionError));
        }
    }
}
#[test]
fn workflow_transition_matrix_is_exhaustive_and_preserves_identity() {
    for from in W_STATES {
        for to in W_STATES {
            let before = at(from);
            let result = before.advance(to);
            let expected = WORKFLOW_LEGAL.contains(&(from, to));
            assert_eq!(result.is_ok(), expected, "{from:?}->{to:?}");
            if let Ok(after) = result {
                assert_eq!(
                    (
                        after.workflow_id(),
                        after.session_id(),
                        after.correlation_id(),
                        after.trace_id()
                    ),
                    (
                        before.workflow_id(),
                        before.session_id(),
                        before.correlation_id(),
                        before.trace_id()
                    )
                );
            }
        }
    }
}
#[test]
fn cancellation_covers_every_nonterminal_and_is_idempotent() {
    for state in W_STATES {
        let w = at(state);
        match state {
            W::Completed | W::Failed => {
                assert_eq!(w.cancel(), Err(WorkflowLifecycleError::IllegalTransition))
            }
            W::Cancelled => assert_eq!(w.cancel(), Ok(w)),
            _ => {
                let cancelled = w.cancel().unwrap();
                assert_eq!(cancelled.state(), W::Cancelled);
                assert_eq!(identities(cancelled), identities(w));
                let repeated = cancelled.cancel().unwrap();
                assert_eq!(repeated, cancelled);
                assert_eq!(identities(repeated), identities(w));
            }
        }
    }
}
#[test]
fn failure_covers_every_live_workflow_and_terminals_reject_operations() {
    for state in W_STATES {
        let w = at(state);
        if matches!(state, W::Completed | W::Cancelled | W::Failed) {
            assert_eq!(w.fail(), Err(WorkflowLifecycleError::IllegalTransition));
        } else {
            let failed = w.fail().unwrap();
            assert_eq!(failed.state(), W::Failed);
            assert_eq!(identities(failed), identities(w));
        }
    }
    for terminal in [W::Completed, W::Failed] {
        let w = at(terminal);
        for next in W_STATES {
            assert_eq!(
                w.advance(next),
                Err(WorkflowLifecycleError::IllegalTransition)
            );
        }
    }
}
#[test]
fn state_vocabularies_have_exact_closed_wire_round_trips() {
    for (state, name) in S_WIRE {
        let json = format!("\"{name}\"");
        assert_eq!(serde_json::to_string(&state).unwrap(), json);
        assert_eq!(serde_json::from_str::<S>(&json).unwrap(), state);
    }
    for (state, name) in W_WIRE {
        let json = format!("\"{name}\"");
        assert_eq!(serde_json::to_string(&state).unwrap(), json);
        assert_eq!(serde_json::from_str::<W>(&json).unwrap(), state);
    }
    assert!(serde_json::from_str::<S>("\"future\"").is_err());
    assert!(serde_json::from_str::<W>("\"future\"").is_err());
}

#[test]
fn wire_is_stable_strict_validating_and_preserves_associations() {
    let w = at(W::Speaking);
    let json = serde_json::to_string(&w).unwrap();
    let decoded = serde_json::from_str::<InteractionWorkflow>(&json).unwrap();
    assert_eq!(decoded, w);
    assert_eq!(identities(decoded), identities(w));
    assert!(
        serde_json::from_str::<InteractionWorkflow>(&json.replace("\"1.0\"", "\"2.0\"")).is_err()
    );
    assert!(serde_json::from_str::<InteractionWorkflow>(
        &json.replace("\"speaking\"", "\"future\"")
    )
    .is_err());
    assert!(
        serde_json::from_str::<InteractionWorkflow>(&json.replace("{", "{\"extra\":true,"))
            .is_err()
    );
    let mut value = serde_json::to_value(w).unwrap();
    for field in ["workflow_id", "session_id", "correlation_id", "trace_id"] {
        let original = value[field].clone();
        value[field] = serde_json::Value::String(Uuid::nil().to_string());
        assert!(
            serde_json::from_value::<InteractionWorkflow>(value.clone()).is_err(),
            "{field}"
        );
        value[field] = original;
    }
}

#[test]
fn trusted_association_validation_rejects_each_reassociated_identity() {
    let w = workflow();
    let (workflow_id, session_id, correlation_id, trace_id) = identities(w);
    assert_eq!(
        w.validate_association(workflow_id, session_id, correlation_id, trace_id),
        Ok(())
    );
    assert_eq!(
        w.validate_association(id(5, WorkflowId::new), session_id, correlation_id, trace_id),
        Err(WorkflowLifecycleError::AssociationMismatch)
    );
    assert_eq!(
        w.validate_association(workflow_id, id(6, SessionId::new), correlation_id, trace_id),
        Err(WorkflowLifecycleError::AssociationMismatch)
    );
    assert_eq!(
        w.validate_association(workflow_id, session_id, id(7, CorrelationId::new), trace_id),
        Err(WorkflowLifecycleError::AssociationMismatch)
    );
    assert_eq!(
        w.validate_association(workflow_id, session_id, correlation_id, id(8, TraceId::new)),
        Err(WorkflowLifecycleError::AssociationMismatch)
    );
}
#[test]
fn errors_have_content_free_stable_diagnostics() {
    let session_error = S::Created.transition_to(S::Ready).unwrap_err();
    assert_eq!(format!("{session_error:?}"), "SessionTransitionError");
    assert_eq!(
        session_error.to_string(),
        "illegal runtime session state transition"
    );
    let illegal = workflow().advance(W::Completed).unwrap_err();
    assert_eq!(format!("{illegal:?}"), "IllegalTransition");
    assert_eq!(illegal.to_string(), "illegal workflow lifecycle transition");

    let w = workflow();
    let (_, session_id, correlation_id, trace_id) = identities(w);
    let mismatch = w
        .validate_association(id(9, WorkflowId::new), session_id, correlation_id, trace_id)
        .unwrap_err();
    assert_eq!(format!("{mismatch:?}"), "AssociationMismatch");
    assert_eq!(
        mismatch.to_string(),
        "workflow lifecycle identity association mismatch"
    );

    let json = serde_json::to_string(&w)
        .unwrap()
        .replace("\"1.0\"", "\"2.0\"");
    let unsupported = serde_json::from_str::<InteractionWorkflow>(&json).unwrap_err();
    assert_eq!(
        unsupported.to_string(),
        "unsupported workflow lifecycle version"
    );
    assert_eq!(
        format!("{unsupported:?}"),
        "Error(\"unsupported workflow lifecycle version\", line: 0, column: 0)"
    );
}
