---
id: tutorial1
title: 1. 初始化项目结构
---

在序章里，我们已经搭建好了开发需要的环境，在本章中将要初始化项目结构，方便后续开发的时候进行预览。

>  如果你使用的是WSL进行开发，同时在进行`Vulkan`设置的时候出了一些问题，导致`Vulkan`并不能如期运行，因此即使编译没有问题，在接下来运行GUI界面的时候很有可能会出现问题，最好的解决方案就是不使用`Vulkan`作为渲染后端，而是替换成`OpenGL`。接下来将会介绍开启的方法。

### 开启需要的功能集

在序章里，我们引入依赖的时候，只是引入了`iced`最小化支持，但有些额外的功能需要我们自行开启，本节将会给出一些我们所需要的所有功能集：

```diff
# 在 Cargo.toml 内
[dependencies]
- iced = "0.3"
localnative_core = { path = "../localnative/localnative-rs/localnative_core", features =["no_print"] }

+ [dependencies.iced]
+ version = "0.3"
+ default-features = false

+ [features]
+ default = ["preview"]
+ wgpu = [
+     "iced/default",
+     "iced/tokio",
+     "iced/qr_code",
+     ]
+ opengl = [
+     "iced/glow",
+     "iced/tokio",
+     "iced/glow_qr_code",
+     "iced/glow_default_system_font"
+     ]
+ preview = [
+     "wgpu"
+ ]
```

我们删掉了之前`[dependencies]`下的`iced`依赖，转而添加了`[dependencies.iced]`。

除了版本依然是`0.3`以外，我们还添加了`default-features = false`这个选项，`default-features`这个字段设置为`false`时，将会关闭掉`iced`默认的`features`。

接着我们构建了自己的`features`，分别是三个新的`feature`：`wgpu`、`opengl`、`preview`。其中默认开启的`feature`是`preview`，这个`feature`将会帮助我们构建预览程序，能够通过添加一些简单的测试用例帮我们熟悉`iced`的视图。

`wgpu`和`opengl`的`feature`想必大家能够看出来是开启了一系列的`iced`的`feature`，而`preview`的内容则是`"wgpu"`，代表的意思是，如果要开启`preview`这个`feature`，就会自动开启`wgpu`这个`feature`。如果你没有特殊需求，我的建议是此处设置为`wgpu`就可以。

> 如果你使用的是WSL，同时遇到了本文开头所说的问题，则此处的preview建议设置为`"opengl"`，它将帮助你解决WSL在进行`Vulkan`设置时遇到的各种奇怪问题。（因为根本就不适用`Vulkan`😀）

### 创建可预览程序集

在接下来的开发过程中，我们会构建各种GUI组件，最后会将这些组件组合成一个完整的应用程序，为了方便教学中进行调试，我们需要构建一个可预览的程序集，因此我们在`ln-iced/`目录下创建一个叫做`previews`文件夹：

```diff
// before:
    .
    ├── Cargo.lock
    ├── Cargo.toml
    ├── src
    │   └── lib.rs
    └── target
        ... // generated files
    
// after:
    .
    ├── Cargo.lock
    ├── Cargo.toml
++  ├── previews
++  │   └── examample.rs
    ├── src
++  │   ├── bin.rs
    │   └── lib.rs
    └── target
        ... // generated files
```

我们不仅创建了新的文件夹，同时还新建了两个`.rs`文件，一个是当前用来作为设置示例的`example.rs`文件，另一个是我们整个项目的最终可执行文件`bin.rs`。

这也是为啥我们在创建项目之初使用的是`cargo new ln-iced --lib`的原因，也正因此我们可以通过开启`preview`这个`feature`，在不增加最终可执行文件的大小情况下，运行我们想要测试的程序中的其中一部分。

好了，除了文件结构上的改变，我们也需要给`Cargo.toml`增加一些东西：

```diff
# 在 Cargo.toml 内
+ [[bin]]
+ name = "ln"
+ path = "./src/bin.rs"

+ [[bin]]
+ name = "preview-example"
+ path = "./previews/example.rs"
+ required-features = ["preview"]
```

我们定义了两个`bin`，这两个同时指定了各自的路径，以及`required-features`。

> 注意： 能够设置成`bin`的可执行文件需要在文件内定义`main`函数。

以及两个文件内的内容：

```rust
// ./previews/example.rs
fn main() {
    println!("Hello!");
}
```

```rust
// ./src/bin.rs
fn main() {
	println!("Hello!");
}
```

我们通过以下命令来运行我们想要运行的可执行文件：

```shell
# 运行名字叫做ln的可执行文件
cargo run --bin ln
# 运行名字叫做preview-example的可执行文件
cargo run --bin preview-example
```

通过上面两个示例，你一定能够知道是通过`--bin`这个选项后面跟上我们在`Cargo.toml`里定义的名字来运行指定的可执行文件的。

如果运行没有什么问题，那么初始化项目结构就算成功了。其中`previews`这个文件夹，我们后续在开发中需要创建多个可执行文件来帮助我们了解和学习`iced`。因此创建一个这样的可执行文件的过程在后续教程中会多次出现，在这里已经详细介绍了，所以后续将不会再着重介绍。

最后，在每一章内容完成之后，不要忘了提交一个commit。