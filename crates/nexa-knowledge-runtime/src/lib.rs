//! Runtime-owned asynchronous Retrieval service and cooperative cancellation boundary.
#![forbid(unsafe_code)]

use nexa_domain::{RetrievalQueryId, RetrievalResultId};
use nexa_knowledge::{RetrievalQuery, RetrievalResult};
use std::{
    collections::VecDeque,
    fmt,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio_util::sync::CancellationToken;

/// An erased, composition-usable future returned by a [`RetrievalService`].
pub type RetrievalFuture<'a> = Pin<Box<dyn Future<Output = RetrievalServiceOutcome> + Send + 'a>>;

/// Provider-neutral asynchronous Retrieval dependency.
///
/// Implementations must observe `cancellation` cooperatively. They must not detach
/// work: returning from this future is the service operation's termination boundary.
pub trait RetrievalService: Send + Sync {
    fn retrieve(
        &self,
        query: RetrievalQuery,
        cancellation: CancellationToken,
    ) -> RetrievalFuture<'_>;
}

/// Precise evidence that the service future observed cancellation and terminated.
/// This does not prove that an external provider stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetrievalCancellation {
    query_id: RetrievalQueryId,
    result_id: RetrievalResultId,
}

impl RetrievalCancellation {
    /// Construct cancellation evidence associated with one exact, valid query.
    ///
    /// The query text is validated but is not retained in the evidence.
    pub fn from_query(query: &RetrievalQuery) -> Result<Self, RetrievalServiceError> {
        query
            .validate()
            .map_err(|_| RetrievalServiceError::InvalidQuery)?;
        Ok(Self {
            query_id: query.query_id,
            result_id: query.result_id,
        })
    }

    pub const fn query_id(&self) -> RetrievalQueryId {
        self.query_id
    }
    pub const fn result_id(&self) -> RetrievalResultId {
        self.result_id
    }
}

impl fmt::Display for RetrievalCancellation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "retrieval cancelled for query {} and result {}",
            self.query_id, self.result_id
        )
    }
}

/// Closed outcome of one service call.
#[derive(Clone, Debug, PartialEq)]
pub enum RetrievalServiceOutcome {
    Success(RetrievalResult),
    Cancelled(RetrievalCancellation),
    DependencyFailure,
}

impl fmt::Display for RetrievalServiceOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Success(_) => f.write_str("retrieval succeeded"),
            Self::Cancelled(evidence) => evidence.fmt(f),
            Self::DependencyFailure => f.write_str("retrieval dependency failed"),
        }
    }
}

/// Closed, content-free host error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetrievalServiceError {
    InvalidQuery,
    AssociationMismatch,
    DependencyFailure,
}

impl fmt::Display for RetrievalServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidQuery => "invalid retrieval query",
            Self::AssociationMismatch => "retrieval result association mismatch",
            Self::DependencyFailure => "retrieval dependency failed",
        })
    }
}

impl std::error::Error for RetrievalServiceError {}

/// Invoke one validated query while enforcing exact query/result association.
pub async fn retrieve(
    service: &dyn RetrievalService,
    query: RetrievalQuery,
    cancellation: CancellationToken,
) -> Result<RetrievalServiceOutcome, RetrievalServiceError> {
    query
        .validate()
        .map_err(|_| RetrievalServiceError::InvalidQuery)?;
    let query_id = query.query_id;
    let result_id = query.result_id;
    match service.retrieve(query, cancellation).await {
        RetrievalServiceOutcome::Success(result)
            if result.query_id == query_id && result.result_id == result_id =>
        {
            Ok(RetrievalServiceOutcome::Success(result))
        }
        RetrievalServiceOutcome::Cancelled(evidence)
            if evidence.query_id == query_id && evidence.result_id == result_id =>
        {
            Ok(RetrievalServiceOutcome::Cancelled(evidence))
        }
        RetrievalServiceOutcome::Success(_) | RetrievalServiceOutcome::Cancelled(_) => {
            Err(RetrievalServiceError::AssociationMismatch)
        }
        RetrievalServiceOutcome::DependencyFailure => Err(RetrievalServiceError::DependencyFailure),
    }
}

/// One deterministic dependency instruction for [`ScriptedRetrievalService`].
#[derive(Clone, Debug, PartialEq)]
pub enum ScriptedRetrievalOutcome {
    Success(RetrievalResult),
    DependencyFailure,
    WaitForCancellation,
}

#[derive(Default)]
struct ScriptState {
    outcomes: VecDeque<ScriptedRetrievalOutcome>,
    received: Vec<RetrievalQuery>,
    consumed: usize,
    active: usize,
}

/// Deterministic FIFO adapter. It creates no task and performs no work after its
/// returned future completes or is dropped.
#[derive(Clone, Default)]
pub struct ScriptedRetrievalService {
    state: Arc<Mutex<ScriptState>>,
}

impl ScriptedRetrievalService {
    pub fn new(outcomes: impl IntoIterator<Item = ScriptedRetrievalOutcome>) -> Self {
        Self {
            state: Arc::new(Mutex::new(ScriptState {
                outcomes: outcomes.into_iter().collect(),
                ..ScriptState::default()
            })),
        }
    }
    pub fn received_queries(&self) -> Vec<RetrievalQuery> {
        self.state
            .lock()
            .expect("script state poisoned")
            .received
            .clone()
    }
    pub fn consumed_outcome_count(&self) -> usize {
        self.state.lock().expect("script state poisoned").consumed
    }
    pub fn remaining_outcome_count(&self) -> usize {
        self.state
            .lock()
            .expect("script state poisoned")
            .outcomes
            .len()
    }
    pub fn active_operation_count(&self) -> usize {
        self.state.lock().expect("script state poisoned").active
    }
}

struct ActiveOperation(Arc<Mutex<ScriptState>>);
impl Drop for ActiveOperation {
    fn drop(&mut self) {
        self.0.lock().expect("script state poisoned").active -= 1;
    }
}

impl RetrievalService for ScriptedRetrievalService {
    fn retrieve(
        &self,
        query: RetrievalQuery,
        cancellation: CancellationToken,
    ) -> RetrievalFuture<'_> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            let query_id = query.query_id;
            let result_id = query.result_id;
            {
                let mut state = state.lock().expect("script state poisoned");
                state.received.push(query);
                state.active += 1;
            }
            let _active = ActiveOperation(Arc::clone(&state));
            if cancellation.is_cancelled() {
                return RetrievalServiceOutcome::Cancelled(RetrievalCancellation {
                    query_id,
                    result_id,
                });
            }
            let outcome = {
                let mut state = state.lock().expect("script state poisoned");
                let outcome = state.outcomes.pop_front();
                if outcome.is_some() {
                    state.consumed += 1;
                }
                outcome
            };
            match outcome {
                Some(ScriptedRetrievalOutcome::Success(result)) => {
                    RetrievalServiceOutcome::Success(result)
                }
                Some(ScriptedRetrievalOutcome::DependencyFailure) | None => {
                    RetrievalServiceOutcome::DependencyFailure
                }
                Some(ScriptedRetrievalOutcome::WaitForCancellation) => {
                    cancellation.cancelled().await;
                    RetrievalServiceOutcome::Cancelled(RetrievalCancellation {
                        query_id,
                        result_id,
                    })
                }
            }
        })
    }
}
