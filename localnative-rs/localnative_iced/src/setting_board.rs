use iced::{button, text_input, tooltip, Element, PickList, Row, Text, TextInput};

// pub fn tip_button<E,M,'a>(state:&'a mut button::State,content:E,msg:M,button_style:impl Into<Renderer::Style>,tip:String) -> Element<'a,M>
// where
//     E: Into<Element<'a, M>>
// {
//     tooltip::Tooltip::new(
//         button::Button::new(state,content)
//         .style(crate::style::symbol::Symbol)
//         .on_press(msg),
//         tip,
//         tooltip::Position::FollowCursor
//     )
//     .style(crate::style::symbol::Symbol)
//     .into()
// }
