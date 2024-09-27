use crate::{problems::Problem, GlyfPoint, Glyph};

const T: f64 = 0.1;
const DEFAULT_KINKINESS_LENGTH: f64 = 0.002;
const DEFAULT_KINKINESS: f64 = 0.5;
const DEFAULT_UPEM: u16 = 1000;

#[allow(clippy::too_many_arguments)]
pub(crate) fn test_kink<'a>(
    glyph_a: &'a Glyph,
    glyph_b: &'a Glyph,
    contour0: &[GlyfPoint],
    contour1: &[GlyfPoint],
    ix: usize,
    tolerance: f64,
    kinkiness: Option<f64>,
    upem: Option<u16>,
) -> Vec<Problem> {
    let kinkiness = kinkiness.unwrap_or(DEFAULT_KINKINESS);
    let deviation_threshold =
        upem.unwrap_or(DEFAULT_UPEM) as f64 * DEFAULT_KINKINESS_LENGTH * DEFAULT_KINKINESS
            / kinkiness;
    let mut problems = vec![];

    for (i, (pt0, pt1)) in contour0.iter().zip(contour1.iter()).enumerate() {
        if !pt0.is_control || !pt1.is_control {
            continue;
        }
        let pt0_prev = &contour0[(i + contour0.len() - 1) % contour0.len()];
        let pt1_prev = &contour1[(i + contour1.len() - 1) % contour1.len()];
        let pt0_next = &contour0[(i + 1) % contour0.len()];
        let pt1_next = &contour1[(i + 1) % contour1.len()];
        if pt0_prev.is_control && pt1_prev.is_control {
            continue;
        }
        let d0_prev = pt0.point - pt0_prev.point;
        let d0_next = pt0_next.point - pt0.point;
        let d1_prev = pt1.point - pt1_prev.point;
        let d1_next = pt1_next.point - pt1.point;

        let sin_0 = d0_prev.cross(d0_next) / (d0_prev.length() * d0_next.length());
        let sin_1 = d1_prev.cross(d1_next) / (d1_prev.length() * d1_next.length());
        // No vector, not colinear, not smooth
        if sin_0.is_nan() || sin_1.is_nan() || sin_0.abs() > T || sin_1.abs() > T {
            continue;
        }

        // Check for sharp corners
        if d0_prev.dot(d0_next) < 0.0 || d1_prev.dot(d1_next) < 0.0 {
            continue;
        }

        // Are handle ratios similar enough?
        let ratio_0 = d0_prev.length() / (d0_prev.length() + d0_next.length());
        let ratio_1 = d1_prev.length() / (d1_prev.length() + d1_next.length());
        if (ratio_0 - ratio_1).abs() < T {
            continue;
        }

        let midpoint = pt0.point.lerp(pt1.point, 0.5);
        let mid_prev = pt0_prev.point.lerp(pt1_prev.point, 0.5);
        let mid_next = pt0_next.point.lerp(pt1_next.point, 0.5);
        let mid_d0 = midpoint - mid_prev;
        let mid_d1 = mid_next - midpoint;
        let sin_mid = mid_d0.cross(mid_d1) / (mid_d0.length() * mid_d1.length());
        if sin_mid.is_nan() || sin_mid.abs() * (tolerance * kinkiness) <= T {
            continue;
        }

        let cross = sin_mid * mid_d0.length() * mid_d1.length();
        let arc_len = mid_d0.length() + mid_d1.length();
        let deviation = (cross / arc_len).abs();
        if deviation < deviation_threshold {
            continue;
        }
        let deviation_ratio = deviation / arc_len;
        if deviation_ratio > T {
            continue;
        }

        let this_tolerance = T / (sin_mid.abs() * kinkiness);
        problems.push(Problem::kink(glyph_a, glyph_b, ix, i, this_tolerance));
    }
    problems
}
