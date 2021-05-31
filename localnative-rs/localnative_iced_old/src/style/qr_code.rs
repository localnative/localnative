// use iced::qr_code;

// pub trait FixedSize {
//     fn fixed_size(&mut self,side_length:f32);
// }

// impl FixedSize for qr_code::QRCode {
//     fn fixed_size(&mut self,side_length:f32) {
//         let cell_size = (side_length/((&*self).width() + 2 * 2) as f32).min(1.0);
//         self.cell_size(
//             cell_size as u16
//         );
//     }
// }
