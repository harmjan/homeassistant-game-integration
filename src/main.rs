use opencv::core::{extract_channel, min_max_loc, norm2_def, Point, Ptr, Rect, Scalar};
use opencv::highgui::{destroy_all_windows, imshow, named_window_def, wait_key};
use opencv::imgcodecs::imread_def;
use opencv::imgproc::{
    match_template_def, put_text_def, resize_def, threshold, FONT_HERSHEY_SIMPLEX, THRESH_BINARY,
    TM_CCOEFF_NORMED,
};
use opencv::prelude::*;
use opencv::text::OCRTesseract;
use opencv::videoio::VideoCapture;

use reqwest;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

const DEBUG_WINDOW: bool = true;

const HOMEASSISTANT_HOST: &str = "http://server.local:8123";
const HOMEASSISTANT_WEBHOOK_ID: &str = "office-dark-souls-dead";

struct Counter {
    max_duration: Duration,
    instants: VecDeque<Instant>,
}

impl Counter {
    fn new() -> Counter {
        let mut instants = VecDeque::new();
        instants.push_front(Instant::now());
        Counter {
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

/// Return if the screens contains the dark souls "You died" text
///
/// In dark souls remastered the game draws a band from left to right that is darker than the rest
/// of the screen with the text "YOU DIED" in red over it.
///
/// Screen is 1054x592
/// (left: 320, right: 727, pixel)
/// (top: 270, bottom: 350)
fn is_dark_souls_you_died(you_died: &Mat, tesseract: &mut Ptr<OCRTesseract>, frame: &Mat) -> bool {
    // Copied from https://github.com/TristoKrempita/ds-death-counter/blob/master/frames.py
    /*
    let mut match_result = Mat::default();
    match_template_def(frame, you_died, &mut match_result, TM_CCOEFF_NORMED)
        .expect("Failed to match template");
    let mask = Mat::default();
    let mut max_val = 0f64;
    min_max_loc(&match_result, None, Some(&mut max_val), None, None, &mask)
        .expect("Failed to find max");

    max_val >= 0.2f64
    */

    // Try to extract the square where the YOU DIED will be
    let you_died_roi = {
        let frame_size = frame.size().unwrap();
        let (mid_x, mid_y) = (frame_size.width / 2, frame_size.height * 21 / 40);
        let (width, height) = (frame_size.width * 2 / 5, frame_size.height * 3 / 20);
        Mat::roi(
            &frame,
            Rect::new(mid_x - width / 2, mid_y - height / 2, width, height),
        )
        .unwrap()
    };

    // Extract the red channel
    let mut red = Mat::default();
    extract_channel(&you_died_roi, &mut red, 2).unwrap();
    // Create a binary threshold
    let mut threshold_red = Mat::default();
    threshold(&red, &mut threshold_red, 100f64, 255f64, THRESH_BINARY).unwrap();
    // Compare the image with the saved one
    let absolute_difference = norm2_def(&threshold_red, &you_died).unwrap();
    imshow("game", &threshold_red).expect("Failed to draw frame");

    absolute_difference < 5000f64
}

fn main() {
    let mut no_video = imread_def("no-video.png").unwrap();
    let you_died_original = imread_def("youdied.png").unwrap();
    let mut you_died = Mat::default();
    extract_channel(&you_died_original, &mut you_died, 2).unwrap();
    let mut tesseract = OCRTesseract::create_def().unwrap();

    named_window_def("game").expect("Failed to create window");

    let mut cap = VideoCapture::new_def(0).expect("Failed to open webcam");

    let mut counter = Counter::new();
    let mut dead_event_dark_souls = DebounceRisingEdge::new();
    loop {
        let mut frame = Mat::default();
        // Read the frame from the stream
        cap.read(&mut frame).expect("Failed to read frame");
        counter.feed();

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

        if has_stream {
            if dead_event_dark_souls.feed(is_dark_souls_you_died(&you_died, &mut tesseract, &frame))
            {
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
            let diff_string = format!("Not running anything");
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
            let fps_string = format!("fps: {:.2}", counter.get_fps());
            put_text_def(
                &mut frame,
                fps_string.as_str(),
                Point { x: 0, y: 50 },
                FONT_HERSHEY_SIMPLEX,
                1.0f64,
                Scalar::new(255f64, 255f64, 255f64, 255f64),
            )
            .expect("Failed to put text on the screen");

            //imshow("game", &frame).expect("Failed to draw frame");
        }

        if wait_key(1).expect("Failed to wait for key") & 0xFF == 'q' as i32 {
            break;
        }
    }

    cap.release().expect("Failed to release video capture");

    destroy_all_windows().expect("Failed to destroy windows");
}
