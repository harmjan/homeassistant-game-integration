use opencv::{
    core::{norm2_def, Point, Scalar},
    highgui::{destroy_all_windows, imshow, named_window_def, wait_key},
    imgcodecs::imread_def,
    imgproc::{put_text_def, FONT_HERSHEY_SIMPLEX},
    prelude::*,
    videoio::VideoCapture,
};

mod action;
mod config;
mod counter;
mod darksouls;
mod debounce;
mod util;

fn main() {
    // Load the config
    let config = config::load_config("config.toml");

    if config.debug_window {
        named_window_def("game").expect("Failed to create window");
    }

    // Create a dark souls detector struct from the dark souls config
    let mut dark_souls: Option<darksouls::DarkSouls> =
        config.dark_souls.map(|config| config.into());

    let mut nzxt_no_video = imread_def("no-video.png").unwrap();

    let mut cap = VideoCapture::new_def(0).expect("Failed to open webcam");

    let mut frame_rate_counter = counter::EventRateCounter::new();
    let mut debounce_stream_on = debounce::DebounceRisingEdge::new();
    let mut debounce_stream_off = debounce::DebounceRisingEdge::new();
    loop {
        let mut frame = Mat::default();
        // Read the frame from the stream
        cap.read(&mut frame).expect("Failed to read frame");
        frame_rate_counter.feed();

        // Try to detect if the no-video screen is on
        util::resize_frames_to_match(&frame, &mut nzxt_no_video).unwrap();
        let absolute_difference = norm2_def(&frame, &nzxt_no_video).unwrap();
        let has_stream = absolute_difference > 5000f64;

        if debounce_stream_on.feed(has_stream) {
            if let Some(ref stream_on_action) = config.stream_on {
                stream_on_action.execute().unwrap();
            }
        }
        if debounce_stream_off.feed(!has_stream) {
            if let Some(ref stream_off_action) = config.stream_off {
                stream_off_action.execute().unwrap();
            }
        }

        // Only run any detection algorithms if there is a stream
        if has_stream {
            if let Some(ref mut dark_souls) = dark_souls {
                dark_souls
                    .execute(&frame)
                    .expect("Failed to run dark souls detection");
            }
        } else {
            let diff_string = format!("No device connected");
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

        // Print the FPS counter on the
        if config.debug_window {
            let fps_string = format!("fps: {:.2}", frame_rate_counter.get_event_rate_per_second());
            put_text_def(
                &mut frame,
                fps_string.as_str(),
                Point { x: 0, y: 50 },
                FONT_HERSHEY_SIMPLEX,
                1.0f64,
                Scalar::new(255f64, 255f64, 255f64, 255f64),
            )
            .expect("Failed to put text on the screen");

            imshow("game", &frame).expect("Failed to draw frame");
        }

        if wait_key(1).expect("Failed to wait for key") & 0xFF == 'q' as i32 {
            break;
        }
    }

    cap.release().expect("Failed to release video capture");

    destroy_all_windows().expect("Failed to destroy windows");
}
