use crate::application::remote_request::{
    RemoteActionOutcome, RemoteActionRejection, SendRemoteAction,
};
use crate::domain::{DeviceDisplay, DeviceId, RemoteAction};

/// Application-owned control availability for the current snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlStatus {
    Available,
    Unavailable(RemoteActionRejection),
}

/// Pairing readiness for the current selection generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PairingState {
    #[default]
    NotStarted,
    InProgress,
    Ready,
    Failed,
}

/// Connection readiness for the current selection generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    NotConnected,
    Connecting,
    Ready,
    Failed,
}

/// The result of applying a generation-scoped lifecycle fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleUpdateResult {
    Applied,
    IgnoredNoSelection,
    IgnoredStaleGeneration,
}

#[derive(Debug, Clone, Default)]
enum Selection {
    #[default]
    None,
    Selected(DeviceDisplay),
}

/// The application's in-memory selected-device and lifecycle snapshot.
#[derive(Debug, Clone, Default)]
pub struct ControlState {
    selection: Selection,
    selection_generation: u64,
    pairing: PairingState,
    connection: ConnectionState,
    verified_actions: Vec<RemoteAction>,
}

impl ControlState {
    /// Creates a new state with no selected TV.
    pub const fn none() -> Self {
        Self {
            selection: Selection::None,
            selection_generation: 0,
            pairing: PairingState::NotStarted,
            connection: ConnectionState::NotConnected,
            verified_actions: Vec::new(),
        }
    }

    /// Returns the currently selected device, if any.
    pub const fn selected_device(&self) -> Option<DeviceId> {
        match &self.selection {
            Selection::None => None,
            Selection::Selected(device) => Some(device.id()),
        }
    }

    /// Returns safe display information for the selected device, if any.
    pub fn selected_device_display(&self) -> Option<&DeviceDisplay> {
        match &self.selection {
            Selection::None => None,
            Selection::Selected(device) => Some(device),
        }
    }

    /// Monotonically identifies the current selection epoch.
    pub const fn selection_generation(&self) -> u64 {
        self.selection_generation
    }

    pub const fn pairing_state(&self) -> PairingState {
        self.pairing
    }

    pub const fn connection_state(&self) -> ConnectionState {
        self.connection
    }

    /// Selects a caller-supplied known device for this session.
    ///
    /// Re-selecting the current identity is an idempotent no-op, including its
    /// display projection. A different identity starts a new selection epoch.
    pub fn select_device(&mut self, device: DeviceDisplay) -> bool {
        if self.selected_device() == Some(device.id()) {
            return false;
        }

        self.selection = Selection::Selected(device);
        self.selection_generation = self.selection_generation.saturating_add(1);
        self.reset_lifecycle();
        true
    }

    /// Starts a new selection epoch for re-pairing the same saved record.
    pub fn restart_selection(&mut self, device: DeviceDisplay) {
        self.selection = Selection::Selected(device);
        self.selection_generation = self.selection_generation.saturating_add(1);
        self.reset_lifecycle();
    }

    /// Clears the session selection and starts a new epoch when one existed.
    pub fn clear_selection(&mut self) -> bool {
        if matches!(self.selection, Selection::None) {
            return false;
        }

        self.selection = Selection::None;
        self.selection_generation = self.selection_generation.saturating_add(1);
        self.reset_lifecycle();
        true
    }

    /// Records a pairing lifecycle fact for the current selection generation.
    pub fn set_pairing_state(
        &mut self,
        generation: u64,
        pairing: PairingState,
    ) -> LifecycleUpdateResult {
        let result = self.validate_lifecycle_generation(generation);
        if result == LifecycleUpdateResult::Applied {
            self.pairing = pairing;
        }
        result
    }

    /// Records a connection lifecycle fact for the current selection generation.
    pub fn set_connection_state(
        &mut self,
        generation: u64,
        connection: ConnectionState,
    ) -> LifecycleUpdateResult {
        let result = self.validate_lifecycle_generation(generation);
        if result == LifecycleUpdateResult::Applied {
            self.connection = connection;
        }
        result
    }

    /// Availability of one action, using the same policy as dispatch admission.
    pub fn control_status(&self, action: RemoteAction) -> ControlStatus {
        let Some(target) = self.selected_device() else {
            return ControlStatus::Unavailable(RemoteActionRejection::NoSelectedTv);
        };

        match self.evaluate_remote_action(SendRemoteAction::new(
            target,
            self.selection_generation,
            action,
        )) {
            RemoteActionOutcome::Eligible(_) => ControlStatus::Available,
            RemoteActionOutcome::Rejected { reason, .. } => ControlStatus::Unavailable(reason),
        }
    }

    /// Records hardware-verified actions for the current selected generation.
    /// Unknown actions remain unavailable until the selected TV has been checked.
    pub fn set_verified_actions(
        &mut self,
        generation: u64,
        actions: impl IntoIterator<Item = RemoteAction>,
    ) -> LifecycleUpdateResult {
        let result = self.validate_lifecycle_generation(generation);
        if result == LifecycleUpdateResult::Applied {
            self.verified_actions = actions
                .into_iter()
                .filter(|action| !action.is_deferred())
                .collect();
        }
        result
    }

    pub fn is_action_verified(&self, action: RemoteAction) -> bool {
        self.verified_actions.contains(&action)
    }

    /// Applies the pure remote-command admission policy to one request.
    ///
    /// An eligible result says only that local policy passed. No request is
    /// queued, sent, or otherwise dispatched by this method.
    pub fn evaluate_remote_action(&self, request: SendRemoteAction) -> RemoteActionOutcome {
        let Some(selected_device) = self.selected_device() else {
            return RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::NoSelectedTv,
            };
        };

        let reason = if request.target() != selected_device {
            Some(RemoteActionRejection::WrongTarget)
        } else if request.selection_generation() != self.selection_generation {
            Some(RemoteActionRejection::StaleSelectionGeneration)
        } else if self.pairing != PairingState::Ready {
            Some(RemoteActionRejection::PairingRequired)
        } else if self.connection != ConnectionState::Ready {
            Some(RemoteActionRejection::NotConnected)
        } else if request.action().is_deferred() {
            Some(RemoteActionRejection::DeferredAction)
        } else if !self.verified_actions.contains(&request.action()) {
            Some(RemoteActionRejection::UnverifiedAction)
        } else {
            None
        };

        match reason {
            Some(reason) => RemoteActionOutcome::Rejected { request, reason },
            None => RemoteActionOutcome::Eligible(request),
        }
    }

    fn reset_lifecycle(&mut self) {
        self.pairing = PairingState::NotStarted;
        self.connection = ConnectionState::NotConnected;
        self.verified_actions.clear();
    }

    fn validate_lifecycle_generation(&self, generation: u64) -> LifecycleUpdateResult {
        if self.selected_device().is_none() {
            LifecycleUpdateResult::IgnoredNoSelection
        } else if generation != self.selection_generation {
            LifecycleUpdateResult::IgnoredStaleGeneration
        } else {
            LifecycleUpdateResult::Applied
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::RemoteAction;

    #[test]
    fn new_state_has_no_selected_tv() {
        let state = ControlState::none();
        assert_eq!(state.selected_device(), None);
        assert_eq!(ControlState::default().selected_device(), None);
        assert_eq!(
            state.control_status(RemoteAction::Up),
            ControlStatus::Unavailable(RemoteActionRejection::NoSelectedTv)
        );
    }

    #[test]
    fn no_selection_rejection_preserves_the_requested_target() {
        let state = ControlState::none();
        let request = SendRemoteAction::new(DeviceId::new(1), 0, RemoteAction::PowerToggle);
        let outcome = state.evaluate_remote_action(request.clone());
        assert_eq!(
            outcome,
            RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::NoSelectedTv,
            }
        );
    }

    #[test]
    fn selection_and_clear_advance_the_generation_only_when_identity_changes() {
        let mut state = ControlState::none();
        let living_room = DeviceDisplay::new(DeviceId::new(1), "Living Room");
        let renamed_living_room = DeviceDisplay::new(DeviceId::new(1), "TV");
        let bedroom = DeviceDisplay::new(DeviceId::new(2), "Bedroom");

        assert_eq!(state.selection_generation(), 0);
        assert!(state.select_device(living_room));
        assert_eq!(state.selected_device(), Some(DeviceId::new(1)));
        assert_eq!(
            state.selected_device_display().map(DeviceDisplay::label),
            Some("Living Room")
        );
        assert_eq!(state.selection_generation(), 1);

        assert!(!state.select_device(renamed_living_room));
        assert_eq!(
            state.selected_device_display().map(DeviceDisplay::label),
            Some("Living Room")
        );
        assert_eq!(state.selection_generation(), 1);

        assert!(state.select_device(bedroom));
        assert_eq!(state.selected_device(), Some(DeviceId::new(2)));
        assert_eq!(state.selection_generation(), 2);

        assert!(state.clear_selection());
        assert_eq!(state.selected_device(), None);
        assert_eq!(state.selected_device_display(), None);
        assert_eq!(state.selection_generation(), 3);
        assert!(!state.clear_selection());
        assert_eq!(state.selection_generation(), 3);
    }

    #[test]
    fn remote_action_admission_checks_each_precondition() {
        let no_selection = ControlState::none();
        let request = SendRemoteAction::new(DeviceId::new(1), 0, RemoteAction::PowerToggle);
        assert_eq!(
            no_selection.evaluate_remote_action(request.clone()),
            RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::NoSelectedTv,
            }
        );

        let mut state = ControlState::none();
        let _ = state.select_device(DeviceDisplay::new(DeviceId::new(1), "Living Room"));
        let generation = state.selection_generation();
        let cases = [
            (
                SendRemoteAction::new(DeviceId::new(2), generation, RemoteAction::PowerToggle),
                RemoteActionRejection::WrongTarget,
            ),
            (
                SendRemoteAction::new(DeviceId::new(1), generation - 1, RemoteAction::PowerToggle),
                RemoteActionRejection::StaleSelectionGeneration,
            ),
            (
                SendRemoteAction::new(DeviceId::new(1), generation, RemoteAction::PowerToggle),
                RemoteActionRejection::PairingRequired,
            ),
        ];
        for (request, reason) in cases {
            assert_eq!(
                state.evaluate_remote_action(request.clone()),
                RemoteActionOutcome::Rejected { request, reason }
            );
        }

        let _ = state.set_pairing_state(generation, PairingState::Ready);
        let request = SendRemoteAction::new(DeviceId::new(1), generation, RemoteAction::Up);
        assert_eq!(
            state.evaluate_remote_action(request.clone()),
            RemoteActionOutcome::Rejected {
                request,
                reason: RemoteActionRejection::NotConnected,
            }
        );

        let _ = state.set_connection_state(generation, ConnectionState::Ready);
        let _ = state.set_verified_actions(generation, [RemoteAction::Up]);
        let request = SendRemoteAction::new(DeviceId::new(1), generation, RemoteAction::Up);
        assert_eq!(
            state.evaluate_remote_action(request.clone()),
            RemoteActionOutcome::Eligible(request)
        );
        assert_eq!(
            state.control_status(RemoteAction::Up),
            ControlStatus::Available
        );
        assert_eq!(
            state.control_status(RemoteAction::PowerToggle),
            ControlStatus::Unavailable(RemoteActionRejection::DeferredAction)
        );
        assert_eq!(
            state.control_status(RemoteAction::Down),
            ControlStatus::Unavailable(RemoteActionRejection::UnverifiedAction)
        );
    }

    #[test]
    fn lifecycle_updates_are_generation_scoped_and_selection_resets_them() {
        let mut state = ControlState::none();
        let _ = state.select_device(DeviceDisplay::new(DeviceId::new(1), "Living Room"));
        let generation = state.selection_generation();

        assert_eq!(
            state.set_pairing_state(generation, PairingState::InProgress),
            LifecycleUpdateResult::Applied
        );
        assert_eq!(
            state.set_pairing_state(generation, PairingState::Ready),
            LifecycleUpdateResult::Applied
        );
        assert_eq!(
            state.set_connection_state(generation, ConnectionState::Connecting),
            LifecycleUpdateResult::Applied
        );
        assert_eq!(
            state.set_connection_state(generation, ConnectionState::Ready),
            LifecycleUpdateResult::Applied
        );
        assert_eq!(state.pairing_state(), PairingState::Ready);
        assert_eq!(state.connection_state(), ConnectionState::Ready);

        let _ = state.select_device(DeviceDisplay::new(DeviceId::new(2), "Bedroom"));
        assert_eq!(state.pairing_state(), PairingState::NotStarted);
        assert_eq!(state.connection_state(), ConnectionState::NotConnected);
        assert_eq!(
            state.set_connection_state(generation, ConnectionState::Failed),
            LifecycleUpdateResult::IgnoredStaleGeneration
        );
        assert_eq!(state.connection_state(), ConnectionState::NotConnected);

        let current_generation = state.selection_generation();
        assert_eq!(
            state.set_pairing_state(current_generation, PairingState::Failed),
            LifecycleUpdateResult::Applied
        );
        assert_eq!(state.pairing_state(), PairingState::Failed);

        let _ = state.clear_selection();
        assert_eq!(
            state.set_connection_state(state.selection_generation(), ConnectionState::Ready),
            LifecycleUpdateResult::IgnoredNoSelection
        );
    }
}
