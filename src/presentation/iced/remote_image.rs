use super::ui_message::Message;
use super::view_model::RemoteControlViewState;
use crate::application::tv_control_coordinator::WakeStage;
use crate::{RemoteAction, SendRemoteAction};
use iced::mouse;
use iced::widget::{canvas, image, Canvas};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};
use std::sync::OnceLock;

const IMAGE_WIDTH: f32 = 1024.0;
const IMAGE_HEIGHT: f32 = 1536.0;
const DISPLAY_WIDTH: f32 = 260.0;
const DISPLAY_HEIGHT: f32 = 600.0;

pub(super) struct RemoteImage {
    controls: RemoteControlViewState,
    wake_stage: WakeStage,
    wake_configured: bool,
}

impl RemoteImage {
    pub(super) fn view(
        controls: &RemoteControlViewState,
        wake_stage: WakeStage,
        wake_configured: bool,
    ) -> Element<'static, Message> {
        Canvas::new(Self {
            controls: controls.clone(),
            wake_stage,
            wake_configured,
        })
        .width(Length::Fixed(DISPLAY_WIDTH))
        .height(Length::Fixed(DISPLAY_HEIGHT))
        .into()
    }

    fn enabled(&self, action: RemoteAction) -> bool {
        if self.controls.selected_device.is_none() {
            return false;
        }
        if action == RemoteAction::PowerToggle {
            let busy = matches!(
                self.wake_stage,
                WakeStage::CheckingConnection | WakeStage::PacketSending | WakeStage::Reconnecting
            );
            !busy
                && (self
                    .controls
                    .action_disabled_reason(RemoteAction::PowerToggle)
                    .is_none()
                    || self.wake_configured)
        } else {
            self.controls.action_disabled_reason(action).is_none()
        }
    }

    fn action_at(&self, bounds: Rectangle, cursor: mouse::Cursor) -> Option<RemoteAction> {
        let position = cursor.position_in(bounds)?;
        let source = source_position(position, bounds.size());
        let action = hit_test(source)?;
        self.enabled(action).then_some(action)
    }
}

impl canvas::Program<Message> for RemoteImage {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if !matches!(
            event,
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        ) {
            return None;
        }
        let action = self.action_at(bounds, cursor)?;
        let device = self.controls.selected_device.as_ref()?;
        let request =
            SendRemoteAction::new(device.id(), self.controls.selection_generation, action);
        let message = if action == RemoteAction::PowerToggle {
            Message::PowerToggle(request)
        } else {
            Message::AttemptRemoteAction(request)
        };
        Some(canvas::Action::publish(message).and_capture())
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let scale = bounds.height / IMAGE_HEIGHT;
        let image_width = IMAGE_WIDTH * scale;
        let left = (bounds.width - image_width) / 2.0;
        frame.with_clip(Rectangle::new(Point::ORIGIN, bounds.size()), |frame| {
            frame.draw_image(
                Rectangle::new(Point::new(left, 0.0), Size::new(image_width, bounds.height)),
                canvas::Image::new(remote_handle().clone()),
            );
            for (action, x, y, radius) in CONTROL_CENTERS {
                if !self.enabled(action) {
                    frame.fill(
                        &canvas::Path::circle(
                            Point::new(left + x * scale, y * scale),
                            radius * scale,
                        ),
                        Color::from_rgba8(6, 9, 12, 0.58),
                    );
                }
            }
            if let Some(action) = self.action_at(bounds, cursor) {
                if let Some((_, x, y, radius)) = CONTROL_CENTERS
                    .iter()
                    .find(|(candidate, _, _, _)| *candidate == action)
                {
                    frame.fill(
                        &canvas::Path::circle(
                            Point::new(left + x * scale, y * scale),
                            radius * scale,
                        ),
                        Color::from_rgba8(91, 166, 224, 0.25),
                    );
                }
            }
        });
        vec![frame.into_geometry()]
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.action_at(bounds, cursor).is_some() {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }
}

fn remote_handle() -> &'static image::Handle {
    static HANDLE: OnceLock<image::Handle> = OnceLock::new();
    HANDLE.get_or_init(|| {
        image::Handle::from_bytes(&include_bytes!("../../../assets/remote-concept-v5.png")[..])
    })
}

fn source_position(position: Point, bounds: Size) -> Point {
    let scale = bounds.height / IMAGE_HEIGHT;
    let left = (bounds.width - IMAGE_WIDTH * scale) / 2.0;
    Point::new((position.x - left) / scale, position.y / scale)
}

fn in_circle(point: Point, x: f32, y: f32, radius: f32) -> bool {
    (point.x - x).powi(2) + (point.y - y).powi(2) <= radius.powi(2)
}

const CONTROL_CENTERS: [(RemoteAction, f32, f32, f32); 12] = [
    (RemoteAction::PowerToggle, 512.0, 184.0, 67.0),
    (RemoteAction::Up, 512.0, 341.0, 57.0),
    (RemoteAction::Left, 370.0, 483.0, 57.0),
    (RemoteAction::Enter, 512.0, 483.0, 103.0),
    (RemoteAction::Right, 654.0, 483.0, 57.0),
    (RemoteAction::Down, 512.0, 625.0, 57.0),
    (RemoteAction::Back, 410.0, 747.0, 66.0),
    (RemoteAction::Home, 614.0, 747.0, 66.0),
    (RemoteAction::PlayPause, 410.0, 899.0, 66.0),
    (RemoteAction::Mute, 410.0, 1060.0, 66.0),
    (RemoteAction::VolumeUp, 614.0, 899.0, 66.0),
    (RemoteAction::VolumeDown, 614.0, 1060.0, 66.0),
];

fn hit_test(point: Point) -> Option<RemoteAction> {
    use RemoteAction as Action;

    if in_circle(point, 512.0, 184.0, 67.0) {
        return Some(Action::PowerToggle);
    }

    let dx = point.x - 512.0;
    let dy = point.y - 483.0;
    let radius_squared = dx * dx + dy * dy;
    if radius_squared <= 103.0_f32.powi(2) {
        return Some(Action::Enter);
    }
    if radius_squared <= 179.0_f32.powi(2) && radius_squared >= 108.0_f32.powi(2) {
        return Some(if dx.abs() > dy.abs() {
            if dx < 0.0 {
                Action::Left
            } else {
                Action::Right
            }
        } else if dy < 0.0 {
            Action::Up
        } else {
            Action::Down
        });
    }

    for (x, y, action) in [
        (410.0, 747.0, Action::Back),
        (614.0, 747.0, Action::Home),
        (410.0, 899.0, Action::PlayPause),
        (410.0, 1060.0, Action::Mute),
    ] {
        if in_circle(point, x, y, 66.0) {
            return Some(action);
        }
    }

    let rocker_x = 614.0;
    if (545.0..=683.0).contains(&point.x)
        && (830.0..=1130.0).contains(&point.y)
        && (point.y >= 899.0 || in_circle(point, rocker_x, 899.0, 69.0))
        && (point.y <= 1061.0 || in_circle(point, rocker_x, 1061.0, 69.0))
    {
        return Some(if point.y < 975.0 {
            Action::VolumeUp
        } else {
            Action::VolumeDown
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ControlState;

    #[test]
    fn no_selected_tv_disables_every_picture_hotspot() {
        let remote = RemoteImage {
            controls: RemoteControlViewState::from_application_state(&ControlState::none()),
            wake_stage: WakeStage::Idle,
            wake_configured: false,
        };
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT));
        let scale = DISPLAY_HEIGHT / IMAGE_HEIGHT;
        let left = (DISPLAY_WIDTH - IMAGE_WIDTH * scale) / 2.0;
        for (action, x, y, _) in CONTROL_CENTERS {
            let cursor = mouse::Cursor::Available(Point::new(left + x * scale, y * scale));
            assert!(!remote.enabled(action));
            assert_eq!(remote.action_at(bounds, cursor), None);
        }
    }

    #[test]
    fn every_visible_control_has_a_distinct_hotspot() {
        let centers = [
            (512.0, 184.0, RemoteAction::PowerToggle),
            (512.0, 341.0, RemoteAction::Up),
            (370.0, 483.0, RemoteAction::Left),
            (512.0, 483.0, RemoteAction::Enter),
            (654.0, 483.0, RemoteAction::Right),
            (512.0, 625.0, RemoteAction::Down),
            (410.0, 747.0, RemoteAction::Back),
            (614.0, 747.0, RemoteAction::Home),
            (410.0, 899.0, RemoteAction::PlayPause),
            (410.0, 1060.0, RemoteAction::Mute),
            (614.0, 899.0, RemoteAction::VolumeUp),
            (614.0, 1060.0, RemoteAction::VolumeDown),
        ];
        for (x, y, expected) in centers {
            assert_eq!(hit_test(Point::new(x, y)), Some(expected));
        }
    }

    #[test]
    fn gaps_and_remote_body_are_not_clickable() {
        for point in [
            Point::new(512.0, 280.0),
            Point::new(512.0, 589.0),
            Point::new(512.0, 900.0),
            Point::new(370.0, 1250.0),
            Point::new(100.0, 500.0),
        ] {
            assert_eq!(hit_test(point), None);
        }
    }

    #[test]
    fn display_coordinates_map_back_to_image_coordinates() {
        let source = Point::new(614.0, 1060.0);
        let bounds = Size::new(DISPLAY_WIDTH, DISPLAY_HEIGHT);
        let scale = DISPLAY_HEIGHT / IMAGE_HEIGHT;
        let left = (DISPLAY_WIDTH - IMAGE_WIDTH * scale) / 2.0;
        let display = Point::new(left + source.x * scale, source.y * scale);
        let result = source_position(display, bounds);
        assert!((result.x - source.x).abs() < 0.001);
        assert!((result.y - source.y).abs() < 0.001);
    }
}
