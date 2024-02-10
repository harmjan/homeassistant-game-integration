use opencv::{
    core::{extract_channel, norm2_def, Rect},
    imgcodecs::imread_def,
    imgproc::{threshold, THRESH_BINARY},
    prelude::*,
};
use serde::Deserialize;

use super::action;
use super::debounce;
use super::util;

/// The dark souls module config
#[derive(Deserialize)]
pub struct Config {
    pub you_died: Option<action::Config>,
}

/// The state for the dark souls
pub struct DarkSouls {
    config: Config,
    you_died_debounce: debounce::DebounceRisingEdge,
    you_died: Mat,
}

impl From<Config> for DarkSouls {
    /// Convert a configuration into an actual detector struct
    fn from(config: Config) -> Self {
        DarkSouls::new(config)
    }
}

impl DarkSouls {
    /// Create a new dark souls detector struct
    pub fn new(config: Config) -> DarkSouls {
        let you_died_original = imread_def("youdied.png").unwrap();
        let mut you_died = Mat::default();
        extract_channel(&you_died_original, &mut you_died, 2).unwrap();
        DarkSouls {
            config,
            you_died_debounce: debounce::DebounceRisingEdge::new(),
            you_died,
        }
    }

    /// Execute all configured detection algorithms on a frame
    pub fn execute(&mut self, frame: &Mat) -> opencv::Result<()> {
        if self.config.you_died.is_some() {
            let is_you_died = self.is_dark_souls_you_died(frame)?;
            if self.you_died_debounce.feed(is_you_died) {
                if let Some(ref you_died_action) = self.config.you_died {
                    you_died_action
                        .execute()
                        .expect("Failed to execute configured action");
                }
            }
        }

        Ok(())
    }

    /// Return if the screens contains the dark souls "You died" text
    ///
    /// There are 3 dark souls games that I currently have:
    ///  - Dark Souls remastered
    ///  - Dark Souls II Scholar of the First Sin
    ///  - Dark Souls III
    /// I want this function to work for all of them without knowledge beforehand which is being
    /// played.
    ///
    /// In dark souls the game draws a band from left to right that is darker than the rest of the
    /// screen with the text "YOU DIED" in red over it. The location it draws this can be slightly
    /// different per game, the text and font seem to be the same though. The text also becomes
    /// slightly bigger during the fade out of the screen. This function should look in a limited area
    /// of the screen for the red text, maybe with different text sizes as well.
    ///
    /// There were other project who did something similar like:
    ///  - https://github.com/TristoKrempita/ds-death-counter/blob/master/frames.py
    fn is_dark_souls_you_died(&mut self, frame: &Mat) -> opencv::Result<bool> {
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
        util::resize_frames_to_match(&you_died_roi, &mut self.you_died)?;

        // Extract the red channel
        let mut red = Mat::default();
        extract_channel(&you_died_roi, &mut red, 2)?;
        // Create a binary threshold
        let mut threshold_red = Mat::default();
        threshold(&red, &mut threshold_red, 80f64, 255f64, THRESH_BINARY)?;

        // Compare the image with the saved one
        let absolute_difference = norm2_def(&threshold_red, &self.you_died)?;

        //imshow("game", &threshold_red).expect("Failed to draw frame");

        //println!("{}", absolute_difference);
        Ok(absolute_difference < 5000f64)
    }
}
