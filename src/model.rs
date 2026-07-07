// Core data model: calibration and measurements.
// Coordinates are stored in IMAGE PIXEL SPACE (origin top-left of the
// original loaded image, independent of on-screen pan/zoom), so everything
// here is resolution/zoom independent.

use egui::Pos2;
use serde::{Deserialize, Serialize};

/// A single reference calibration: two points of known real-world distance.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Calibration {
    pub point_a: Pos2,
    pub point_b: Pos2,
    /// The real-world length the user says exists between point_a and point_b.
    pub real_length: f64,
    /// Unit label, e.g. "mm", "in", "cm". Purely cosmetic / for display.
    pub unit: String,
}

impl Calibration {
    pub fn pixel_distance(&self) -> f64 {
        crate::geometry::distance(self.point_a, self.point_b)
    }

    /// Scale factor: real-world units per pixel. None if degenerate (zero px distance).
    pub fn units_per_pixel(&self) -> Option<f64> {
        let px = self.pixel_distance();
        if px > 1e-9 {
            Some(self.real_length / px)
        } else {
            None
        }
    }
}

/// A user-recorded measurement between two arbitrary points, derived from
/// whichever calibration was active when it was taken.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Measurement {
    pub label: String,
    pub point_a: Pos2,
    pub point_b: Pos2,
    pub pixel_distance: f64,
    /// Real-world distance at time of measurement, if a calibration was active.
    pub real_distance: Option<f64>,
    pub unit: String,
}

/// What the next click(s) should do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    /// Clicking sets the calibration's Point A, then Point B.
    Calibrate,
    /// Clicking sets a new measurement's Point A, then Point B.
    Measure,
    /// No point placement; just pan/inspect.
    Idle,
}

/// Tracks how many points of the *current* click-pair have been placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PendingClick {
    AwaitingFirst,
    AwaitingSecond(Pos2), // holds point A in image space
}

/// Whole persisted project state (so a session can be saved/reloaded as JSON,
/// separate from the image bytes themselves which are referenced by path).
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProjectState {
    pub image_path: Option<String>,
    pub calibration: Option<Calibration>,
    pub measurements: Vec<Measurement>,
}
