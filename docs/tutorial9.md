---
id: tutorial9
title: 9. 初始化和打包
---
### 浏览器host初始化

Local Native的应用程序实际上是由两个部分组成的，一个是GUI应用程序，我们使用了iced来进行开发，另一个是浏览器插件外部调用程序，用来给浏览器插件与本地的数据库进行交互，这部分程序编译后和Local Native放在同一个文件夹内。

> 在linux上由于打包成Appimage，在程序运行时，可执行文件会被挂载到一个临时的文件夹内，因此浏览器扩展只能在本地App打开时使用修复按钮恢复通讯，和我们对程序预期效果不一致。因此在Linux上的实现该用了将包内的Host程序（浏览器插件外部调用程序）复制到一个固定的位置。

浏览器通讯时依赖于相应的配置文件，该文件可以查阅各大浏览器文档找到相关信息，在linux和mac上你只需要将该json放到特定的位置，浏览器扩展在打开后会自动从特定位置读取相应配置文件进行通讯。Windows上需要将json文件自己指定位置保存，同时修改相应的注册表内容，这部分操作实际上应该放到打包安装时进行，但是会加大开发难度。因此当前的做法是通过运行时进行初始化操作。

这部分代码在`src/init.rs`内可以查阅到，代码内容只是生成json配置文件，放到特定的位置（或者修改对应的注册表内容），比较复杂的部分在于多个平台不同的位置，给开发过程中调试带来不少难度。我此前只有Windows设备因此只调试了Windows平台和linux平台（通过WSL），现在多了Mac，在打包后才明确知道具体文件路径应该怎么写才合适。

之前获取当前运行程序位置调用的是`std::env::current_dir()`，在mac平台打包之后获取的位置是诸如`/Contents/MacOS`此类打包后app内部的文件夹，这并不符合我们的预期，因此后续改成了`std::env::current_exe()`，这样获取的地址就是绝对路径了，拿到之后调用`.parent()`即可获取当前运行程序的实际位置。

获取该位置是用来配置json文件，指定host程序的位置，配置文件基本是这个样子：

```rust
#[derive(Debug, Default, Serialize)]
pub struct AppHost {
    name: String,
    description: String,
    path: PathBuf,
  // type 是rust关键字，不能用做字段名字
  // 因此需要我们使用serde的rename来在序列化的时候重命名为type
    #[serde(rename = "type")]
    tp: String,
    allowed_extensions: Vec<String>,
}
```

各字段需要和浏览器插件一一对应，出了path是动态的，其他字段基本是静态的：

```rust
impl AppHost {
  // 目前主要支持的浏览器内核就只有这两种，基本也只是稍有区别
    pub fn firefox() -> Self {
        Self {
            name: "app.localnative".to_owned(),
            description: "Local Native Host".to_owned(),
          // path是动态获取的
            path: Self::path(),
            tp: "stdio".to_owned(),
            allowed_extensions: vec!["localnative@example.org".to_owned()],
        }
    }
    pub fn chrome() -> Self {
        Self {
            name: "app.localnative".to_owned(),
            description: "Local Native Host".to_owned(),
            path: Self::path(),
            tp: "stdio".to_owned(),
            allowed_extensions: vec![
                "chrome-extension://oclkmkeameccmgnajgogjlhdjeaconnb/".to_owned()
            ],
        }
    }
}
```

### 打包和简便开发

打包使用的是[`tauri-bundler`](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler)一个基于[`cargo-bundle`](https://github.com/burtonageo/cargo-bundle)开发的打包器，前者支持多个平台多种打包方式，后者和Rust项目耦合性要高一些，同时支持的平台更少，在这里再次感谢`tauri`团队，感谢他们的开发人员，因为在Windows下打包使用固定模版，他们的安装包在运行之前会检测并安装`Webview`程序，对于我们的程序是不需要的，先是在他们的帮助下添加了模版替换功能，后续因为我的需求而增加`Webview`安装的可关闭选项。

Local Native打包并没有使用`tauri.conf.json`文件进行打包，而是直接将其当作库来调用，只需要在工作区间内执行命令即可：

```cargo xtask release -v [version]```

你可以指定打包时的版本，xtask是一个被配置在工作区间的`.cargo/config`文件内的子命令：

```toml
[alias]
# 即调用cargo xtask 和 cargo run --package xtask --bin xtask 是一致的
xtask = "run --package xtask --bin xtask"
fmt-check = "fmt --all -- --check"
lint = "clippy --workspace --all-targets  -- -D warnings -A clippy::type_complexity"
iced = "run --bin local-native"

[target.x86_64-pc-windows-msvc]
linker = "rust-lld"
```

此前还有多个不同的子命令，考虑到下一个版本的wgpu即将支持包括`OpenGL`在内的全平台图形后端，已经将别名做了简化。

> 说一下使用xtask的理由，xtask如果此前你没有接触过，你可以理解成它就是一个命令行工具库，可以让你方便的编写命令行工具，同时因为所需的依赖很小，可以很快就编译，因此用来作为脚本文件是比较合适的。但是最重要的理由是，我们的应用程序需要在多个平台进行打包，使用shell脚本并不能满足多个平台下的方便调用，使用xtask已经帮我们提供了对应多个平台的一致性操作，加上我们打包是必然是需要Rust工具链的，因此使用xtask并不会造成额外的困扰。社区里诸如**[`rust-analyzer`](https://github.com/rust-analyzer/rust-analyzer)**都使用了xtask来管理项目，**[`bevy`](https://github.com/bevyengine/bevy)**的CI也使用了xtask来进行项目测试。

xtask的文档很详细，这里就不再深入探究，Local Native的xtask除了`release`子命令之外，还提供了`ndkbd`（即ndk build）的子命令，用于方便的构建安卓移动端所需的Rust依赖，需要安装**[`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk)**工具，该工具要求配置两个环境变量，同时需要完整的安卓开发环境支持，这是打包安卓开发时需要执行的第一步。

目前来说因为引入了[`tauri-bundler`](https://github.com/tauri-apps/tauri/tree/dev/tooling/bundler)，所以实际上编译的时候xtask所需的依赖很大，解决方案是采用条件编译的方式，在引入依赖的时候，限定为某个条件，同时再更改对应别名即可。目前因为编译不是十分慢，在我可以接受的范围，因此没有动力去做出更改，但提出解决方案，是为了给以后使用xtask时可能会遇到的坑做个总结。

还有一个坑，在mac上编译时，因为我们项目xtask和其他项目是在同一个工作区间，因此造成了一定耦合，如果两者同时使用release进行编译，会造成文件占用的问题，在Windows上居然没出现问题，好在xtask都是一些工具化脚本命令，对性能几乎没有需求，因此别名是使用debug模式进行编译的。正确的解决方案是xtask应该要比工作区间更高一级，用来当作脚本处理即可。

## 课后练习（Quiz）

本章我们介绍了`cargo`别名配置文件，我们可以把常用的一些命令变得简单容易。
我们有什么理由用Rust来写脚本文件呢？


A) 
Rust学习难度陡峭，使用过程繁琐，不该用来写脚本文件。

B) 
脚本文件重在便捷性，Rust还需要编译才能运行，不应该用来写脚本。

C)
在一个Rust项目里，我们考虑到跨平台运行，使用一些部署脚本，用Rust来写，真是太方便了，虽然需要编译，但是在所有平台都是保持一致性的！

答案（Explanation）

所有选项都没错，但是我们最后还是用Rust写了部分脚本文件，多个平台下的开发，确实很便捷。