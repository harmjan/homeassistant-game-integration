use opencv::highgui::{destroy_all_windows, imshow, named_window_def, wait_key};
use opencv::prelude::*;
use opencv::videoio::VideoCapture;

use std::time::Instant;

fn main() {
    named_window_def("game").expect("Failed to create window");

    let mut cap = VideoCapture::new_def(0).expect("Failed to open webcam");

    let mut frame = Mat::default();
    let mut num_frames = 0u64;
    let start = Instant::now();
    loop {
        cap.read(&mut frame).expect("Failed to read frame");

        imshow("game", &frame).expect("Failed to draw frame");

        num_frames += 1;
        let elapsed = (Instant::now() - start) / num_frames as u32;

        // Frame time of about 43ms so about 23fps
        println!("{:?}", elapsed);

        if wait_key(1).expect("Failed to wait for key") & 0xFF == 'q' as i32 {
            break;
        }
    }

    cap.release().expect("Failed to release video capture");

    destroy_all_windows().expect("Failed to destroy windows");
}
