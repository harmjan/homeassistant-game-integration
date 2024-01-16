use opencv::core::{Point, Scalar};
use opencv::highgui::{destroy_all_windows, imshow, named_window_def, wait_key};
use opencv::imgproc::{put_text_def, FONT_HERSHEY_SIMPLEX};
use opencv::prelude::*;
use opencv::videoio::VideoCapture;

use std::collections::VecDeque;
use std::time::Instant;

fn main() {
    named_window_def("game").expect("Failed to create window");

    let mut cap = VideoCapture::new_def(0).expect("Failed to open webcam");

    let mut frame = Mat::default();
    let mut frame_instants = VecDeque::<Instant>::new();
    frame_instants.push_front(Instant::now());
    loop {
        cap.read(&mut frame).expect("Failed to read frame");
        frame_instants.push_front(Instant::now());
        if frame_instants.len() >= 60 {
            frame_instants.pop_back().unwrap();
        }

        let elapsed = (*frame_instants.front().unwrap() - *frame_instants.back().unwrap())
            / frame_instants.len() as u32;

        let fps_string = format!("{:.2}ms", elapsed.as_micros() as f64 / 1000f64);
        put_text_def(
            &mut frame,
            fps_string.as_str(),
            Point { x: 0, y: 50 },
            FONT_HERSHEY_SIMPLEX,
            1.0f64,
            Scalar::new(255f64, 255f64, 255f64, 128f64),
        )
        .expect("Failed to put text on the screen");

        imshow("game", &frame).expect("Failed to draw frame");

        if wait_key(1).expect("Failed to wait for key") & 0xFF == 'q' as i32 {
            break;
        }
    }

    cap.release().expect("Failed to release video capture");

    destroy_all_windows().expect("Failed to destroy windows");
}
