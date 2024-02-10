use opencv::imgproc::resize_def;
use opencv::prelude::*;

/// Resize frame b so it matches the size of frame a
pub fn resize_frames_to_match(frame_a: &Mat, frame_b: &mut Mat) -> opencv::Result<()> {
    if frame_a.size()? != frame_b.size()? {
        // If the images aren't the same size resize frame_b
        let mut resized_frame_b = Mat::default();
        resize_def(frame_b, &mut resized_frame_b, frame_a.size()?)?;
        resized_frame_b.copy_to(frame_b)?;
    }

    Ok(())
}
