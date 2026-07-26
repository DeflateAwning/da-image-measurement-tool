// Small geometry + coordinate transform helpers shared across the app.

use egui::{Pos2, Rect, Vec2};

pub fn distance(a: Pos2, b: Pos2) -> f64 {
    (((a.x - b.x) as f64).powi(2) + ((a.y - b.y) as f64).powi(2)).sqrt()
}

/// Maps between IMAGE PIXEL SPACE (origin top-left of the source image,
/// 1 unit = 1 source pixel) and SCREEN SPACE (the egui rect the image is
/// currently painted into, accounting for pan offset and zoom).
#[derive(Clone, Copy, Debug)]
pub struct ViewTransform {
    /// Screen-space rect the image is drawn into.
    pub screen_rect: Rect,
    /// Size of the source image, in pixels.
    pub image_size: Vec2,
    /// Current zoom multiplier (1.0 = image pixel maps to 1 screen point at fit scale).
    pub zoom: f32,
    /// Additional pan offset in screen points, applied after fit-to-rect scaling.
    pub pan: Vec2,
}

impl ViewTransform {
    /// Base scale that fits the whole image inside `screen_rect` (before zoom/pan).
    fn fit_scale(&self) -> f32 {
        if self.image_size.x <= 0.0 || self.image_size.y <= 0.0 {
            return 1.0;
        }
        let sx = self.screen_rect.width() / self.image_size.x;
        let sy = self.screen_rect.height() / self.image_size.y;
        sx.min(sy)
    }

    pub fn effective_scale(&self) -> f32 {
        self.fit_scale() * self.zoom
    }

    /// Convert an image-pixel-space point to screen-space.
    pub fn image_to_screen(&self, p: Pos2) -> Pos2 {
        let scale = self.effective_scale();
        let center = self.screen_rect.center() + self.pan;
        let half_img = self.image_size * scale * 0.5;
        let top_left = center - half_img;
        top_left + Vec2::new(p.x * scale, p.y * scale)
    }

    /// Convert a screen-space point back into image-pixel-space.
    pub fn screen_to_image(&self, p: Pos2) -> Pos2 {
        let scale = self.effective_scale();
        let center = self.screen_rect.center() + self.pan;
        let half_img = self.image_size * scale * 0.5;
        let top_left = center - half_img;
        let rel = p - top_left;
        Pos2::new(rel.x / scale, rel.y / scale)
    }

    /// Whether an image-space point is inside the actual image bounds.
    pub fn in_bounds(&self, p: Pos2) -> bool {
        p.x >= 0.0 && p.y >= 0.0 && p.x <= self.image_size.x && p.y <= self.image_size.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_computes_euclidean_length() {
        let a = Pos2::new(0.0, 0.0);
        let b = Pos2::new(3.0, 4.0);
        assert_eq!(distance(a, b), 5.0);
    }

    #[test]
    fn screen_to_image_inverts_image_to_screen() {
        let transform = ViewTransform {
            screen_rect: Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(200.0, 100.0)),
            image_size: Vec2::new(400.0, 200.0),
            zoom: 1.5,
            pan: Vec2::new(5.0, -3.0),
        };

        let original = Pos2::new(123.0, 45.0);
        let screen = transform.image_to_screen(original);
        let round_tripped = transform.screen_to_image(screen);

        assert!((round_tripped.x - original.x).abs() < 1e-3);
        assert!((round_tripped.y - original.y).abs() < 1e-3);
    }

    #[test]
    fn in_bounds_respects_image_edges() {
        let transform = ViewTransform {
            screen_rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(100.0, 100.0)),
            image_size: Vec2::new(50.0, 80.0),
            zoom: 1.0,
            pan: Vec2::ZERO,
        };

        assert!(transform.in_bounds(Pos2::new(0.0, 0.0)));
        assert!(transform.in_bounds(Pos2::new(50.0, 80.0)));
        assert!(!transform.in_bounds(Pos2::new(-1.0, 0.0)));
        assert!(!transform.in_bounds(Pos2::new(50.1, 80.0)));
    }
}
