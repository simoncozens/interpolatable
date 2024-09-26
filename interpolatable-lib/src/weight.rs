use greencurves::ComputeGreenStatistics;
use kurbo::BezPath;

use crate::{problems::Problem, stats_to_vectors, Glyph};

pub(crate) fn test_over_underweight<'a>(
    glyph_a: &'a Glyph,
    glyph_b: &'a Glyph,
    m0_vector: &[f64],
    m1_vector: &[f64],
    mid: &BezPath,
    tolerance: f64,
    ix: usize,
) -> Vec<Problem<'a>> {
    let mut problems = vec![];
    if (m0_vector[0] < 0.0) == (m1_vector[0] < 0.0) {
        return problems;
    }
    let mid_stats = stats_to_vectors(&mid.green_statistics());
    let size0 = m0_vector[0] * m0_vector[0];
    let size1 = m1_vector[0] * m1_vector[0];
    let mid_size = mid_stats[0] * mid_stats[0];

    // Check for overweight
    let expected = size0.max(size1);
    if 1e-5f64 + expected / tolerance < mid_size {
        let this_tolerance = if mid_size == 0.0 {
            0.0
        } else {
            expected / mid_size
        };
        problems.push(Problem::overweight(
            glyph_a,
            glyph_b,
            ix,
            this_tolerance,
            size0,
            size1,
        ));
    }

    // Check for underweight
    let expected = (size0 * size1).sqrt();
    if expected * tolerance > mid_size + 1e-5f64 {
        let this_tolerance = if expected == 0.0 {
            0.0
        } else {
            mid_size / expected
        };
        problems.push(Problem::underweight(
            glyph_a,
            glyph_b,
            ix,
            this_tolerance,
            size0,
            size1,
        ));
    }
    problems
}
