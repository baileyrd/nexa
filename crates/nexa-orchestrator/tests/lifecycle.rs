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
                assert_eq!(cancelled.cancel(), Ok(cancelled));
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
            assert_eq!(w.fail().unwrap().state(), W::Failed);
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
fn wire_is_stable_strict_validating_and_preserves_associations() {
    let w = at(W::Speaking);
    let json = serde_json::to_string(&w).unwrap();
    assert_eq!(
        serde_json::from_str::<InteractionWorkflow>(&json).unwrap(),
        w
    );
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
    assert!(serde_json::from_str::<InteractionWorkflow>(&json.replace(
        "00000000-0000-0000-0000-000000000001",
        "00000000-0000-0000-0000-000000000000"
    ))
    .is_err());
}
#[test]
fn errors_have_content_free_stable_diagnostics() {
    assert_eq!(
        format!("{SessionTransitionError:?}"),
        "SessionTransitionError"
    );
    assert_eq!(
        SessionTransitionError.to_string(),
        "illegal runtime session state transition"
    );
    for e in [
        WorkflowLifecycleError::UnsupportedVersion,
        WorkflowLifecycleError::IllegalTransition,
    ] {
        assert!(!format!("{e:?} {e}").contains("00000000"));
    }
}
