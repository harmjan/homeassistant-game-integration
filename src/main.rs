use opencv::core::{norm2_def, Point, Scalar};
use opencv::highgui::{destroy_all_windows, imshow, named_window_def, wait_key};
use opencv::imgcodecs::imread_def;
use opencv::imgproc::{put_text_def, resize_def, FONT_HERSHEY_SIMPLEX};
use opencv::prelude::*;
use opencv::videoio::VideoCapture;

use std::collections::VecDeque;
use std::time::Instant;

fn main() {
    let mut no_video = imread_def("no-video.png").expect("");

    named_window_def("game").expect("Failed to create window");

    let mut cap = VideoCapture::new_def(0).expect("Failed to open webcam");

    let mut frame_instants = VecDeque::<Instant>::new();
    frame_instants.push_front(Instant::now());
    loop {
        let mut frame = Mat::default();
        cap.read(&mut frame).expect("Failed to read frame");
        frame_instants.push_front(Instant::now());
        if frame_instants.len() >= 60 {
            frame_instants.pop_back().unwrap();
        }
        let elapsed = (*frame_instants.front().unwrap() - *frame_instants.back().unwrap())
            / frame_instants.len() as u32;

        // Try to detect if the no-video screen is on
        if no_video.size().unwrap() != frame.size().unwrap() {
            // If the images aren't the same size resize the no_video frame we have
            let mut no_video_new = Mat::default();
            resize_def(&no_video, &mut no_video_new, frame.size().unwrap())
                .expect("Failed to resize");
            no_video = no_video_new;
        }
        let absolute_difference = norm2_def(&frame, &no_video).unwrap();
        let has_stream = absolute_difference > 5000f64;

        let fps_string = format!("{:.2}ms", elapsed.as_micros() as f64 / 1000f64);
        put_text_def(
            &mut frame,
            fps_string.as_str(),
            Point { x: 0, y: 50 },
            FONT_HERSHEY_SIMPLEX,
            1.0f64,
            Scalar::new(255f64, 255f64, 255f64, 255f64),
        )
        .expect("Failed to put text on the screen");
        if has_stream {
            let diff_string = format!("On");
            put_text_def(
                &mut frame,
                diff_string.as_str(),
                Point { x: 0, y: 100 },
                FONT_HERSHEY_SIMPLEX,
                1.0f64,
                Scalar::new(255f64, 255f64, 255f64, 255f64),
            )
            .expect("Failed to put text on the screen");
        }

        imshow("game", &frame).expect("Failed to draw frame");

        if wait_key(1).expect("Failed to wait for key") & 0xFF == 'q' as i32 {
            break;
        }
    }

    cap.release().expect("Failed to release video capture");

    destroy_all_windows().expect("Failed to destroy windows");
}
