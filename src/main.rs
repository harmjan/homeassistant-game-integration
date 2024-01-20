use opencv::core::{extract_channel, norm2_def, Point, Rect, Scalar};
use opencv::highgui::{destroy_all_windows, imshow, named_window_def, wait_key};
use opencv::imgcodecs::imread_def;
use opencv::imgproc::{put_text_def, resize_def, threshold, FONT_HERSHEY_SIMPLEX, THRESH_BINARY};
use opencv::prelude::*;
use opencv::videoio::VideoCapture;

use reqwest;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

const DEBUG_WINDOW: bool = false;

const HOMEASSISTANT_HOST: &str = "http://server.local:8123";
const HOMEASSISTANT_WEBHOOK_ID: &str = "office-dark-souls-dead";

/// Count events in a period to calculate an FPS
struct EventRateCounter {
    max_duration: Duration,
    instants: VecDeque<Instant>,
}

impl EventRateCounter {
    fn new() -> EventRateCounter {
        let mut instants = VecDeque::new();
        instants.push_front(Instant::now());
        EventRateCounter {
            instants,
            max_duration: Duration::from_secs(60),
        }
    }

    fn feed(&mut self) {
        self.instants.push_front(Instant::now());
        // Remove instants that are too old but always keep 2
        while self.instants.len() > 2 && self.total_duration() >= self.max_duration {
            self.instants.pop_back().unwrap();
        }
    }

    fn total_duration(&self) -> Duration {
        *self.instants.front().unwrap() - *self.instants.back().unwrap()
    }

    fn get_fps(&self) -> f32 {
        let duration_per_frame = self.total_duration() / self.instants.len() as u32;

        1f32 / duration_per_frame.as_secs_f32()
    }
}

/// The webhook only needs to get called once, only pass the true result on once for a stretch of
/// true signals
struct DebounceRisingEdge {
    period: Duration,
    last_rising: Instant,
}

impl DebounceRisingEdge {
    fn new() -> DebounceRisingEdge {
        DebounceRisingEdge {
            period: Duration::from_secs(5),
            last_rising: Instant::now(),
        }
    }

    fn feed(&mut self, event: bool) -> bool {
        let now = Instant::now();
        let result = event && (now - self.last_rising) >= self.period;
        if event {
            self.last_rising = now;
        }
        result
    }
}

/// Resize frame b so it matches the size of frame a
fn resize_frames_to_match(frame_a: &Mat, frame_b: &mut Mat) -> opencv::Result<()> {
    if frame_a.size()? != frame_b.size()? {
        // If the images aren't the same size resize frame_b
        let mut resized_frame_b = Mat::default();
        resize_def(frame_b, &mut resized_frame_b, frame_a.size()?)?;
        resized_frame_b.copy_to(frame_b)?;
    }

    Ok(())
}

/// Return if the screens contains the dark souls "You died" text
///
/// In dark souls remastered the game draws a band from left to right that is darker than the rest
/// of the screen with the text "YOU DIED" in red over it.
///
/// There were other project who did something similar like:
///  - https://github.com/TristoKrempita/ds-death-counter/blob/master/frames.py
fn is_dark_souls_you_died(you_died: &mut Mat, frame: &Mat) -> opencv::Result<bool> {
    // Try to extract the square where the YOU DIED will be as the region of interest
    let you_died_roi = {
        let frame_size = frame.size()?;
        let (mid_x, mid_y) = (frame_size.width / 2, frame_size.height * 21 / 40);
        let (width, height) = (frame_size.width * 2 / 5, frame_size.height * 3 / 20);
        Mat::roi(
            &frame,
            Rect::new(mid_x - width / 2, mid_y - height / 2, width, height),
        )?
    };

    // Resize the reference if it is somehow different from the actual ROI
    resize_frames_to_match(&you_died_roi, you_died)?;

    // Extract the red channel
    let mut red = Mat::default();
    extract_channel(&you_died_roi, &mut red, 2)?;
    // Create a binary threshold
    let mut threshold_red = Mat::default();
    threshold(&red, &mut threshold_red, 100f64, 255f64, THRESH_BINARY)?;

    // Compare the image with the saved one
    let absolute_difference = norm2_def(&threshold_red, you_died)?;

    Ok(absolute_difference < 5000f64)
}

fn main() {
    let mut nzxt_no_video = imread_def("no-video.png").unwrap();
    let you_died_original = imread_def("youdied.png").unwrap();
    let mut you_died = Mat::default();
    extract_channel(&you_died_original, &mut you_died, 2).unwrap();

    if DEBUG_WINDOW {
        named_window_def("game").expect("Failed to create window");
    }

    let mut cap = VideoCapture::new_def(0).expect("Failed to open webcam");

    let mut frame_rate_counter = EventRateCounter::new();
    let mut dead_event_dark_souls = DebounceRisingEdge::new();
    loop {
        let mut frame = Mat::default();
        // Read the frame from the stream
        cap.read(&mut frame).expect("Failed to read frame");
        frame_rate_counter.feed();

        // Try to detect if the no-video screen is on
        resize_frames_to_match(&frame, &mut nzxt_no_video).unwrap();
        let absolute_difference = norm2_def(&frame, &nzxt_no_video).unwrap();
        let has_stream = absolute_difference > 5000f64;

        if has_stream {
            if dead_event_dark_souls.feed(
                is_dark_souls_you_died(&mut you_died, &frame)
                    .expect("Failed to run YOU DIED detection"),
            ) {
                let client = reqwest::blocking::Client::new();
                client
                    .post(format!(
                        "{}/api/webhook/{}",
                        HOMEASSISTANT_HOST, HOMEASSISTANT_WEBHOOK_ID
                    ))
                    .send()
                    .expect("Failed to post webhook")
                    .error_for_status()
                    .expect("Webhook didn't return good status code");
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
        if DEBUG_WINDOW {
            let fps_string = format!("fps: {:.2}", frame_rate_counter.get_fps());
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
