use ::interpolatable::{BezGlyph, Glyph as TwisterGlyph};
use pyo3::{exceptions::PyTypeError, prelude::*};
use pythonize::pythonize;

#[pyclass]
pub struct Glyph(pub TwisterGlyph);

fn decompose_quadratic_segment(points: Vec<(f32, f32)>) -> Vec<((f32, f32), (f32, f32))> {
    let mut quad_segments = Vec::new();
    for i in 0..points.len() - 1 {
        let (x, y) = points[i];
        let (nx, ny) = points[i + 1];
        let implied_pt = (0.5 * (x + nx), 0.5 * (y + ny));
        quad_segments.push((points[i], implied_pt));
    }
    quad_segments
}

fn quad_to_one(bezglyph: &mut BezGlyph, p1: (f32, f32), p2: (f32, f32)) {
    bezglyph.current().quad_to(p1, p2);
}

fn replay_recording(bezglyph: &mut BezGlyph, value: Vec<(String, Vec<(f32, f32)>)>) {
    for (command, points) in value {
        match command.as_str() {
            "moveTo" => {
                bezglyph.next().move_to((points[0].0, points[0].1));
            }
            "lineTo" => {
                bezglyph.current().line_to((points[0].0, points[0].1));
            }
            "qCurveTo" => {
                // in theory handle the zero case heres
                for (pt1, pt2) in decompose_quadratic_segment(points) {
                    quad_to_one(bezglyph, pt1, pt2);
                }
            }
            "curveTo" => {
                // in theory handle the polycubic case here
                bezglyph.current().curve_to(
                    (points[0].0, points[0].1),
                    (points[1].0, points[1].1),
                    (points[2].0, points[2].1),
                );
            }
            "closePath" => {
                bezglyph.current().close_path();
            }
            _ => {}
        }
    }
}

#[pymethods]
impl Glyph {
    #[new]
    fn __new__(
        py: Python,
        master_name: String,
        master_index: usize,
        obj: PyObject,
        glyphset: PyObject,
    ) -> PyResult<Self> {
        let recordingpen_m = PyModule::import_bound(py, "fontTools.pens.recordingPen")?;
        let recordingpen = recordingpen_m.getattr("DecomposingRecordingPen")?;
        let pen = recordingpen.call1((glyphset,))?;
        obj.call_method1(py, "draw", (&pen,))?;
        let value: Vec<(String, Vec<(f32, f32)>)> = pen.getattr("value")?.extract()?;
        let mut bezglyph = BezGlyph::default();
        replay_recording(&mut bezglyph, value);
        let mut glyph: TwisterGlyph = bezglyph.into();
        glyph.master_name = master_name;
        glyph.master_index = master_index;

        Ok(Glyph(glyph))
    }
}

#[pyfunction]
#[pyo3(signature = (glyph_a, glyph_b, tolerance=None, kinkiness=None, upem=None))]
fn test_interpolatability<'py>(
    py: Python<'py>,
    glyph_a: &Glyph,
    glyph_b: &Glyph,
    tolerance: Option<f64>,
    kinkiness: Option<f64>,
    upem: Option<u16>,
) -> PyResult<Bound<'py, PyAny>> {
    let result = ::interpolatable::run_tests(&glyph_a.0, &glyph_b.0, tolerance, kinkiness, upem);
    println!("{:?}", result);
    pythonize(py, &result).map_err(|e| PyErr::new::<PyTypeError, _>("Error message"))
}

#[pymodule]
fn interpolatable(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Glyph>()?;
    m.add_function(wrap_pyfunction!(test_interpolatability, m)?)?;
    Ok(())
}
