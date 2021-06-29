---
id: tutorial4
title: 4. 实现TagView
---

有了实现`NoteView`的经验，接下来实现`TagView`会快很多。我们将按照之前的思路，声明一个由`Tag`结构体`into`的`Tagview`结构，通过给`TagView`实现`view`方法和`update`方法，并进行一些美化。


### 添加依赖

在实现`TagView`之前，我们先添加接下来会用到的依赖：


```diff
# 在 Cargo.toml 内
    [dependencies]
    localnative_core = { path = "../localnative_core" }
    open = "1"
    once_cell = "1.7"

++  serde = {version = "1",features = ["derive"]}
++  serde_json = "1"
```

我们新增了`serde`和`serde_json`两个依赖，前者我们还开启了其`deriver`feature，可以帮助我们更简单的实现序列化。后者是因为`localnative_core`的返回值是`json`，我们之后会需要将从`localnative_core`获取的值转化为绘制GUI所需要的数据，也就需要用到这个依赖了。

### 开始构建

```diff
// 在 lib.rs 内
    mod note;
    mod style;
++  mod tags;
    use iced::Command;
    pub use note::NoteView;
++  pub use tags::TagView;
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
                // 直接使用NoteView时定义的tag风格
                .style(style::tag(theme))
                .on_press(Message::Search(self.tag.name.to_string())),
            )
            .push(
                Button::new(
                    &mut self.count_button,
                    Text::new(self.tag.count.to_string()).color([1.0, 0.0, 0.0]),
                )
                .on_press(Message::Search(self.tag.count.to_string()))
                // 这是专门给Count定义的，接下来会介绍
                .style(style::count(theme)),
            )
            .into()
    }
}

// 给TagView实现SandBox
#[cfg(feature = "preview")]
impl iced::Sandbox for TagView {
    type Message = Message;

    fn new() -> Self {
        Tag {
            name:"testtag".to_owned(),
            count: 16
        }.into()
    }

    fn title(&self) -> String {
        "tagview preview".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        // 简单打印即可，后续需要交给更高的层次处理
        match message {
            Message::Search(s) => println!("{}", s),
        }
    }
//	在view里调用TagView的view方法，预览部分提供自己想要预览的主题即可
    fn view(&mut self) -> Element<'_, Self::Message> {
        self.view(Theme::Light)
    }
}
```
同时不要忘记了给`count`定义风格：
```rust
// 在 style.rs 内
pub struct Count {
    theme: Theme
}

impl button::StyleSheet for Count {
    fn active(&self) -> button::Style {
        // 只需要简单定义颜色即可，同时将边框置为透明
        let text_color = match self.theme {
            Theme::Light => Color::from_rgb(1.0, 0.0, 0.0),
            Theme::Dark =>  Color::from_rgb(0.2, 0.1, 1.0)
        };
        button::Style {
            background: None,
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: text_color,
            ..Default::default()
        }
    }
}
pub fn count(theme: Theme) -> Count {
    Count {
        theme
    }
}
```
和之前调用`NoteView`的预览一样，我们也需要在`preview`文件夹下新建一个`tagview.rs`文件：
```rust
// 在 preview/tagview.rs 下
use iced::Sandbox;
use localnative_iced::TagView;

fn main() -> iced::Result {
    TagView::run(localnative_iced::settings())
}
```
并且将bin属性添加到`Cargo.toml`内：
```diff
# 在 Cargo.toml 内
++ [[bin]]
++ name = "tagview"
++ path = "./previews/tagview.rs"
++ required-features = ["preview"]
```
完成这些常规操作之后我们运行`tagview`,看一下我们的最终结果了：
```shell
cargo run --bin tagview
```
得到以下结果：

![tagview](/img/tutorial-04-00.png)

到这里，我们已经实现了`TagView`，你可以根据你自己的喜好反复修改主题风格，直到满足你的要求。在下一节我们将会实现一个完整的搜索页面和一个完整的tags页面。