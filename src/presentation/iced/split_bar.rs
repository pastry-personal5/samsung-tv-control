use ::iced::advanced::layout;
use ::iced::advanced::renderer;
use ::iced::advanced::widget::tree::{self, Tree};
use ::iced::advanced::{Clipboard, Layout, Shell, Widget};
use ::iced::{border, mouse, Background, Element, Event, Length, Point, Rectangle, Size, Theme};

#[derive(Debug, Default)]
struct State {
    origin: Option<(Point, u16)>,
}

/// A compact horizontal divider that reports a pane height while it is dragged.
pub struct SplitBar<'a, Message> {
    height: u16,
    on_resize: Box<dyn Fn(u16) -> Message + 'a>,
}

impl<'a, Message> SplitBar<'a, Message> {
    pub fn new(height: u16, on_resize: impl Fn(u16) -> Message + 'a) -> Self {
        Self {
            height,
            on_resize: Box::new(on_resize),
        }
    }
}

impl<Message, Renderer> Widget<Message, Theme, Renderer> for SplitBar<'_, Message>
where
    Message: Clone,
    Renderer: renderer::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(10.0))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, Length::Fill, 10.0)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(layout.bounds()) =>
            {
                state.origin = cursor.position().map(|position| (position, self.height));
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.origin.is_some() => {
                let (origin, initial_height) = state.origin.expect("drag origin is present");
                let delta = origin.y - position.y;
                let height = (f32::from(initial_height) + delta)
                    .round()
                    .clamp(0.0, f32::from(u16::MAX)) as u16;
                shell.publish((self.on_resize)(height));
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.origin = None;
            }
            _ => {}
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let palette = theme.extended_palette();
        let color = if state.origin.is_some() || cursor.is_over(layout.bounds()) {
            palette.primary.strong.color
        } else {
            palette.background.strong.color
        };
        let bounds = layout.bounds();
        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    y: bounds.y + 4.0,
                    height: 2.0,
                    ..bounds
                },
                border: border::rounded(1),
                ..renderer::Quad::default()
            },
            Background::Color(color),
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<State>().origin.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(layout.bounds()) {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::None
        }
    }
}

impl<'a, Message> From<SplitBar<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(split_bar: SplitBar<'a, Message>) -> Self {
        Self::new(split_bar)
    }
}
