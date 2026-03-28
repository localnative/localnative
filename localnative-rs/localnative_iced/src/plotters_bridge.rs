//! Minimal plotters-to-iced bridge for iced 0.14.
//!
//! Replaces the `plotters-iced` crate which is only compatible with iced 0.13.
//! Provides a `Chart` trait and `ChartWidget` that render plotters charts
//! onto iced's canvas widget.

use iced::mouse::Cursor;
use iced::widget::canvas;
use iced::{Element, Length, Rectangle, Size, Theme};
use plotters::drawing::IntoDrawingArea;
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend,
    DrawingErrorKind,
};

pub use plotters::chart::ChartBuilder;
pub use plotters_backend::DrawingBackend as PlottersDrawingBackend;

/// Trait for rendering a plotters chart inside an iced canvas.
///
/// Mirrors the plotters-iced `Chart` trait.
pub trait Chart<Message> {
    /// Per-canvas state (e.g. cursor position, selection).
    type State: Default + 'static;

    /// Build and draw the chart into the given drawing area.
    fn build_chart<DB: DrawingBackend>(&self, state: &Self::State, builder: ChartBuilder<DB>);

    /// Handle canvas events (mouse clicks, scrolls, etc.).
    fn update(
        &self,
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        (iced::event::Status::Ignored, None)
    }
}

/// Widget that displays a plotters `Chart` inside an iced canvas.
pub struct ChartWidget<'a, Message, C: Chart<Message>> {
    chart: &'a C,
    width: Length,
    height: Length,
    _marker: std::marker::PhantomData<Message>,
}

impl<'a, Message, C: Chart<Message>> ChartWidget<'a, Message, C> {
    pub fn new(chart: &'a C) -> Self {
        Self {
            chart,
            width: Length::Fill,
            height: Length::Fill,
            _marker: std::marker::PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[allow(dead_code)]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }
}

/// Canvas program that delegates to a `Chart` implementation.
struct ChartProgram<'a, Message, C: Chart<Message>> {
    chart: &'a C,
    _marker: std::marker::PhantomData<Message>,
}

impl<Message, C: Chart<Message>> canvas::Program<Message, Theme> for ChartProgram<'_, Message, C> {
    type State = C::State;

    fn update(
        &self,
        state: &mut Self::State,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> Option<canvas::Action<Message>> {
        let (status, msg) = self.chart.update(state, event, bounds, cursor);
        match (status, msg) {
            (_, Some(m)) => Some(canvas::Action::publish(m)),
            (iced::event::Status::Captured, None) => Some(canvas::Action::request_redraw()),
            _ => None,
        }
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: Cursor,
    ) -> Vec<canvas::Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let w = bounds.width as u32;
        let h = bounds.height as u32;

        {
            let backend = IcedBackend {
                frame: &mut frame,
                size: (w, h),
            };
            let root = backend.into_drawing_area();
            let builder = ChartBuilder::on(&root);
            self.chart.build_chart(state, builder);
            root.present().ok();
        }

        vec![frame.into_geometry()]
    }
}

type BackendResult = Result<(), DrawingErrorKind<std::convert::Infallible>>;

/// A plotters `DrawingBackend` that renders onto an iced canvas `Frame`.
pub struct IcedBackend<'a> {
    frame: &'a mut canvas::Frame,
    size: (u32, u32),
}

impl DrawingBackend for IcedBackend<'_> {
    type ErrorType = std::convert::Infallible;

    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    fn ensure_prepared(&mut self) -> BackendResult {
        Ok(())
    }

    fn present(&mut self) -> BackendResult {
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> BackendResult {
        if color.alpha == 0.0 {
            return Ok(());
        }
        let p = iced::Point::new(point.0 as f32, point.1 as f32);
        self.frame.fill_rectangle(
            p,
            Size::new(1.0, 1.0),
            iced::Color::from_rgba8(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha as f32),
        );
        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> BackendResult {
        let color = style.color();
        if color.alpha == 0.0 {
            return Ok(());
        }
        let stroke = canvas::Stroke::default()
            .with_width(style.stroke_width() as f32)
            .with_color(iced::Color::from_rgba8(
                color.rgb.0,
                color.rgb.1,
                color.rgb.2,
                color.alpha as f32,
            ));
        let path = canvas::Path::new(|p| {
            p.move_to(iced::Point::new(from.0 as f32, from.1 as f32));
            p.line_to(iced::Point::new(to.0 as f32, to.1 as f32));
        });
        self.frame.stroke(&path, stroke);
        Ok(())
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> BackendResult {
        let color = style.color();
        if color.alpha == 0.0 {
            return Ok(());
        }
        let ic = iced::Color::from_rgba8(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha as f32);
        let top_left = iced::Point::new(upper_left.0 as f32, upper_left.1 as f32);
        let size = Size::new(
            (bottom_right.0 - upper_left.0) as f32,
            (bottom_right.1 - upper_left.1) as f32,
        );
        if fill {
            self.frame.fill_rectangle(top_left, size, ic);
        } else {
            let stroke = canvas::Stroke::default()
                .with_width(style.stroke_width() as f32)
                .with_color(ic);
            let path = canvas::Path::new(|p| {
                p.rectangle(top_left, size);
            });
            self.frame.stroke(&path, stroke);
        }
        Ok(())
    }

    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: BackendCoord,
    ) -> BackendResult {
        let color = style.color();
        if color.alpha == 0.0 {
            return Ok(());
        }
        let ic = iced::Color::from_rgba8(color.rgb.0, color.rgb.1, color.rgb.2, color.alpha as f32);
        let size = style.size();

        let position = iced::Point::new(pos.0 as f32, pos.1 as f32);
        self.frame.fill_text(canvas::Text {
            content: text.to_string(),
            position,
            color: ic,
            size: iced::Pixels(size as f32),
            ..canvas::Text::default()
        });
        Ok(())
    }
}

impl<'a, Message: 'a, C: Chart<Message> + 'a> From<ChartWidget<'a, Message, C>>
    for Element<'a, Message>
where
    Message: Clone,
{
    fn from(widget: ChartWidget<'a, Message, C>) -> Element<'a, Message> {
        canvas(ChartProgram {
            chart: widget.chart,
            _marker: std::marker::PhantomData,
        })
        .width(widget.width)
        .height(widget.height)
        .into()
    }
}
