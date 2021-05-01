use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Day {
    #[serde(rename = "k")]
    pub time: String,
    #[serde(rename = "v")]
    pub count: i32,
}

// impl Day {
//     fn time(&self) -> Date<Local> {
//         let ymd: Vec<&str> = self.time.split('-').into_iter().map(|time| time).collect();

//         Local.ymd_opt(
//             ymd[0].parse().unwrap(),
//             ymd[1].parse().unwrap(),
//             ymd[2].parse().unwrap(),
//         )
//         .unwrap()
//     }

// }

// pub struct DayView {
//     days: Vec<(Date<Local>,i32)>,
//     max_count:i32,
//     state: State,
//     cache:Cache,
//     line_width:f32
// }

// impl Program<Message> for DayView {
//     fn draw(&self, bounds: Rectangle, cursor: Cursor) -> Vec<Geometry> {
//         let size = bounds.size();
//         let len = self.days.len();
//         self.cache.draw(size, |frame| {
//             let x_step = self.time_length().unwrap().num_days_from_ce() as f32 / size.width;
//             let x = Path::line(Point::ORIGIN, Point::new(size.width, 0.0));
//             let y = Path::line(Point::ORIGIN, Point::new(0.0, size.height));
//             let mut xp = 0.0;
//             for (time,count) in self.days.iter() {
//                 let height = count/self.max_count*size.height;
//                 let cube = Path::rectangle(Point::new(xp,height), Size::new(self.line_width, height));
//             }

//         })

//         todo!()
//     }
// }
// impl DayView {
//     fn sort_time(&mut self) {
//         self.days.sort_by(|(a,_),(b,_)|a.cmp(b));
//     }
//     fn time_length(&self) ->Option<Date<Local>> {
//         if let Some((first,_)) = self.days.first() {
//             if let Some((last,_)) = self.days.last() {
//                 let res = last-first;
//                 Some(res)
//             }
//         }
//         None
//     }

// }
// pub enum State {}
// #[derive(Debug, Clone)]
// pub enum Message {
//     Search(String),
// }
