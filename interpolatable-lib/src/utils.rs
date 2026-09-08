#[cfg(feature = "fontations")]
use std::collections::HashMap;

#[cfg(feature = "fontations")]
use fontdrasil::{
    coords::{
        CoordConverter, DesignCoord, NormalizedCoord, NormalizedLocation, UserCoord, UserLocation,
    },
    types::Axes,
};
use kurbo::{BezPath, Vec2};
use munkres::Position;
#[cfg(feature = "fontations")]
use skrifa::{
    raw::{ReadError, TableProvider},
    setting::VariationSetting,
    FontRef, GlyphId, MetadataProvider as _, Tag,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Matching(pub(crate) Vec<Position>);

impl Matching {
    pub fn reorder<T: Clone>(&self, data: &[T]) -> Vec<T> {
        let mut result = vec![];
        for pos in self.iter() {
            result.push(data[pos.column].clone());
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

    pub fn iter(&self) -> std::slice::Iter<'_, Position> {
        self.0.iter()
    }
}

#[cfg(feature = "fontations")]
/// A trait for denormalizing a location tuple into a friendly representation in userspace.
pub trait DenormalizeLocation {
    /// Given a normalized location tuple, turn it back into a friendly representation in userspace
    fn denormalize_location(&self, tuple: &[f32]) -> Result<UserLocation, ReadError>;
}

#[cfg(feature = "fontations")]
impl DenormalizeLocation for FontRef<'_> {
    fn denormalize_location(&self, tuple: &[f32]) -> Result<UserLocation, ReadError> {
        let per_axis_maps = if let Ok(segments) = self.avar().map(|x| x.axis_segment_maps()) {
            segments.iter().collect::<Result<Vec<_>, _>>()?
        } else {
            vec![]
        };
        let axes: Axes = self
            .axes()
            .iter()
            .enumerate()
            .map(|(ix, axis)| {
                let min = UserCoord::new(axis.min_value() as f64);
                let default = UserCoord::new(axis.default_value() as f64);
                let max = UserCoord::new(axis.max_value() as f64);
                #[allow(clippy::unwrap_used)]
                let mut fd_axis = fontdrasil::types::Axis {
                    converter: CoordConverter::default_normalization(min, default, max),
                    hidden: axis.is_hidden(),
                    tag: Tag::new_checked(axis.tag().to_string().as_bytes()).unwrap(),
                    name: "Doesn't matter".to_string(),
                    min,
                    default,
                    max,
                    localized_names: HashMap::new(), // Let's not
                };
                if let Some(map) = per_axis_maps.get(ix) {
                    let desired_mapping: Vec<(
                        fontdrasil::coords::Coord<fontdrasil::coords::UserSpace>,
                        fontdrasil::coords::Coord<fontdrasil::coords::DesignSpace>,
                    )> = map
                        .axis_value_maps
                        .iter()
                        .map(|mapping| {
                            let from = mapping.from_coordinate().to_f32();
                            let to = mapping.to_coordinate().to_f32();
                            // These are both normalized coordinates. Turn the `from` back into
                            // userspace using default normalization
                            let user_from =
                                NormalizedCoord::new(from as f64).to_user(&fd_axis.converter);
                            // Let's pretend design space is just normalized space
                            let design_to = DesignCoord::new(to as f64);
                            (user_from, design_to)
                        })
                        .collect();
                    let default_idx = desired_mapping
                        .iter()
                        .position(|(_, to)| to.to_f64() == 0.0)
                        .unwrap_or(0);
                    fd_axis.converter = CoordConverter::new(desired_mapping, default_idx)
                        .unwrap_or_else(|_| {
                            CoordConverter::default_normalization(min, default, max)
                        })
                }
                fd_axis
            })
            .collect();
        let mut normal_location = NormalizedLocation::new();
        for (axis, value) in axes.iter().zip(tuple.iter()) {
            normal_location.insert(axis.tag, NormalizedCoord::new(*value as f64));
        }
        normal_location
            .convert(&axes)
            .map_err(|_| ReadError::MalformedData("Axis conversion error"))
    }
}

#[cfg(feature = "fontations")]
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
            font.denormalize_location(&tuple).map(|loc|
                // Now back to Skrifa
                loc.iter()
                    .map(|(tag, value)| VariationSetting {
                        selector: *tag,
                        value: value.to_f64() as f32,
                    })
                    .collect::<Vec<_>>())
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
