use std::sync::mpsc::Receiver;

use iced_native::{
    event, layout, Clipboard, Color, Event, Hasher, HorizontalAlignment, Layout, Length, Point,
    Rectangle, Size, VerticalAlignment, Widget,
};

use std::hash::Hash;

#[derive(Debug)]
pub struct Logger {
    state: State,
}

impl Logger {
    pub fn new<T: Into<String>>(record: Receiver<String>, label: T) -> Self {
        Self {
            state: State::new(record, label),
        }
    }
    pub fn state(&mut self) -> &mut State {
        &mut self.state
    }
}
#[derive(Debug)]
pub struct State {
    record: Receiver<String>,
    content: String,
}
impl State {
    pub fn new<T: Into<String>>(record: Receiver<String>, content: T) -> Self {
        Self {
            record,
            content: content.into(),
        }
    }
}
#[derive(Debug)]
pub struct Record<'a, Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    state: &'a mut State,
    size: Option<u16>,
    color: Option<Color>,
    font: Renderer::Font,
    width: Length,
    height: Length,
    horizontal_alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl<'a, Renderer> Record<'a, Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    /// Create a new fragment of [`Record`] with the given contents.
    pub fn new(state: &'a mut State) -> Self {
        Self {
            state,
            size: None,
            color: None,
            font: Default::default(),
            width: Length::Shrink,
            height: Length::Shrink,
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
        }
    }

    /// Sets the size of the [`Record`].
    pub fn size(mut self, size: u16) -> Self {
        self.size = Some(size);
        self
    }

    /// Sets the [`Color`] of the [`Record`].
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Sets the [`Font`] of the [`Record`].
    ///
    /// [`Font`]: Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = font.into();
        self
    }

    /// Sets the width of the [`Record`] boundaries.
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    /// Sets the height of the [`Record`] boundaries.
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// Sets the [`HorizontalAlignment`] of the [`Record`].
    pub fn horizontal_alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = alignment;
        self
    }

    /// Sets the [`VerticalAlignment`] of the [`Record`].
    pub fn vertical_alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = alignment;
        self
    }
}

impl<'a, Renderer, Message> Widget<Message, Renderer> for Record<'a, Renderer>
where
    Renderer: iced_native::text::Renderer,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let size = self.size.unwrap_or_else(|| renderer.default_size());

        let bounds = limits.max();

        let (width, height) = renderer.measure(&self.state.content, size, self.font, bounds);

        let size = limits.resolve(Size::new(width, height));

        layout::Node::new(size)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            layout.bounds(),
            &self.state.content,
            self.size.unwrap_or_else(|| renderer.default_size()),
            self.font,
            self.color,
            self.horizontal_alignment,
            self.vertical_alignment,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.state.content.hash(state);
        self.size.hash(state);
        self.width.hash(state);
        self.height.hash(state);
    }
    fn on_event(
        &mut self,
        _event: Event,
        _layout: Layout<'_>,
        _cursor_positionn: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        _messages: &mut Vec<Message>,
    ) -> event::Status {
        if let Ok(rd) = self.state.record.try_recv() {
            self.state.content = rd;
            event::Status::Captured
        } else {
            event::Status::Ignored
        }
    }
}

impl<'a, Renderer, Message> From<Record<'a, Renderer>>
    for iced_native::Element<'a, Message, Renderer>
where
    Renderer: iced_native::text::Renderer + 'a,
{
    fn from(rd: Record<'a, Renderer>) -> Self {
        iced_native::Element::new(rd)
    }
}
