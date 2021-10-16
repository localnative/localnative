---
id: tutorial0
title: 0. 序章
---

从今天起，我们将要使用[`iced`](https://github.com/hecrj/iced)来构建一个跨平台的网站书签管理工具。

![localnative演示](/img/tutorial/0-00.gif)

当然在开始正式开发之前，我们需要做一些准备：

1. 安装Rust，请参考[官方安装引导](https://www.rust-lang.org/learn/get-started)。

2. 安装依赖：

   - **Ubuntu Linux**:

     ```sudo apt install pkg-config libfreetype6-dev libfontconfig1-dev libxkbcommon-dev```

   - **Windows**：

     确定你的电脑安装有[VS2019 构建工具](https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=16)

   - **MacOS**：

     没啥特殊依赖

3. 代码编辑器/IDE

   - 我强烈推介[Visual Studio Code](https://code.visualstudio.com/)和[Rust Analyzer](https://github.com/rust-analyzer/rust-analyzer)插件
   - 另一个选择是[CLion](https://www.jetbrains.com/clion/)以及其Rust插件

4. 如果你想要使用WSL进行GUI开发，请参考[这篇文章](https://wiki.ubuntu.com/WSL)进行设置。（在Running Graphical Applications这节有详细介绍）

5. 在进行开发之前，你需要对Rust有一定认识，在此推介官方的教程：[**The Rust Book**](https://doc.rust-lang.org/book/)。

6. 你需要安装有[Git](https://git-scm.com/)命令行工具，我们将使用它作为版本管理工具。

当你做好上面的准备之后，我们还需要做一些小工作，方便我们之后的开发。

首先我们需要获取`LocalNative`的代码核心库，这部分代码同样是用Rust编写的，它作为跨平台支持的核心库，不仅仅是桌面端，移动端同样依赖于它，因此单独作为一个`crate`进行管理。

新建一个用作项目开发的空文件夹，名字随意，我们将在这个文件夹下克隆核心库，方便我们之后开发。

```shell
git clone https://gitlab.com/localnative/localnative.git
```

中国读者如果在克隆的过程中出现网络异常的问题，建议通过国内git托管平台作为中转站，具体操作方法是在[`gitee`](https://gitee.com/)等类似平台内导入仓库，接着再从`gitee`克隆就可以了，在此我已经帮大家建了一个中转仓库了：

```shell
git clone https://gitee.com/localnative/localnative.git
```

克隆之后我们将在新项目中，将核心库作为依赖，方便我们后续开发，为了不影响主代码库，我们将把新项目和克隆来的`localnative`文件夹放到同一个文件夹内。

即在当前克隆文件夹内，我们创建一个新项目：

```shell
cargo new ln-iced --lib
```

此时我们的文件夹结构应当是这样的：

```shell
/current/folder
> tree
.
├── ln-iced
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
├── localnative
...// localnative files
```

我们创建新项目的时候，使用`--lib`的原因是为了后续方便构建多个不同的GUI测试用例，这样更方便我们熟悉`iced`的视图如何构建。

好了现在我们进入`ln-iced`文件夹：

```shell
cd ln-iced/
```

接着可以使用你喜欢的代码编辑器打开这个项目，我们需要将核心代码库作为依赖加到我们的`Cargo.toml`里去。

在我们添加新内容到项目中之前，我们需要提交以下commit，方便记录项目的变化：

```shell
git add .
git commit -m "init project"
```

接着我们在`Cargo.toml`中加入我们需要的依赖：

```diff
# ln-iced/Cargo.toml

[dependencies]
+ iced = "0.3"
+ localnative_core = { path = "../localnative/localnative-rs/localnative_core", features =["no_print"] }
```

添加依赖之后你可以在命令行里编译一下，看看我们的依赖是否添加正确：

```shell
cargo build
```

编译的过程十分漫长，经过几分钟的等待之后，如果编译没有出现问题，那么我们的依赖环境搭建算是完毕了。如果在整个过程中出现了任何问题，欢迎留言评论。

## 课后练习（Quiz）

在最后我们加入依赖时使用了`features`，相对于未使用`features`编译结果有什么区别？

A) 
相对于未使用`features`，通常使用之后可以减少编译文件大小。

B) 
没什么区别，主要是能让添加的依赖增加一些指定的功能。

C)
既能在一定程度上缓解编译文件的大小，同时也能增加一些指定的功能。

答案（Explanation）

正确的选项应该是C，A和B均回答到了一些，但是并不全面，使用`features`在引用足够大的依赖时能让我们选择想要的功能集，避免程序因为不需要的额外引入依赖占据多余空间。同时有些依赖的某些`feature`是不常用的，因此并没有默认提供，需要我们自行在引入依赖时添加`features`。