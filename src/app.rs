use eframe::egui;
use egui::{Color32, Pos2, Rect, Sense, Stroke, Vec2};
use std::path::PathBuf;

use crate::export;
use crate::geometry::ViewTransform;
use crate::model::{Calibration, InputMode, Measurement, PendingClick, ProjectState};

pub struct MeasurementApp {
    texture: Option<egui::TextureHandle>,
    image_size: Vec2,
    image_path: Option<PathBuf>,

    mode: InputMode,
    pending: PendingClick,

    calibration_points: Option<(Pos2, Pos2)>,
    calib_length_input: String,
    calib_unit_input: String,

    measurements: Vec<Measurement>,
    next_measurement_id: usize,
    selected_measurement: Option<usize>,

    zoom: f32,
    pan: Vec2,

    status_message: String,
    error_message: Option<String>,
}

impl MeasurementApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            texture: None,
            image_size: Vec2::ZERO,
            image_path: None,
            mode: InputMode::Idle,
            pending: PendingClick::AwaitingFirst,
            calibration_points: None,
            calib_length_input: "10.0".to_owned(),
            calib_unit_input: "mm".to_owned(),
            measurements: Vec::new(),
            next_measurement_id: 1,
            selected_measurement: None,
            zoom: 1.0,
            pan: Vec2::ZERO,
            status_message: "Load an image to begin.".to_owned(),
            error_message: None,
        }
    }

    fn reset_view(&mut self) {
        self.zoom = 1.0;
        self.pan = Vec2::ZERO;
    }

    fn load_image_path(&mut self, ctx: &egui::Context, path: PathBuf) {
        match image::open(&path) {
            Ok(img) => {
                let rgba = img.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let pixels = rgba.into_raw();
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
                let texture =
                    ctx.load_texture("loaded_image", color_image, egui::TextureOptions::LINEAR);
                self.image_size = Vec2::new(size[0] as f32, size[1] as f32);
                self.texture = Some(texture);
                self.image_path = Some(path);
                self.calibration_points = None;
                self.measurements.clear();
                self.next_measurement_id = 1;
                self.pending = PendingClick::AwaitingFirst;
                self.reset_view();
                self.error_message = None;
                self.status_message = "Image loaded. Pick a mode to start.".to_owned();
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to load image: {e}"));
            }
        }
    }

    fn current_calibration(&self) -> Option<Calibration> {
        let (a, b) = self.calibration_points?;
        let length: f64 = self.calib_length_input.trim().parse().ok()?;
        if length <= 0.0 {
            return None;
        }
        Some(Calibration {
            point_a: a,
            point_b: b,
            real_length: length,
            unit: self.calib_unit_input.trim().to_owned(),
        })
    }

    fn handle_calibration_click(&mut self, pt: Pos2) {
        match self.pending {
            PendingClick::AwaitingFirst => {
                self.pending = PendingClick::AwaitingSecond(pt);
                self.status_message =
                    "Calibration: Point A set. Click Point B (the other end of the known length)."
                        .to_owned();
            }
            PendingClick::AwaitingSecond(a) => {
                self.calibration_points = Some((a, pt));
                self.pending = PendingClick::AwaitingFirst;
                self.status_message =
                    "Calibration set. Enter the real length + unit in the side panel."
                        .to_owned();
            }
        }
    }

    fn handle_measurement_click(&mut self, pt: Pos2) {
        match self.pending {
            PendingClick::AwaitingFirst => {
                self.pending = PendingClick::AwaitingSecond(pt);
                self.status_message = "Measure: Point A set. Click Point B.".to_owned();
            }
            PendingClick::AwaitingSecond(a) => {
                let pixel_distance = crate::geometry::distance(a, pt);
                let calib = self.current_calibration();
                let (real_distance, unit) = match &calib {
                    Some(c) => (
                        c.units_per_pixel().map(|upp| upp * pixel_distance),
                        c.unit.clone(),
                    ),
                    None => (None, String::new()),
                };
                let label = format!("M{}", self.next_measurement_id);
                self.next_measurement_id += 1;
                self.measurements.push(Measurement {
                    label,
                    point_a: a,
                    point_b: pt,
                    pixel_distance,
                    real_distance,
                    unit,
                });
                self.selected_measurement = Some(self.measurements.len() - 1);
                self.pending = PendingClick::AwaitingFirst;
                self.status_message = "Measurement recorded. Click to start another.".to_owned();
            }
        }
    }

    fn recompute_measurements_from_calibration(&mut self) {
        if let Some(calib) = self.current_calibration() {
            if let Some(upp) = calib.units_per_pixel() {
                for m in &mut self.measurements {
                    m.real_distance = Some(upp * m.pixel_distance);
                    m.unit = calib.unit.clone();
                }
                self.status_message = "All measurements recomputed from current calibration."
                    .to_owned();
            }
        } else {
            self.error_message =
                Some("Set a valid calibration (two points + positive length) first.".to_owned());
        }
    }

    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("📂 Load Image").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Images", &["png", "jpg", "jpeg", "bmp", "gif", "tiff", "webp"])
                        .pick_file()
                    {
                        self.load_image_path(ctx, path);
                    }
                }
                ui.separator();
                if ui.button("💾 Save Project").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .set_file_name("measurement_project.json")
                        .save_file()
                    {
                        let state = ProjectState {
                            image_path: self.image_path.as_ref().map(|p| p.display().to_string()),
                            calibration: self.current_calibration(),
                            measurements: self.measurements.clone(),
                        };
                        if let Err(e) = export::save_project_json(&path, &state) {
                            self.error_message = Some(format!("Save failed: {e}"));
                        } else {
                            self.status_message = "Project saved.".to_owned();
                        }
                    }
                }
                if ui.button("📁 Load Project").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("JSON", &["json"])
                        .pick_file()
                    {
                        match export::load_project_json(&path) {
                            Ok(state) => {
                                if let Some(img_path) = state.image_path.clone() {
                                    self.load_image_path(ctx, PathBuf::from(img_path));
                                }
                                if let Some(c) = state.calibration {
                                    self.calibration_points = Some((c.point_a, c.point_b));
                                    self.calib_length_input = format!("{}", c.real_length);
                                    self.calib_unit_input = c.unit;
                                }
                                self.measurements = state.measurements;
                                self.next_measurement_id = self.measurements.len() + 1;
                                self.status_message = "Project loaded.".to_owned();
                            }
                            Err(e) => self.error_message = Some(format!("Load failed: {e}")),
                        }
                    }
                }
                ui.separator();
                if ui.button("🔍 Reset View").clicked() {
                    self.reset_view();
                }
                ui.separator();
                ui.label(format!("Zoom: {:.0}%", self.zoom * 100.0));
            });
        });
    }

    fn side_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("side_panel")
            .resizable(true)
            .default_width(340.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Mode");
                    ui.radio_value(&mut self.mode, InputMode::Idle, "Idle / Pan");
                    ui.radio_value(&mut self.mode, InputMode::Calibrate, "Calibrate (known length)");
                    ui.radio_value(&mut self.mode, InputMode::Measure, "Measure (read off length)");
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(&self.status_message).italics().weak());

                    ui.separator();
                    ui.heading("Calibration");
                    match self.calibration_points {
                        Some((a, b)) => {
                            ui.label(format!("Point A: ({:.1}, {:.1}) px", a.x, a.y));
                            ui.label(format!("Point B: ({:.1}, {:.1}) px", b.x, b.y));
                            let px_dist = crate::geometry::distance(a, b);
                            ui.label(format!("Pixel distance: {:.2} px", px_dist));
                        }
                        None => {
                            ui.label("Not set. Switch to Calibrate mode and click two points spanning a known length.");
                        }
                    }
                    ui.horizontal(|ui| {
                        ui.label("Real length:");
                        ui.text_edit_singleline(&mut self.calib_length_input);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Unit:");
                        ui.text_edit_singleline(&mut self.calib_unit_input);
                    });
                    if let Some(calib) = self.current_calibration() {
                        if let Some(upp) = calib.units_per_pixel() {
                            ui.colored_label(
                                Color32::from_rgb(90, 200, 120),
                                format!("Scale: 1 px = {:.5} {}", upp, calib.unit),
                            );
                        }
                    } else if self.calibration_points.is_some() {
                        ui.colored_label(Color32::from_rgb(220, 140, 60), "Enter a valid positive length to activate calibration.");
                    }
                    ui.horizontal(|ui| {
                        if ui.button("Clear Calibration").clicked() {
                            self.calibration_points = None;
                        }
                        if ui.button("Recompute Measurements").clicked() {
                            self.recompute_measurements_from_calibration();
                        }
                    });

                    ui.separator();
                    ui.heading(format!("Measurements ({})", self.measurements.len()));
                    let mut to_delete: Option<usize> = None;
                    for (i, m) in self.measurements.iter_mut().enumerate() {
                        let selected = self.selected_measurement == Some(i);
                        egui::Frame::group(ui.style())
                            .fill(if selected {
                                ui.style().visuals.selection.bg_fill
                            } else {
                                ui.style().visuals.faint_bg_color
                            })
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.text_edit_singleline(&mut m.label);
                                    if ui.small_button("🗑").clicked() {
                                        to_delete = Some(i);
                                    }
                                });
                                ui.label(format!("{:.2} px", m.pixel_distance));
                                if let Some(real) = m.real_distance {
                                    ui.label(format!("{:.4} {}", real, m.unit));
                                } else {
                                    ui.weak("no calibration at capture time");
                                }
                            });
                    }
                    if let Some(i) = to_delete {
                        self.measurements.remove(i);
                        if self.selected_measurement == Some(i) {
                            self.selected_measurement = None;
                        }
                    }

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Clear All Measurements").clicked() {
                            self.measurements.clear();
                            self.selected_measurement = None;
                        }
                        if ui.button("⬇ Export CSV").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("CSV", &["csv"])
                                .set_file_name("measurements.csv")
                                .save_file()
                            {
                                if let Err(e) = export::save_csv_to_path(&path, &self.measurements) {
                                    self.error_message = Some(format!("Export failed: {e}"));
                                } else {
                                    self.status_message = "Measurements exported.".to_owned();
                                }
                            }
                        }
                    });

                    if let Some(err) = self.error_message.clone() {
                        ui.add_space(8.0);
                        ui.colored_label(Color32::from_rgb(220, 80, 80), err);
                    }
                });
            });
    }

    fn canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(texture) = &self.texture else {
                ui.centered_and_justified(|ui| {
                    ui.label("No image loaded. Use \"Load Image\" above.");
                });
                return;
            };
            let texture_id = texture.id();

            let available = ui.available_size();
            let (response, painter) =
                ui.allocate_painter(available, Sense::click_and_drag());
            let screen_rect = response.rect;

            let transform = ViewTransform {
                screen_rect,
                image_size: self.image_size,
                zoom: self.zoom,
                pan: self.pan,
            };

            // Draw the image itself, stretched into its transformed rect.
            let img_top_left = transform.image_to_screen(Pos2::ZERO);
            let img_bottom_right = transform.image_to_screen(Pos2::new(
                self.image_size.x,
                self.image_size.y,
            ));
            let img_rect = Rect::from_two_pos(img_top_left, img_bottom_right);
            painter.image(
                texture_id,
                img_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );

            // Zoom with scroll wheel, centered on cursor.
            if let Some(hover_pos) = response.hover_pos() {
                let scroll = ctx.input(|i| i.raw_scroll_delta.y);
                if scroll.abs() > 0.0 {
                    let old_zoom = self.zoom;
                    let factor = (scroll * 0.0015).exp();
                    let new_zoom = (old_zoom * factor).clamp(0.05, 40.0);
                    let img_pt = transform.screen_to_image(hover_pos);
                    self.zoom = new_zoom;
                    let transform_after = ViewTransform {
                        screen_rect,
                        image_size: self.image_size,
                        zoom: self.zoom,
                        pan: self.pan,
                    };
                    let screen_of_pt = transform_after.image_to_screen(img_pt);
                    self.pan += hover_pos - screen_of_pt;
                }
            }

            // Pan with middle-mouse or right-mouse drag (any mode), or
            // left-drag while in Idle mode.
            if response.dragged() {
                let dragging_pan_button = ctx.input(|i| {
                    i.pointer.middle_down() || i.pointer.secondary_down()
                });
                if dragging_pan_button || self.mode == InputMode::Idle {
                    self.pan += response.drag_delta();
                }
            }

            // Re-derive transform in case pan/zoom changed above.
            let transform = ViewTransform {
                screen_rect,
                image_size: self.image_size,
                zoom: self.zoom,
                pan: self.pan,
            };

            // Left-click places points, in Calibrate/Measure modes.
            if response.clicked() && self.mode != InputMode::Idle {
                if let Some(click_pos) = response.interact_pointer_pos() {
                    let img_pt = transform.screen_to_image(click_pos);
                    if transform.in_bounds(img_pt) {
                        match self.mode {
                            InputMode::Calibrate => self.handle_calibration_click(img_pt),
                            InputMode::Measure => self.handle_measurement_click(img_pt),
                            InputMode::Idle => {}
                        }
                    }
                }
            }

            // --- Drawing overlays ---
            let gold = Color32::from_rgb(230, 190, 60);
            let cyan = Color32::from_rgb(70, 210, 220);
            let magenta = Color32::from_rgb(230, 90, 200);

            // Calibration overlay.
            if let Some((a, b)) = self.calibration_points {
                draw_marker(&painter, transform.image_to_screen(a), gold);
                draw_marker(&painter, transform.image_to_screen(b), gold);
                painter.line_segment(
                    [transform.image_to_screen(a), transform.image_to_screen(b)],
                    Stroke::new(2.0_f32, gold),
                );
                if let Some(calib) = self.current_calibration() {
                    let mid = transform.image_to_screen(midpoint(a, b));
                    painter.text(
                        mid,
                        egui::Align2::CENTER_BOTTOM,
                        format!("{:.3} {}", calib.real_length, calib.unit),
                        egui::FontId::proportional(14.0),
                        gold,
                    );
                }
            }

            // Existing measurements.
            for (i, m) in self.measurements.iter().enumerate() {
                let color = if self.selected_measurement == Some(i) {
                    magenta
                } else {
                    cyan
                };
                let sa = transform.image_to_screen(m.point_a);
                let sb = transform.image_to_screen(m.point_b);
                draw_marker(&painter, sa, color);
                draw_marker(&painter, sb, color);
                painter.line_segment([sa, sb], Stroke::new(2.0_f32, color));
                let mid = transform.image_to_screen(midpoint(m.point_a, m.point_b));
                let label_text = match m.real_distance {
                    Some(real) => format!("{}: {:.3} {}", m.label, real, m.unit),
                    None => format!("{}: {:.1} px", m.label, m.pixel_distance),
                };
                painter.text(
                    mid,
                    egui::Align2::CENTER_BOTTOM,
                    label_text,
                    egui::FontId::proportional(14.0),
                    color,
                );
            }

            // Rubber-band preview line from the first point to the cursor.
            if let PendingClick::AwaitingSecond(a) = self.pending {
                if self.mode != InputMode::Idle {
                    if let Some(hover_pos) = response.hover_pos() {
                        let color = if self.mode == InputMode::Calibrate {
                            gold
                        } else {
                            cyan
                        };
                        let sa = transform.image_to_screen(a);
                        draw_marker(&painter, sa, color);
                        painter.line_segment(
                            [sa, hover_pos],
                            Stroke::new(1.5_f32, color.gamma_multiply(0.7)),
                        );
                        let img_pt = transform.screen_to_image(hover_pos);
                        let px_dist = crate::geometry::distance(a, img_pt);
                        let text = if let Some(calib) = self.current_calibration() {
                            if let Some(upp) = calib.units_per_pixel() {
                                format!("{:.3} {}", upp * px_dist, calib.unit)
                            } else {
                                format!("{:.1} px", px_dist)
                            }
                        } else {
                            format!("{:.1} px", px_dist)
                        };
                        painter.text(
                            hover_pos + Vec2::new(12.0, -12.0),
                            egui::Align2::LEFT_BOTTOM,
                            text,
                            egui::FontId::proportional(13.0),
                            color,
                        );
                    }
                }
            }
        });
    }
}

fn draw_marker(painter: &egui::Painter, p: Pos2, color: Color32) {
    let r = 4.5;
    painter.circle_stroke(p, r, Stroke::new(2.0_f32, color));
    painter.line_segment([p - Vec2::new(r + 3.0, 0.0), p + Vec2::new(r + 3.0, 0.0)], Stroke::new(1.0_f32, color));
    painter.line_segment([p - Vec2::new(0.0, r + 3.0), p + Vec2::new(0.0, r + 3.0)], Stroke::new(1.0_f32, color));
}

fn midpoint(a: Pos2, b: Pos2) -> Pos2 {
    Pos2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
}

impl eframe::App for MeasurementApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.top_bar(ctx);
        self.side_panel(ctx);
        self.canvas(ctx);
    }
}
