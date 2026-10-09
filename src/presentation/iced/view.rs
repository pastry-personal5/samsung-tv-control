use super::split_bar::SplitBar;
use super::ui_message::Message;
use super::view_model::{feed_is_at_bottom, PrimaryView, RemoteControlViewState, ViewModel};
use crate::application::tv_control_coordinator::{WakeFailure, WakeStage};
use crate::application::wake::WakeInterface;
use crate::{RemoteAction, SendRemoteAction};
use ::iced::widget::{
    button, column, container, row, scrollable, text, text_input, tooltip, Space,
};
use ::iced::{Alignment, Element, Length};

pub const MESSAGE_FEED_ID: &str = "global-message-feed";

pub fn main_window(
    view_model: &ViewModel,
    wake_stage: WakeStage,
    wake_configured: bool,
    wake_tick: bool,
) -> Element<'_, Message> {
    let sidebar = column![
        navigation_button(PrimaryView::Power, view_model.primary_view()),
        navigation_button(PrimaryView::Remote, view_model.primary_view()),
        navigation_button(PrimaryView::Sources, view_model.primary_view()),
        navigation_button(PrimaryView::Apps, view_model.primary_view()),
        navigation_button(PrimaryView::TextInput, view_model.primary_view()),
        Space::new().height(Length::Fill),
        tooltip(
            button(text("⚙").size(20))
                .on_press(Message::OpenSettings)
                .width(Length::Fill),
            "Settings",
            tooltip::Position::Top,
        ),
    ]
    .padding(16)
    .spacing(10)
    .width(180);

    let main_pane = column![
        scrollable(primary_view(
            view_model.primary_view(),
            view_model.control_state(),
            wake_stage,
            wake_configured,
            wake_tick
        ))
        .height(Length::Fill),
        split_bar(view_model.message_pane_height()),
        global_messages(view_model),
    ]
    .spacing(8)
    .padding(16)
    .width(Length::Fill)
    .height(Length::Fill);

    row![sidebar, main_pane].height(Length::Fill).into()
}

fn navigation_button(
    destination: PrimaryView,
    selected: PrimaryView,
) -> ::iced::widget::Button<'static, Message> {
    let label = if destination == selected {
        format!("{} ✓", destination.title())
    } else {
        destination.title().to_owned()
    };

    button(text(label))
        .on_press(Message::Navigate(destination))
        .style(if destination == selected {
            button::secondary
        } else {
            button::primary
        })
        .width(Length::Fill)
}

fn primary_view(
    primary_view: PrimaryView,
    control_state: &RemoteControlViewState,
    wake_stage: WakeStage,
    wake_configured: bool,
    wake_tick: bool,
) -> Element<'static, Message> {
    match primary_view {
        PrimaryView::Power => power_view(control_state, wake_stage, wake_configured, wake_tick),
        PrimaryView::Remote => remote_view(control_state, wake_stage, wake_configured),
        PrimaryView::Sources => empty_primary_view(
            "Sources View",
            "No TV selected. Source choices will appear after choosing a TV in Settings.",
        ),
        PrimaryView::Apps => empty_primary_view(
            "Apps View",
            "No TV selected. Installed apps will appear after choosing a TV in Settings.",
        ),
        PrimaryView::TextInput => empty_primary_view(
            "Text Input View",
            "No TV selected. Text input is unavailable until a TV is selected.",
        ),
    }
}

fn empty_primary_view(title: &'static str, status: &'static str) -> Element<'static, Message> {
    container(column![text(title).size(26), Space::new().height(12), text(status)].spacing(8))
        .width(Length::Fill)
        .into()
}

fn remote_view(
    control_state: &RemoteControlViewState,
    wake_stage: WakeStage,
    wake_configured: bool,
) -> Element<'static, Message> {
    let directional_pad = column![
        remote_button("Up", RemoteAction::Up, control_state),
        row![
            remote_button("Left", RemoteAction::Left, control_state),
            remote_button("Enter", RemoteAction::Enter, control_state),
            remote_button("Right", RemoteAction::Right, control_state)
        ]
        .spacing(8),
        remote_button("Down", RemoteAction::Down, control_state),
    ]
    .align_x(Alignment::Center)
    .spacing(8);

    container(
        column![
            text("Remote View").size(26),
            Space::new().height(8),
            power_toggle_button(control_state, wake_stage, wake_configured),
            directional_pad,
            row![
                remote_button("Back", RemoteAction::Back, control_state),
                remote_button("Home", RemoteAction::Home, control_state)
            ]
            .spacing(8),
            text("Volume"),
            tooltip(
                button("Volume Slider"),
                "Exact volume is unavailable in this milestone.",
                tooltip::Position::Top
            ),
            row![
                remote_button("Mute", RemoteAction::Mute, control_state),
                remote_button("Volume Down", RemoteAction::VolumeDown, control_state),
                remote_button("Volume Up", RemoteAction::VolumeUp, control_state)
            ]
            .spacing(8),
            text("Keyboard: arrows, Return, Esc, Home, M, + (Shift+=), and - (Remote View only)."),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
}

fn power_toggle_button(
    control_state: &RemoteControlViewState,
    stage: WakeStage,
    configured: bool,
) -> Element<'static, Message> {
    let button = button("Power Toggle");
    let connected = control_state
        .action_disabled_reason(RemoteAction::PowerToggle)
        .is_none();
    let busy = matches!(
        stage,
        WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
    );
    if let Some(device) = control_state
        .selected_device
        .as_ref()
        .filter(|_| !busy && (connected || configured))
    {
        button
            .on_press(Message::PowerToggle(SendRemoteAction::new(
                device.id(),
                control_state.selection_generation,
                RemoteAction::PowerToggle,
            )))
            .into()
    } else {
        let reason = if busy {
            "Power is already in progress."
        } else if control_state.selected_device.is_none() {
            "No TV selected. Open Settings to choose a TV."
        } else if !configured {
            "Set an active Wake MAC in TV Settings before using Power while disconnected."
        } else {
            "Power is unavailable for this TV."
        };
        tooltip(button, reason, tooltip::Position::Top).into()
    }
}

fn power_view(
    control_state: &RemoteControlViewState,
    stage: WakeStage,
    configured: bool,
    tick: bool,
) -> Element<'static, Message> {
    let status = match stage {
        WakeStage::ConfigurationRequired => "Configure a wired or Wi-Fi MAC in TV Settings.",
        WakeStage::Idle => "Ready to send one wake packet.",
        WakeStage::CheckingConnection => "Checking the saved TV connection first…",
        WakeStage::PacketSending => "Sending one magic packet…",
        WakeStage::Reconnecting => "Magic packet sent. Waiting for the paired remote channel…",
        WakeStage::Connected => "Paired remote channel is ready. No wake packet was needed.",
        WakeStage::Ready => "Paired remote channel ready. Physical panel state is not measured.",
        WakeStage::Failed(WakeFailure::Timeout) | WakeStage::FailedAfterSend(WakeFailure::Timeout) => "Remote was not ready within 30 seconds. Check the TV and network, then try again.",
        WakeStage::Failed(WakeFailure::PairingRequired) | WakeStage::FailedAfterSend(WakeFailure::PairingRequired) => "Saved pairing needs attention. Re-pair in TV Settings.",
        WakeStage::Failed(WakeFailure::Local(crate::application::wake_transport::WakeSendError::InvalidTarget)) => "Saved TV address is not a valid local target. Check TV Settings.",
        WakeStage::Failed(WakeFailure::Local(crate::application::wake_transport::WakeSendError::NoIpv4Route)) => "No local IPv4 route to this TV. Check the Mac's network connection.",
        WakeStage::Failed(WakeFailure::Local(crate::application::wake_transport::WakeSendError::NoBroadcast)) => "TV is not on a usable local broadcast subnet. Check the saved TV address and network.",
        WakeStage::Failed(WakeFailure::Local(crate::application::wake_transport::WakeSendError::Permission)) => "Local Network permission denied. Allow this app access in macOS Settings, then try again.",
        WakeStage::Failed(WakeFailure::Local(crate::application::wake_transport::WakeSendError::SendFailed)) => "Magic packet could not be sent. Check the local network and try again.",
        WakeStage::Failed(WakeFailure::Local(crate::application::wake_transport::WakeSendError::Cancelled)) => "Wake cancelled before the packet was sent.",
        WakeStage::Failed(WakeFailure::Connection) | WakeStage::FailedAfterSend(WakeFailure::Connection) => "Could not establish the paired remote channel. Check TV Settings and try again.",
        WakeStage::FailedAfterSend(WakeFailure::Local(_)) => "Magic packet sent, but remote readiness could not be established.",
        WakeStage::Cancelled => "Wake cancelled before a packet was sent.",
        WakeStage::CancelledDuringSend => "Wake cancelled. The packet may already have been sent.",
        WakeStage::CancelledAfterSend => "Reconnect cancelled. The sent packet cannot be recalled.",
    };
    let active = matches!(
        stage,
        WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
    );
    let configured_text = if configured {
        "Configured"
    } else {
        "Needs setup"
    };
    let packet_text = match stage {
        WakeStage::PacketSending => "Sending",
        WakeStage::Reconnecting
        | WakeStage::Ready
        | WakeStage::FailedAfterSend(_)
        | WakeStage::CancelledAfterSend => "Sent; delivery unconfirmed",
        WakeStage::Failed(WakeFailure::Local(_)) => "Failed locally",
        WakeStage::CancelledDuringSend => "Send outcome unknown",
        _ => "Waiting",
    };
    let packet_step = match stage {
        WakeStage::PacketSending => StepState::Active,
        WakeStage::Reconnecting
        | WakeStage::Ready
        | WakeStage::FailedAfterSend(_)
        | WakeStage::CancelledAfterSend => StepState::Complete,
        WakeStage::Failed(WakeFailure::Local(_)) => StepState::Failed,
        WakeStage::CancelledDuringSend => StepState::Failed,
        _ => StepState::Waiting,
    };
    let reconnect_text = match stage {
        WakeStage::CheckingConnection | WakeStage::Reconnecting => "In progress",
        WakeStage::Connected | WakeStage::Ready => "Complete",
        WakeStage::Failed(_)
        | WakeStage::FailedAfterSend(_)
        | WakeStage::CancelledAfterSend
        | WakeStage::CancelledDuringSend
        | WakeStage::Cancelled => "Stopped",
        _ => "Waiting",
    };
    let reconnect_step = match stage {
        WakeStage::CheckingConnection | WakeStage::Reconnecting => StepState::Active,
        WakeStage::Connected | WakeStage::Ready => StepState::Complete,
        WakeStage::Failed(
            WakeFailure::Timeout | WakeFailure::PairingRequired | WakeFailure::Connection,
        )
        | WakeStage::FailedAfterSend(_) => StepState::Failed,
        _ => StepState::Waiting,
    };
    let ready_text = if matches!(stage, WakeStage::Connected | WakeStage::Ready) {
        "Ready"
    } else {
        "Waiting"
    };
    let ready_step = if matches!(stage, WakeStage::Connected | WakeStage::Ready) {
        StepState::Complete
    } else if matches!(stage, WakeStage::FailedAfterSend(_)) {
        StepState::Failed
    } else {
        StepState::Waiting
    };
    let connected = control_state
        .action_disabled_reason(RemoteAction::PowerToggle)
        .is_none();
    let wake_button = button("Wake").on_press_maybe(
        (configured
            && !connected
            && !active
            && stage != WakeStage::Ready
            && control_state.selected_device.is_some())
        .then_some(Message::Wake),
    );
    let wake_button = tooltip(
        wake_button,
        if control_state.selected_device.is_none() {
            "Select a saved TV in Settings first."
        } else if !configured {
            "Set an active Wake MAC in TV Settings first."
        } else if connected {
            "The paired remote channel is already connected."
        } else if active {
            "A power operation is already in progress."
        } else {
            "Send one magic packet to the selected TV."
        },
        tooltip::Position::Top,
    );
    let retry_button = button("Try again").on_press_maybe(
        (configured
            && !connected
            && matches!(
                stage,
                WakeStage::Failed(_)
                    | WakeStage::FailedAfterSend(_)
                    | WakeStage::Cancelled
                    | WakeStage::CancelledDuringSend
                    | WakeStage::CancelledAfterSend
            )
            && !matches!(
                stage,
                WakeStage::Failed(WakeFailure::PairingRequired)
                    | WakeStage::FailedAfterSend(WakeFailure::PairingRequired)
            ))
        .then_some(Message::Wake),
    );
    container(
        column![
            text("Power View").size(26),
            text("Wake Steps").size(20),
            wake_step(
                "Wake configuration",
                configured_text,
                if configured {
                    StepState::Complete
                } else {
                    StepState::Failed
                },
                tick
            ),
            wake_step("Magic packet", packet_text, packet_step, tick),
            wake_step("Reconnecting", reconnect_text, reconnect_step, tick),
            wake_step("Remote ready", ready_text, ready_step, tick),
            text(status),
            row![
                wake_button,
                power_toggle_button(control_state, stage, configured),
                retry_button,
                button("Cancel").on_press_maybe(active.then_some(Message::CancelWake))
            ]
            .spacing(8),
        ]
        .spacing(10),
    )
    .width(Length::Fill)
    .into()
}

#[derive(Clone, Copy)]
enum StepState {
    Waiting,
    Active,
    Complete,
    Failed,
}

fn wake_step(
    label: &'static str,
    status: &'static str,
    state: StepState,
    tick: bool,
) -> Element<'static, Message> {
    let mark = match state {
        StepState::Waiting => "○",
        StepState::Active if tick => "●",
        StepState::Active => "○",
        StepState::Complete => "✓",
        StepState::Failed => "!",
    };
    text(format!("{mark} {label}: {status}"))
        .style(match state {
            StepState::Waiting => ::iced::widget::text::primary,
            StepState::Active => ::iced::widget::text::warning,
            StepState::Complete => ::iced::widget::text::success,
            StepState::Failed => ::iced::widget::text::danger,
        })
        .into()
}

fn remote_button(
    label: &'static str,
    action: RemoteAction,
    control_state: &RemoteControlViewState,
) -> Element<'static, Message> {
    let button = button(label);
    if let Some(device) = control_state
        .selected_device
        .as_ref()
        .filter(|_| control_state.action_disabled_reason(action).is_none())
    {
        button
            .on_press(Message::AttemptRemoteAction(SendRemoteAction::new(
                device.id(),
                control_state.selection_generation,
                action,
            )))
            .into()
    } else {
        tooltip(
            button,
            control_state
                .action_disabled_reason(action)
                .unwrap_or("Remote actions are unavailable."),
            tooltip::Position::Top,
        )
        .into()
    }
}

fn split_bar(height: u16) -> Element<'static, Message> {
    SplitBar::new(height, Message::ResizeMessages).into()
}

fn global_messages(view_model: &ViewModel) -> Element<'_, Message> {
    let entries = if view_model.messages().entries().is_empty() {
        column![text("Global Messages Pane"), text("No messages yet.")].spacing(4)
    } else {
        let mut entries = column![text("Global Messages Pane").size(14)].spacing(4);
        for entry in view_model.messages().entries() {
            let severity = text(entry.severity.label())
                .size(12)
                .style(match entry.severity {
                    super::view_model::MessageSeverity::Information => {
                        ::iced::widget::text::primary
                    }
                    super::view_model::MessageSeverity::Warning => ::iced::widget::text::warning,
                });
            entries = entries.push(
                row![
                    container(severity).width(72),
                    text(&entry.text).size(13).width(Length::Fill),
                ]
                .spacing(8),
            );
        }
        entries
    };

    let feed_status = if view_model.messages().unread_count() > 0 {
        format!(
            "{} new message(s) below",
            view_model.messages().unread_count()
        )
    } else {
        String::new()
    };

    container(
        column![
            scrollable(entries)
                .id(MESSAGE_FEED_ID)
                .height(Length::Fill)
                .on_scroll(|viewport| {
                    let bounds = viewport.bounds();
                    let content = viewport.content_bounds();
                    let at_bottom = feed_is_at_bottom(
                        content.height,
                        bounds.height,
                        viewport.absolute_offset().y,
                    );
                    Message::FeedScrolled { at_bottom }
                }),
            (!feed_status.is_empty()).then(|| text(feed_status).size(12)),
        ]
        .spacing(4),
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(f32::from(view_model.message_pane_height())))
    .into()
}

pub struct SettingsView<'a> {
    pub address: &'a str,
    pub candidates: &'a [crate::application::tv_address::TvHost],
    pub saved_devices: &'a [crate::application::device_repository::SavedDevice],
    pub fingerprint: Option<String>,
    pub observed_name: Option<String>,
    pub observed_model: Option<String>,
    pub status: &'a str,
    pub selected_label: Option<&'a str>,
    pub pairing_pending: bool,
    pub forget_pending: bool,
    pub wake_wired: &'a str,
    pub wake_wifi: &'a str,
    pub wake_active: Option<WakeInterface>,
}

pub fn settings_window(options: SettingsView<'_>) -> Element<'_, Message> {
    let SettingsView {
        address,
        candidates,
        saved_devices,
        fingerprint,
        observed_name,
        observed_model,
        status,
        selected_label,
        pairing_pending,
        forget_pending,
        wake_wired,
        wake_wifi,
        wake_active,
    } = options;
    let sidebar = column![text("Settings").size(18), text("TV (current view)")]
        .padding(16)
        .spacing(10)
        .width(150);

    let selected = selected_label.unwrap_or("None");
    let has_fingerprint = fingerprint.is_some();
    let fingerprint_text = fingerprint
        .map(|value| format!("Observed certificate SHA-256: {value}"))
        .unwrap_or_else(|| "No certificate observed yet.".to_owned());
    let mut main_content = column![
        text("TV settings").size(26),
        text(format!("Selected TV: {selected}")),
        text("Enter TV Address"),
        button("Discover TVs").on_press(Message::DiscoverTv),
    ]
    .spacing(12);
    for candidate in candidates {
        main_content = main_content.push(
            button(text(format!(
                "Unconfirmed candidate: {} — Probe",
                candidate.as_str()
            )))
            .on_press(Message::UseCandidate(candidate.clone())),
        );
    }
    main_content = main_content
            .push(
            text_input("Local IP address or host name", address)
                .on_input(Message::TvAddressChanged)
                .on_submit(Message::ProbeTv),
            )
            .push(button("Probe Secure TV (8002)").on_press(Message::ProbeTv))
            .push(text(fingerprint_text).size(12))
            .push(text(format!(
                "Observed name: {}",
                observed_name.unwrap_or_else(|| "Unavailable".to_owned())
            )))
            .push(text(format!(
                "Observed model: {}",
                observed_model.unwrap_or_else(|| "Unavailable".to_owned())
            )))
            .push(text("Confirm the address and certificate on the intended TV. Approve the matching TV prompt."))
            .push(button("Confirm TV and Pair").on_press_maybe(
                if has_fingerprint && !pairing_pending && !forget_pending { Some(Message::ConfirmAndPair) } else { None }
            ))
            .push(button("Re-pair Selected TV").on_press_maybe(
                if has_fingerprint && selected_label.is_some() && !pairing_pending && !forget_pending { Some(Message::ConfirmAndRepair) } else { None }
            ))
            .push(row![
                button("Retry Connection").on_press_maybe(
                    selected_label.filter(|_| !forget_pending).map(|_| Message::ConnectSelected)
                ),
                button("Forget Selected TV").on_press_maybe(
                    selected_label.filter(|_| !forget_pending).map(|_| Message::ForgetSelected)
                )
            ].spacing(8))
            .push(text("Wake configuration").size(20))
            .push(text("Use the MAC shown by the TV for its wired or wireless network interface."))
            .push(text_input("Wired MAC (AA:BB:CC:DD:EE:FF)", wake_wired).on_input(Message::WakeWiredChanged))
            .push(text_input("Wi-Fi MAC (AA:BB:CC:DD:EE:FF)", wake_wifi).on_input(Message::WakeWifiChanged))
            .push(row![
                button(if wake_active == Some(WakeInterface::Wired) { "Wired selected" } else { "Use Wired" }).on_press(Message::WakeInterfaceSelected(WakeInterface::Wired)),
                button(if wake_active == Some(WakeInterface::WiFi) { "Wi-Fi selected" } else { "Use Wi-Fi" }).on_press(Message::WakeInterfaceSelected(WakeInterface::WiFi)),
                button("Disable Wake").on_press(Message::WakeInterfaceCleared),
                button("Save Wake configuration").on_press_maybe(selected_label.filter(|_| !forget_pending).map(|_| Message::SaveWakeConfiguration)),
            ].spacing(8))
            .push(text(status));
    for device in saved_devices {
        main_content = main_content.push(
            button(text(format!("Select saved TV: {}", device.label)))
                .on_press_maybe((!forget_pending).then_some(Message::SelectSaved(device.id))),
        );
    }
    let main_pane = container(scrollable(main_content))
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    row![sidebar, main_pane].height(Length::Fill).into()
}
