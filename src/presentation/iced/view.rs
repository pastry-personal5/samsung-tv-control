use super::icons::Icon;
use super::remote_image::RemoteImage;
use super::split_bar::SplitBar;
use super::ui_message::{Message, SettingsPage};
use super::view_model::{feed_is_at_bottom, PrimaryView, RemoteControlViewState, ViewModel};
use crate::application::tv_control_coordinator::{WakeFailure, WakeStage};
use crate::application::wake::WakeInterface;
use crate::{RemoteAction, SendRemoteAction};
use ::iced::alignment::Horizontal;
use ::iced::widget::{
    button, column, container, radio, row, scrollable, text, text_input, tooltip, Space,
};
use ::iced::{Alignment, Background, Border, Color, Element, Length, Theme};

pub const MESSAGE_FEED_ID: &str = "global-message-feed";
const SIDEBAR_WIDTH: f32 = 68.0;
const SIDEBAR_SLOT_HEIGHT: f32 = 58.0;
const VIEW_BOX_WIDTH: f32 = 660.0;
const TAB_TITLE_SIZE: u32 = 14;
const WAKE_STEP_MARKER_WIDTH: f32 = 16.0;
const SETTINGS_SIDEBAR_WIDTH: f32 = 186.0;
const SETTINGS_SIDEBAR_ITEM_HEIGHT: f32 = 52.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WakeChoice {
    Wired,
    WiFi,
    Disabled,
}

impl WakeChoice {
    fn selected(active: Option<WakeInterface>) -> Self {
        match active {
            Some(WakeInterface::Wired) => Self::Wired,
            Some(WakeInterface::WiFi) => Self::WiFi,
            None => Self::Disabled,
        }
    }
}

fn graphite(value: u8) -> Color {
    Color::from_rgb8(value, value, value)
}

fn sidebar_color() -> Color {
    graphite(25)
}

fn sidebar_button_style(selected: bool, _: &Theme, status: button::Status) -> button::Style {
    let background = match (selected, status) {
        (true, button::Status::Hovered | button::Status::Pressed) => Color::from_rgb8(48, 65, 82),
        (true, _) => Color::from_rgb8(42, 50, 60),
        (false, button::Status::Hovered | button::Status::Pressed) => graphite(48),
        _ => sidebar_color(),
    };
    button::Style {
        background: Some(Background::Color(background)),
        border: Border {
            width: if selected { 2.0 } else { 0.0 },
            color: if selected {
                Color::from_rgb8(91, 166, 224)
            } else {
                background
            },
            ..Border::default()
        },
        ..button::Style::default()
    }
}

fn settings_sidebar_color() -> Color {
    Color::from_rgb8(32, 39, 46)
}

fn settings_sidebar_button_style(
    selected: bool,
    _: &Theme,
    status: button::Status,
) -> button::Style {
    let background = match (selected, status) {
        (true, button::Status::Hovered) => Color::from_rgb8(53, 91, 121),
        (true, button::Status::Pressed) => Color::from_rgb8(39, 72, 100),
        (true, _) => Color::from_rgb8(45, 78, 105),
        (false, button::Status::Hovered) => Color::from_rgb8(48, 65, 78),
        (false, button::Status::Pressed) => Color::from_rgb8(40, 57, 72),
        _ => settings_sidebar_color(),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if status == button::Status::Disabled {
            Color::from_rgb8(145, 157, 167)
        } else if selected {
            Color::from_rgb8(250, 252, 255)
        } else {
            Color::from_rgb8(226, 234, 241)
        },
        border: Border::default(),
        ..button::Style::default()
    }
}

fn action_button_style(_: &Theme, status: button::Status) -> button::Style {
    let background = match status {
        button::Status::Active => graphite(52),
        button::Status::Hovered => Color::from_rgb8(55, 77, 99),
        button::Status::Pressed => Color::from_rgb8(38, 91, 139),
        button::Status::Disabled => graphite(39),
    };
    button::Style {
        background: Some(Background::Color(background)),
        text_color: if status == button::Status::Disabled {
            graphite(130)
        } else {
            Color::WHITE
        },
        border: Border {
            color: if status == button::Status::Hovered {
                Color::from_rgb8(107, 165, 212)
            } else {
                graphite(76)
            },
            width: 1.0,
            radius: 8.0.into(),
        },
        ..button::Style::default()
    }
}

fn box_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(graphite(37))),
        border: Border {
            color: graphite(76),
            width: 1.0,
            radius: 12.0.into(),
        },
        ..container::Style::default()
    }
}

fn card_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(graphite(46))),
        border: Border {
            color: graphite(80),
            width: 1.0,
            radius: 9.0.into(),
        },
        ..container::Style::default()
    }
}

fn active_tab_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(46, 46, 48))),
        text_color: Some(Color::from_rgb8(232, 232, 232)),
        border: Border {
            color: graphite(82),
            width: 1.0,
            radius: 6.0.into(),
        },
        ..container::Style::default()
    }
}

fn message_feed_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(graphite(28))),
        border: Border {
            color: graphite(92),
            width: 1.0,
            radius: 8.0.into(),
        },
        ..container::Style::default()
    }
}

fn active_tab(title: &'static str) -> Element<'static, Message> {
    container(text(title).size(TAB_TITLE_SIZE))
        .padding([5, 10])
        .style(active_tab_style)
        .into()
}

fn wake_choice_radio(
    label: &'static str,
    choice: WakeChoice,
    selected: WakeChoice,
    enabled: bool,
) -> Element<'static, Message> {
    if enabled {
        return radio(label, choice, Some(selected), move |choice| match choice {
            WakeChoice::Wired => Message::WakeInterfaceSelected(WakeInterface::Wired),
            WakeChoice::WiFi => Message::WakeInterfaceSelected(WakeInterface::WiFi),
            WakeChoice::Disabled => Message::WakeInterfaceCleared,
        })
        .size(18)
        .text_size(14)
        .into();
    }

    row![
        text(if choice == selected { "◉" } else { "○" }).style(|_| {
            ::iced::widget::text::Style {
                color: Some(graphite(118)),
            }
        }),
        text(label).size(14).style(|_| ::iced::widget::text::Style {
            color: Some(graphite(118)),
        })
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn wake_choice_enabled(choice: WakeChoice, wired: &str, wifi: &str, forget_pending: bool) -> bool {
    if forget_pending {
        return false;
    }

    match choice {
        WakeChoice::Wired => wired.parse::<crate::domain::MacAddress>().is_ok(),
        WakeChoice::WiFi => wifi.parse::<crate::domain::MacAddress>().is_ok(),
        WakeChoice::Disabled => true,
    }
}

fn saved_tv_radio(
    id: crate::DeviceId,
    selected_id: Option<crate::DeviceId>,
    enabled: bool,
) -> Element<'static, Message> {
    if enabled {
        return radio("", id, selected_id, Message::SelectSaved)
            .size(18)
            .into();
    }

    text(if selected_id == Some(id) {
        "◉"
    } else {
        "○"
    })
    .style(|_| ::iced::widget::text::Style {
        color: Some(graphite(118)),
    })
    .into()
}

fn tooltip_style(_: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgb8(14, 23, 33))),
        text_color: Some(Color::from_rgb8(244, 248, 252)),
        border: Border {
            color: Color::from_rgb8(96, 165, 212),
            width: 1.0,
            radius: 7.0.into(),
        },
        ..container::Style::default()
    }
}

fn view_box(title: &'static str, content: Element<'static, Message>) -> Element<'static, Message> {
    container(
        container(column![text(title).size(26), content].spacing(18))
            .padding(22)
            .width(Length::Fill)
            .max_width(VIEW_BOX_WIDTH)
            .style(box_style),
    )
    .center_x(Length::Fill)
    .padding([18, 22])
    .into()
}

fn card<'a>(title: &'static str, content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(column![text(title).size(18), content.into()].spacing(12))
        .padding(16)
        .width(Length::Fill)
        .style(card_style)
        .into()
}

fn titled_panel<'a>(
    title: &'static str,
    content: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    container(
        column![
            row![text(title).size(16), Space::new().width(Length::Fill)].align_y(Alignment::Center),
            content.into()
        ]
        .spacing(12),
    )
    .padding(16)
    .width(Length::Fill)
    .style(card_style)
    .into()
}

fn titled_frame(
    title: &'static str,
    content: Element<'static, Message>,
    max_width: Option<f32>,
) -> Element<'static, Message> {
    let frame = container(
        column![
            row![active_tab(title), Space::new().width(Length::Fill)].align_y(Alignment::End),
            content
        ]
        .spacing(12),
    )
    .padding(22)
    .width(Length::Fill)
    .style(box_style);

    container(frame.max_width(max_width.unwrap_or(VIEW_BOX_WIDTH)))
        .center_x(Length::Fill)
        .padding([18, 22])
        .into()
}

pub fn main_window(
    view_model: &ViewModel,
    wake_stage: WakeStage,
    wake_configured: bool,
    wake_tick: bool,
) -> Element<'_, Message> {
    let sidebar = container(
        column![
            navigation_button(PrimaryView::Power, view_model.primary_view()),
            navigation_button(PrimaryView::Remote, view_model.primary_view()),
            navigation_button(PrimaryView::Sources, view_model.primary_view()),
            navigation_button(PrimaryView::Apps, view_model.primary_view()),
            navigation_button(PrimaryView::TextInput, view_model.primary_view()),
            Space::new().height(Length::Fill),
            tooltip(
                button(container(Icon::Settings.image(30.0, true)).center(Length::Fill))
                    .on_press(Message::OpenSettings)
                    .width(Length::Fill)
                    .height(SIDEBAR_SLOT_HEIGHT)
                    .padding(0)
                    .style(|theme, status| sidebar_button_style(false, theme, status)),
                "Settings",
                tooltip::Position::Right,
            )
            .style(tooltip_style),
        ]
        .spacing(0)
        .width(SIDEBAR_WIDTH)
        .height(Length::Fill),
    )
    .width(SIDEBAR_WIDTH)
    .height(Length::Fill)
    .style(|_| container::Style::default().background(sidebar_color()));

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

fn navigation_icon(destination: PrimaryView) -> Icon {
    match destination {
        PrimaryView::Power => Icon::Power,
        PrimaryView::Remote => Icon::Remote,
        PrimaryView::Sources => Icon::Sources,
        PrimaryView::Apps => Icon::Apps,
        PrimaryView::TextInput => Icon::TextInput,
    }
}

fn navigation_button(destination: PrimaryView, selected: PrimaryView) -> Element<'static, Message> {
    let is_selected = destination == selected;
    tooltip(
        button(container(navigation_icon(destination).image(30.0, true)).center(Length::Fill))
            .on_press(Message::Navigate(destination))
            .width(Length::Fill)
            .height(SIDEBAR_SLOT_HEIGHT)
            .padding(0)
            .style(move |theme, status| sidebar_button_style(is_selected, theme, status)),
        destination.title(),
        tooltip::Position::Right,
    )
    .style(tooltip_style)
    .into()
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
    view_box(title, text(status).into())
}

fn remote_view(
    control_state: &RemoteControlViewState,
    wake_stage: WakeStage,
    wake_configured: bool,
) -> Element<'static, Message> {
    titled_frame(
        "Remote View",
        container(RemoteImage::view(
            control_state,
            wake_stage,
            wake_configured,
        ))
        .center_x(Length::Fill)
        .into(),
        None,
    )
}

fn power_toggle_button(
    control_state: &RemoteControlViewState,
    stage: WakeStage,
    configured: bool,
    with_label: bool,
) -> Element<'static, Message> {
    let connected = control_state
        .action_disabled_reason(RemoteAction::PowerToggle)
        .is_none();
    let busy = matches!(
        stage,
        WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
    );
    let enabled_device = control_state
        .selected_device
        .as_ref()
        .filter(|_| !busy && (connected || configured));
    let reason = if enabled_device.is_some() {
        "Power Toggle"
    } else if busy {
        "Power is already in progress."
    } else if control_state.selected_device.is_none() {
        "No TV selected. Open Settings to choose a TV."
    } else if !configured {
        "Set an active Wake MAC in TV Settings before using Power while disconnected."
    } else {
        "Power is unavailable for this TV."
    };
    let button = action_button(
        Icon::Power,
        "Power Toggle",
        with_label,
        enabled_device.is_some(),
    );
    let button = if with_label {
        button
    } else {
        button.height(48).width(Length::Fixed(56.0))
    };
    tooltip(
        button.on_press_maybe(enabled_device.map(|device| {
            Message::PowerToggle(SendRemoteAction::new(
                device.id(),
                control_state.selection_generation,
                RemoteAction::PowerToggle,
            ))
        })),
        reason,
        tooltip::Position::Top,
    )
    .style(tooltip_style)
    .into()
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
        WakeStage::Ready => "Paired remote channel is ready. Physical panel state is not measured.",
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
    let wake_enabled = configured
        && !connected
        && !active
        && stage != WakeStage::Ready
        && control_state.selected_device.is_some();
    let wake_button = action_button(Icon::Wake, "Wake", true, wake_enabled)
        .on_press_maybe(wake_enabled.then_some(Message::Wake));
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
    )
    .style(tooltip_style);
    let retry_enabled = configured
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
        );
    let retry_button = action_button(Icon::Retry, "Try again", true, retry_enabled)
        .on_press_maybe(retry_enabled.then_some(Message::Wake));
    titled_frame(
        "Power View",
        column![
            titled_panel(
                "Wake Steps",
                column![
                    wake_step(
                        "Wake configuration",
                        configured_text,
                        if configured {
                            StepState::Complete
                        } else {
                            StepState::Failed
                        },
                        tick,
                        active
                    ),
                    wake_step("Magic packet", packet_text, packet_step, tick, active),
                    wake_step("Reconnecting", reconnect_text, reconnect_step, tick, active),
                    wake_step("Remote ready", ready_text, ready_step, tick, active),
                    text(status).style(|_| ::iced::widget::text::Style {
                        color: Some(graphite(145)),
                    }),
                ]
                .spacing(10)
            ),
            titled_panel(
                "Power Controls",
                container(
                    row![
                        wake_button,
                        power_toggle_button(control_state, stage, configured, true),
                        retry_button,
                        action_button(Icon::Cancel, "Cancel", true, active)
                            .on_press_maybe(active.then_some(Message::CancelWake))
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .center_x(Length::Fill),
            ),
        ]
        .spacing(14)
        .into(),
        None,
    )
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
    active: bool,
) -> Element<'static, Message> {
    if !active {
        return row![
            Space::new().width(WAKE_STEP_MARKER_WIDTH),
            text(format!("{label}: {status}")).style(|_| ::iced::widget::text::Style {
                color: Some(graphite(145)),
            })
        ]
        .align_y(Alignment::Center)
        .into();
    }

    let mark = match state {
        StepState::Waiting => "○",
        StepState::Active if tick => "●",
        StepState::Active => "○",
        StepState::Complete => "✓",
        StepState::Failed => "!",
    };
    let style = match state {
        StepState::Waiting => ::iced::widget::text::primary,
        StepState::Active => ::iced::widget::text::warning,
        StepState::Complete => ::iced::widget::text::success,
        StepState::Failed => ::iced::widget::text::danger,
    };
    row![
        container(text(mark).style(style))
            .width(WAKE_STEP_MARKER_WIDTH)
            .align_x(Horizontal::Center),
        text(format!("{label}: {status}")).style(style)
    ]
    .align_y(Alignment::Center)
    .into()
}

fn action_button(
    icon: Icon,
    label: &'static str,
    with_label: bool,
    enabled: bool,
) -> ::iced::widget::Button<'static, Message> {
    let content: Element<'static, Message> = if with_label {
        container(
            row![icon.image(22.0, enabled), text(label).size(14)]
                .spacing(7)
                .align_y(Alignment::Center),
        )
        .center_y(Length::Fill)
        .into()
    } else {
        container(icon.image(28.0, enabled))
            .center(Length::Fill)
            .into()
    };
    button(content)
        .height(50)
        .width(if with_label {
            Length::Shrink
        } else {
            Length::Fixed(58.0)
        })
        .padding(if with_label { 10 } else { 0 })
        .style(action_button_style)
}

fn split_bar(height: u16) -> Element<'static, Message> {
    SplitBar::new(height, Message::ResizeMessages).into()
}

fn global_messages(view_model: &ViewModel) -> Element<'_, Message> {
    let entries = if view_model.messages().entries().is_empty() {
        column![text("No messages yet.").size(13)].spacing(6)
    } else {
        let mut entries = column![].spacing(6);
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

    let feed = container(
        scrollable(entries)
            .id(MESSAGE_FEED_ID)
            .height(Length::Fill)
            .on_scroll(|viewport| {
                let bounds = viewport.bounds();
                let content = viewport.content_bounds();
                let at_bottom =
                    feed_is_at_bottom(content.height, bounds.height, viewport.absolute_offset().y);
                Message::FeedScrolled { at_bottom }
            }),
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fill)
    .style(message_feed_style);

    container(
        column![
            row![active_tab("Output"), Space::new().width(Length::Fill)].align_y(Alignment::End),
            feed,
            (!feed_status.is_empty()).then(|| text(feed_status).size(12)),
        ]
        .spacing(4),
    )
    .width(Length::Fill)
    .height(Length::Fixed(f32::from(view_model.message_pane_height())))
    .into()
}

pub struct SettingsView<'a> {
    pub settings_page: SettingsPage,
    pub address: &'a str,
    pub candidates: &'a [crate::application::tv_address::TvHost],
    pub saved_devices: &'a [crate::application::device_repository::SavedDevice],
    pub fingerprint: Option<String>,
    pub observed_name: Option<String>,
    pub observed_model: Option<String>,
    pub observed_host: Option<&'a crate::application::tv_address::TvHost>,
    pub status: &'a str,
    pub selected_id: Option<crate::DeviceId>,
    pub selected_label: Option<&'a str>,
    pub pairing_pending: bool,
    pub forget_pending: bool,
    pub wake_wired: &'a str,
    pub wake_wifi: &'a str,
    pub wake_active: Option<WakeInterface>,
}

fn candidate_hosts(
    discovered: &[crate::application::tv_address::TvHost],
    saved: &[crate::application::device_repository::SavedDevice],
    address: &str,
) -> Vec<crate::application::tv_address::TvHost> {
    let is_saved = |host: &crate::application::tv_address::TvHost| {
        saved.iter().any(|device| device.host == *host)
    };
    let mut hosts = Vec::new();
    for host in discovered {
        if !is_saved(host) && !hosts.contains(host) {
            hosts.push(host.clone());
        }
    }
    if let Ok(manual) = crate::application::tv_address::TvHost::parse(address) {
        if !is_saved(&manual) && !hosts.contains(&manual) {
            hosts.push(manual);
        }
    }
    hosts
}

fn observed_matches(
    host: &crate::application::tv_address::TvHost,
    observed_host: Option<&crate::application::tv_address::TvHost>,
    has_fingerprint: bool,
) -> bool {
    has_fingerprint && observed_host == Some(host)
}

pub fn settings_window(options: SettingsView<'_>) -> Element<'_, Message> {
    let SettingsView {
        settings_page,
        address,
        candidates,
        saved_devices,
        fingerprint,
        observed_name,
        observed_model,
        observed_host,
        status,
        selected_id,
        selected_label,
        pairing_pending,
        forget_pending,
        wake_wired,
        wake_wifi,
        wake_active,
    } = options;
    let settings_item = |label, page| {
        button(
            container(text(label).size(15))
                .width(Length::Fill)
                .height(Length::Fill)
                .align_y(Alignment::Center),
        )
        .padding([0, 16])
        .width(Length::Fill)
        .height(Length::Fixed(SETTINGS_SIDEBAR_ITEM_HEIGHT))
        .on_press(Message::SelectSettingsPage(page))
        .style(move |theme, status| {
            settings_sidebar_button_style(settings_page == page, theme, status)
        })
    };
    let sidebar = container(
        column![
            container(
                text("Settings")
                    .size(17)
                    .style(|_| ::iced::widget::text::Style {
                        color: Some(Color::from_rgb8(183, 196, 207)),
                    })
            )
            .width(Length::Fill)
            .height(Length::Fixed(58.0))
            .padding([0, 16])
            .align_y(Alignment::Center),
            settings_item("Discovery", SettingsPage::Discovery),
            settings_item("Wake on LAN", SettingsPage::WakeOnLan),
        ]
        .spacing(0)
        .width(Length::Fill),
    )
    .width(Length::Fixed(SETTINGS_SIDEBAR_WIDTH))
    .height(Length::Fill)
    .style(|_| container::Style::default().background(settings_sidebar_color()));

    let selected = selected_label.unwrap_or("None");
    let has_fingerprint = fingerprint.is_some();
    let fingerprint_text = fingerprint
        .map(|value| format!("Observed certificate SHA-256: {value}"))
        .unwrap_or_else(|| "No certificate observed yet.".to_owned());
    let mut selected_content = column![text(format!("Current selection: {selected}"))].spacing(10);
    if !saved_devices.is_empty() {
        let header = row![
            container(text("Select").size(13)).width(Length::Fixed(48.0)),
            container(text("TV name").size(13)).width(Length::FillPortion(3)),
            container(text("TV IP Address").size(13)).width(Length::FillPortion(2)),
            container(text("Actions").size(13)).width(Length::Fixed(180.0)),
        ]
        .align_y(Alignment::Center)
        .spacing(8)
        .padding([5, 8]);
        let mut rows = column![header].spacing(2);
        for device in saved_devices {
            let selected_row = selected_id == Some(device.id);
            let checked_row = observed_matches(&device.host, observed_host, has_fingerprint);
            let actions: Element<'_, Message> = if selected_row {
                row![
                    button("Check TV")
                        .on_press_maybe(
                            (!pairing_pending && !forget_pending)
                                .then(|| { Message::ProbeCandidate(device.host.clone()) })
                        )
                        .style(action_button_style),
                    button("Re-pair")
                        .on_press_maybe(
                            (checked_row && !pairing_pending && !forget_pending)
                                .then_some(Message::ConfirmAndRepair)
                        )
                        .style(action_button_style),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
                .into()
            } else {
                Space::new().into()
            };
            rows = rows.push(
                row![
                    container(saved_tv_radio(device.id, selected_id, !forget_pending))
                        .width(Length::Fixed(48.0)),
                    container(text(&device.label)).width(Length::FillPortion(3)),
                    container(text(device.host.as_str())).width(Length::FillPortion(2)),
                    container(actions).width(Length::Fixed(180.0)),
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .padding([6, 8]),
            );
        }
        selected_content = selected_content
            .push(text("Saved TVs").size(13))
            .push(container(rows).style(message_feed_style));
    }
    if saved_devices.is_empty() {
        selected_content = selected_content.push(text("No saved TVs yet.").size(13));
    }

    let candidates = candidate_hosts(candidates, saved_devices, address);
    if !candidates.is_empty() {
        let header = row![
            container(text("Select").size(13)).width(Length::Fixed(48.0)),
            container(text("TV name").size(13)).width(Length::Fixed(90.0)),
            container(text("TV IP Address").size(13)).width(Length::Fixed(128.0)),
            container(text("Observed TV").size(13)).width(Length::Fill),
            container(text("Actions").size(13)).width(Length::Fixed(180.0)),
        ]
        .align_y(Alignment::Center)
        .spacing(8)
        .padding([5, 8]);
        let selected_candidate = candidates
            .iter()
            .position(|candidate| candidate.as_str() == address);
        let mut rows = column![header].spacing(2);
        for (index, candidate) in candidates.iter().enumerate() {
            let target = candidate.clone();
            let checked_row = observed_matches(candidate, observed_host, has_fingerprint);
            let details: Element<'_, Message> = if checked_row {
                column![
                    text(format!(
                        "Name: {}",
                        observed_name.as_deref().unwrap_or("Unavailable")
                    ))
                    .size(12),
                    text(format!(
                        "Model: {}",
                        observed_model.as_deref().unwrap_or("Unavailable")
                    ))
                    .size(12),
                ]
                .spacing(2)
                .into()
            } else {
                text("Not checked").size(12).into()
            };
            rows = rows.push(
                row![
                    container(
                        radio("", index, selected_candidate, move |_| {
                            Message::UseCandidate(target.clone())
                        })
                        .size(18),
                    )
                    .width(Length::Fixed(48.0)),
                    container(text("Candidate")).width(Length::Fixed(90.0)),
                    container(text(candidate.as_str().to_owned())).width(Length::Fixed(128.0)),
                    container(details).width(Length::Fill),
                    container(
                        row![
                            button("Check TV")
                                .on_press_maybe(
                                    (!pairing_pending && !forget_pending)
                                        .then(|| { Message::ProbeCandidate(candidate.clone()) })
                                )
                                .style(action_button_style),
                            button("Pair")
                                .on_press_maybe(
                                    (checked_row && !pairing_pending && !forget_pending)
                                        .then_some(Message::ConfirmAndPair)
                                )
                                .style(action_button_style),
                        ]
                        .spacing(6)
                        .align_y(Alignment::Center),
                    )
                    .width(Length::Fixed(180.0)),
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .padding([6, 8]),
            );
        }
        selected_content = selected_content
            .push(text("Discovered candidates").size(13))
            .push(container(rows).style(message_feed_style));
    }

    let mut pairing_content = column![
        text("Enter or discover the address of the intended TV.").size(13),
        button("Discover TVs")
            .on_press(Message::DiscoverTv)
            .style(action_button_style),
    ]
    .spacing(10);
    pairing_content = pairing_content
        .push(
            text_input("Local IP address or host name", address)
                .on_input(Message::TvAddressChanged)
                .on_submit(Message::ProbeTv),
        )
            .push(text(fingerprint_text).size(12))
            .push(text("Check the intended TV, confirm its address and certificate, then approve the pairing prompt on the TV."));

    let connection_content = column![
        text("Retry the trusted remote connection or forget this TV and its stored pairing.")
            .size(13),
        row![
            button("Retry Connection")
                .on_press_maybe(
                    selected_label
                        .filter(|_| !forget_pending)
                        .map(|_| Message::ConnectSelected)
                )
                .style(action_button_style),
            button("Forget Selected TV")
                .on_press_maybe(
                    selected_label
                        .filter(|_| !forget_pending)
                        .map(|_| Message::ForgetSelected)
                )
                .style(action_button_style)
        ]
        .spacing(8)
    ]
    .spacing(10);

    let wake_content = column![
        text("Enter the MAC shown by the TV for its wired or Wi-Fi network interface.").size(13),
        text_input("Wired MAC (AA:BB:CC:DD:EE:FF)", wake_wired).on_input(Message::WakeWiredChanged),
        text_input("Wi-Fi MAC (AA:BB:CC:DD:EE:FF)", wake_wifi).on_input(Message::WakeWifiChanged),
        row![
            wake_choice_radio(
                "Wired",
                WakeChoice::Wired,
                WakeChoice::selected(wake_active),
                wake_choice_enabled(WakeChoice::Wired, wake_wired, wake_wifi, forget_pending),
            ),
            wake_choice_radio(
                "Wi-Fi",
                WakeChoice::WiFi,
                WakeChoice::selected(wake_active),
                wake_choice_enabled(WakeChoice::WiFi, wake_wired, wake_wifi, forget_pending),
            ),
            wake_choice_radio(
                "Disabled",
                WakeChoice::Disabled,
                WakeChoice::selected(wake_active),
                wake_choice_enabled(WakeChoice::Disabled, wake_wired, wake_wifi, forget_pending),
            ),
        ]
        .spacing(8),
    ]
    .spacing(10);

    let main_content: Element<'_, Message> = match settings_page {
        SettingsPage::Discovery => column![
            text("Discovery").size(26),
            card("TV List", selected_content),
            card("Discovery and Pairing", pairing_content),
            card("Connection recovery", connection_content),
            card("Guidance", text(status)),
        ]
        .spacing(14)
        .into(),
        SettingsPage::WakeOnLan => column![
            text("Wake on LAN").size(26),
            card("Wake Configuration", wake_content),
            card("Guidance", text(status)),
        ]
        .spacing(14)
        .into(),
    };
    let main_pane = container(scrollable(main_content))
        .padding(20)
        .width(Length::Fill)
        .height(Length::Fill);

    row![sidebar, main_pane].height(Length::Fill).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_rows_omit_saved_tvs_and_keep_a_manual_target() {
        use crate::application::device_repository::SavedDevice;
        use crate::application::tv_address::TvHost;

        let saved_host = TvHost::parse("192.168.1.10").unwrap();
        let new_host = TvHost::parse("192.168.1.11").unwrap();
        let manual_host = TvHost::parse("192.168.1.12").unwrap();
        let saved = [SavedDevice {
            id: crate::DeviceId::new(1),
            label: "Living Room".to_owned(),
            host: saved_host.clone(),
            wake: crate::application::wake::WakeConfiguration::default(),
        }];

        assert_eq!(
            candidate_hosts(
                &[saved_host, new_host.clone(), new_host.clone(),],
                &saved,
                manual_host.as_str(),
            ),
            vec![new_host, manual_host]
        );
    }

    #[test]
    fn only_the_checked_tv_can_offer_pairing() {
        use crate::application::tv_address::TvHost;

        let checked = TvHost::parse("192.168.1.10").unwrap();
        let other = TvHost::parse("192.168.1.11").unwrap();
        assert!(observed_matches(&checked, Some(&checked), true));
        assert!(!observed_matches(&other, Some(&checked), true));
        assert!(!observed_matches(&checked, Some(&checked), false));
        assert!(!observed_matches(&checked, None, true));
    }

    #[test]
    fn sidebar_buttons_rest_on_the_rail_and_distinguish_hover_from_selection() {
        let resting = sidebar_button_style(false, &Theme::Dark, button::Status::Active);
        let hovered = sidebar_button_style(false, &Theme::Dark, button::Status::Hovered);
        let selected = sidebar_button_style(true, &Theme::Dark, button::Status::Active);

        assert_eq!(resting.background, Some(Background::Color(sidebar_color())));
        assert_ne!(hovered.background, resting.background);
        assert_ne!(selected.background, resting.background);
        assert_eq!(selected.border.width, 2.0);
    }

    #[test]
    fn settings_sidebar_labels_keep_readable_contrast_in_every_state() {
        fn luminance(color: Color) -> f32 {
            let linear = |value: f32| {
                if value <= 0.04045 {
                    value / 12.92
                } else {
                    ((value + 0.055) / 1.055).powf(2.4)
                }
            };
            0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
        }

        for selected in [false, true] {
            for status in [
                button::Status::Active,
                button::Status::Hovered,
                button::Status::Pressed,
            ] {
                let style = settings_sidebar_button_style(selected, &Theme::Dark, status);
                let Some(Background::Color(background)) = style.background else {
                    panic!("sidebar items need a solid background");
                };
                let foreground = luminance(style.text_color);
                let background = luminance(background);
                let contrast =
                    (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05);
                assert!(contrast >= 4.5, "insufficient sidebar label contrast");
                assert_eq!(style.border.width, 0.0);
            }
        }
    }

    #[test]
    fn action_buttons_distinguish_hover_and_disabled_states() {
        let active = action_button_style(&Theme::Dark, button::Status::Active);
        let hovered = action_button_style(&Theme::Dark, button::Status::Hovered);
        let disabled = action_button_style(&Theme::Dark, button::Status::Disabled);

        assert_ne!(active.background, hovered.background);
        assert_ne!(hovered.background, disabled.background);
        assert_ne!(active.text_color, disabled.text_color);
    }

    #[test]
    fn every_primary_destination_has_a_distinct_bitmap_icon() {
        let icons = [
            PrimaryView::Power,
            PrimaryView::Remote,
            PrimaryView::Sources,
            PrimaryView::Apps,
            PrimaryView::TextInput,
        ]
        .map(navigation_icon);

        for (index, icon) in icons.iter().enumerate() {
            assert!(!icons[..index].contains(icon));
        }
    }

    #[test]
    fn wake_choices_unlock_from_their_valid_mac_without_a_saved_tv_selection() {
        let wired = "02:11:22:33:44:55";

        assert!(wake_choice_enabled(WakeChoice::Wired, wired, "", false));
        assert!(!wake_choice_enabled(WakeChoice::WiFi, wired, "", false));
        assert!(wake_choice_enabled(WakeChoice::Disabled, wired, "", false));
        assert!(!wake_choice_enabled(WakeChoice::Wired, wired, "", true));
    }
}
