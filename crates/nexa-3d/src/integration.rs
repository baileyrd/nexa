//! Synchronous, headless NBP-to-avatar acceptance composition.
use nexa_avatar::{
    AvatarPort, AvatarReport, AvatarRequest, OutputConversionError, RequestConversionError,
};
use nexa_domain::{EndpointId, EventId, MessageId, Sequence};
use nexa_events::{
    AvatarBehaviorAccepted, AvatarBehaviorCancelled, AvatarBehaviorCompleted,
    AvatarBehaviorDegraded, AvatarBehaviorFailed, AvatarBehaviorStarted, DomainEvent, Event,
};
use nexa_nbp::{NbpMessage, RuntimeStatus};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum LifecycleEvent {
    Accepted(Event<AvatarBehaviorAccepted>),
    Started(Event<AvatarBehaviorStarted>),
    Completed(Event<AvatarBehaviorCompleted>),
    Cancelled(Event<AvatarBehaviorCancelled>),
    Degraded(Event<AvatarBehaviorDegraded>),
    Failed(Event<AvatarBehaviorFailed>),
}

/// Caller-owned identities make the synchronous core deterministic and replay-safe.
pub struct FlowIdentity {
    pub source: EndpointId,
    pub first_output_sequence: Sequence,
    pub output_message_ids: Vec<MessageId>,
    pub first_event_sequence: Sequence,
    pub event_ids: Vec<EventId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlowResult {
    pub report: AvatarReport,
    pub outputs: Vec<NbpMessage>,
    pub events: Vec<LifecycleEvent>,
}

#[derive(Debug, Error)]
pub enum FlowError {
    #[error(transparent)]
    InvalidInput(#[from] RequestConversionError),
    #[error(transparent)]
    Output(#[from] OutputConversionError),
    #[error("not enough event identities supplied for lifecycle output")]
    InsufficientEventIds,
    #[error("avatar event sequence exceeds u64::MAX")]
    EventSequenceOverflow,
    #[error("avatar adapter returned a different report than its preflight preview")]
    AdapterReportMismatch,
}

pub fn execute<A: AvatarPort>(
    adapter: &mut A,
    input: &NbpMessage,
    identity: FlowIdentity,
) -> Result<FlowResult, FlowError> {
    let request = AvatarRequest::try_from(input)?;
    let report = adapter.preview(&request);

    // Complete every fallible identity and sequence conversion before dispatching the request.
    let outputs = report.to_nbp_messages(
        input,
        identity.source.clone(),
        identity.first_output_sequence,
        identity.output_message_ids.iter().copied(),
    )?;
    let events = lifecycle_events(input, &report, &identity)?;

    let actual = adapter.handle(request);
    if actual != report {
        return Err(FlowError::AdapterReportMismatch);
    }
    Ok(FlowResult {
        report,
        outputs,
        events,
    })
}

fn lifecycle_events(
    input: &NbpMessage,
    report: &AvatarReport,
    identity: &FlowIdentity,
) -> Result<Vec<LifecycleEvent>, FlowError> {
    let mut ids = identity.event_ids.iter().copied();
    report
        .lifecycle()
        .iter()
        .filter(|status| !matches!(status, RuntimeStatus::Queued | RuntimeStatus::Rejected))
        .enumerate()
        .map(|(offset, status)| {
            let event_id = ids.next().ok_or(FlowError::InsufficientEventIds)?;
            let sequence = Some(Sequence::new(
                identity
                    .first_event_sequence
                    .get()
                    .checked_add(offset as u64)
                    .ok_or(FlowError::EventSequenceOverflow)?,
            ));
            let event = match status {
                RuntimeStatus::Accepted => LifecycleEvent::Accepted(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorAccepted {
                        message_id: report.message_id,
                        behavior_id: report.behavior_id,
                    },
                )),
                RuntimeStatus::Started => LifecycleEvent::Started(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorStarted {
                        message_id: report.message_id,
                        behavior_id: report.behavior_id,
                    },
                )),
                RuntimeStatus::Completed => LifecycleEvent::Completed(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorCompleted {
                        message_id: report.message_id,
                        behavior_id: report.behavior_id,
                    },
                )),
                RuntimeStatus::Cancelled => LifecycleEvent::Cancelled(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorCancelled {
                        message_id: report.message_id,
                        behavior_id: report.behavior_id,
                    },
                )),
                RuntimeStatus::Degraded => LifecycleEvent::Degraded(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorDegraded {
                        message_id: report.message_id,
                        behavior_id: report.behavior_id,
                        reason: report
                            .error()
                            .expect("validated report has an error")
                            .code
                            .clone(),
                    },
                )),
                RuntimeStatus::Failed => LifecycleEvent::Failed(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorFailed {
                        message_id: report.message_id,
                        behavior_id: report.behavior_id,
                        reason: report
                            .error()
                            .expect("validated report has an error")
                            .code
                            .clone(),
                    },
                )),
                RuntimeStatus::Queued | RuntimeStatus::Rejected => unreachable!("filtered above"),
            };
            Ok(event)
        })
        .collect()
}

fn make_event<T: DomainEvent>(
    input: &NbpMessage,
    event_id: EventId,
    sequence: Option<Sequence>,
    source: EndpointId,
    payload: T,
) -> Event<T> {
    Event::new(
        input.nbp_version,
        event_id,
        input.timestamp,
        Some(input.session_id),
        sequence,
        source,
        None,
        input.correlation_id,
        None,
        None,
        payload,
        Default::default(),
    )
}
