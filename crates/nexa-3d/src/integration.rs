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
}

pub fn execute<A: AvatarPort>(
    adapter: &mut A,
    input: &NbpMessage,
    identity: FlowIdentity,
) -> Result<FlowResult, FlowError> {
    let request = AvatarRequest::try_from(input)?;
    let report = adapter.handle(request);
    let outputs = report.to_nbp_messages(
        input,
        identity.source.clone(),
        identity.first_output_sequence,
        identity.output_message_ids,
    )?;
    let mut ids = identity.event_ids.into_iter();
    let mut events = Vec::new();
    for (offset, status) in report.lifecycle.iter().enumerate() {
        let event_id = ids.next().ok_or(FlowError::InsufficientEventIds)?;
        let sequence = Some(Sequence::new(
            identity.first_event_sequence.get() + offset as u64,
        ));
        let basic = || (report.message_id, report.behavior_id);
        let event = match status {
            RuntimeStatus::Accepted => {
                let (message_id, behavior_id) = basic();
                LifecycleEvent::Accepted(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorAccepted {
                        message_id,
                        behavior_id,
                    },
                ))
            }
            RuntimeStatus::Started => {
                let (message_id, behavior_id) = basic();
                LifecycleEvent::Started(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorStarted {
                        message_id,
                        behavior_id,
                    },
                ))
            }
            RuntimeStatus::Completed => {
                let (message_id, behavior_id) = basic();
                LifecycleEvent::Completed(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorCompleted {
                        message_id,
                        behavior_id,
                    },
                ))
            }
            RuntimeStatus::Cancelled => {
                let (message_id, behavior_id) = basic();
                LifecycleEvent::Cancelled(make_event(
                    input,
                    event_id,
                    sequence,
                    identity.source.clone(),
                    AvatarBehaviorCancelled {
                        message_id,
                        behavior_id,
                    },
                ))
            }
            RuntimeStatus::Degraded => LifecycleEvent::Degraded(make_event(
                input,
                event_id,
                sequence,
                identity.source.clone(),
                AvatarBehaviorDegraded {
                    message_id: report.message_id,
                    behavior_id: report.behavior_id,
                    reason: report
                        .error
                        .as_ref()
                        .expect("degraded report has reason")
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
                        .error
                        .as_ref()
                        .expect("failed report has reason")
                        .code
                        .clone(),
                },
            )),
            RuntimeStatus::Queued | RuntimeStatus::Rejected => continue,
        };
        events.push(event);
    }
    Ok(FlowResult {
        report,
        outputs,
        events,
    })
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
