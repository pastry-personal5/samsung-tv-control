use super::ui_message::Message;
use super::view_model::{
    feed_is_at_bottom, remote_action_label, PrimaryView, RemoteControlViewState, ViewModel,
    MAX_MESSAGE_PANE_HEIGHT, MIN_MESSAGE_PANE_HEIGHT,
};
use crate::{RemoteAction, SendRemoteAction};
use ::iced::widget::{
    button, column, container, row, scrollable, slider, text, text_input, tooltip, Space,
};
use ::iced::{Alignment, Element, Length};

pub const MESSAGE_FEED_ID: &str = "global-message-feed";

pub fn main_window(view_model: &ViewModel) -> Element<'_, Message> {
    let sidebar = column![
        text("Samsung TV Remote").size(22),
        Space::new().height(12),
        navigation_button(PrimaryView::Remote, view_model.primary_view()),
        navigation_button(PrimaryView::Sources, view_model.primary_view()),
        navigation_button(PrimaryView::Apps, view_model.primary_view()),
        navigation_button(PrimaryView::TextInput, view_model.primary_view()),
        text("Views: ⌘1–4").size(12),
        Space::new().height(Length::Fill),
        text("Main Toolbar").size(14),
        button("Settings (⌘,)")
            .on_press(Message::OpenSettings)
            .width(Length::Fill),
    ]
    .padding(16)
    .spacing(10)
    .width(180);

    let main_pane = column![
        scrollable(primary_view(
            view_model.primary_view(),
            view_model.control_state()
        ))
        .height(Length::Fill),
        split_bar(view_model.message_pane_height()),
        global_messages(view_model),
        activity_view(view_model),
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
) -> Element<'static, Message> {
    match primary_view {
        PrimaryView::Remote => remote_view(control_state),
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

fn remote_view(control_state: &RemoteControlViewState) -> Element<'static, Message> {
    let availability = control_state
        .disabled_reason
        .unwrap_or("Verified remote actions are available.");
    let selected_device = control_state
        .selected_device
        .as_ref()
        .map(|device| format!("Selected TV: {}", device.label()))
        .unwrap_or_else(|| "Selected TV: None".to_owned());

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
            text(selected_device),
            text(format!("Pairing: {}", control_state.pairing.label)),
            text(control_state.pairing.guidance),
            text(format!("Connection: {}", control_state.connection.label)),
            text(control_state.connection.guidance),
            text(availability),
            Space::new().height(8),
            remote_button("Power Toggle", RemoteAction::PowerToggle, control_state),
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
    row![
        text("Messages height"),
        slider(
            MIN_MESSAGE_PANE_HEIGHT..=MAX_MESSAGE_PANE_HEIGHT,
            height,
            Message::ResizeMessages,
        )
        .width(Length::Fill),
        text(format!("{height}px · ⌘⇧↑/↓")),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn global_messages(view_model: &ViewModel) -> Element<'_, Message> {
    let entries = if view_model.messages().entries().is_empty() {
        column![text("Global Messages Pane"), text("No messages yet.")].spacing(4)
    } else {
        let mut entries = column![text("Global Messages Pane")].spacing(4);
        for entry in view_model.messages().entries() {
            entries = entries.push(text(format!(
                "#{} · {} · {} · {}",
                entry.sequence,
                entry.severity.label(),
                entry.source.label(),
                entry.text
            )));
        }
        entries
    };

    let feed_status = if view_model.messages().unread_count() == 0 {
        if view_model.messages().at_bottom() {
            "Following new messages.".to_owned()
        } else {
            "Reading earlier messages.".to_owned()
        }
    } else {
        format!(
            "{} new message(s) below",
            view_model.messages().unread_count()
        )
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
            text(feed_status),
        ]
        .spacing(4),
    )
    .padding(10)
    .width(Length::Fill)
    .height(Length::Fixed(f32::from(view_model.message_pane_height())))
    .into()
}

fn activity_view(view_model: &ViewModel) -> Element<'static, Message> {
    let mut entries = column![text("Activity View")].spacing(4);
    if view_model.activity().is_empty() {
        entries = entries.push(text("No activity yet."));
    } else {
        for item in view_model.activity().iter().rev().take(3) {
            entries = entries.push(text(item.clone()));
        }
    }
    container(entries)
        .padding(10)
        .width(Length::Fill)
        .height(80)
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
    pub verified_actions: [(RemoteAction, bool); 10],
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
        verified_actions,
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
            .push(text(status))
            .push(text("After testing each key on the physical TV, mark only the keys that worked."));
    if selected_label.is_some() {
        for (action, verified) in verified_actions {
            main_content = main_content.push(
                button(text(format!(
                    "{}: {}",
                    remote_action_label(action),
                    if verified {
                        "Verified ✓"
                    } else {
                        "Unverified"
                    }
                )))
                .on_press(Message::SetActionVerified {
                    action,
                    verified: !verified,
                }),
            );
        }
    }
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
