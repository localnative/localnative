---
id: tutorial4
title: 4. 实现tags
---





### 添加依赖

有了实现`NoteView`的经验，接下来实现`tags`会快很多，在构建tags模块之前，我们先添加上一会儿会用到的依赖：

```diff
# 在 Cargo.toml 内

++  serde = {version = "1",features = ["derive"]}
++  serde_json = "1"

++  [dependencies.rand]
++  version = "0.8"
++  optional = true

    [dependencies.iced_aw]
    git = "https://github.com/iced-rs/iced_aw"
    branch = "main"
    default-features = false
    features = ["wrap"] 

    [dependencies.iced]
    version = "0.3.0"
    default-features = false

    [features]
    default = ["preview"]
    wgpu = [
        "iced/default",
        "iced/tokio",
        "iced/qr_code",
        "iced/canvas",
        ]
    opengl = [
        "iced/glow",
        "iced/tokio",
        "iced/glow_qr_code",
        "iced/glow_canvas",
        "iced/glow_default_system_font"
        ]
    preview = [
        "wgpu",
++      "rand"
    ]
```

总得来说就是添加了三个依赖，以及将其中一个依赖放入了特定features下才会开启。

> 你可以像我一样给依赖开启optional，并且在特定feature里指定该依赖的名字，这样这个依赖在编译的时候，只会在该feature开启的情况下参与编译。

我们需要使用rand产生一些随机数帮助我们测试接下来的一些用例。

除此之外还添加了`serde`和`serde_json`两个依赖，前者我们还开启了`deriver`feature，可以帮助我们更简单的实现序列化。后者是因为`core`的返回值是`json`，我们之后会需要将从`core`获取的值转化为绘制GUI所需要的数据结构，也就需要用到这个依赖了。

### 开始构建

```diff
// 在 lib.rs 内
    mod note;
    mod style;
++  mod tags;
    use iced::Command;
    pub use note::NoteView;
++  #[cfg(feature = "preview")]
++  pub use tags::Tags;
```

我们需要先在`lib.rs`内添加上新的模块名，同时创建对应的`tags.rs`文件：

```rust
// 在 tags.rs 内
use iced::{button, Button, Element, Row, Text};
use serde::{Deserialize, Serialize};

use crate::style::{self, Theme};
// 和NoteView的时候一样，定义一个Message，tag只需要作为按钮，因此也只有一个Search消息
#[derive(Debug, Clone)]
pub enum Message {
    Search(String),
}
// 此处我们没有和NoteView中那样，直接使用core里面的Tag
// 实际上，在core代码里没有对应的Tag，而是使用这样的一个结构体：
// #[allow(clippy::upper_case_acronyms)]
// #[derive(Serialize, Deserialize, Debug)]
// pub struct KVStringI64 {
//     pub k: String,
//     pub v: i64,
// }
// 为了与其名字对应，我们将使用serde的rename宏将其一一对应
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Tag {
// 只需要在rename后指定对应的名称即可
    #[serde(rename = "k")]
    pub name: String,
    #[serde(rename = "v")]
    pub count: i64,
}
// 同样构建一个保存tag的TagView结构
#[derive(Debug, Default)]
pub struct TagView {
    pub tag: Tag,
    pub search_button: button::State,
    pub count_button: button::State,
}
// 通过实现From<Tag>，方便后续将Tag转变为TagView
impl From<Tag> for TagView {
    fn from(tag: Tag) -> Self {
        Self {
            tag,
            search_button: button::State::new(),
            count_button: button::State::new(),
        }
    }
}
impl TagView {
// 同样的view方法，相对于NoteView，只需要一个Row，放入两个button即可
    pub fn view(&mut self,theme:Theme) -> Element<Message> {
        Row::new()
            .push(
                Button::new(
                    &mut self.search_button,
                    Text::new(self.tag.name.as_str()).size(16),
                )
                // 直接使用NoteView时定义的style
                .style(style::tag(theme))
                .on_press(Message::Search(self.tag.name.clone())),
            )
            .push(
                Button::new(
                    &mut self.count_button,
                    Text::new(self.tag.count.to_string()).color([1.0, 0.0, 0.0]),
                )
                .on_press(Message::Search(self.tag.count.to_string()))
                // 这是专门个Count定义的，接下来会介绍
                .style(style::count(theme)),
            )
            .into()
    }
}
// 为了方便预览，我们构建一个Vec来预览多个tag
#[cfg(feature = "preview")]
pub struct Tags {
    tags: Vec<TagView>,
}
// 同样要实现SandBox
#[cfg(feature = "preview")]
impl iced::Sandbox for Tags {
    type Message = Message;

    fn new() -> Self {
        let mut tags =Vec::new();
        // 这里使用随机数帮我们生成一些不一样的数据进行预览
        for _ in 0..50 {
            tags.push(
                Tag {
                    name: format!("test {}",rand::random::<i32>()),
                    count: rand::random()
                }
            );
        }
        Self {
            tags: tags.into_iter().map(|tag| TagView::from(tag)).collect(),
        }
    }

    fn title(&self) -> String {
        "tags preview".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::Search(s) => println!("{}", s),
        }
    }
//	使用wrap将所有的tag包裹起来
    fn view(&mut self) -> Element<'_, Self::Message> {
        let wrap = iced_aw::Wrap::new().push(Text::new("tags:"));
        self.tags
            .iter_mut()
            .fold(wrap, |wrap, tag| wrap.push(tag.view(Theme::Light)))
            .into()
    }
}
```
同时我们
