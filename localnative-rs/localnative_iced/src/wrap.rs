use iced_graphics::{Backend, Defaults, Primitive, Renderer};
use iced_native::{
    event,
    layout::{self, Limits, Node},
    mouse, overlay, Clipboard, Element, Event, Hasher, Layout, Length, Point, Rectangle, Size,
    Widget,
};

// TODO：wrap是一个很实用的组件，所以最好将现有实现广泛化，加入到iced的源码中
pub struct Wrap<'a, B, Message>
where
    B: Backend,
{
    pub elements: Vec<Element<'a, Message, Renderer<B>>>,
    // 目前这两个对齐并没有什么用，如果不需要，甚至可以删除掉
    // pub horizontal_alignment: Align,
    // pub vertical_alignment: Align,
    pub width: Length,
    pub height: Length,
    pub max_width: u32,
    pub max_height: u32,
    pub padding: u16,
    pub spacing: u16,
    pub line_spacing: u16,
    pub line_height: u32,
}

impl<'a, Message, B> Widget<Message, Renderer<B>> for Wrap<'a, B, Message>
where
    B: Backend,
{
    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer<B>, limits: &layout::Limits) -> layout::Node {
        let padding = self.padding as f32;
        let spacing = self.spacing as f32;
        let line_spacing = self.line_spacing as f32;
        let line_height = self.line_height as f32;
        let limits = limits
            .pad(padding)
            .width(self.width)
            .height(self.height)
            .max_width(self.max_width)
            .max_height(self.max_height);
        let max_width = limits.max().width;

        let mut curse = padding;
        let mut deep_curse = padding;
        let mut current_line_height = line_height;
        let mut max_main = curse;

        let nodes: Vec<Node> = self
            .elements
            .iter()
            .map(|elem| {
                let node_limit =
                    Limits::new(Size::new(limits.min().width, line_height), limits.max());
                let mut node = elem.layout(renderer, &node_limit);

                let size = node.size();

                let offset_init = size.width + spacing;
                let offset = curse + offset_init;

                if offset > max_width {
                    deep_curse += current_line_height + line_spacing;
                    current_line_height = line_height;
                    node.move_to(Point::new(padding, deep_curse));
                    curse = offset_init + padding;
                } else {
                    node.move_to(Point::new(curse, deep_curse));
                    curse = offset;
                }
                current_line_height = current_line_height.max(size.height);
                max_main = max_main.max(curse);

                // node.align(self.horizontal_alignment, self.vertical_alignment, size);

                node
            })
            .collect();

        let (width, height) = (
            max_main - padding,
            deep_curse - padding + current_line_height,
        );
        // let size = limits.resolve(Size::new(width, height));

        Node::with_children(Size::new(width, height).pad(padding), nodes)
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        // self.vertical_alignment.hash(state);
        // self.horizontal_alignment.hash(state);

        self.width.hash(state);
        self.height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.line_spacing.hash(state);
        self.padding.hash(state);
        self.spacing.hash(state);

        for elem in &self.elements {
            elem.hash_layout(state)
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer<B>,
        defaults: &Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        let mut mouse_interaction = mouse::Interaction::default();
        let content = &self.elements[..];
        (
            Primitive::Group {
                primitives: content
                    .iter()
                    .zip(layout.children())
                    .map(|(child, layout)| {
                        let (primitive, new_mouse_interaction) =
                            child.draw(renderer, defaults, layout, cursor_position, viewport);

                        if new_mouse_interaction > mouse_interaction {
                            mouse_interaction = new_mouse_interaction;
                        }

                        primitive
                    })
                    .collect(),
            },
            mouse_interaction,
        )
    }
    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        renderer: &Renderer<B>,
        clipboard: &mut dyn Clipboard,
        messages: &mut Vec<Message>,
    ) -> event::Status {
        self.elements
            .iter_mut()
            .zip(layout.children())
            .map(|(child, layout)| {
                child.on_event(
                    event.clone(),
                    layout,
                    cursor_position,
                    renderer,
                    clipboard,
                    messages,
                )
            })
            .fold(event::Status::Ignored, event::Status::merge)
    }
    fn overlay(
        &mut self,
        layout: Layout<'_>,
    ) -> Option<overlay::Element<'_, Message, Renderer<B>>> {
        self.elements
            .iter_mut()
            .zip(layout.children())
            .filter_map(|(child, layout)| child.overlay(layout))
            .next()
    }
}

impl<'a, Message, B> From<Wrap<'a, B, Message>> for Element<'a, Message, Renderer<B>>
where
    B: Backend + 'a,
    Message: 'a,
{
    fn from(wrap: Wrap<'a, B, Message>) -> Self {
        Element::new(wrap)
    }
}

impl<'a, B, Message> Wrap<'a, B, Message>
where
    B: Backend,
{
    pub fn new() -> Self {
        Self::with_elements(Vec::new())
    }

    pub fn with_elements(elements: Vec<Element<'a, Message, Renderer<B>>>) -> Self {
        Self {
            elements,
            ..Default::default()
        }
    }

    pub fn spacing(mut self, units: u16) -> Self {
        self.spacing = units;
        self
    }
    pub fn line_spacing(mut self, units: u16) -> Self {
        self.line_spacing = units;
        self
    }

    pub fn padding(mut self, units: u16) -> Self {
        self.padding = units;
        self
    }

    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    pub fn max_width(mut self, max_width: u32) -> Self {
        self.max_width = max_width;
        self
    }

    pub fn max_height(mut self, max_height: u32) -> Self {
        self.max_height = max_height;
        self
    }

    // pub fn vertical_alignment(mut self, align: Align) -> Self {
    //     self.vertical_alignment = align;
    //     self
    // }

    // pub fn horizontal_alignment(mut self, align: Align) -> Self {
    //     self.horizontal_alignment = align;
    //     self
    // }

    pub fn push(mut self, element: Element<'a, Message, Renderer<B>>) -> Self {
        self.elements.push(element);
        self
    }
}
impl<'a, B, Message> Default for Wrap<'a, B, Message>
where
    B: Backend,
{
    fn default() -> Self {
        Self {
            elements: vec![],
            // horizontal_alignment: Align::Center,
            // vertical_alignment: Align::Center,
            width: Length::Shrink,
            height: Length::Shrink,
            max_width: u32::MAX,
            max_height: u32::MAX,
            padding: 0,
            spacing: 0,
            line_spacing: 0,
            line_height: 10,
        }
    }
}
