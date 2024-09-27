use kurbo::{BezPath, PathEl};
use skrifa::outline::OutlinePen;

trait Draw {
    fn draw(&self, cairo: &cairo::Context);
}

impl Draw for BezPath {
    fn draw(&self, cairo: &cairo::Context) {
        for el in self.elements() {
            match el {
                PathEl::MoveTo(p) => cairo.move_to(p.x, p.y),
                PathEl::LineTo(p) => cairo.line_to(p.x, p.y),
                PathEl::QuadTo(p0, p1) => {
                    let (px, py) = cairo.current_point().unwrap();
                    let cx0 = (px + 2.0 * p0.x) / 3.0;
                    let cy0 = (py + 2.0 * p1.x) / 3.0;
                    let cx1 = (p1.x + 2.0 * p0.x) / 3.0;
                    let cy1 = (p1.y + 2.0 * p0.y) / 3.0;
                    cairo.curve_to(cx0, cy0, cx1, cy1, p1.x, p1.y);
                }
                PathEl::CurveTo(p0, p1, p2) => cairo.curve_to(p0.x, p0.y, p1.x, p1.y, p2.x, p2.y),
                PathEl::ClosePath => cairo.close_path(),
            }
        }
    }
}
pub(crate) struct CairoPen<'a>(pub &'a cairo::Context);

impl<'a> CairoPen<'a> {
    pub fn new(ctx: &'a cairo::Context) -> CairoPen<'a> {
        CairoPen(ctx)
    }
}

impl OutlinePen for CairoPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x as f64, y as f64);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x as f64, y as f64);
    }

    fn quad_to(&mut self, qx1: f32, qy1: f32, x: f32, y: f32) {
        // Convert to cubic
        let (px, py) = self.0.current_point().unwrap();
        let cx0 = (px + 2.0 * qx1 as f64) / 3.0;
        let cy0 = (py + 2.0 * qy1 as f64) / 3.0;
        let cx1 = (x as f64 + 2.0 * qx1 as f64) / 3.0;
        let cy1 = (y as f64 + 2.0 * qy1 as f64) / 3.0;
        self.0.curve_to(cx0, cy0, cx1, cy1, x as f64, y as f64);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.0.curve_to(
            cx0 as f64, cy0 as f64, cx1 as f64, cy1 as f64, x as f64, y as f64,
        );
    }

    fn close(&mut self) {
        self.0.close_path();
    }
}
