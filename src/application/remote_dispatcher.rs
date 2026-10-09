use std::collections::VecDeque;

use super::remote_request::{RemoteActionOutcome, RemoteActionRejection, SendRemoteAction};
use super::ControlState;

/// A dispatcher owns one waiting FIFO and at most one transport write.
/// Terminal results stay in its journal until an observer acknowledges them.
#[derive(Debug)]
pub struct RemoteDispatcher {
    waiting: VecDeque<QueuedRequest>,
    in_flight: Option<QueuedRequest>,
    terminal: VecDeque<TerminalResult>,
    next_id: u64,
    revision: u64,
    queue_capacity: usize,
    journal_capacity: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(u64);

impl RequestId {
    pub const fn value(self) -> u64 {
        self.0
    }

    #[cfg(test)]
    pub(crate) const fn from_test_value(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueuedRequest {
    pub id: RequestId,
    pub request: SendRemoteAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchRejection {
    Policy(RemoteActionRejection),
    Busy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    Queued(RequestId),
    Rejected(DispatchRejection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotSentReason {
    Cancelled,
    Policy(RemoteActionRejection),
    TransportUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalOutcome {
    NotSent(NotSentReason),
    Written,
    Uncertain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalResult {
    pub id: RequestId,
    pub request: SendRemoteAction,
    pub outcome: TerminalOutcome,
}

impl RemoteDispatcher {
    pub fn new(queue_capacity: usize, journal_capacity: usize) -> Self {
        Self {
            waiting: VecDeque::new(),
            in_flight: None,
            terminal: VecDeque::new(),
            next_id: 1,
            revision: 0,
            queue_capacity,
            journal_capacity,
        }
    }

    pub const fn revision(&self) -> u64 {
        self.revision
    }

    pub fn terminal_results(&self) -> &VecDeque<TerminalResult> {
        &self.terminal
    }

    pub fn pending_ids(&self) -> impl Iterator<Item = RequestId> + '_ {
        self.in_flight
            .iter()
            .chain(self.waiting.iter())
            .map(|item| item.id)
    }

    pub fn admit(&mut self, state: &ControlState, request: SendRemoteAction) -> Admission {
        if let RemoteActionOutcome::Rejected { reason, .. } =
            state.evaluate_remote_action(request.clone())
        {
            return Admission::Rejected(DispatchRejection::Policy(reason));
        }
        // Reserve journal space for every admitted request, including one
        // currently in the transport. Results are never evicted silently.
        let reserved =
            self.terminal.len() + self.waiting.len() + usize::from(self.in_flight.is_some());
        if self.waiting.len() >= self.queue_capacity || reserved >= self.journal_capacity {
            return Admission::Rejected(DispatchRejection::Busy);
        }
        let id = RequestId(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.waiting.push_back(QueuedRequest { id, request });
        self.bump_revision();
        Admission::Queued(id)
    }

    /// Returns the next request that may start a write. Rechecks every queued
    /// request against current state and records a terminal result for stale work.
    pub fn start_next(&mut self, state: &ControlState) -> Option<QueuedRequest> {
        if self.in_flight.is_some() {
            return None;
        }
        while let Some(item) = self.waiting.pop_front() {
            match state.evaluate_remote_action(item.request.clone()) {
                RemoteActionOutcome::Rejected { reason, .. } => {
                    self.record(
                        item,
                        TerminalOutcome::NotSent(NotSentReason::Policy(reason)),
                    );
                }
                RemoteActionOutcome::Eligible(_) => {
                    self.in_flight = Some(item.clone());
                    self.bump_revision();
                    return Some(item);
                }
            }
        }
        None
    }

    /// Completes the active write exactly once. A stale or duplicate completion
    /// cannot replace the active request or mutate selected-device lifecycle.
    pub fn finish(&mut self, id: RequestId, outcome: TerminalOutcome) -> bool {
        if self.in_flight.as_ref().map(|item| item.id) != Some(id) {
            return false;
        }
        let item = self.in_flight.take().expect("matched active request");
        self.record(item, outcome);
        true
    }

    /// Cancels queued work during a selection change or forget operation.
    /// The socket owner must separately close/cancel an in-progress write.
    pub fn cancel_waiting(&mut self) {
        while let Some(item) = self.waiting.pop_front() {
            self.record(item, TerminalOutcome::NotSent(NotSentReason::Cancelled));
        }
    }

    /// Aborts an in-progress socket write during a selection change. Its
    /// outcome remains uncertain because the transport may have written bytes.
    pub fn abort_in_flight(&mut self) {
        if let Some(item) = self.in_flight.take() {
            self.record(item, TerminalOutcome::Uncertain);
        }
    }

    /// Acknowledges journal entries through an ID after the observer has
    /// incorporated them into its own ordered snapshot.
    pub fn acknowledge_through(&mut self, id: RequestId) {
        let mut changed = false;
        while self
            .terminal
            .front()
            .is_some_and(|result| result.id.0 <= id.0)
        {
            let _ = self.terminal.pop_front();
            changed = true;
        }
        if changed {
            self.bump_revision();
        }
    }

    fn record(&mut self, item: QueuedRequest, outcome: TerminalOutcome) {
        assert!(
            self.terminal.len() < self.journal_capacity,
            "journal reservation invariant"
        );
        self.terminal.push_back(TerminalResult {
            id: item.id,
            request: item.request,
            outcome,
        });
        self.bump_revision();
    }

    fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::{ConnectionState, PairingState};
    use crate::domain::{DeviceDisplay, DeviceId, RemoteAction};

    fn connected_state() -> ControlState {
        let mut state = ControlState::none();
        state.select_device(DeviceDisplay::new(DeviceId::new(1), "TV"));
        let generation = state.selection_generation();
        state.set_pairing_state(generation, PairingState::Ready);
        state.set_connection_state(generation, ConnectionState::Ready);
        state
    }

    fn request(state: &ControlState, action: RemoteAction) -> SendRemoteAction {
        SendRemoteAction::new(DeviceId::new(1), state.selection_generation(), action)
    }

    #[test]
    fn orders_writes_and_rejects_queue_or_journal_overflow() {
        let state = connected_state();
        let mut dispatcher = RemoteDispatcher::new(2, 2);
        let first = dispatcher.admit(&state, request(&state, RemoteAction::Up));
        let second = dispatcher.admit(&state, request(&state, RemoteAction::Down));
        assert!(matches!(first, Admission::Queued(_)));
        assert!(matches!(second, Admission::Queued(_)));
        assert_eq!(
            dispatcher.admit(&state, request(&state, RemoteAction::Left)),
            Admission::Rejected(DispatchRejection::Busy)
        );
        let active = dispatcher.start_next(&state).unwrap();
        assert_eq!(active.request.action(), RemoteAction::Up);
        assert!(dispatcher.start_next(&state).is_none());
        assert!(dispatcher.finish(active.id, TerminalOutcome::Written));
        assert!(!dispatcher.finish(active.id, TerminalOutcome::Written));
        assert_eq!(
            dispatcher.start_next(&state).unwrap().request.action(),
            RemoteAction::Down
        );
        assert_eq!(
            dispatcher.admit(&state, request(&state, RemoteAction::Left)),
            Admission::Rejected(DispatchRejection::Busy)
        );
    }

    #[test]
    fn stale_selection_is_terminal_without_a_write() {
        let mut state = connected_state();
        let mut dispatcher = RemoteDispatcher::new(2, 2);
        let admission = dispatcher.admit(&state, request(&state, RemoteAction::Up));
        state.select_device(DeviceDisplay::new(DeviceId::new(2), "Other"));
        assert!(dispatcher.start_next(&state).is_none());
        let Admission::Queued(id) = admission else {
            panic!("not queued")
        };
        assert_eq!(dispatcher.terminal_results().front().unwrap().id, id);
        assert_eq!(
            dispatcher.terminal_results().front().unwrap().outcome,
            TerminalOutcome::NotSent(NotSentReason::Policy(RemoteActionRejection::WrongTarget))
        );
    }

    #[test]
    fn cancellation_and_observer_reattachment_preserve_terminal_results() {
        let state = connected_state();
        let mut dispatcher = RemoteDispatcher::new(2, 2);
        let Admission::Queued(first) = dispatcher.admit(&state, request(&state, RemoteAction::Up))
        else {
            panic!("not queued")
        };
        let Admission::Queued(second) =
            dispatcher.admit(&state, request(&state, RemoteAction::Down))
        else {
            panic!("not queued")
        };
        dispatcher.cancel_waiting();
        let revision = dispatcher.revision();
        assert_eq!(dispatcher.terminal_results().len(), 2);
        assert_eq!(dispatcher.terminal_results()[0].id, first);
        assert_eq!(dispatcher.terminal_results()[1].id, second);
        assert_eq!(
            dispatcher.terminal_results()[0].outcome,
            TerminalOutcome::NotSent(NotSentReason::Cancelled)
        );
        assert_eq!(
            dispatcher.admit(&state, request(&state, RemoteAction::Left)),
            Admission::Rejected(DispatchRejection::Busy)
        );
        assert_eq!(dispatcher.revision(), revision);
        dispatcher.acknowledge_through(first);
        assert_eq!(dispatcher.terminal_results().front().unwrap().id, second);
        assert!(matches!(
            dispatcher.admit(&state, request(&state, RemoteAction::Left)),
            Admission::Queued(_)
        ));
    }

    #[test]
    fn power_is_admitted_only_for_a_connected_session() {
        let state = connected_state();
        let mut dispatcher = RemoteDispatcher::new(1, 1);
        assert!(matches!(
            dispatcher.admit(&state, request(&state, RemoteAction::PowerToggle)),
            Admission::Queued(_)
        ));
    }

    #[test]
    fn aborted_write_is_uncertain_and_queued_followup_is_cancelled() {
        let state = connected_state();
        let mut dispatcher = RemoteDispatcher::new(2, 2);
        let Admission::Queued(first) = dispatcher.admit(&state, request(&state, RemoteAction::Up))
        else {
            panic!("not queued")
        };
        let Admission::Queued(second) =
            dispatcher.admit(&state, request(&state, RemoteAction::Down))
        else {
            panic!("not queued")
        };
        assert_eq!(dispatcher.start_next(&state).unwrap().id, first);

        dispatcher.abort_in_flight();
        dispatcher.cancel_waiting();

        let outcomes = dispatcher.terminal_results();
        assert_eq!(outcomes.len(), 2);
        assert_eq!(outcomes[0].id, first);
        assert_eq!(outcomes[0].outcome, TerminalOutcome::Uncertain);
        assert_eq!(outcomes[1].id, second);
        assert_eq!(
            outcomes[1].outcome,
            TerminalOutcome::NotSent(NotSentReason::Cancelled)
        );
        assert!(!dispatcher.finish(first, TerminalOutcome::Written));
        assert!(dispatcher.start_next(&state).is_none());
    }
}
