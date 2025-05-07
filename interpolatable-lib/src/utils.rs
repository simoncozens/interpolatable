use kurbo::{BezPath, Vec2};
use munkres::Position;
#[cfg(feature = "skrifa")]
use skrifa::{
    raw::ReadError,
    raw::{tables::fvar::VariationAxisRecord, TableProvider},
    setting::VariationSetting,
    FontRef, GlyphId,
};

pub(crate) fn lerp_curve(c0: &BezPath, c1: &BezPath) -> Option<BezPath> {
    let mut new = BezPath::new();
    for (e0, e1) in c0.elements().iter().zip(c1.elements()) {
        match (e0, e1) {
            (kurbo::PathEl::MoveTo(p0), kurbo::PathEl::MoveTo(p1)) => {
                new.push(kurbo::PathEl::MoveTo(p0.lerp(*p1, 0.5)));
            }
            (kurbo::PathEl::LineTo(p0), kurbo::PathEl::LineTo(p1)) => {
                new.push(kurbo::PathEl::LineTo(p0.lerp(*p1, 0.5)));
            }
            (kurbo::PathEl::QuadTo(p0, p1), kurbo::PathEl::QuadTo(q0, q1)) => {
                new.push(kurbo::PathEl::QuadTo(p0.lerp(*q0, 0.5), p1.lerp(*q1, 0.5)));
            }
            (kurbo::PathEl::CurveTo(p0, p1, p2), kurbo::PathEl::CurveTo(q0, q1, q2)) => {
                new.push(kurbo::PathEl::CurveTo(
                    p0.lerp(*q0, 0.5),
                    p1.lerp(*q1, 0.5),
                    p2.lerp(*q2, 0.5),
                ));
            }
            (kurbo::PathEl::ClosePath, kurbo::PathEl::ClosePath) => {
                new.push(kurbo::PathEl::ClosePath);
            }
            _ => return None,
        }
    }
    Some(new)
}

pub(crate) trait VdiffHypo2 {
    fn vdiff_hypot2(&self, other: &Self) -> f64;
}

impl VdiffHypo2 for Vec<f64> {
    fn vdiff_hypot2(&self, other: &Self) -> f64 {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
    }
}

impl VdiffHypo2 for Vec<Vec2> {
    fn vdiff_hypot2(&self, other: &Self) -> f64 {
        self.iter()
            .zip(other.iter())
            .map(|(a, b)| (*a - *b).hypot2())
            .sum::<f64>()
    }
}

pub(crate) struct Matching(pub(crate) Vec<Position>);

impl Matching {
    pub fn reorder<T: Clone>(&self, data: &[T]) -> Vec<T> {
        let mut result = vec![];
        for pos in self.iter() {
            result.push(data[pos.row].clone());
        }
        result
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> std::slice::Iter<Position> {
        self.0.iter()
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(feature = "skrifa")]
fn poor_mans_denormalize(peak: f32, axis: &VariationAxisRecord) -> f32 {
    // Insert avar here

    if peak > 0.0 {
        lerp(
            axis.default_value().to_f32(),
            axis.max_value().to_f32(),
            peak,
        )
    } else {
        lerp(
            axis.default_value().to_f32(),
            axis.min_value().to_f32(),
            -peak,
        )
    }
}

#[cfg(feature = "skrifa")]
/// A trait for denormalizing a location tuple into a friendly representation in userspace.
pub trait DenormalizeLocation {
    /// Given a normalized location tuple, turn it back into a friendly representation in userspace
    fn denormalize_location(&self, tuple: &[f32]) -> Result<Vec<VariationSetting>, ReadError>;
}

#[cfg(feature = "skrifa")]
impl DenormalizeLocation for FontRef<'_> {
    fn denormalize_location(&self, tuple: &[f32]) -> Result<Vec<VariationSetting>, ReadError> {
        let all_axes = self.fvar()?.axes()?;
        Ok(all_axes
            .iter()
            .zip(tuple)
            .filter(|&(_axis, peak)| *peak != 0.0)
            .map(|(axis, peak)| {
                let value = poor_mans_denormalize(*peak, axis);
                (axis.axis_tag().to_string().as_str(), value).into()
            })
            .collect())
    }
}

#[cfg(feature = "skrifa")]
/// Find all the variations for a given glyph id.
///
/// Given a font and a glyph id, this function will return all the locations at
/// which the glyph is defined in the font. This includes all the locations
/// defined in the `gvar` table, as well as the default location.
pub fn glyph_variations(
    font: &FontRef,
    gid: GlyphId,
) -> Result<Vec<Vec<VariationSetting>>, ReadError> {
    let Some(variation_data) = font.gvar()?.glyph_variation_data(gid)? else {
        return Ok(vec![]);
    };

    let variations: Result<Vec<Vec<VariationSetting>>, ReadError> = variation_data
        .tuples()
        .map(|t| {
            let tuple: Vec<f32> = t.peak().values.iter().map(|v| v.get().to_f32()).collect();
            font.denormalize_location(&tuple)
        })
        .collect();
    let mut variations = variations?;
    // Sort by length of non-default locations, and then from min to max
    variations.sort_by(|a, b| {
        let a_nondefault = a.iter().filter(|v| v.value != 0.0).count();
        let b_nondefault = b.iter().filter(|v| v.value != 0.0).count();
        let length_ordering = a_nondefault.cmp(&b_nondefault);
        if length_ordering != std::cmp::Ordering::Equal {
            return length_ordering;
        }
        a.iter()
            .zip(b.iter())
            .fold(std::cmp::Ordering::Equal, |acc, (a, b)| {
                if acc != std::cmp::Ordering::Equal {
                    return acc;
                }
                a.selector.cmp(&b.selector)
            })
    });
    Ok(variations)
}
