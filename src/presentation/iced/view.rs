use super::message::Message;
use super::view_model::{
    feed_is_at_bottom, ControlState, PrimaryView, ViewModel, MAX_MESSAGE_PANE_HEIGHT,
    MIN_MESSAGE_PANE_HEIGHT,
};
use ::iced::widget::{button, column, container, row, scrollable, slider, text, tooltip, Space};
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
        activity_view(),
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
    control_state: &ControlState,
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

fn remote_view(control_state: &ControlState) -> Element<'static, Message> {
    let reason = control_state.disabled_reason;
    let disabled = |label| tooltip(button(label), reason, tooltip::Position::Top);

    let directional_pad = column![
        disabled("Up"),
        row![disabled("Left"), disabled("Enter"), disabled("Right")].spacing(8),
        disabled("Down"),
    ]
    .align_x(Alignment::Center)
    .spacing(8);

    container(
        column![
            text("Remote View").size(26),
            text(reason),
            Space::new().height(8),
            disabled("Power Toggle"),
            directional_pad,
            row![disabled("Back"), disabled("Home")].spacing(8),
            text("Volume"),
            disabled("Volume Slider"),
            row![
                disabled("Mute"),
                disabled("Volume Down"),
                disabled("Volume Up")
            ]
            .spacing(8),
        ]
        .spacing(10)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .into()
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

fn activity_view() -> Element<'static, Message> {
    container(column![text("Activity View"), text("No activity yet.")].spacing(4))
        .padding(10)
        .width(Length::Fill)
        .height(80)
        .into()
}

pub fn settings_window() -> Element<'static, Message> {
    let severity = super::view_model::MessageSeverity::Warning;
    let sidebar = column![text("Settings").size(18), text("TV (current view)")]
        .padding(16)
        .spacing(10)
        .width(150);

    let main_pane = container(
        column![
            text("TV settings").size(26),
            Space::new().height(Length::Fill),
            text("No TVs are listed in this presentation shell."),
            text(format!(
                "{}: discovery is not available in this shell.",
                severity.label()
            )),
            tooltip(
                button("Discover TVs"),
                "Discovery is not available in this shell.",
                tooltip::Position::Bottom,
            ),
            Space::new().height(Length::Fill),
        ]
        .align_x(Alignment::Center)
        .spacing(12),
    )
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill);

    row![sidebar, main_pane].height(Length::Fill).into()
}
