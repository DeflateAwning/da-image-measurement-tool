// CSV / JSON export helpers for measurements.

use crate::model::{Measurement, ProjectState};
use std::io::Write;
use std::path::Path;

pub fn measurements_to_csv(measurements: &[Measurement]) -> Result<String, csv::Error> {
    let mut wtr = csv::WriterBuilder::new().from_writer(vec![]);
    wtr.write_record([
        "label",
        "point_a_x_px",
        "point_a_y_px",
        "point_b_x_px",
        "point_b_y_px",
        "pixel_distance",
        "real_distance",
        "unit",
    ])?;
    for m in measurements {
        wtr.write_record(&[
            m.label.clone(),
            format!("{:.3}", m.point_a.x),
            format!("{:.3}", m.point_a.y),
            format!("{:.3}", m.point_b.x),
            format!("{:.3}", m.point_b.y),
            format!("{:.4}", m.pixel_distance),
            m.real_distance
                .map(|v| format!("{:.4}", v))
                .unwrap_or_default(),
            m.unit.clone(),
        ])?;
    }
    let bytes = wtr.into_inner().map_err(|e| e.into_error())?;
    Ok(String::from_utf8(bytes).unwrap_or_default())
}

pub fn save_csv_to_path(path: &Path, measurements: &[Measurement]) -> anyhow::Result<()> {
    let csv_text = measurements_to_csv(measurements)?;
    let mut f = std::fs::File::create(path)?;
    f.write_all(csv_text.as_bytes())?;
    Ok(())
}

pub fn save_project_json(path: &Path, state: &ProjectState) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_project_json(path: &Path) -> anyhow::Result<ProjectState> {
    let text = std::fs::read_to_string(path)?;
    let state: ProjectState = serde_json::from_str(&text)?;
    Ok(state)
}
