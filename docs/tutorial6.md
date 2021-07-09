---
id: tutorial6
title: 6. 使用Canvas构建日期过滤器
---

在本节我们将使用`iced`的`Canvas`控件来构建一个日期过滤器。

### 一些小准备

我们之前在接收后端传回的数据时，有一个数据到目前仍然没有使用过，就是days:
```rust
#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct MiddleDate {
    pub count: u32,
    pub notes: Vec<Note>,
    pub days: Vec<Day>,
    pub tags: Vec<Tag>,
}
```
但是实际上在使用`electron`构建的GUI里，里面包含的日志过滤器控件使用的数据并不是后端返回的days字段，而是`Note`结构体内的`created_at`字段。今天我们不用考虑这么多，我们就直接用days字段即可。当然，还需要做一些小的修改。
1. 首先添加一个后端的过滤方法：
```rust
// 在 middle_date.rs -> impl MiddleDate 内
// 有了之前的经验，这里的接口对接都很简单，因此不需要过度关注
    // 这是指定某个时间段的查询
    pub async fn from_filter(
        conn: Arc<Mutex<Connection>>,
        query: String,
        limit: u32,
        offset: u32,
        from: time::Date,
        to: time::Date,
    ) -> Option<Self> {
        let conn = &*conn.lock().await;
        let from = from.to_string();
        let to = to.to_string();
        Self::from_filter_inner(conn, &query, &limit, &offset, &from, &to)
    }
    // 这是指定某一天的过滤查询
    pub async fn from_someday(
        conn: Arc<Mutex<Connection>>,
        query: String,
        limit: u32,
        offset: u32,
        day: time::Date,
    ) -> Option<Self> {
        let conn = &*conn.lock().await;
        let day = day.to_string();
        Self::from_filter_inner(conn, &query, &limit, &offset, &day, &day)
    }
    // 这是直接调用后端的过滤方法
    fn from_filter_inner(
        conn: &Connection,
        query: &str,
        limit: &u32,
        offset: &u32,
        from: &str,
        to: &str,
    ) -> Option<Self> {
        let filter_result = localnative_core::exe::do_filter(conn, query, limit, offset, from, to);

        serde_json::from_str::<Self>(&filter_result).ok()
    }
```
需要注意的是，因为之前设计后端的时候，始终没有将days作为需要渲染的数据，因此  `localnative_core::exe::do_filter(conn, query, limit, offset, from, to)`返回的结果并没有days字段，如果只是为了接收结果，可以将现有的`MiddleDate`结构体的days字段变为：`Option<Vec<Day>>`。但是我更推介直接在后端的返回值内加上days字段（拉取最新的代码库，立面的do_filter函数返回的是四个字段）。

### 另一个小准备
此前我们定义Day结构体是这样的：
```rust
// 在days.rs内
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Day {
    #[serde(rename = "k")]
    pub day: String,
    #[serde(rename = "v")]
    pub count: i64,
}
```
但是这样的结构体并不能让我们更直观的处理日期，因此我们改成这样：
```rust
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct Day {
    // Date是time::Date，顺便连字段名字都改了，
    // 更改了字段名字会设计部分代码重构，在此省略重构的部分
    #[serde(rename = "k")]
    pub date: Date,
    #[serde(rename = "v")]
    pub count: i64,
}
```
只是这样的话，并不能正确的序列化返回的结构，我们还需要开启一个feature：
```toml
# 在 Cargo.toml 内
# before：
# time = { version = "0.3.0-alpha-2",features = ["macros","formatting","parsing"] }
# now：
time = { version = "0.3.0-alpha-2",features = ["serde-human-readable","macros"] }
```
`serde-human-readable`本身包含了`formatting`、`parsing`、`serde`三个feature，因此开启这个feature之后可以将另外几个feature关闭。仅在原来的基础上加上`serde`并不能满足我们的需求，查看`time`的文档可以了解到更多信息。

### 一个完整的简单小实现

有了以上的准备，我们可以放开手脚使用`Canvas`控件了，这是一个相对特殊的控件，提供了更多的自由性，同时可以让你做出很多你能想象的功能，用来做数据可视化是最合适的了。当然当前的`Canvas`还有很多问题需要解决，我们将在实现的过程中对其一一介绍。

作为用来熟悉`Canvas`的实现，是一个很常见的功能，当鼠标左键按下会出现一个选取框，鼠标放开之后选取框会消失。就像这样一样：

![01](img/tutorial/6-01.gif)

在这个过程中，我们需要储存的数据很简单，就是鼠标左键按下的那个点，除此之外，鼠标的当前位置，由trait提供给我们。

思路已经有了，我们可以来看看具体的实现过程：

```rust
// 我们将需要用于绘制的数据放到DateChart这个数据结构内
pub struct DateChart {
    // Point是iced::Point，当Option为None的时候即是不需要绘制的时候
    pub pending: Option<Point>,
}
#[derive(Clone, Debug)]
pub enum ChartMsg {
}
// 需要给需要绘制的数据实现iced::widget::canvas::Program这个trait
// 这个trait内提供了三个方法，分别是update用来处理事件
// draw用来绘制数据
// mouse_interaction用来处理鼠标状态
// 其中draw是必须要实现的，而另两个可以选择性实现
// 当前的实例将会三个都接触到
// 和给iced的主程序类似，处理事件需要提供一个枚举体用来保存事件
// 我们使用的是ChartMsg,简单实现Clone和Debug这两个trait
impl Program<ChartMsg> for DateChart {
    fn update(
        &mut self,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (canvas::event::Status, Option<ChartMsg>) {
        // 这是用来获取当前的bounds内的鼠标位置，鼠标不在绘制范围内时暂时不需要处理
        if let Some(cursor_position) = cursor.position_in(&bounds) {
            // 同样使用模式匹配，找到想要处理的事件进行处理即可
            match event {
                // 我们找到鼠标事件，再进一步找到我们想要处理的鼠标事件
                canvas::Event::Mouse(me) => match me {
                    // 我们暂时只对左键按下和松开两个事件进行处理
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        // 当鼠标按下左键时，我们替换需要绘制的框型固定点
                        self.pending.replace(cursor_position);
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        // 当松开鼠标左键的时候，清空pending的值
                        let pending = self.pending.take();
                    }                    
                    _ => {}
                },
                canvas::Event::Keyboard(_) => {}
            };
        }
        (canvas::event::Status::Ignored, None)
    }
    // 绘制方法提供了三个参数，一个是用于绘制的数据即self
    // 一个是绘制的框bounds，和绘制能用到的鼠标cursor
    // 返回值是一系列的图形Geometry
    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let size = bounds.size();
        let mut res = vec![];
        // 判断是否存在需要绘制的点
        if let Some(pending) = self.pending.map(|pending| {
            // 构建一个帧结构体，提供画框的大小即可构建
            let mut frame = Frame::new(size);
            // 判断鼠标是否在画框内
            if let Some(cursor_position) = cursor.position_in(&bounds) {
                // 当鼠标在画框内的时候，构建出需要绘制的方框左上角
                // 需要注意的是iced左上角的坐标值为0.0,0.0
                // 越往下y越大，越往右x越大，这种坐标设计在布局更改的时候会有更多的好处
                // 我们在绘制date的时候会感受到这种坐标设置的优势
                let top_left = Point::new(
                    cursor_position.x.min(pending.x),
                    cursor_position.y.min(pending.y),
                );
                // 同时构建需要绘制的方框大小
                let size = Size::new(
                    (cursor_position.x - pending.x).abs(),
                    (cursor_position.y - pending.y).abs(),
                );
                // 调用帧绘制方框的方法
                frame.fill_rectangle(
                    top_left,
                    size,
                    // 目前只能提供颜色填充
                    Fill::from(Color::from_rgba(0.1, 0.5, 0.8, 0.2)),
                );
            }
            // 将帧转化为图形，最后放入结果内返回
            frame.into_geometry()
        }) {
            res.push(pending);
        }

        res
    }

    fn mouse_interaction(
        &self,
        _bounds: Rectangle,
        _cursor: Cursor,
    ) -> iced_native::mouse::Interaction {
        // 当pending的值不为空时，将鼠标的状态定位指针
        if self.pending.is_some() {
            return iced_native::mouse::Interaction::Pointer;
        }
        iced_native::mouse::Interaction::default()
    }
}
```
通过以上简单的实现一个trait，我们`Canvas`控件就已经实现了一大半了，我们还需要再和应用程序接洽之前给它再包裹抽象一层，使用一个Chart结构体来包裹它：
```rust
// 暂时不需要任何字段，后续复杂时会添加其它字段
#[derive(Debug)]
pub struct Chart {
}
impl Chart {
    pub fn new() -> Self {
        Self {}
    }
    // 主要就是实现一个view方法，上层只需要提供相应的数据即可获得一个Canvas控件
    pub fn chart_view(&self) -> Element<ChartMsg> {
        Canvas::new(DateChart {
            pending: None
        })
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }
}
```
通过包裹了一层抽象之后，我们在调用之后绘制更多数据的时候，会更容易。
和之前一样，我们也构建一个preview：
```rust
#[cfg(feature = "preview")]
impl iced::Sandbox for Chart {
    type Message = ChartMsg;

    fn new() -> Self {
        Self::new()
    }

    fn title(&self) -> String {
        "chart".to_owned()
    }

    fn update(&mut self, message: Self::Message) {
        // todo!()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        self.chart_view()
    }
}
```
```diff
// 在lib.rs内
+ pub use days::Chart;
# 在Cargo.toml内：
+ [[bin]]
+ name = "chart"
+ path = "./previews/days.rs" 
+ required-features = ["preview"]
```
同时我们在previews文件夹内创建一个days.rs文件：
```rust
// ./previews/days.rs 内
use iced::Sandbox;
use localnative_iced::Chart;

fn main() -> iced::Result {
    Chart::run(localnative_iced::settings())
}
```
最后我们运行：`cargo run --bin chart`，即可得到一个绘制方框的Canvas。

### 一个较为复杂的实现
经过一个简单的实例，我们已经掌握了实现一个Canvas控件的过程，简单实现`Program`trait即可获得一个Canvas控件，在draw方法内，我们可以绘制上我们想要绘制的内容，`iced`内部提供了多种绘制方法，目前我们的需求仅仅需要方框和文本，方框我们已经见过了，接下来的一节将介绍文本绘制。

在这之前，我们需要介绍一下`fill_text`这个[方法](https://docs.rs/iced/0.3.0/iced/widget/canvas/struct.Frame.html#method.fill_text)：
```rust
pub fn fill_text(&mut self, text: impl Into<Text>)
```
 > 注意：text需要Into的Text不是iced::Text，而是canvas内的Text（iced::widget::canvas::Text）
在文档内特意给了警告：
 > Warning: Text currently does not work well with rotations and scale transforms! The position will be correctly transformed, but the resulting glyphs will not be rotated or scaled properly.
 简单来说就是当前的文本在绘制上属于最高层，同时并不兼容于旋转和放大这两种平移，这只会对我们后续绘图有唯一一个影响，绘制的文本会在给定绘图框外显示，当然我们在绘制的时候会有相应的解决方案。

在开始前，我们需要了解一下需求，我们其实只需要在给定画框内选取之后，就能够获得一个日期，通过这个日期从后端获取过滤后的数据用于绘制即可。实现的思路上也很简单，以当天为基准，即使用`now_utc`方法获取今天的日期，得到之后作为最右边绘制的数据，从最右边向左边逐渐绘制。通过使用translation来改变镜头的平移，每次绘制都用以确定需要绘制的范围。思路大致上就是这样，文本是每一个都需要绘制，日期只需要绘制后端返回的值即可，难点在于确定哪些日期需要绘制，日期绘制的位置。后端返回的days是排序过的，从旧到新的时间顺序排列。

```rust
    // 在绘制之前，我们从后端接收到的数据需要做一些预处理：
    // handle_days我放在了impl Day立面，它并不适用self，
    // 因此单独放在外部也没有问题
    pub fn handle_days(days: Vec<Day>) -> HandleDays {
        let len = days.len();

        if len == 0 {
            return HandleDays::default();
        }
        let mut monthviews = Vec::new();
        // 最大count用于确定竖向绘制的单位高度
        // 我们绘制的横向单位有天为单位和月为单位，因此有两种类型的count
        let mut max_day_count = 0;
        let mut max_month_count = 0;
        let mut monthcount_temp = 0;
        // 获取days的第一个日期的年份和月份
        let (mut pre_year, mut pre_month, _) = days.first().unwrap().date.to_calendar_date();

        for day in days.iter() {
            // 判断年份是否变动，变动即构建一个MonthView结构体，并更新count数
            if day.date.year() != pre_year || day.date.month() != pre_month {
                if monthcount_temp != 0 {
                    max_month_count = max_month_count.max(monthcount_temp);
                    monthviews.push(MonthView {
                        month: pre_month,
                        year: pre_year,
                        count: monthcount_temp,
                    });
                    monthcount_temp = day.count;
                } else {
                    monthcount_temp += day.count;
                }
                pre_year = day.date.year();
                pre_month = day.date.month();
            } else {
                monthcount_temp += day.count;
            }
            // 更新最大天为单位下的count数
            max_day_count = max_day_count.max(day.count);
        }
        // 当月的count临时数不为零时还需要进行最后的month插入
        if monthcount_temp != 0 {
            max_month_count = max_month_count.max(monthcount_temp);

            monthviews.push(MonthView {
                month: pre_month,
                year: pre_year,
                count: monthcount_temp,
            });
        }
        // 计算出最旧的天和最新的天之间的天数间隔
        let full_days = days.last().and_then(|last| {
            days.first()
                .map(|first| (last.date - first.date).whole_days())
        });
        // 同上，但是计算的单位是月,months_num会在之后定义
        let full_months = monthviews
            .last()
            .and_then(|last| monthviews.first().map(|first| months_num(first, last)));
        // last_day计算的是接近作为基准的天的距离
        let last_day = days.last().map(|day| (base_day() - day.date).whole_days());
        // 同上
        let last_month = monthviews
            .last()
            .map(|month| months_num(month, &base_month()));
        HandleDays {
            days,
            months: monthviews,
            max_day_count,
            max_month_count,
            full_days,
            full_months,
            last_day,
            last_month,
        }
    }
```
```rust 
// 定义MonthView用来绘制月为单位的情况
#[derive(Debug, Clone)]
pub struct MonthView {
    // Month是time::month
    pub month: Month,
    pub year: i32,
    // 后端返回的count就是i64
    pub count: i64,
}
#[derive(Debug, Default, Clone)]
pub struct HandleDays {
    pub days: Vec<Day>,
    pub months: Vec<MonthView>,
    // count用来确定单位高度
    pub max_day_count: i64,
    pub max_month_count: i64,
    // full 用来确定单位宽度
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    // last 用来确定初始化镜头平移位置
    // 之前说过canvas左上角是0，0
    // 而从给定基准天或者月作为最右边来进行绘制的话
    // 我们需要获取一个平移位置，
    // 这个位置的值就需要拥有数据的最新一天/月
    // 和基准月之间的距离
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
}
// 以下方法都是用以方便计算的一些简单抽线
pub fn months_num(start: &MonthView, end: &MonthView) -> i32 {
    (end.year - start.year - 1) * 12 + 13 + month_to_num(end.month) - month_to_num(start.month)
}
pub fn month_to_num(month: Month) -> i32 {
    match month {
        Month::January => 1,
        Month::February => 2,
        Month::March => 3,
        Month::April => 4,
        Month::May => 5,
        Month::June => 6,
        Month::July => 7,
        Month::August => 8,
        Month::September => 9,
        Month::October => 10,
        Month::November => 11,
        Month::December => 12,
    }
}
// 暂时使用的是当前日期的后一天作为基准日
// 这里直接使用unwrap即可，不需要做特殊的错误处理
pub fn base_day() -> Date {
    time::OffsetDateTime::now_utc().date().next_day().unwrap()
}
// 同上，只不过time没有提供方便的next_month方法，因此自己写了
pub fn base_month() -> MonthView {
    let today = time::OffsetDateTime::now_utc().date();
    let tomorrow = today.next_day().unwrap();
    let tomorrow_month = tomorrow.month();
    if tomorrow_month != today.month() {
        MonthView {
            month: tomorrow_month,
            year: tomorrow.year(),
            count: 0,
        }
    } else {
        let year = if tomorrow_month == Month::December {
            tomorrow.year() + 1
        } else {
            tomorrow.year()
        };
        MonthView {
            month: tomorrow_month.next(),
            year,
            count: 0,
        }
    }
}
```
从后端得到days之后，进行处理，处理好的days再赋予到需要绘制的结构体上，最后进行绘制。

```rust
// 数据结构变得复杂起来
pub struct DateChart<'a> {
    // 每一帧都需要构建新的DateChart
    // 因此较大的数据我们直接传递指针来构建
    pub days: &'a [Day],
    pub months: &'a [MonthView],
    // Cache是用来存储绘制的帧的
    // 同时拥有Cache可以很方便的刷新绘制
    pub cache: &'a Cache,
    pub selected_cache: &'a Cache,
    // 和pending类似，但是是一个另外定义的结构体
    pub selected: Option<&'a Selected>,
    pub pending: Option<Point>,
    // 用来确定单位高度
    pub maximal_day_count: i64,
    pub maximal_month_count: i64,
    // 确定好的固定单位宽度
    pub day_uw: f32,
    pub month_uw: f32,
    // 用来确定不固定的单位宽度，当为None的时候取固定单位宽度
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    // 用来确定平移值
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
    // 用来确定当前所处的等级
    pub level: ChartLevel,
    // 提供一些风格化数据
    pub style: Style,
    // 需要应用到帧的平移
    pub translation: Vector,
    // 基准日/月
    pub base_day: Date,
    // 如果你给你定义的MonthView实现了Copy
    // 你可以直接传递MonthView，而不是引用
    pub base_month: &'a MonthView,
}
impl <'a> DateChart<'a> {
    // 初始化trnslation和单位宽度
    fn init(
        &self,
        size: Size,
        translation: &mut Vector,
        last_day: Option<i64>,
        last_month: Option<i32>,
        selected: &Option<&Selected>,
    ) -> f32 {
        match self.level {
            ChartLevel::Day => {
                if let Some(num) = self.full_days {
                    // 最大的坑就是as，使用as需要仔细考虑溢出的情况
                    // 最好的方式是给定一个行限定范围
                    let uw = (size.width as i64 / (num + 3)).max(1).min(50);
                    // 镜头平移的计算方法很简单，因为选取范围代表的是距离最右边的天数/月数差
                    // 只需要使用月数差乘上单位宽度即可
                    if let Some(s) = selected {
                        // 当有选取范围时，镜头平移到选取范围内
                        *translation = Vector::new(
                            ((s.end.max(1) - 1) * uw as usize).max(0).min(10000) as f32,
                            0.0,
                        )
                    } else {
                        // 没有选取范围时镜头平移到最后一天有数据的即可
                        if let Some(last) = last_day {
                            *translation =
                                Vector::new(((last - 1).max(0) * uw).max(0).min(10000) as f32, 0.0);
                        }
                    }
                    uw as f32
                } else {
                    self.day_uw
                }
            }
            ChartLevel::Month => {
                if let Some(num) = self.full_months {
                    let uw = (size.width as i32 / (num + 3)).max(1).min(50);
                    if let Some(s) = selected {
                        *translation = Vector::new(
                            ((s.end.max(1) - 1) * uw as usize).max(0).min(10000) as f32,
                            0.0,
                        )
                    } else {
                        if let Some(last) = last_month {
                            *translation =
                                Vector::new(((last - 1) * uw).max(0).min(10000) as f32, 0.0);
                        }
                    }
                    uw as f32
                } else {
                    self.month_uw
                }
            }
        }
    }
}
// 用来控制当前的单位
#[derive(Debug, Clone, Copy)]
pub enum ChartLevel {
    Day,
    Month,
}
// 存放一些用来控制风格的数据
pub struct Style {
    fill_color: Color,
    font_size: f32,
    big_font_size: f32,
}
// 用来代表选取的范围
// 每个值都代表和基准日/月的距离
#[derive(Debug, Clone, Copy)]
pub struct Selected {
    start: usize,
    end: usize,
}
// 同时给selected实现一些方便后续调用的方法
impl Selected {
    // into到Date
    pub fn get_months_range(&self, base_month: &MonthView) -> (Date, Date) {
        (
            get_offset_date(self.start, 31, base_month),
            get_offset_date(self.end, 31, base_month),
        )
    }
    pub fn get_days_range(&self, base_day: Date) -> (Date, Date) {
        (
            base_day - Duration::days(self.start as i64 + 1),
            base_day - Duration::days(self.end as i64 + 1),
        )
    }
    pub fn date_into_days_selected(&mut self, start: Date, end: Date, base_day: Date) {
        self.start = (base_day - start).whole_days().max(1) as usize - 1;
        self.end = (base_day - end).whole_days().max(1) as usize - 1;
    }
    pub fn date_into_months_selected(
        &mut self,
        start: Date,
        end: Date,
        base_month: &MonthView,
        base_day: Date,
    ) {
        let ds = Selected {
            start: (base_day - start).whole_days().max(1) as usize - 1,
            end: (base_day - end).whole_days().max(1) as usize - 1,
        };
        *self = ds.days_to_months(base_day, base_month);
    }
    // 从天为单位转换到月为单位
    pub fn days_to_months(self, base_day: Date, base_month: &MonthView) -> Self {
        let (start, end) = self.get_days_range(base_day);
        let start = months_num(&MonthView::from_date(start, 0), base_month);
        let end = months_num(&MonthView::from_date(end, 0), base_month);
        Self {
            start: start.max(1) as usize - 1,
            end: end.max(1) as usize - 1,
        }
    }
    // 从月为单位转换到天为单位
    pub fn months_to_days(self, base_day: Date, base_month: &MonthView) -> Self {
        let (start, end) = self.get_months_range(base_month);
        let start = (base_day - start).whole_days() as usize;
        let end = (base_day - end).whole_days() as usize;
        Self {
            start: start.max(1) - 1,
            end: end.max(1) - 1,
        }
    }
    // 最重要的绘制函数
    // 其中frame在传入前需要应用translation
    pub fn draw(&self, frame: &mut Frame, uw: f32) {
        let Size { width, height } = frame.size();
        if self.start == self.end {
            // 相等的情况即为指定单个单位
            frame.fill_rectangle(
                Point::new(width - self.start as f32 * uw - uw, 0.0),
                Size::new(uw, height),
                Fill::from(Color::from_rgba(0.8, 0.0, 0.0, 0.3)),
            );
        } else {
            let rw = (self.start as f32 - self.end as f32) * uw;
            frame.fill_rectangle(
                Point::new(width - self.start as f32 * uw, 0.0),
                Size::new(rw, height),
                Fill::from(Color::from_rgba(0.8, 0.0, 0.0, 0.3)),
            );
        }
    }
}
// 用来获取给定月偏移量（即与基准月的距离）的实际Date
[inline]
fn get_offset_date(mut offset: usize, mut day: u8, base_month: &MonthView) -> Date {
    let MonthView {
        mut month,
        mut year,
        ..
    } = base_month;

    year -= offset as i32 / 12;
    offset %= 12;

    if offset != 0 {
        let mut m_num = month_to_num(month) - offset as i32;
        if m_num <= 0 {
            m_num += 12;
            year -= 1;
        }
        month = match m_num {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => unreachable!(),
        };
    }

    let mut res = Date::from_calendar_date(year, month, day);

    while res.is_err() && day > 0 {
        day -= 1;
        res = Date::from_calendar_date(year, month, day);
    }

    res.unwrap()
}
```
定义了selected之后，我们需要在`Program`trait里调用其draw方法：
```diff
    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
+       let size = bounds.size();
+       let mut translation = self.translation;
+       let uw = self.init(
+           size,
+           &mut translation,
+           self.last_day,
+           self.last_month,
+           &self.selected,
+       );
        let mut res = vec![];
+// 主要是添加了draw方法的调用，值得注意的是没有直接构建frame，
+// 而是采用cache的draw方法获取frame
+       if let Some(selected) = self.selected.map(|selected| {
+           self.selected_cache.draw(size, |frame| {
+               frame.translate(translation);
+               selected.draw(frame, uw)
+           })
+       }) {
+           res.push(selected);
+       }

        if let Some(pending) = self.pending.map(|pending| {
            let mut frame = Frame::new(size);
            if let Some(cursor_position) = cursor.position_in(&bounds) {
                let top_left = Point::new(
                    cursor_position.x.min(pending.x),
                    cursor_position.y.min(pending.y),
                );
                let size = Size::new(
                    (cursor_position.x - pending.x).abs(),
                    (cursor_position.y - pending.y).abs(),
                );
                frame.fill_rectangle(
                    top_left,
                    size,
                    Fill::from(Color::from_rgba(0.1, 0.5, 0.8, 0.2)),
                );
            }
            frame.into_geometry()
        }) {
            res.push(pending);
        }
        res
    }
```
绘制好了选框之后，我们不着急预览，在预览之前先绘制days：
```rust
impl Day {
    pub fn draw_all_day(
        days: &[Day],
        frame: &mut Frame,
        max_count: i64,
        fill_color: Color,
        font_size: f32,
        big_font_size: f32,
        uw: f32,
        translation: f32,
        base_day: Date,
        skip_text: bool,
    ) {
        // 绘制文本
        let Size { width, height } = frame.size();
        let num = (width / uw).max(0.0).min(10000.0) as i64;
        let overflow_num = (translation / uw).max(0.0).min(10000.0) as i64;
        // 计算出当前需要绘制的最右边的日期
        let rightmost_day = base_day - Duration::days(overflow_num + 1);

        let mut date_pointer = rightmost_day;
        let day_text_y = height - font_size;
        // 用来居中文本
        let gt9_offset = 0.5 * (uw - font_size);
        let lt9_offset = 0.5 * (uw - 0.5 * font_size);

        for idx in 1..=num {
            let day = date_pointer.day();
            let x = width - idx as f32 * uw - translation;
            if !skip_text {
                // 绘制年份文本
                if idx == num || (date_pointer.month() == Month::January && date_pointer.day() == 1)
                {
                    let year = canvas::Text {
                        content: date_pointer.year().to_string(),
                        position: Point::new(x, 0.0),
                        size: big_font_size,
                        ..Default::default()
                    };
                    frame.fill_text(year);
                }
                // 绘制月份文本
                if day == 1 || idx == num {
                    let month = canvas::Text {
                        content: date_pointer.month().to_string(),
                        position: Point::new(x, big_font_size),
                        size: big_font_size,
                        ..Default::default()
                    };
                    frame.fill_text(month);
                }
                // 绘制天份文本
                if uw >= font_size * 1.8 {
                    let day_text_offset = if day > 9 { gt9_offset } else { lt9_offset };
                    let position = Point::new(x + day_text_offset, day_text_y);
                    let day = canvas::Text {
                        content: day.to_string(),
                        position,
                        size: font_size,
                        ..Default::default()
                    };
                    frame.fill_text(day);
                }
            }
            // 绘制好了这一天的，就绘制上一天
            if let Some(pd) = date_pointer.previous_day() {
                date_pointer = pd;
            }
        }
        // 绘制方块
        let uh = height / max_count as f32;
        for day in days {
            // 日期大于最右侧时停止绘制
            if day.date > rightmost_day {
                break;
            }
            // 日期小于最左侧时跳过绘制
            if day.date < date_pointer {
                continue;
            }

            let num = (base_day - day.date).whole_days().max(0).min(10000) as f32;
            let x = width - num * uw;
            let dh = uh * day.count as f32;
            // 绘制count文本
            if uw >= font_size * 1.8 && (height - dh < day_text_y - font_size) {
                let count_offset = if day.count > 9 {
                    gt9_offset
                } else {
                    lt9_offset
                };
                let count = canvas::Text {
                    content: day.count.to_string(),
                    position: Point::new(x + count_offset, height - dh),
                    size: font_size,
                    ..Default::default()
                };
                frame.fill_text(count);
            }
            // 绘制数据块
            frame.fill_rectangle(
                Point::new(x, height - dh),
                Size::new(uw, dh),
                Fill::from(fill_color),
            );
        }
    }
}
```
同样，绘制月份：
```rust
impl MonthView {
    // 和day类似，不同的是month没有提供previous的方法，需要我们自己写
    pub fn draw_all_month(
        months: &[MonthView],
        frame: &mut Frame,
        max_count: i64,
        fill_color: Color,
        font_size: f32,
        big_font_size: f32,
        uw: f32,
        translation: f32,
        base_month: &MonthView,
    ) {
        // 绘制文本
        let Size { width, height } = frame.size();
        let num = (width / uw).min(10000.0).max(0.0) as i64;
        let MonthView {
            mut month,
            mut year,
            ..
        } = base_month;
        let mut offset = (translation / uw) as i64;

        year -= offset as i32 / 12;
        offset = offset % 12;
    
        if offset != 0 {
            let mut m_num = month_to_num(month) - offset as i32;
            if m_num <= 0 {
                m_num += 12;
                year -= 1;
            }
            month = match m_num {
                1 => Month::January,
                2 => Month::February,
                3 => Month::March,
                4 => Month::April,
                5 => Month::May,
                6 => Month::June,
                7 => Month::July,
                8 => Month::August,
                9 => Month::September,
                10 => Month::October,
                11 => Month::November,
                12 => Month::December,
                _ => unreachable!(),
            };
        }

        let rightmost_month = (year, month);

        let mut date_pointer = rightmost_month;

        let month_text_y = height - font_size;
        let gt9_offset = 0.5 * (uw - font_size);
        let lt9_offset = 0.5 * (uw - 0.5 * font_size);

        for idx in 1..=num {
            let month = date_pointer;
            let x = width - idx as f32 * uw - translation;
            if idx == num || date_pointer.1 == Month::January {
                let year = canvas::Text {
                    content: date_pointer.0.to_string(),
                    position: Point::new(x, 0.0),
                    size: big_font_size,
                    ..Default::default()
                };
                frame.fill_text(year);
            }

            let content = month_to_num(month.1);
            if uw >= font_size * 1.8 {
                let month_text_offset = if content > 9 { gt9_offset } else { lt9_offset };
                let position = Point::new(x + month_text_offset, month_text_y);
    
                let month = canvas::Text {
                    content: content.to_string(),
                    position,
                    size: font_size,
                    ..Default::default()
                };
    
                frame.fill_text(month);
            }

            if let Month::January = date_pointer.1 {
                date_pointer = (date_pointer.0 - 1, Month::December);
            } else {
                date_pointer.1 = date_pointer.1.previous();
            }
        }
        // 绘制方块
        let uh = height / max_count as f32;
        let rm_num = month_to_num(rightmost_month.1);
        let dm_num = month_to_num(date_pointer.1);
        let MonthView {
            month: tm,
            year: ty,
            ..
        } = base_month;
        let tm_num = month_to_num(*tm);
        for month in months {
            let cy = month.year;
            let cm = month.month;
            let cm_num = month_to_num(cm);
            if cy > rightmost_month.0 || (cy == rightmost_month.0 && cm_num > rm_num) {
                break;
            }

            if (cy == date_pointer.0 && cm_num < dm_num) || (cy < date_pointer.0) {
                continue;
            }
            let num = (*ty - cy - 1) * 12 + 13 + tm_num - cm_num;
            let x = width - num as f32 * uw;

            let dh = uh * month.count as f32;
            let content = month.count.to_string();
            if font_size * content.len() as f32 <= uw && height - dh < month_text_y - font_size {
                let count_offset = if month.count > 9 {
                    gt9_offset
                } else {
                    lt9_offset
                };
                let count = canvas::Text {
                    content,
                    position: Point::new(x + count_offset, height - dh),
                    size: font_size,
                    ..Default::default()
                };
                frame.fill_text(count);
            }
            frame.fill_rectangle(
                Point::new(x, height - dh),
                Size::new(uw, dh),
                Fill::from(fill_color),
            );
        }
    }
}
```
> 注意：在绘制文本时，有部分过滤使用了字体大小和单位宽度作比较，这是为了防止文本绘制时全部挤压到一块，可以按照自己喜好来设置相关参数。另外，此前我们提到的文本绘制永远在上层，会导致一些意想不到的问题，比如超出了绘图框，仍有文字被绘制到上面，解决的方法就是在绘制的时候，指定特定的位置范围内绘制，超出了不进行绘制即可。

有了两个绘制方法，我们在`Program`trait里调用即可：
```rust
    fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
        let size = bounds.size();
        let mut translation = self.translation;
        let uw = self.init(
            size,
            &mut translation,
            self.last_day,
            self.last_month,
            &self.selected,
        );
        let mut res = vec![];
        // 实际上增加的仅仅是这部分
        match &self.level {
            ChartLevel::Day => {
                let days = &self.days[..];
                let rects = self.cache.draw(size, |frame| {
                    frame.translate(translation);
                    Day::draw_all_day(
                        days,
                        frame,
                        self.maximal_day_count,
                        self.style.fill_color,
                        self.style.font_size,
                        self.style.big_font_size,
                        uw,
                        translation.x,
                        self.base_day,
                        false,
                    );
                });
                res.push(rects);
            }
            ChartLevel::Month => {
                let months = &self.months[..];

                let rects = self.cache.draw(size, |frame| {
                    frame.translate(translation);
                    MonthView::draw_all_month(
                        months,
                        frame,
                        self.maximal_month_count,
                        self.style.fill_color,
                        self.style.font_size,
                        self.style.big_font_size,
                        uw,
                        translation.x,
                        self.base_month,
                    );
                });

                let mut sub_frame = Frame::new(Size {
                    width: size.width / 2.0,
                    height: size.height / 2.0,
                });

                if let Some(s) = self.selected {
                    let day_range = s.months_to_days(self.base_day, self.base_month);
                    let day_uw = (size.width
                        / (day_range.start as f32 - day_range.end as f32 + 3.0) as f32)
                        .max(1.0)
                        .min(50.0);
                    let day_translation = Vector::new((day_range.end - 1) as f32 * day_uw, 0.0);
                    sub_frame.translate(day_translation);
                    let days = &self.days[..];
                    Day::draw_all_day(
                        days,
                        &mut sub_frame,
                        self.maximal_day_count,
                        self.style.fill_color,
                        self.style.font_size,
                        self.style.big_font_size,
                        day_uw,
                        day_translation.x,
                        self.base_day,
                        true,
                    );
                }
                res.push(rects);
                res.push(sub_frame.into_geometry());
            }
        };
        // 以下都是和之前一致的了，需要注意的是，在绘制月份的时候，除了月份本身外，
        // 我们还绘制了月份内选择范围之后，天份的预览，作为sub_frame进行绘制。
        if let Some(selected) = self.selected.map(|selected| {
            self.selected_cache.draw(size, |frame| {
                frame.translate(translation);
                selected.draw(frame, uw)
            })
        }) {
            res.push(selected);
        }

        if let Some(pending) = self.pending.map(|pending| {
            let mut frame = Frame::new(size);
            if let Some(cursor_position) = cursor.position_in(&bounds) {
                let top_left = Point::new(
                    cursor_position.x.min(pending.x),
                    cursor_position.y.min(pending.y),
                );
                let size = Size::new(
                    (cursor_position.x - pending.x).abs(),
                    (cursor_position.y - pending.y).abs(),
                );
                frame.fill_rectangle(
                    top_left,
                    size,
                    Fill::from(Color::from_rgba(0.1, 0.5, 0.8, 0.2)),
                );
            }
            frame.into_geometry()
        }) {
            res.push(pending);
        }

        res
    }
```
到目前为止，绘制的部分基本结束了，紧接着就可以写事件更新了，之前的事件更新很简单，只是考虑了选择的框绘制，接下来我们除了选择框绘制之外，还需要进行一些处理，有了以上的一些方法的帮助，做处理会容易很多：
```rust
    fn update(
        &mut self,
        event: canvas::Event,
        bounds: Rectangle,
        cursor: Cursor,
    ) -> (canvas::event::Status, Option<ChartMsg>) {
        if let Some(cursor_position) = cursor.position_in(&bounds) {
            match event {
                canvas::Event::Mouse(me) => match me {
                    mouse::Event::ButtonReleased(mouse::Button::Right) => {
                        println!("🖱: {:?}", cursor_position);
                        // 鼠标右键点击时，触发清理range的事件，返回的事件需要我们在上层处理
                        return (canvas::event::Status::Captured, Some(ChartMsg::ClearRange));
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Left) => {
                        // 鼠标左键按下时，替换pending的值
                        self.pending.replace(cursor_position);
                    }
                    mouse::Event::ButtonPressed(mouse::Button::Middle) => {
                        // 鼠标中间按下时，触发进入range的事件，这个事件将会在天和月之间进行切换
                        if self.selected.is_some() {
                            return (canvas::event::Status::Captured, Some(ChartMsg::EnterRange));
                        }
                    }
                    mouse::Event::ButtonReleased(mouse::Button::Left) => {
                        // 除了此前的take之外，我们还额外做了一些处理
                        let pending = self.pending.take();
                        let size = bounds.size();
                        let width = size.width;
                        // 最重要的就是返回一个selected，所以我们需要回去当前的uw，
                        // 通过uw来确定start和end的值
                        if let Some(pd) = pending {
                            let mut translation = self.translation;
                            let uw = self.init(
                                size,
                                &mut translation,
                                self.last_day,
                                self.last_month,
                                &self.selected,
                            );
                            let fix = translation.x;

                            let a = (width - pd.x + fix) / uw;
                            let b = (width - cursor_position.x + fix) / uw;
                            let (start, end) = if a > b {
                                (a.ceil() as usize, b.floor() as usize)
                            } else {
                                (b.ceil() as usize, a.floor() as usize)
                            };

                            return (
                                canvas::event::Status::Captured,
                                Some(ChartMsg::FilterSearch(Selected { start, end })),
                            );
                        }
                    }
                    mouse::Event::WheelScrolled { delta } => match delta {
                        // 我们仅考虑竖直方向上的滑轮滚动
                        mouse::ScrollDelta::Lines { y,.. } => {
                            return (canvas::event::Status::Captured, Some(ChartMsg::Scroll(y)));
                        }
                        mouse::ScrollDelta::Pixels { y,.. } => {
                            return (canvas::event::Status::Captured, Some(ChartMsg::Scroll(y)));
                        }
                    },
                    _ => {}
                },
                canvas::Event::Keyboard(_) => {}
            };
        }
        (canvas::event::Status::Ignored, None)
    }
```
至此，canvas的部分基本都实现了，事件处理还需要上层来配合：
```rust
// 补充好用来包裹DateChart的Chart数据结构
#[derive(Debug)]
pub struct Chart {
    pub cache: Cache,
    pub selected_cache: Cache,
    pub selected: Option<Selected>,
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
    pub max_day_count: i64,
    pub max_month_count: i64,
    pub day_uw: f32,
    pub month_uw: f32,
    pub level: ChartLevel,
    pub translation: Vector,
    pub base_day: Date,
    pub base_month: MonthView,
}

impl Default for Chart {
    fn default() -> Self {
        Self::new()
    }
}

impl Chart {
    pub fn uw(&self) -> f32 {
        match self.level {
            ChartLevel::Day => self.day_uw,
            ChartLevel::Month => self.month_uw,
        }
    }
    pub fn new() -> Self {
        Self {
            cache: Cache::new(),
            selected_cache: Cache::new(),
            selected: None,
            max_day_count: 10,
            max_month_count: 10,
            level: ChartLevel::Day,
            translation: Vector::new(0.0, 0.0),
            day_uw: 17.0,
            month_uw: 17.0,
            full_days: None,
            full_months: None,
            last_day: None,
            last_month: None,
            base_day: base_day(),
            base_month: base_month(),
        }
    }
    pub fn chart_view<'a>(
        &'a self,
        days: &'a [Day],
        months: &'a [MonthView],
    ) -> Element<'a, ChartMsg> {
        Canvas::new(DateChart {
            days,
            months,
            cache: &self.cache,
            selected_cache: &self.selected_cache,
            selected: self.selected.as_ref(),
            pending: None,
            maximal_day_count: self.max_day_count,
            maximal_month_count: self.max_month_count,
            style: Style {
                fill_color: Color::from_rgba(0.0, 0.0, 0.8, 0.5),
                font_size: self.uw() * 2.0 / 3.0,
                big_font_size: 17.0,
            },
            level: self.level,
            translation: self.translation,
            day_uw: self.day_uw,
            month_uw: self.month_uw,
            base_day: self.base_day,
            base_month: &self.base_month,
            full_days: self.full_days,
            full_months: self.full_months,
            last_day: self.last_day,
            last_month: self.last_month,
        })
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
    }
}
// 声明一个用来包裹Chart的数据结构
#[derive(Debug)]
pub struct DateView {
    pub days: Vec<Day>,
    pub months: Vec<MonthView>,
    pub is_show: bool,
    pub chart: Chart,
    pub full_days: Option<i64>,
    pub full_months: Option<i32>,
    pub last_day: Option<i64>,
    pub last_month: Option<i32>,
    close: button::State,
    full_or_adjustable: button::State,
    max_or_min: button::State,
    day_or_month: button::State,
    uw_input: number_input::State,
    pub is_full: bool,
}

impl Default for DateView {
    fn default() -> Self {
        Self {
            is_show: true,
            is_full: true,
            chart: Chart::new(),
            close: button::State::new(),
            full_or_adjustable: button::State::new(),
            max_or_min: button::State::new(),
            day_or_month: button::State::new(),
            uw_input: number_input::State::new(),
            days: Vec::new(),
            months: Vec::new(),
            full_days: None,
            full_months: None,
            last_day: None,
            last_month: None,
        }
    }
}
// 后续方法实现都很常规，基本都是view，update的套路
impl DateView {
    pub fn view(&mut self, theme: Theme) -> Element<Message> {
        let DateView {
            close,
            full_or_adjustable,
            max_or_min,
            day_or_month,
            chart,
            days,
            months,
            uw_input,
            ..
        } = self;
        let close_button = Button::new(close, Text::new("X")).on_press(Message::Close);
        let minimize_or_maximize_button = Button::new(
            max_or_min,
            Text::new(if self.is_show { "min" } else { "max" }),
        )
        .on_press(Message::MaxOrMin);

        let mut ctrl_row = Row::new();

        if !self.is_full {
            // 记得，要在Cargo.toml内开启numer_input的feature
            let uw_input = iced_aw::NumberInput::new(uw_input, chart.uw(), 30.0, Message::UwChange)
                .min(1.0)
                .step(0.1);
            ctrl_row = ctrl_row.push(uw_input);
        }
        ctrl_row = ctrl_row.push(style::horizontal_rule());
        if self.is_show {
            let full_or_adjustable_button = Button::new(
                full_or_adjustable,
                Text::new(if self.is_full { "adjustable" } else { "full" }),
            )
            .on_press(Message::FullOrAdjustable);
            let day_or_month_button =
                Button::new(day_or_month, Text::new("changing view")).on_press(Message::DayOrMonth);
            ctrl_row = ctrl_row
                .push(day_or_month_button)
                .push(full_or_adjustable_button);
        }
        ctrl_row = ctrl_row
            .push(minimize_or_maximize_button)
            .push(close_button);

        let mut content = Column::new();
        content = content.push(ctrl_row);
        if self.is_show {
            let chart = chart.chart_view(&*days, &*months).map(Message::ChartMsg);
            content = content.push(chart);
        }
        Container::new(content).into()
    }
    pub fn clear_cache_and_convert_selected_range(&mut self) {
        if let Some(s) = self.chart.selected {
            match self.chart.level {
                crate::days::ChartLevel::Day => self
                    .chart
                    .selected
                    .replace(s.months_to_days(self.chart.base_day, &self.chart.base_month)),
                crate::days::ChartLevel::Month => self
                    .chart
                    .selected
                    .replace(s.days_to_months(self.chart.base_day, &self.chart.base_month)),
            };
        }
        self.clear_cahce();
    }
    // 手动调用clear可以触发刷新绘制的效果
    pub fn clear_cahce(&mut self) {
        self.chart.cache.clear();
        self.chart.selected_cache.clear();
    }
    pub fn get_range(&mut self, selected: Selected) -> (Date, Date) {
        self.chart.selected.replace(selected);
        match self.chart.level {
            ChartLevel::Day => selected.get_days_range(self.chart.base_day),
            ChartLevel::Month => selected.get_months_range(&self.chart.base_month),
        }
    }
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Close => {
                // 上层处理
            }
            Message::MaxOrMin => {
                self.is_show = !self.is_show;
            }
            Message::FullOrAdjustable => {
                self.is_full = !self.is_full;
                if self.is_full {
                    self.chart.full_days = self.full_days;
                    self.chart.full_months = self.full_months;
                } else {
                    self.chart.full_days.take();
                    self.chart.full_months.take();
                    self.align();
                }
                self.clear_cahce();
            }
            Message::ChartMsg(cm) => match cm {
                ChartMsg::EnterRange => {
                    self.day_or_month();
                    self.clear_cache_and_convert_selected_range();
                }
                ChartMsg::ClearRange => {
                    self.chart.selected.take();
                    self.clear_cahce();
                }
                ChartMsg::Scroll(x) => {
                    // 滑轮向下是负数
                    if self.is_full {
                        if let Some(s) = self.chart.selected {
                            if x > 0.0 {
                                // 基本上都是通过缩减或增大绘制需要的范围来控制滑轮缩放
                                // 目前滑轮缩放和鼠标的位置并不是绑定的，从效果上来说与鼠标
                                // 位置绑定效果要好一些，会在未来考虑加入
                                let val = s.start - s.end;
                                match self.chart.level {
                                    ChartLevel::Day => {
                                        self.chart.full_days = Some(val as i64);
                                        self.chart.last_day = Some(s.end as i64);
                                    }
                                    ChartLevel::Month => {
                                        self.chart.full_months = Some(val as i32);
                                        self.chart.last_month = Some(s.end as i32);
                                    }
                                }
                            } else {
                                // 当full开启的时候，需要做一些对齐操作
                                // 对齐也是通过给last、full这两个变量进行赋值
                                match self.chart.level {
                                    ChartLevel::Day => {
                                        let fd = self.full_days.and_then(|original| {
                                            self.chart.full_days.map(|now| {
                                                let new = now - x.floor() as i64;
                                                if new > original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        let ld = self.last_day.and_then(|original| {
                                            self.chart.last_day.map(|now| {
                                                let new = now + x.floor() as i64;
                                                if new < original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        self.chart.full_days = fd;
                                        self.chart.last_day = ld;
                                    }
                                    ChartLevel::Month => {
                                        let fm = self.full_months.and_then(|original| {
                                            self.chart.full_months.map(|now| {
                                                let new = now - x.floor() as i32;
                                                if new > original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        let lm = self.last_month.and_then(|original| {
                                            self.chart.last_month.map(|now| {
                                                let new = now + x.floor() as i32;
                                                if new < original {
                                                    original
                                                } else {
                                                    new
                                                }
                                            })
                                        });
                                        self.chart.full_months = fm;
                                        self.chart.last_month = lm;
                                    }
                                }
                            }
                        }
                    } else {
                        // 如果不是full模式，仅仅需要对translation赋值即可
                        let new_x = x * self.chart.uw() + self.chart.translation.x;
                        if new_x < 0.0 {
                            self.chart.translation = Vector::new(0.0, 0.0);
                        } else {
                            self.chart.translation = Vector::new(new_x, 0.0);
                        }
                    }
                    // 在赋值后需要清理cache用以刷新绘制窗口
                    self.chart.cache.clear();
                    self.chart.selected_cache.clear();
                }
                _ => {}
            },
            Message::DayOrMonth => {
                // 上层处理
            }
            Message::UwChange(uw) => {
                self.set_uw(uw);
                self.clear_cahce();
                self.align();
            }
        }
    }
    pub fn day_or_month(&mut self) {
        match self.chart.level {
            ChartLevel::Day => {
                self.chart.level = ChartLevel::Month;
            }
            ChartLevel::Month => {
                self.chart.level = ChartLevel::Day;
            }
        }
        self.align();
    }
    pub fn align(&mut self) {
        match self.chart.level {
            ChartLevel::Day => {
                self.chart.last_day.map(|last| {
                    self.chart.translation =
                        Vector::new((last as f32 - 1.0) * self.chart.day_uw, 0.0)
                });
            }
            ChartLevel::Month => {
                self.chart.last_month.map(|last| {
                    self.chart.translation =
                        Vector::new((last - 1) as f32 * self.chart.month_uw, 0.0)
                });
            }
        }
    }
    pub fn set_uw(&mut self, uw: f32) {
        match self.chart.level {
            ChartLevel::Day => {
                self.chart.day_uw = uw;
            }
            ChartLevel::Month => {
                self.chart.month_uw = uw;
            }
        }
    }
}
```
至此，大部分都实现了，我们还需要做的就是和后端的接洽，以及将当前的控件嵌入到search_page中。
```rust
// 在 lib.rs -> impl Application for LoclaNative 内
    fn update(
        &mut self,
        message: Self::Message,
        clipboard: &mut iced::Clipboard,
    ) -> Command<Self::Message> {
        // 从下方注释开始看即可
        match self {
            LocalNative::Loading => match message {
                Message::Loading(..) => {
                    let conn = Arc::new(Mutex::new(get_sqlite_connection()));
                    let data = Data {
                        search_page: Default::default(),
                        conn,
                        theme: Theme::Light,
                        limit: 10,
                    };

                    *self = LocalNative::Loaded(data);
                    if let LocalNative::Loaded(data) = self {
                        data.search_page
                            .update(search_page::Message::Refresh, data.limit, data.conn.clone())
                            .map(Message::SearchPageMessage)
                    } else {
                        unreachable!()
                    }
                }
                _ => Command::none(),
            },
            LocalNative::Loaded(data) => match message {
                Message::SearchPageMessage(search_page_msg) => match search_page_msg {
                    search_page::Message::Receiver(Some(md)) => {
                        let MiddleDate {
                            tags,
                            notes,
                            count,
                            days,
                        } = md;
                        data.search_page.count = count;
                    // 如果你升级了Rust1.53.0,之前我们需要使用的IntoIter在这里就不再需要了
                        Command::batch([
                            Command::perform(
                                async move {
                                    let mut tags = tags;
                                    tags.sort_by(|a, b| b.count.cmp(&a.count));
                                    tags.into_iter().map(TagView::from).collect()
                                },
                                Message::TagView,
                            ),
                            Command::perform(
                                async move { notes.into_iter().map(NoteView::from).collect() },
                                Message::NoteView,
                            ),
                            {
                                // days的返回是Option（如果你直接修改了后端的返回，
                                // 则不需要用Option来包裹）
                                if let Some(days) = days {
                                    Command::perform(
                                        async move { days::Day::handle_days(days) },
                                        Message::DayView,
                                    )
                                } else {
                                    Command::none()
                                }
                            },
                        ])
                    }
                    msg => data
                        .search_page
                        .update(msg, data.limit, data.conn.clone())
                        .map(Message::SearchPageMessage),
                },
                Message::NoteView(notes) => {
                    data.search_page.notes = notes;
                    Command::none()
                }
                Message::TagView(tags) => {
                    data.search_page.tags = tags;
                    Command::none()
                }
                Message::Loading(..) => Command::none(),
                // 获取到处理好的HandleDays之后，赋值即可。
                Message::DayView(HandleDays {
                    days,
                    months,
                    max_day_count,
                    max_month_count,
                    full_days,
                    full_months,
                    last_day,
                    last_month,
                }) => {
                    data.search_page.days.days = days;
                    data.search_page.days.months = months;
                    data.search_page.days.full_days = full_days;
                    data.search_page.days.full_months = full_months;
                    data.search_page.days.last_day = last_day;
                    data.search_page.days.last_month = last_month;
                    data.search_page.days.chart.last_day = last_day;
                    data.search_page.days.chart.last_month = last_month;
                    data.search_page.days.chart.max_day_count = max_day_count;
                    data.search_page.days.chart.max_month_count = max_month_count;
                    if data.search_page.range.is_none() {
                        data.search_page.days.align();
                    }
                    data.search_page.days.clear_cahce();
                    Command::none()
                }
            },
        }
    }
```
后端接洽比较容易，嵌入到search_page也很简单：
```rust
// 此前的search没有考虑过滤情况，现在我们加上
fn search(
    conn: Arc<Mutex<Connection>>,
    query: String,
    limit: u32,
    offset: u32,
    range: Option<(time::Date, time::Date)>,
) -> Command<Message> {
    if let Some((from, to)) = range {
        Command::perform(
            MiddleDate::from_filter(conn, query, limit, offset, from, to),
            Message::Receiver,
        )
    } else {
        Command::perform(
            MiddleDate::from_select(conn, query, limit, offset),
            Message::Receiver,
        )
    }
}
#[derive(Default)]
pub struct SearchPage {
    pub notes: Vec<NoteView>,
    pub tags: Vec<TagView>,
    // 主要添加了days和range两个字段
    pub days: DateView,
    pub range: Option<(time::Date, time::Date)>,
    search_value: String,
    pub offset: u32,
    pub count: u32,
    input_state: text_input::State,
    clear_button: button::State,
    refresh_button: button::State,
    notes_scrollable: scrollable::State,
    tags_scrollable: scrollable::State,
    next_button: button::State,
    pre_button: button::State,
}
// 在 impl SearchPage 内
    pub fn view(&mut self, theme: Theme, limit: u32) -> Element<Message> {
        let Self {
            notes,
            tags,
            days,
            search_value,
            input_state,
            clear_button,
            refresh_button,
            notes_scrollable,
            tags_scrollable,
            next_button,
            pre_button,
            ..
        } = self;
        let mut search_bar = Row::new().push(
            TextInput::new(
                input_state,
                "Type your search...",
                &search_value,
                Message::SearchInput,
            )
            .on_submit(Message::Search),
        );
        if !self.search_value.is_empty() || self.range.is_some() {
            search_bar =
                search_bar.push(Button::new(clear_button, Text::new("X")).on_press(Message::Clear));
        }
        let refresh_button = Button::new(refresh_button, Text::new("O")).on_press(Message::Refresh);
        search_bar = search_bar.push(refresh_button);
        let tags = Scrollable::new(tags_scrollable)
            .push(Container::new(tags.iter_mut().fold(
                iced_aw::Wrap::new().spacing(5).push(Text::new("tags:")),
                |tags, tag| tags.push(tag.view(theme).map(Message::TagMessage)),
            )))
            .width(iced::Length::FillPortion(2));
        let is_show = days.is_show;
        // 主要是添加了days
        let days = Container::new(days.view(theme).map(Message::DayMessage)).height({
            if is_show {
                iced::Length::FillPortion(4)
            } else {
                iced::Length::Shrink
            }
        });
        // 同时为了保证高度比，将页面控制栏固定在了底部，不再同note数挂钩
        let next_button = Button::new(next_button, Text::new("->")).on_press(Message::NextPage);
        let pre_button = Button::new(pre_button, Text::new("<-")).on_press(Message::PrePage);
        let page_info = Text::new(format!(
            "{}-{}/{}",
            self.offset + 1,
            (self.offset + limit).min(self.count),
            self.count
        ));
        let page_ctrl = Row::new()
            .push(style::horizontal_rule())
            .push(pre_button)
            .push(page_info)
            .push(next_button)
            .push(style::horizontal_rule());
        let note_page = if self.count > 0 {
            let notes = Container::new(notes.iter_mut().enumerate().fold(
                Scrollable::new(notes_scrollable).padding(10),
                |notes, (idx, note_view)| {
                    notes.push(
                        note_view
                            .view(theme)
                            .map(move |note_msg| Message::NoteMessage(note_msg, idx)),
                    )
                },
            ))
            .height(iced::Length::FillPortion(8));

            Column::new()
                .push(search_bar)
                .push(days)
                .push(notes)
                .push(page_ctrl)
        } else {
            let tip = if self.search_value.is_empty() && self.range.is_none() {
                "Not Created"
            } else {
                "Not Founded"
            };
            let tip = Container::new(
                Column::new()
                    .push(style::vertical_rule())
                    .push(Text::new(tip).size(50))
                    .push(style::vertical_rule()),
            )
            .height(iced::Length::FillPortion(8));
            Column::new()
                .push(search_bar)
                .push(days)
                .push(tip)
                .push(page_ctrl)
        }
        .align_items(iced::Align::Center)
        .width(iced::Length::FillPortion(8));
        Container::new(Row::new().push(note_page).push(tags)).into()
    }
    // 主要是增加了days的事件处理
    pub fn update(
        &mut self,
        message: Message,
        limit: u32,
        conn: Arc<Mutex<Connection>>,
    ) -> Command<Message> {
        match message {
            Message::Search => search(
                conn,
                self.search_value.to_owned(),
                limit,
                self.offset,
                self.range,
            ),
            Message::SearchInput(search_value) => {
                self.search_value = search_value;
                search(
                    conn,
                    self.search_value.to_owned(),
                    limit,
                    self.offset,
                    self.range,
                )
            }
            Message::Clear => {
                self.search_value.clear();
                self.range.take();
                self.days.clear_cache_and_convert_selected_range();
                search(
                    conn,
                    self.search_value.to_owned(),
                    limit,
                    self.offset,
                    self.range,
                )
            }
            Message::Refresh => search(
                conn,
                self.search_value.to_owned(),
                limit,
                self.offset,
                self.range,
            ),
            Message::NextPage => {
                let current_count = self.offset + limit;
                if current_count < self.count {
                    self.offset = current_count;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                } else {
                    Command::none()
                }
            }
            Message::PrePage => {
                if self.offset >= limit {
                    self.offset -= limit;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                } else if self.offset != 0 {
                    self.offset = 0;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                } else {
                    Command::none()
                }
            }
            Message::NoteMessage(msg, idx) => match msg {
                crate::note::Message::Delete(rowid) => Command::perform(
                    MiddleDate::delete(
                        conn,
                        self.search_value.to_string(),
                        limit,
                        self.offset,
                        rowid,
                    ),
                    Message::Receiver,
                ),
                crate::note::Message::Search(s) => {
                    self.search_value = s;
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                }
                msg => {
                    if let Some(note) = self.notes.get_mut(idx) {
                        note.update(msg)
                    };
                    Command::none()
                }
            },
            Message::Receiver(_) => Command::none(),
            Message::TagMessage(tag_msg) => {
                match tag_msg {
                    crate::tags::Message::Search(text) => self.search_value = text,
                }
                search(
                    conn,
                    self.search_value.to_owned(),
                    limit,
                    self.offset,
                    self.range,
                )
            }
            // 增加的事件处理基本在这
            Message::DayMessage(dm) => match dm {
                crate::days::Message::DayOrMonth => {
                    self.days.day_or_month();
                    self.days.clear_cache_and_convert_selected_range();
                    Command::none()
                }
                crate::days::Message::Close => {
                    // TODO：在添加config时处理
                    Command::none()
                }
                crate::days::Message::ChartMsg(crate::days::ChartMsg::ClearRange) => {
                    self.days.clear_cahce();
                    self.days.chart.selected.take();
                    self.range.take();
                    if self.days.is_full {
                        // 同步full和last
                        match self.days.chart.level {
                            crate::days::ChartLevel::Day => {
                                self.days.chart.full_days = self.days.full_days;
                                self.days.chart.last_day = self.days.last_day;
                            }
                            crate::days::ChartLevel::Month => {
                                self.days.chart.full_months = self.days.full_months;
                                self.days.chart.last_month = self.days.last_month;
                            }
                        }
                    }
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                }
                crate::days::Message::ChartMsg(crate::days::ChartMsg::FilterSearch(selected)) => {
                    self.days.clear_cahce();
                    let range = self.days.get_range(selected);
                    self.range = Some(range);
                    search(
                        conn,
                        self.search_value.to_owned(),
                        limit,
                        self.offset,
                        self.range,
                    )
                }
                dm => {
                    self.days.update(dm);
                    Command::none()
                }
            },
        }
    }
```
至此，实现上基本结束。相对于之前的几章，本章在写的时候，重点放到了如何实现Canvas控件，诸如此前的那些固定的iced的写法，都比较简略的跳过了，没有很详细的记录是如何写成的。

到了这一步，基本可以运行最终结果了：`cargo run --bin ln`

![2](img/tutorial/6-02.gif)

