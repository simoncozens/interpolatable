#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
pub use bezglyph::BezGlyph;
use greencurves::{ComputeControlStatistics, ComputeGreenStatistics, CurveStatistics};
use isomorphism::Isomorphisms;
use itertools::Itertools;
use kurbo::{BezPath, Point};
pub use problems::{Problem, ProblemDetails};

#[cfg(feature = "skrifa")]
use skrifa::{prelude::*, setting::VariationSetting};

use startingpoint::test_starting_point;
use utils::lerp_curve;

mod basiccompat;
mod bezglyph;
mod contourorder;
mod isomorphism;
mod kink;
mod problems;
mod startingpoint;
pub mod utils;
mod weight;

#[derive(Debug)]
enum NodeType {
    MoveTo,
    LineTo,
    QuadTo,
    OffCurve,
    CurveTo,
    ClosePath,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GlyfPoint {
    pub point: Point,
    pub is_control: bool,
}
impl GlyfPoint {
    fn offcurve(pt: Point) -> Self {
        Self {
            point: pt,
            is_control: false,
        }
    }
    fn oncurve(pt: Point) -> Self {
        Self {
            point: pt,
            is_control: true,
        }
    }
}

/// A glyph at a given location, containing per-contour information
///
/// The easiest way to construct a glyph for testing is to start with
/// a [BezGlyph] and call `into()` on it, then modify its master name
/// and index.
///
/// Once you have two glyphs, you can test their interpolability by
/// passing them to `run_tests`.
#[derive(Default)]
pub struct Glyph {
    pub master_name: String,
    pub master_index: usize,
    // types: Vec<Vec<NodeType>>,
    pub curves: Vec<BezPath>,
    green_stats: Vec<greencurves::GreenStatistics>,
    control_stats: Vec<greencurves::ControlStatistics>,
    green_vectors: Vec<Vec<f64>>,
    control_vectors: Vec<Vec<f64>>,
    pub points: Vec<Vec<GlyfPoint>>,
    isomorphisms: Vec<Isomorphisms>,
}

impl Glyph {
    fn new() -> Self {
        Self::default()
    }
}

fn stats_to_vectors(stats: &dyn CurveStatistics) -> Vec<f64> {
    let area = stats.area();
    let com = stats.center_of_mass();
    let size = area.abs().sqrt();
    let stdev = stats.stddev();
    vec![
        size.copysign(area),
        com.x,
        com.y,
        stdev.x * 2.0,
        stdev.y * 2.0,
        stats.correlation() * size,
    ]
}

impl From<BezGlyph> for Glyph {
    fn from(val: BezGlyph) -> Self {
        let mut glyph = Glyph::new();
        for path in val.0 {
            let green_stats = path.green_statistics();
            let control_stats = path.control_statistics();
            glyph.green_vectors.push(stats_to_vectors(&green_stats));
            glyph.control_vectors.push(stats_to_vectors(&control_stats));
            glyph.green_stats.push(green_stats);
            glyph.control_stats.push(control_stats);
            let mut points = vec![];
            let mut types = vec![];
            for el in path.iter() {
                match el {
                    kurbo::PathEl::MoveTo(p) => {
                        points.push(GlyfPoint::oncurve(p));
                        types.push(NodeType::MoveTo);
                    }
                    kurbo::PathEl::LineTo(p) => {
                        points.push(GlyfPoint::oncurve(p));
                        types.push(NodeType::LineTo);
                    }
                    kurbo::PathEl::QuadTo(p0, p1) => {
                        points.push(GlyfPoint::offcurve(p0));
                        types.push(NodeType::OffCurve);
                        points.push(GlyfPoint {
                            point: p1,
                            is_control: true,
                        });
                        types.push(NodeType::QuadTo);
                    }
                    kurbo::PathEl::CurveTo(p0, p1, p2) => {
                        points.push(GlyfPoint::offcurve(p0));
                        types.push(NodeType::OffCurve);
                        points.push(GlyfPoint::offcurve(p1));
                        types.push(NodeType::OffCurve);
                        points.push(GlyfPoint {
                            point: p2,
                            is_control: true,
                        });
                        types.push(NodeType::CurveTo);
                    }
                    kurbo::PathEl::ClosePath => {
                        types.push(NodeType::ClosePath);
                    }
                }
            }
            if points.first() == points.last() && points.len() > 1 {
                points.pop();
                types.pop();
            }

            glyph.isomorphisms.push(Isomorphisms::new(&points));
            glyph.points.push(points);
            glyph.curves.push(path);
        }
        glyph
    }
}

#[cfg(feature = "skrifa")]
impl Glyph {
    pub fn new_from_font(
        font: &FontRef,
        glyph_id: GlyphId,
        location: &[VariationSetting],
    ) -> Option<Self> {
        let collection = font.outline_glyphs();
        let loc = font.axes().location(location);
        let outlined = collection.get(glyph_id)?;
        let settings =
            skrifa::outline::DrawSettings::unhinted(skrifa::prelude::Size::unscaled(), &loc);
        let mut bezglyph = BezGlyph::default();
        outlined.draw(settings, &mut bezglyph).ok()?;
        let mut glyph: Glyph = bezglyph.into();
        glyph.master_name = location
            .iter()
            .map(|x| format!("{}={}", x.selector, x.value))
            .join(" ");
        Some(glyph)
    }
}

/// The main interpolatability testing function
///
/// Returns a list of [Problem]s, which are serializable and can be
/// converted to JSON.
///
/// Arguments:
///
/// * `glyph_a` - the first glyph to test
/// * `glyph_b` - the second glyph to test
/// * `tolerance` - the maximum tolerance for problems; defaults to 0.95
/// * `kinkiness` - the maximum tolerance for kinks; defaults to 0.5
/// * `upem` - the UPEM value; defaults to 1000
pub fn run_tests<'a>(
    glyph_a: &'a Glyph,
    glyph_b: &'a Glyph,
    tolerance: Option<f64>,
    kinkiness: Option<f64>,
    upem: Option<u16>,
) -> Vec<Problem> {
    let tolerance = tolerance.unwrap_or(0.95);
    let mut problems = vec![];

    problems.extend(basiccompat::test_compatibility(glyph_a, glyph_b));

    if !problems.is_empty() {
        return problems;
    }

    let (contour_tolerance, matching) = contourorder::test_contour_order(glyph_a, glyph_b);
    if let Some(matching) = matching.as_ref() {
        if contour_tolerance < tolerance {
            problems.push(Problem::contour_order(
                glyph_a,
                glyph_b,
                tolerance,
                (0..matching.len()).collect::<Vec<usize>>(),
                matching.iter().map(|x| x.column).collect(),
            ));
        }
    }
    let m0_isomorphisms = &glyph_a.isomorphisms;
    let m0_vectors = &glyph_a.green_vectors;
    let m0_curves = &glyph_a.curves;
    let m0_points = &glyph_a.points;

    let (m1_isomorphisms, m1_vectors, m1_curves, m1_points) =
        if let Some(matching) = matching.as_ref() {
            (
                &matching.reorder(&glyph_b.isomorphisms),
                &matching.reorder(&glyph_b.green_vectors),
                &matching.reorder(&glyph_b.curves),
                &matching.reorder(&glyph_b.points),
            )
        } else {
            (
                &glyph_b.isomorphisms,
                &glyph_b.green_vectors,
                &glyph_b.curves,
                &glyph_b.points,
            )
        };
    let midpoint_interpolations: Vec<Option<BezPath>> = m0_curves
        .iter()
        .zip(m1_curves.iter())
        .map(|(c0, c1)| lerp_curve(c0, c1))
        .collect();

    for (ix, (contour_0, contour_1)) in m0_isomorphisms
        .iter()
        .zip(m1_isomorphisms.iter())
        .enumerate()
    {
        if contour_0.len() == 0 || contour_1.len() != contour_1.len() {
            continue;
        }
        if let Some((this_tolerance, proposed_point, reverse)) = test_starting_point(
            glyph_b, contour_0, contour_1, m0_vectors, m1_vectors, ix, tolerance,
        ) {
            if this_tolerance < tolerance {
                problems.push(Problem::wrong_start_point(
                    glyph_a,
                    glyph_b,
                    this_tolerance,
                    ix,
                    proposed_point,
                    reverse,
                ));
            }
        }
        if let Some(Some(mid)) = midpoint_interpolations.get(ix) {
            problems.extend(weight::test_over_underweight(
                glyph_a,
                glyph_b,
                &m0_vectors[ix],
                &m1_vectors[ix],
                mid,
                tolerance,
                ix,
            ));
        }

        problems.extend(kink::test_kink(
            glyph_a,
            glyph_b,
            &m0_points[ix],
            &m1_points[ix],
            ix,
            tolerance,
            kinkiness,
            upem,
        ));
    }

    problems
}

#[cfg(test)]
#[cfg(feature = "skrifa")]
mod tests {
    #![allow(clippy::expect_used)]
    #![allow(clippy::unwrap_used)]
    use serde_json::json;
    use skrifa::{FontRef, MetadataProvider};

    use super::*;

    #[test]
    fn test_stuff() {
        let fontdata = include_bytes!("../NotoSerif-Italic.ttf");
        let font = FontRef::new(fontdata).expect("Can't parse font");
        let glyph_id = font.charmap().map('C').unwrap();
        assert_eq!(glyph_id.to_u32(), 38);
        let interpolatable_glyph = Glyph::new_from_font(&font, glyph_id, &[]).expect("Fail");

        assert_eq!(
            interpolatable_glyph.green_vectors[0],
            vec![
                -347.58620033981043,
                281.0901509455822,
                365.18227893066177,
                342.7222793630012,
                485.41182923392773,
                120.83807623607606
            ]
        );
    }

    #[test]
    fn test_contour_order() {
        let fontdata = include_bytes!("../variable_ttf/TwisterTest-VF.ttf");
        let font = FontRef::new(fontdata).expect("Can't parse font");
        let glyph_id = font.charmap().map('A').unwrap();
        let glyph1 = Glyph::new_from_font(&font, glyph_id, &[]).expect("Fail");
        let glyph2 =
            Glyph::new_from_font(&font, glyph_id, &[("wght", 800.0).into()]).expect("Fail");
        let problems = run_tests(&glyph1, &glyph2, None, None, None);
        assert_eq!(problems.len(), 1);
        let problem = serde_json::to_value(&problems[0]).unwrap();
        let problem = problem.as_object().unwrap();
        assert_eq!(problem["type"], "ContourOrder");
        assert_eq!(problem["value_1"], json!([0, 1, 2]));
        assert_eq!(problem["value_2"], json!([2, 1, 0]));
    }

    #[test]
    fn test_isomorphisms() {
        let fontdata = include_bytes!("../NotoSerif-Italic.ttf");
        let font = FontRef::new(fontdata).expect("Can't parse font");
        let glyph_id = font.charmap().map('b').unwrap();
        let interpolatable_glyph = Glyph::new_from_font(&font, glyph_id, &[]).expect("Fail");
        assert_eq!(interpolatable_glyph.points[0].len(), 42);
        assert_eq!(interpolatable_glyph.isomorphisms.len(), 2);
        assert_eq!(interpolatable_glyph.isomorphisms[0].len(), 1);
        assert_eq!(interpolatable_glyph.isomorphisms[1].len(), 2);
        assert_eq!(
            interpolatable_glyph.isomorphisms[0]
                .iter()
                .next()
                .unwrap()
                .rotated_list
                .len(),
            168
        );
        assert_eq!(
            interpolatable_glyph.isomorphisms[1]
                .iter()
                .next()
                .unwrap()
                .rotated_list
                .len(),
            108
        );
        let last = interpolatable_glyph.isomorphisms[1].iter().last().unwrap();
        assert_eq!(last.rotated_list.len(), 108);
        assert_eq!(last.rotation, 18);
    }
}
