---
id: tutorial2
title: 2. 运行通用示例
---

在上一章里，我们已经初始化项目结构，本章主要是介绍这样一个项目结构是如何运行的。

首先我们需要删除掉原来`lib.rs`的所有内容，同时添加以下内容（直接复制到lib.rs就行，下文会详细介绍写了啥）：

```rust
// 在 lib.rs 内
use iced::Command;
pub enum LocalNative {
    Loading,
    Loaded(Data),
}
pub struct Data {
}
#[derive(Debug)]
pub enum Message {
}
impl iced::Application for LocalNative {
    type Executor = iced::executor::Default;
    
    type Message = Message;

    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (LocalNative::Loading,Command::none())
    }

    fn title(&self) -> String {
        "ln-iced".to_owned()
    }

    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        iced::Text::new("ln-iced").into()
    }
}

pub fn settings() -> iced::Settings<()> {
    iced::Settings {
        ..Default::default()
    }
}
```

同时复制以下内容到`bin.rs`文件内：

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ln_iced::LocalNative;
use ln_iced::settings;
use iced::Application;

fn main() -> iced::Result {
    LocalNative::run(settings())
}
```

>正常的`iced`开发，是不需要通过`lib`的方式来管理整个项目的。而是通过常规的`mian.rs`。但是现在我们为了更方便的预览，因此需要多个可执行文件，所以将整个程序作为库来实现，而可执行部分分别放到不同的文件里去调用。

`lib.rs`声明了一个`LocalNative`的枚举，这个枚举是这样定义的：

```rust
pub enum LocalNative {
    Loading,
    Loaded(Data),
}
```

它代表着程序的不同状态，即正在加载状态和加载完成状态。

`Data`是我们需要绘制的数据，目前只是一个空的结构体，后续我们会和`localnative_core`交互，从数据库里获取需要绘制的数据。

我们定义了一个用来代表更新消息的`Message`：

```rust
#[derive(Debug)]
pub enum Message {
}
```

目前同样也是一个空的枚举体，我们将在后续的开发中大量的和它打交道。

> 注意：
>
> 查看[`iced::Application`文档](https://docs.rs/iced/0.3.0/iced/trait.Application.html)，[关联类型部分](https://docs.rs/iced/0.3.0/iced/trait.Application.html#associated-types)，`Message`是这样定义的：
>
> ```rust
> type Message: Debug + Send
> ```
>
> 因此我们在定义`Message`的时候，需要实现`Debug`这个`trait`，我们通过`derive`宏来帮助我们自动化实现这个`trait`。
>
> `Send`的话只要你的`Message`内所有成员都实现了`Send`，那么`Message`本身也是实现了`Send`的（详情看[`Send`文档](https://doc.rust-lang.org/nightly/core/marker/trait.Send.html)）。
>

> 题外话： `Send`大部分时候是不需要我们手动实现的，`Send`这个trait标识的类型代表了可以在多线程间安全传递所有权，这是一个用来给编译器确定多线程编程条件的`trait`，你只要知道一个类型标识`Send`，那么这个类型的所有权就可以在线程间安全传递，当然通常是函数签名里限制你只能传递实现了`Send`的类型。

紧接着我们给`LocalNative`这个枚举体实现了`iced::Application`这个`trait`:

```rust
// 在 lib.rs 中
impl iced::Application for LocalNative {
    // 执行器的关联类型定义为默认即可，
    // 后续我们使用Command的时候，这里会详细介绍
    type Executor = iced::executor::Default;
	// 外部定义的Message需要在此处给Application这个trati指明
    type Message = Message;
	// 这个关联变量是为了在new的时候多一些选择，
    // 比如我们通过传入一些参数来构建不同的GUI应用程序
    // 当前我们没有特殊需求，所以这个关联类型定义为()即可
    type Flags = ();
	// 正如刚刚所说，我们可以通过读取flags来构建不同的Self,
    // 目前我们仅仅需要一个普通的示例，因此并没有用到flags
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (LocalNative::Loading,Command::none())
    }
	// 这个方法用来定义应用程序的标题
    fn title(&self) -> String {
        "ln-iced".to_owned()
    }
	// 这里用来处理message，当前我们并没有什么需要处理的，
    // 因此这里直接返回Command::none()即可
    // 关于Command，我们在后续会详细介绍
    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        iced::Command::none()
    }
	// 这部分就是用来描述GUI程序视图的地方
    // 当前只是定义了一个显示字符的文本控件
    // 如果你当前你把字换成中文，运行之后会看到乱码
    // 我们将会在后续解决这个问题
    fn view(&mut self) -> iced::Element<'_, Self::Message> {
        iced::Text::new("ln-iced").into()
    }
}
```

以上，是一个普通的`iced`应用程序示例，我们将在接下来的开发中充实它，不过在此之前，既然这是一个普通的示例，那就代表我们能够运行它，现在我们来介绍一下`bin.rs`：

```rust
// 在 bin.rs 中
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ln_iced::LocalNative;
use ln_iced::settings;
use iced::Application;

fn main() -> iced::Result {
    // run 方法是我们实现Application这个trait的时候
    // 这个trait自动实现的，也因为是这个trait实现的
    // 所以要调用这个方法，我们需要在程序内引入
    // iced::Application这个trait
    // 这个 trait 需要提供一个Settings结构体
    LocalNative::run(settings())
}
```

我们把这个结构体放在了lib.rs里去实现
```rust
// 在 lib.rs 中
// 需要指明泛型，泛型要和Application中的关联类型的类型一致
// 如果我们不是用lib的方式来写，那么它将会自动推导泛型
// 但是现在我们是lib的形式来写的，所以需要我们特别标注出来
// 此时的Settings使用的是默认值
pub fn settings() -> iced::Settings<()> {
    iced::Settings {
        ..Default::default()
    }
}
```

> 注意，在第一行：
>
> `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`
>
> 是用来标识在windows操作系统下，如果是release模式发布应用程序的话，则在运行时禁用终端窗口弹出。

> 题外话：
>
> Rust的结构体在定义的时候可以通过`..`语法糖来减少不必要的输入，比如此处的`..Default::default()`，就是所有字段全部和默认的一致的意思。
>
> 我们之前创建项目的时候使用的名字是:`ln-iced`，但是在`bin.rs`里引用的时候，使用的是`ln_iced`，这是Rust自动转换的结果，另外你也只能通过`ln_iced`引用到你想引用的值，用`ln-iced`是没有用的。

在完成上述操作之后，保存之后就可以跑一下程序试试了：

```shell
cargo run --bin ln
```

编译没有问题的话，会得到以下结果（编译过程中出现的warning请暂时忽略）：

![普通示例](/img/tutorial/2-00.png)

> 关于WSL的一个warning：
>
> 如果你使用WSL开发，在编译的时候感受到了异常的缓慢，同时看到这个warning：
>
> `warning: Hard linking files in the incremental compilation cache failed.`
>
> 你可以尝试将当前工作的文件夹移动到WSL的文件管理系统内部去，通常能够解决问题。

最后，每次在git需要提交之前，别忘了给你的代码格式化：

```shell
cargo fmt
```

当然这个需要你安装`rustfmt`组件，如果在运行的时候报错了，按照报错的提示进行安装即可。
