---
id: tutorial7
title: 7. 多语言支持
---

在本章节，我们将介绍多语言支持。目前 `iced`并没有原生的多语言支持，所以实际上这方面并不是很友好。在介绍解决方案之前，首先需要解决的问题是中文乱码。

### 中文乱码

之前如果添加的note里有中文的小伙伴，应该已经发现了`iced`对中文支持不友好，即使当前我们已经知道了解决方案只需要替换字体即可，但是实际上还是有些地方并没有完美的解决。

替换默认字体：
 ```rust
 // 在 lib.rs 内
 pub fn settings() -> iced::Settings<()> {
    iced::Settings {
        default_font:font(),
        ..Default::default()
    }
}
// OnceCell可以帮助我们安全的定义一个生存期为 static的全局变量
static FONT: OnceCell<Option<Vec<u8>>> = OnceCell::new();

fn font() -> Option<&'static [u8]> {
    FONT.get_or_init(|| {
        // 需要添加iced_graphics这个crate
        use iced_graphics::font::Family;
        let source = iced_graphics::font::Source::new();
        source
            .load(&[
                Family::Title("PingFang SC".to_owned()),
                Family::Title("Hiragino Sans GB".to_owned()),
                Family::Title("Heiti SC".to_owned()),
                Family::Title("Microsoft YaHei".to_owned()),
                Family::Title("WenQuanYi Micro Hei".to_owned()),
                Family::Title("Microsoft YaHei".to_owned()),
                // TODO:iced 目前没有字体fallback，所以我们只能尽可能选择中英文支持的字体
                Family::Title("Helvetica".to_owned()),
                Family::Title("Tahoma".to_owned()),
                Family::Title("Arial".to_owned()),
                Family::SansSerif,
            ])
            .ok()
    })
    .as_ref()
    .map(|f| f.as_slice())
}
 ```
 > 不完美的地方在于，目前`iced`并不支持富文本，同时也没有相应的字体fallback，因此，当你指定了默认字体的时候，无论该字符是否正确绘制，都会默认使用该字体。这造成的重要问题是，我们想要app正确渲染中文字符，就只能使用中文的字体。

 ### 简陋的多语言支持

 目前 `iced`并没有原生多语言支持的功能，因此我们只能通过其他的相关crate来完成这个需求。可以选择的方案很多，最后选择的方案是使用fluent_bundle，在这里谈一下其他的解决方案。

 在选择方案前，首先对当前的需求做了大致的预估，可以明确的知道当前的app在多语言支持需求上比较简陋，仅仅是一些简易的文本替换。也就是在这个预估下，选择方案上偏向于简易实现的方案。

 最先考虑到的是fluent-rs系列的选择方案，[fluent-rs](https://github.com/projectfluent/fluent-rs)提供了一系列的对多语言支持的crate，从资源管理到高级抽象，你都能够在fluent-rs里找到。但是，在`iced`里，我们仍然不能直接爽快的使用fluent-rs提供的高级抽象（特指[fluent-fallback](https://crates.io/crates/fluent_fallback))我们在每次调用文本切换时，都需要提供一个资源管理的变量用于查询，可以说是十分不爽，可以看看下面的官方实例：

  ```rust
  // 以下代码都来自fluent-fallback的官方文档，不用看的很仔细，大致了解调用过程即可
use fluent::{FluentBundle, FluentValue, FluentResource, FluentArgs};

// Used to provide a locale for the bundle.
use unic_langid::LanguageIdentifier;
// 在app当中，此处对应的ftl文件
let ftl_string = String::from("
hello-world = Hello, world!
intro = Welcome, { $name }.
");
let res = FluentResource::try_new(ftl_string)
    .expect("Failed to parse an FTL string.");

let langid_en: LanguageIdentifier = "en-US".parse().expect("Parsing failed");
let mut bundle = FluentBundle::new(vec![langid_en]);
// bundle提供了一系列的语言标识符，需要和资源相对应，当查找不到相应语言的资源时
// 会有一个fallback的过程，默认的fallbakc语言是bundle构建时提供的数组里索引为
// 0的语言
bundle
    .add_resource(res)
    .expect("Failed to add FTL resources to the bundle.");
// 对，如果我们使用fluent-fallback的话，最大的问题就是调用的时候
// 我们需要提供一个bundle变量，通过bundle变量去获取相应多语言文本
let msg = bundle.get_message("hello-world")
    .expect("Message doesn't exist.");
let mut errors = vec![];
let pattern = msg.value()
    .expect("Message has no value.");
let value = bundle.format_pattern(&pattern, None, &mut errors);

assert_eq!(&value, "Hello, world!");

let mut args = FluentArgs::new();
args.set("name", FluentValue::from("John"));

let msg = bundle.get_message("intro")
    .expect("Message doesn't exist.");
let mut errors = vec![];
let pattern = msg.value().expect("Message has no value.");
let value = bundle.format_pattern(&pattern, Some(&args), &mut errors);

// The FSI/PDI isolation marks ensure that the direction of
// the text from the variable is not affected by the translation.
assert_eq!(value, "Welcome, \u{2068}John\u{2069}.");
  ```
以上仅仅是一个fluent-fallback的一个简短实例，我们可以发现整个调用过程十分繁杂，说是一个高级抽象，但用起来一点都不高级。其实在脑海中我们希望的调用方式更像是这样：
```rust
// App本身提供一个枚举用来表示当前的语言，切换语言只需要切换该枚举的值即可
let hello = tr("hello");
assert_eq!(hello, "哈喽！");
```
对，我们需要的抽象，应该要如此简单才对，其实在Rust社区里，还真有一个类似这样抽象的包：[tr](https://lib.rs/crates/tr)。也正是我们上面使用的函数名来源，tr->translate（或者translation）的缩写，它本身来源于qt中的tr函数。

那已经有了类似的包，我们直接用不就可以了么？确实是这样，但是相对于当前版本的实现，使用tr的话，我们仍然需要考虑和iced的交互，以及多语言的切换，这也是为什么最后的方案没有选择这个，而仅仅是参考了部分api的原因。

还有个非fluent-rs官方做的包：[fluent-templates](https://github.com/XAMPPRocky/fluent-templates)，官方的例子是这样的：

```rust
// 在 src/translate.rs 下
use std::collections::HashMap;

use unic_langid::{LanguageIdentifier, langid};
use fluent_templates::{Loader, static_loader};

const US_ENGLISH: LanguageIdentifier = langid!("en-US");
const FRENCH: LanguageIdentifier = langid!("fr");
const GERMAN: LanguageIdentifier = langid!("de");

static_loader! {
    static LOCALES = {
        locales: "./tests/locales",
        fallback_language: "en-US",
        // Removes unicode isolating marks around arguments, you typically
        // should only set to false when testing.
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

fn main() {
    assert_eq!("Hello World!", LOCALES.lookup(&US_ENGLISH, "hello-world"));
    assert_eq!("Bonjour le monde!", LOCALES.lookup(&FRENCH, "hello-world"));
    assert_eq!("Hallo Welt!", LOCALES.lookup(&GERMAN, "hello-world"));

    let args = {
        let mut map = HashMap::new();
        map.insert(String::from("name"), "Alice".into());
        map
    };

    assert_eq!("Hello Alice!", LOCALES.lookup_with_args(&US_ENGLISH, "greeting", &args));
    assert_eq!("Bonjour Alice!", LOCALES.lookup_with_args(&FRENCH, "greeting", &args));
    assert_eq!("Hallo Alice!", LOCALES.lookup_with_args(&GERMAN, "greeting", &args));
}
```
距离我们想要的效果，已经很接近了，只需要写一个tr宏来包裹看上去很丑的`LOCALES`调用就可以了。

事实上最终方案也确实是参考刚刚谈到的这两个包：
 ```rust
 use std::{borrow::Cow, fmt::Display, sync::Arc};

use elsa::sync::FrozenMap;
use fluent_bundle::FluentResource;
use fluent_bundle::{bundle::FluentBundle, FluentArgs};
use intl_memoizer::concurrent::IntlLangMemoizer;
use serde::{Deserialize, Serialize};
use tokio::sync::{OnceCell, RwLock};
use unic_langid::langid;
use unic_langid::LanguageIdentifier;

use crate::error_handle;

// OnceCell是用来安全的申明全局变量的
// FrozenMap是hash表的变种，只支持添加新的key
// bundle_cache用来储存读取好了的bundle
static BUNDLE_CACHE: OnceCell<FrozenMap<Language, Arc<FluentBundle<FluentResource, IntlLangMemoizer>>>> =
    OnceCell::const_new();
// bundle是用来查询多语言的ftl文件的
pub static BUNDLE: OnceCell<RwLock<&FluentBundle<FluentResource, IntlLangMemoizer>>> =
    OnceCell::const_new();
// 读取ftl文件
async fn read_file(path: &str) -> Option<String> {
    tokio::fs::read_to_string(path)
        .await
        .map_err(error_handle)
        .ok()
}

// 创建resource，出错的时候打印错误即可
async fn create_resource(path: &str) -> Option<FluentResource> {
    let content = read_file(path).await?;
    match FluentResource::try_new(content) {
        Ok(res) => Some(res),
        Err((res, err)) => {
            err.into_iter().for_each(error_handle);
            Some(res)
        }
    }
}
// 创建用于查询的bundle
async fn create_bundle(
    locale: Language,
) -> Option<FluentBundle<FluentResource, IntlLangMemoizer>> {
    let path = locale.path();
    let res = create_resource(&path).await?;
    let mut bundle = FluentBundle::new_concurrent(vec![locale.locale()]);
    bundle.set_use_isolating(false);
    if let Err(e) = bundle.add_resource(res) {
        e.into_iter().for_each(error_handle);
    }
    Some(bundle)
}
// 初始化bundle缓冲
async fn init_bundle_cache(
    locale: Language,
) -> Option<&'static FluentBundle<FluentResource, IntlLangMemoizer>> {
    let ress = BUNDLE_CACHE.get_or_init(|| async { FrozenMap::new() }).await;
    ress.get(&locale)
        .or({
            let bundle = create_bundle(locale).await?;
            Some(ress.insert(locale, Arc::new(bundle)))
        })
}
// 初始化bundle，同时也用于语言切换
pub async fn init_bundle(locale: Language) -> Option<()> {
    let bundle = init_bundle_cache(locale).await?;
    if BUNDLE.initialized() {
        let mut bundle_inner = BUNDLE.get()?.write().await;
        *bundle_inner = bundle;
    } else {
        BUNDLE
            .set(RwLock::new(bundle))
            //.map_err(error_handle)
            .ok()?;
    }
    Some(())
}
// 这里的就是实际调用tr宏展开的函数了
// 返回值为什么是Cow，主要原因是BUNDLE查询之后的返回值就是Cow
#[inline]
pub fn tr_with_args<'a, 'arg: 'a>(
    key: &'a str,
    args: Option<&'a FluentArgs<'arg>>,
) -> Cow<'a, str> {
    let res = BUNDLE
        .get()
        .and_then(|bundle| bundle.try_read().ok())
        .and_then(|bundle| {
            bundle
                .get_message(key)
                .and_then(|msg| msg.value())
                .map(|p| {
                    let mut errors = vec![];
                    let res = bundle.format_pattern(p, args, &mut errors);
                    errors.into_iter().for_each(error_handle);
                    res
                })
        });
    if let Some(res) = res {
        res
    } else {
        Cow::from(key)
    }
}
// 目前只支持代码内定义多语言
// 最初我自己的构想是通过读取指定文件夹内的内容来生成多语言
// 但是因为过于复杂而放弃了这种构想
#[derive(Copy, Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    Chinese,
}
const EN_US: LanguageIdentifier = langid!("en-US");
const ZH_CN: LanguageIdentifier = langid!("zh-CN");

impl Language {
    pub fn locale(&self) -> LanguageIdentifier {
        match self {
            Language::English => EN_US,
            Language::Chinese => ZH_CN,
        }
    }
    // 提供资源的路径即可，ftl文件有专门的语法，看fluent-rs的文档即可
    pub fn path(&self) -> String {
        let locale = self.locale().to_string();
        format!("./locales/{}/tr.ftl", locale)
    }
}
impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}
// 实现display是为了后续的设置界面UI作准备
impl Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::English => write!(f, "english"),
            Language::Chinese => write!(f, "中文"),
        }
    }
}
// 这就是我们的宏了，和tr包比起来十分简陋
// 但是已经满足我们的需求了
#[macro_export]
macro_rules! tr {
    ($msg:expr) => {
        crate::translate::tr_with_args($msg, None)
    };
    ($msg:expr; $args:expr ) => {{
        crate::translate::tr_with_args($msg, Some($args))
    }};
}
// 用于生成参数的宏，为什么不和tr集成到一块呢？
#[macro_export]
macro_rules! args {
    ($($key:expr => $value:expr),+ ) => {
        {
            let mut args: fluent_bundle::FluentArgs = fluent_bundle::FluentArgs::new();
            $(
                args.set($key, $value);
            )+
            args
        }
    };
}
 ```
 同时，用到的包也比较多：
 ```toml
 # 在 Cargo.toml 内
[deoendencies]
# -- 多语言支持需要用到的包 --
fluent-bundle = "0.15"
once_cell = "1.8"
unic-langid = { version = "0.9",features = ["macros"] }
elsa = "1.4"
intl-memoizer = "0.5"
fern = "0.6"
tokio = {version = "1",features = ["fs","io-util","macros"]}
# -- 就这些了，需要注意once_cell 之前第一章的时候已经添加过了--
 ```
这样基本就完成了多语言的支持，当然，我们还需要在初始化的时候去加载多语言支持：
```rust
// 在 lib.rs 内
// 新增了translate模块
mod translate;

impl iced::Application for LocalNative {
    // 其他的基本没有变动，仅仅是new的时候多执行了一条初始化bundle的命令
    // 语言切换的UI我们将在下一章介绍，因此此处只需要直接传入任意一个已定义的语言
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            LocalNative::Loading,
            Command::batch(
                [
                    Command::perform(async {}, Message::Loading),
                    Command::perform(translate::init_bundle(translate::Language::Chinese), Message::ApplyLanguage)
                ]
            )
        )
    }
    // unpdate里同样需要接受ApplyLanguage进行处理，不过不需要特殊的处理
    // 因此返回Command::none()即可
}
```

目前基本已经搞定了，接下来就是一些文本的切换了，首先需要创建对应语言的ftl文件，我定义的路径就是工作区间的同等目录下的locales文件夹，在里面创建了两个文件夹：`en-US`和`zh-CN`，名字命名也是有规则的，规则可以在`fluent-rs`的文档里面找。在这两个文件夹内都创建了名为`tr.ftl`的文件，这在定义路径的时候确定了的。

文件的内容：
```ftl
# 在 en-US 下的tr.ftl文件
tags = tags:
not-found = Sorry, the result you want was not found...
nothing = You don’t have any notes yet, you can add notes through the browser extension.
search = Type your search...
```
类似的定义中文下的ftl文件即可：
```ftl
# 在 zh-CN 下的tr.ftl文件
tags = 标签：
not-found = 对不起，没有您想要的结果。
nothing = 您还没有任何note，您可以通过浏览器扩展添加note。
search = 输入您的搜索…
```
定义好之后我们可以将此前的固定文本替换成tr了，以search为例：
 ```rust
 // 在 search_page.rs -> impl SearchPage -> view内
 // before:
        let mut search_bar = Row::new().push(
            TextInput::new(
                input_state,
                "Type your search...",
                &search_value,
                Message::SearchInput,
            )
            .on_submit(Message::Search),
        );

 // after:
        let mut search_bar = Row::new().push(
            TextInput::new(
                input_state,
                &tr!("search"),
                &search_value,
                Message::SearchInput,
            )
            .on_submit(Message::Search),
        );
 ```

 类似的其他部分再此就不贴出来了，更改结束之后大功告成：

 ![0](/img/tutorial/7-00.png)

 ### 漂亮的图标

 正好本章节是和字体有关，因此将图标替换也放在此处吧，其实`iced`是支持svg的，你只需要准备好svg，替换掉之前的字符就可以了，但是选择svg就要承担相应的后果：你不能使用glow，因为svg只支持wgpu。
 
 并且，当前版本的wgpu对OpenGL支持并不好（估计下下个版本能有更好的支持），因此使用字体的形式来使用这些icon，是个更合适的选择。

 找到你喜欢的svg图标，通常能够在相关网站上找到svg对应字体的下载，下载之后，通过下面的方式引入iced即可。

 ```rust
use iced::{Font, Text};
// 通过引入字体的方式，再使用font函数时应用字体即可
// 如果想要icon和普通文本放到一块
// 可以考虑使用Row将他们放到同一行
const ICONS: Font = Font::External {
    // 注意，如果使用诸如iiced_aw此类的crrate时，不要将自己的字体命名为Icons，因为会和内部的字体冲突
    name: "LocalNativeIcons",
    bytes: include_bytes!("../fonts/icons.ttf"),
};


pub enum IconItem {
    Search,
    Clear,
    Delete,
    Settings,
    Filter,
    FilterOff,
    Refresh,
    Next,
    Pre,
    Full,
    FullExit,
    QRCode,
    DayTime,
    MonthTime,
    SyncFromFile,
    SyncFromServer,
    SyncToServer,
    OpenServer,
    CloseServer,
    Sync,
    Dark,
    Light,
    Date,
    Note,
}

impl IconItem {
    fn into_char(self) -> char {
        // 在svg图标网站上下载到的字体，通常包含诸如css之类的文件
        // 这些文件里将每个图标及其对应的字符号都标识出来了，
        // 我们在代码中将这些一一对应即可
        match self {
            IconItem::Search => '\u{f0d1}',
            IconItem::Clear => '\u{eb99}',
            IconItem::Delete => '\u{ec1e}',
            IconItem::Settings => '\u{f0e6}',
            IconItem::Filter => '\u{ed27}',
            IconItem::FilterOff => '\u{ed29}',
            IconItem::Refresh => '\u{ed2a}',
            IconItem::Next => '\u{ea6e}',
            IconItem::Pre => '\u{ea64}',
            IconItem::Full => '\u{ed9c}',
            IconItem::FullExit => '\u{ed9a}',
            IconItem::QRCode => '\u{f03d}',
            IconItem::DayTime => '\u{f20f}',
            IconItem::MonthTime => '\u{f20e}',
            IconItem::SyncFromFile => '\u{eccf}',
            IconItem::SyncFromServer => '\u{ec58}',
            IconItem::SyncToServer => '\u{f24d}',
            IconItem::OpenServer => '\u{eb9d}',
            IconItem::CloseServer => '\u{eb9f}',
            IconItem::Sync => '\u{eba1}',
            IconItem::Dark => '\u{ef72}',
            IconItem::Light => '\u{f1bf}',
            IconItem::Date => '\u{eb29}',
            IconItem::Note => '\u{ea7e}',
        }
    }
    pub fn into_text(self) -> Text {
        Text::new(&self.into_char().to_string()).font(ICONS)
    }
}
// 实现了from方法可以更方便的调用
impl<'a, Message> From<IconItem> for Element<'a, Message>
where
    Message: 'a,
{
    fn from(icon: IconItem) -> Element<'a, Message> {
        Element::new(icon.into_text())
    }
}
 ```

 接着将之前的文本替换成你想要的图标即可：

![1](/img/tutorial/7-01.png)

替换图标之后，属实清爽多了。在下一章我们将会实现一个左边栏，通过左边栏能够切换页面到诸如同步等多个选项。

## 课后练习（Quiz）

在本章里我们使用了不少的字符串，在创建字符串的过程中，肯定会有不少埋怨的想法，毕竟是真的麻烦。
那如何简便的创建一个字符串`String`类型呢？

A) 
使用`String::from("你的&str内容")`。

B) 
使用这种：
```rust
let this_string = "这段&str需要转换成String".to_owned();
```

C)
大部分时候这种更简洁：
```rust
fn return_string() -> String{
    "这段&str需要转换成String".into();
}
```

答案（Explanation）

每一个选项都能够创建`String`，在不同的场景下可以选择适宜的方法，重点介绍一下第三种，当你需要返回或者传入一个`String`时，你只需要传递`&str`加上`.into()`即可。我们讨论一下为什么Rust里要将`String`设计的如此繁琐？我的看法是`String`本身就是复杂的东西，同时相对于使用`&str`而言，会有更多的性能负担，所以通常是能使用`&str`的地方，尽可能的不适用`String`。