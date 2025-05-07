use kurbo::BezPath;

#[derive(Default, Debug)]
/// A `BezGlyph` is a collection of `BezPath`s, which represent the outline of a glyph.
/// It is used to store the paths of a glyph in a vector, allowing for multiple paths
/// to be stored in a single glyph.
pub struct BezGlyph(pub(crate) Vec<BezPath>);

impl BezGlyph {
    /// Creates a new `BezGlyph` from a vector of `BezPath`s.
    pub fn new_from_paths(b: Vec<BezPath>) -> Self {
        BezGlyph(b)
    }
    /// Adds a new `BezPath` to the `BezGlyph` and returns a mutable reference to it.
    pub fn next(&mut self) -> &mut BezPath {
        self.0.push(BezPath::new());
        #[allow(clippy::unwrap_used)] // We just added it
        self.0.last_mut().unwrap()
    }
    /// Returns a mutable reference to the current `BezPath`. If there are no paths,
    /// it creates a new one and returns it.
    pub fn current(&mut self) -> &mut BezPath {
        if self.0.is_empty() {
            self.0.push(BezPath::new());
        }
        #[allow(clippy::unwrap_used)] // We know it's not empty
        self.0.last_mut().unwrap()
    }

    /// Iterates over the paths in the `BezGlyph`, returning an iterator of references to `BezPath`.
    pub fn iter(&self) -> impl Iterator<Item = &BezPath> {
        self.0.iter()
    }
}

#[cfg(feature = "skrifa")]
impl skrifa::outline::OutlinePen for BezGlyph {
    fn move_to(&mut self, x: f32, y: f32) {
        self.next().move_to((x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.current().line_to((x, y));
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.current().quad_to((cx0, cy0), (x, y));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.current().curve_to((cx0, cy0), (cx1, cy1), (x, y));
    }

    fn close(&mut self) {
        self.current().close_path();
    }
}
