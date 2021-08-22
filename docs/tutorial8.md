---
id: tutorial8
title: 8. 配置和侧边栏
---

在此前我们已经实现了app的绝大多数功能，在本章中，我们将会实现侧边栏的功能，也同时是app剩下的所有功能。

### 配置文件

实际上，有很多信息需要我们保存下来，在下一次打开app之前读取这部分信息，用以创建应用程序。我们的想法是这样的：

> 打开应用程序 -> 读取配置文件（如果没有的话，按照默认配置文件创建） -> 由读取到的配置文件构建app -> 在app运行时，根据不同的设置对配置文件作出更改 -> app退出前，将配置文件进行保存到指定位置

使用到的操作也很简单，就是读取和保存，因为我们的配置文件也很小，不需要做太特别的优化，因此只需要用简单的方法来实现读取和保存两个方法即可：

```rust
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config;// 我们的配置文件里，暂时什么都没放
```

`Config`实现序列化和反序列化有利于我们对文件进行读取和写入。

```rust
impl Config {
    pub const APP_NAME: &'static str = "LocalNative";
    pub const CONFIG_NAME: &'static str = "config.json";
  	// 获取app_dir，默认是home下的LocalNative文件
  	// 当然，特殊情况下可能没有home文件夹
  	// 在获取home失败的前提下，我们使用临时文件夹
    pub fn app_dir() -> PathBuf {
        if let Some(home) = localnative_core::dirs::home_dir() 				{
            home.join(Self::APP_NAME)
        } else {
            std::env::temp_dir().join(Self::APP_NAME)
        }
    }
  	// 配置文件夹的目录
    pub fn config_path() -> PathBuf {
        Self::app_dir().join(Self::CONFIG_NAME)
    }
}
```

实现了两个简单的文件获取方法之后，我们来实现读取方法：

```rust
impl Config {
  // 根据我们的构思，读取只发生在app打开之前
  // 因此使用同步方法会更符合我们的预期
    pub fn load() -> Option<Self> {
      // 引入Read之后才能使用诸如read_to_string等方法
        use std::io::Read;
        let mut contents = String::new();
      // 获取config的路径
        let path = Self::config_path();
			// 尝试打开配置文件
        let mut file = std::fs::File::open(path).map_err(error_handle).ok()?;
			// 配置打开之后，尝试读取文件到字符串
        file.read_to_string(&mut contents)
            .map_err(error_handle)
            .ok()?;

      // 最后反序列化读取到的字符串，成功则返回Config，否则返回None
      serde_json::from_str(&contents).map_err(error_handle).ok()
    }
}
```

读取的方法也很简单粗暴，中间任何一个环节出错了，都直接返回`None`即可。接着我们实现保存方法：

```rust
// 按照之前的构思，app在退出之前会进行保存操作
// 保存操作完成之后有一个异步回调告知程序可以退出了
// 这个时候再进行实际上的程序退出
// 这种写法在配置较低的电脑上可能会造成关闭延迟的问题
// 之后的版本应该会考虑加入部分优化
// 有更好构思的想法，可以在issue里讨论
pub async fn save(json: String) -> Option<()> {
  	// 虽然是实际上使用同步应该没多大区别
  	// 甚至有可能性能上会更好一些
  	// 但是为了满足写异步函数的好奇，最终这里选择了使用tokio来写
  	// 引入这个trait之后才能使用tokio的一些写入函数
    use tokio::io::AsyncWriteExt;
		// 我们可以试着输出一下即将保存的json文件
  	// 选择json的原因是我们的依赖里，在之后必然会使用到json序列化
  	// 为了减少更多依赖关系，我们最终配置文件也选择json
    println!("json:{}", json);
  	// 这一步简单看作类型转换即可，将字符串转换为写入时需要的文件
    let raw_data = json.as_bytes();
    let path = Config::config_path();
		// 判断一下config的文件夹是否存在，不存在的话，我们需要创建一下
    if let Some(dir) = path.parent() {
        if !dir.exists() {
            tokio::fs::create_dir_all(dir)
                .await
                .map_err(error_handle)
                .ok()?;
        }
    }
		// 如果路径是一个文件夹，我们需要删除掉config.json这个文件夹
    if path.is_dir() {
        tokio::fs::remove_dir(&path)
            .await
            .map_err(error_handle)
            .ok()?;
    }
  	// 最后我们直接创建文件写入即可，这里可以做打开文件来进行更多判断
  	// 但是最后写入的时候也是直接全部序列化写入
  	// 所以直接创建写入是没有问题的
    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(error_handle)
        .ok()?;
		// 写入数据即可
    file.write_all(raw_data).await.map_err(error_handle).ok()?;
    Some(())
}

```

我们将`Config`放到`LocalNative`结构体里，因为需要在app启动之前就读取到配置文件，因此我们的`LocalNative`结构体需要一些变动：

```rust
pub struct LocalNative {
    config: Config,
    should_exit: bool,// 这个变量用来控制是否该退出app
    state: State,
}
#[allow(clippy::large_enum_variant)]// 这个是为了过clippy的lint检查
pub enum State {// 之前的LocalNative编程了State
    Loading,
    Loaded(Data),
}
```

> clippy是rust的一个lint工具，可以让你的代码更rust，安装clippy：
>
> `rustup component add clippy`。
>
> 执行clippy检查：
>
>`cargo clippy --workspace --all-targets  -- -D warnings -A clippy::type_complexity`。
>
> 你可以和我一样在工作区间的`.cargo`目录下创建`config`文件，使用`Cargo`的别名系统来简化这个命令：
>
> ```
> [alias]
> lint = "clippy --workspace --all-targets  -- -D warnings -A clippy::type_complexity"
> ```
>
> 这样每次使用的时候，只需要：`cargo lint`就会执行clippy的检测了。
>
> 在此前的代码中，会有很多报错，你可以按照报错信息逐行了解，根据你的喜好和判断来选择是否要开启某个报错，比如此处我就允许了在一个枚举内拥有两个大小相差很大的数据结构。在clippy内，会提示你使用Box来将大的data放到堆上，而实际上我们这里的Loading阶段存在时间很少，基本上大部分都是data存在，以及根本没有大量的状态交换的情况，所以允许将内存占用相差过大的两个结构体放在一个枚举内，在这里是可以被允许的。
>
> 还有一个地方需要注意，报错提示中给定的clippy报错类型，是使用`-`来链接，替换到Rust代码中的allow里，需要将`-`替换成`_`。

我们也需要对Application实现两个之前没有实现的方法：

```rust
impl iced::Application for LocalNative {
    type Executor = iced::executor::Default;

    type Message = Message;
		// flags我们定义为config
  	// 使用Option的原因是我们的load返回值就是这个
    type Flags = Option<Config>;
  	// new方法也和此前有点出入
    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
      	// 如果读取到的config是空的，则说明是第一次打开app
      	let is_first_open = flags.is_none();
        let config = flags.unwrap_or_default();
      	// 直接先获取出language是因为接下来config的所有权会移动
        let language = config.language;
      	// 同时，现在你已经知道了我们config中使用到language这个字段
      	// language用于记录当前app语言设置
        (
            LocalNative {
                config,
                should_exit: false,
                state: State::Loading,
            },
          	// 使用Command::batch此前我们已经介绍过了
          	// 如果你想要和我下面这种简洁的写法
          	// 那至少要使用1.53.0版本的Rust
          	// 否则编译过程中，这里会报错，提醒你使用迭代器
            Command::batch([
                Command::perform(async {}, Message::Loading),
                Command::perform(translate::init_bundle(language), Message::ApplyLanguage),
              // 如果是第一次打开，我们会做一个初始化操作
              // 这个命令会初始化浏览器和host程序之间的交互
              // 我们将在最后一个章节介绍
                if is_first_open {
                    Command::perform(init::WebKind::init_all(), Message::InitHost)
                } else {
                    Command::none()
                },
            ]),
        )
    }
  // Application还给我们提供了一个订阅的方法
  // 通过这个方法你可以简单的将窗口事件、鼠标事件、键盘事件等
  // 映射为iced的Message进行更新处理，需要注意的是
  // 如果你使用的控件中，有和刚刚说到的这几个事件进行交互的
  // 需要一些特殊的过滤，以免事件被你在此全部截断
  fn subscription(&self) -> iced::Subscription<Self::Message> {
    		// 其中events_handler将在之后介绍
        iced_native::subscription::events_with(events_handler)
    }
  	// 这是用于控制iced程序是否退出的flag，当我们将config保存之后
  	// 将self.should_exit设置为ture，则程序将会正确退出
  	// 也正是因为这样，在配置较低的电脑上，对配置进行写入
  	// 会花费一定事件，只有写入完成之后，才能进行退出
  	// 即造成了延迟退出的问题，如果你有更好的解决方法
  	// 请在gitlab或者gitee的官方代码库内提出issue
  	// LocalNative团队将会十分感谢你的帮助
    fn should_exit(&self) -> bool {
        self.should_exit
    }
  	// update方法不会全部列出，因为我们对LocalNative结构体作出了
  	// 比较重大的改变，所以update又些地方需要重新写
  	// 因为处理的事件比较多，且杂，这里就不放出所有的代码了
  	// 你可以到gitlab或者gitee的源码库内找到这部分的完整源码
    fn update(
        &mut self,
        message: Self::Message,
        _clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
      // 和之前模式匹配一样，是为了分离所有权
      let LocalNative { config, state, .. } = self;
	        match state {
            State::Loading => match message {
                Message::Loading(..) => {
                    let conn = Arc::new(Mutex::new(get_sqlite_connection()));
										// 根据config来生成搜索页面
                  	// 这些陌生的字段，我们都将在本章进行介绍
                    let data = Data {
                        search_page: SearchPage::from_config(&*config),
                        sidebar: Sidebar::default(),
                        delete_tip: DeleteTip {
                            rowid: -1,
                            tip_state: Default::default(),
                        },
                        sync_view: SyncView::new(),
                        settings: settings::Settings {
                            disable_delete_tip_temp: config.disable_delete_tip,
                            language_temp: config.language,
                            limit_temp: config.limit,
                            state: Default::default(),
                        },
                        conn,
                    };
										// 切换state
                    self.state = State::Loaded(data);
                    if let State::Loaded(ref mut data) = self.state {
                        let Data {
                            conn, search_page, ..
                        } = data;
												// 执行此前我们写好的upgrade方法
                        Command::perform(
                            MiddleDate::upgrade(
                                conn.clone(),
                                search_page.search_value.clone(),
                                config.limit,
                                search_page.offset,
                            ),
                            Message::Receiver,
                        )
                    } else {
                      // unreachable宏用来做一些优化
                      // 当代码真的到达这里的时候，程序会panic
                        unreachable!()
                    }
                }
                Message::InitHost(_) => Command::none(),
                _ => Command::none(),
            },
            State::Loaded(data) => match message {
	
                Message::ApplyLanguage(..) => Command::none(),
              // 接收到请求关闭窗口事件
              // 我们将config进行保存
              Message::RequestClosed => {
                // 这些字段都是创建deteview的时候用到的
                // 我们将这些保存到config，写入硬盘
                // 方便下一次打开的时候，仍然能保持之前的操作参数
                    config.date_filter_is_show = data.search_page.days.is_show;
                    config.date_mode_is_full = data.search_page.days.is_full;
                    config.day_uw = data.search_page.days.chart.day_uw;
                    config.month_uw = data.search_page.days.chart.month_uw;
                		// 这里我们绝对不允许失败
                		// 因此直接unwrap就可以了
                    let json = serde_json::to_string_pretty(&*config).unwrap();
                		// 执行save命令
                    Command::perform(config::save(json), Message::CloseWindow)
                }
              // 接收到保存文件的message，我们将should_exit设置为true
                Message::CloseWindow(res) => {
                    if res.is_some() {
                        println!("ok!");
                    }
                    self.should_exit = true;
                    Command::none()
                }
              // ... 其他的Message，照常处理即可
                Message::InitHost(..) => Command::none(),
            },
        }
  }
}
```

其中`events_handler`是一个用于过滤事件的函数：

```rust
fn events_handler(event: Event, states: Status) -> Option<Message> {
  	// 判断此事件的状态是否属于忽略，如果属于的话，再进行下一步
    if states == Status::Ignored {
        if let Event::Window(window::Event::CloseRequested) = event {
          	// 我们关心的只有这个事件，返回即可
            return Some(Message::RequestClosed);
        }
    }
    None
}
```

### 删除警告

在我们开始写侧边栏之前，我们需要啊写一个删除警告，当note被用户删除的时候，会给用户一个警告提示。

![0](/img/tutorial/8-00.png)

使用到一个控件叫做`Modal`是`iced_aw`里的一个控件，我们需要在`features`里手动开启：

```toml
[dependencies.iced_aw]
git = "https://github.com/iced-rs/iced_aw"
branch = "main"
default-features = false
# 除了modal，还有card和icons我们都需要开启
features = [ "wrap", "number_input", "modal", "date_picker","icons","card"] 
```

开启之后我们创建一个`delete_tip`模块，然后在里面构建提示的界面：

```rust
pub struct DeleteTip {
  	// rowid 是存放note的rowid号，在后续删除的时候
 		// 需要提供这个号码给后端
    pub rowid: i64,
  	// 构建modal，相当于在当前的ui界面上绘制另一层ui
  	// 我们把绘制另一层ui的状态作为范型参数传递给modal
  	// 的state
    pub tip_state: modal::State<TipState>,
}
#[derive(Default)]
pub struct TipState {
    pub ok_button: button::State,
    pub cancel_button: button::State,
}
```

了解了结构体之后，我们给它实现`view`方法：

```rust
#[derive(Debug, Clone)]
pub enum Message {
    Enter,
    Cancel,
    SearchPageMessage(crate::search_page::Message),// 用于映射搜索页面的message
}
impl DeleteTip {
  	// 我们的tip将在搜索页面点击删除时出现
  	// 因此可以将search_page传递进来
    pub fn view<'tip, 'page: 'tip>(
        &'tip mut self,
        theme: Theme,
        limit: u32,
        search_page: &'page mut crate::SearchPage,
    ) -> Element<'tip, Message> {
      	// 调用search_page的view方法，接着映射Message到本地的Message
        let underlay = search_page
            .view(theme, limit)
            .map(Message::SearchPageMessage);
        let Self { tip_state, .. } = self;
				// 这个控件的构建方法和iced官方的其他空间不太一样
      	// 它的state，是由必包提供的
        Modal::new(tip_state, underlay, |state| {
          	// 构建ok按钮
            let ok_button =
                Button::new(&mut state.ok_button, Text::new(tr!("ok"))).on_press(Message::Enter);
          	// 构建cancel按钮
            let cancel_button = Button::new(&mut state.cancel_button, Text::new(tr!("cancel")))
                .on_press(Message::Cancel);
          	// 构建一个card控件
          	// 需要提供head显示的element
          	// body显示的element
          	// 和foot显示的element
            Card::new(
                Row::new()
                    .push(
                        Text::new(iced_aw::Icon::ExclamationDiamond)
                            .font(iced_aw::ICON_FONT)
                            .color(iced::Color::from_rgba(1.0, 0.0, 0.0, 0.7)),
                    )
                    .push(Text::new(tr!("delete-tip"))),
                Text::new(tr!("delete-tip-content")),
            )
            .foot(
                Row::new()
                    .push(style::horizontal_rule())
                    .push(ok_button)
                    .push(cancel_button)
                    .push(style::horizontal_rule())
                    .spacing(10),
            )
            .on_close(Message::Cancel)// 当点击关闭时对应的事件
            .max_width(300)// 设置好最大宽度
            .into()
        })
        .style(style::transparent(theme))// 在style内按照自己喜好，实现style，也可以不选
        .on_esc(Message::Cancel)// 当按esc按键时对应的事件
        .backdrop(Message::Cancel)// 当鼠标点击非card部分时对应的事件
        .into()
    }
}
```

这样就把delete时的警告视图完成了。只需要将其添加到删除按钮对应的message即可，这部分的工作很简单，就不再赘述，你可以参考完整代码库查看对应内容。

### 设置

设置我们也会用modal来做，虽然和删除时的警告用的控件差不多，但是有一个稍微有一个区别，我们需要在不同页面下都能够打开settings，而删除时的警告，只需要考虑搜索页面下能够打开。

```rust
// 和删除时的警告一样，需要将state作为范型参数放到modal里
pub struct Settings {
    pub language_temp: Language,
    pub disable_delete_tip_temp: bool,
    pub limit_temp: u32,
    pub state: modal::State<State>,
}
#[derive(Default)]
pub struct State {
    pub save_button: button::State,
    pub cancel_button: button::State,
    pub try_fix_host: button::State,
    pub limit_input: number_input::State,
    pub language_selector: pick_list::State<Language>,
}
```

稍微不一样的地方在于view方法的实现：

```rust
#[derive(Debug, Clone, Copy)]
pub enum Message {
    Save,
    Cancel,
    TryFixHost,
    DisableTip(bool),
    LanguageChanged(Language),
    LimitChanged(u32),
  // 这个message用来将传入的element映射到这个
    Other,
}
impl Settings {
    pub fn view<'settings, 'underlay: 'settings>(
        &'settings mut self,
        theme: Theme,
      // 最大的区别在于这里传入的是Element
        underlay: Element<'underlay, Message>,
        config: &'underlay Config,
    ) -> Element<'settings, Message> {
			// 具体实现可以看代码库，传入的underlay直接用就可以了
      // 需要注意的是，语言的选择部分原本是想着用picklist来做
      // 但是picklist和modal使用的时候会出现一个bug，也就是
      // 不能正常选择，因此暂时选择了radio来进行替代，此前我们
      // 还没有用过这个，所以这里贴出一些用法，以供参考：
                let language_selector = Row::new()
                .push(style::horizontal_rule())
      // 使用radio你甚至不需要提供state
                .push(Radio::new(
                  // 被选中时，message返回的语言
                    Language::English,
                  // 显示时的内容
                    tr!("english"),
                  // 当前被选中的值，如果和第一个参数一致，则处于被选中状态
                    Some(language),
                    Message::LanguageChanged,
                ))
                .push(Radio::new(
                    Language::Chinese,
                    tr!("chinese"),
                    Some(language),
                    Message::LanguageChanged,
                ))
                .push(style::horizontal_rule())
                .spacing(30);
    }
}
```

### 同步页面

同步页面的实现很简单，用到的基本都是此前的空间，值得说的是文本输入框，使用文本输入框返回得到的字符串，你可以做出过滤，我们可以用这个方法来实现一个支持ipv4地址过滤的输入栏：

```rust
    pub fn update(&mut self, message: Message, conn: Conn) -> Command<crate::Message> {
        match message {
            Message::IpInput(input) => {
              // 这里用了正则表达式来过滤，因为很麻烦，所以
              // ipv6的多久没有这样做，而是使用了解析器来做的
              // 只要输入的字符串能够解析为ipv6，则输入成功
              // 
                let ip_regex = IP_REGEX_SET.get_or_init(||{
                    RegexSet::new(&[
                        r"^$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.?$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.?$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.?$",
                        r"^(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|2[0-4]\d|[0-1]?\d?\d)\.(25[0-5]|[0-4]\d|[0-1]?\d?\d)$",
                    ]).unwrap()
                });
                if ip_regex.is_match(&input) || Ipv6Addr::from_str(&input).is_ok() {
                    self.ip = input;
                }
            }
      // ... 其他upddate
      }
```

同步这部分的view方法，用到的控件基本是此前用到的，但是又个列外，就是这个：![1](/img/tutorial/8-01.png)


当点击从文件同步的按钮时，会弹出这样一个选择文件的界面，我们使用了另一个包来完成这个事情：

```rust
// 获取同步文件的路径
pub fn get_sync_file_path() -> Option<PathBuf> {
    localnative_core::dirs::desktop_dir()
        .unwrap_or_else(std::env::temp_dir)
        .to_str()
        .and_then(|path| {
          // 使用的是tinyfiledialogs这个包
          // 这个包在跨平台的支持上很不错
            open_file_dialog(
                &tr!("sync-file-title"),
                path,
                Some((&["*.sqlite3"], &tr!("sync-file"))),
            )
        })
        .map(PathBuf::from)
}

pub async fn sync_via_file(path: PathBuf, conn: Conn) -> Option<()> {
  // 得到文件的路径之后，调用core的同步方法即可
    tokio::task::spawn(async move {
        if let Some(uri) = path.to_str() {
            let conn = &*conn.lock().await;
            localnative_core::cmd::sync_via_attach(conn, uri);
        }
    })
    .await
    .map_err(error_handle)
    .ok()
}
```

除此之外，还有一个地方也比较棘手，我们可以将自己的设备作为服务器给其他设备进行同步，服务器开启的方式在后端实际上是用一个异步任务来做，那么服务器关闭就需要我们将这个异步任务给终结掉，在早起的Electron版本中，因为使用的是FFI Rust进程来开启服务器，只需要将开启的Rust进程终结掉就可以。而iced直接调用了后端的包，本质上和后端就是同一个程序，不可能简单的调用程序退出。

但其实解决的方案也很简单，而且有很多中解决方案，最终的解决方案是采用类似通道的解决方案在开启服务器时，会保存一个handle但服务器需要关闭的时候，drop掉这个handle即可，其他的方案五花八门，感兴趣的话，可以去搜搜看怎么结束一个正在执行的异步任务。

```rust
// 开启服务的时候，返回一个handle
pub async fn start_server(port: u16) -> std::io::Result<Stop> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);
    localnative_core::rpc::server::iced_start_server(addr).await
}
// 要关闭服务，将handle给drop掉即可
pub async fn stop_server(stop: Stop) -> Option<()> {
    let res = stop.await.map_err(error_handle).ok()?;
    drop(res);
    Some(())
}
```

还有一个难点在于怎么确定本机的ip坐标，这个用于展示本机的地址，方便其他设备连接服务器时进行输入，这个解决方案也很多，这里采用的是这种：

```rust
pub fn get_ip() -> Option<String> {
    use std::net::UdpSocket;
  // 使用udp进行通讯连接之后获取本地地址
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:90").and_then(|_| s.local_addr()))
        .map(|addr| addr.ip().to_string())
        .map_err(error_handle)
        .ok()
}
```

### 侧边栏

侧边栏的view方法很简单，就四个不同的按钮，我们就不在赘述，可以直接看源码，可能只有使用state来控制界面这个可以说一说，但是因为之前些dateview的时候接触过类似的做法，所以实际上也并不需要特别说明。

### 总结

至此，整个app 基本功能完全实现，变化最大的莫属`lib.rs`，很多地方都有了巨大的变化，甚至在写教程的同时，我也发现了很多此前写的不够好，可以写更好的地方，做出了很多重构。虽然app实现了，但是打包却没有实现，下一章会详细介绍一下我们打包是使用什么工具来帮助我们的。
