use munkres::{Position, Weights};

use crate::utils::{Matching, VdiffHypo2};
use crate::Glyph;

pub(crate) fn test_contour_order<'a>(
    glyph1: &'a Glyph,
    glyph2: &'a Glyph,
) -> (f64, Option<Matching>) {
    let n = glyph1.control_vectors.len();
    if n <= 1 {
        return (1.0, None);
    }

    let (matching_control, matching_cost_control, identity_cost_control) =
        matching_for_vectors(&glyph1.control_vectors, &glyph2.control_vectors);
    if matching_cost_control == identity_cost_control {
        return (1.0, None);
    }

    let (matching_green, matching_cost_green, identity_cost_green) =
        matching_for_vectors(&glyph1.green_vectors, &glyph2.green_vectors);
    if matching_cost_green == identity_cost_green {
        return (1.0, None);
    }

    // Maybe they're OK, but the contours are reversed.
    let g2_control_reversed: Vec<Vec<f64>> = glyph2
        .control_vectors
        .iter()
        .map(|v| {
            // Reverse the sign of the first element
            let mut v = v.clone();
            v[0] = -v[0];
            v
        })
        .collect();
    let (_, matching_cost_control_reversed, identity_cost_control_reversed) =
        matching_for_vectors(&glyph1.control_vectors, &g2_control_reversed);
    if matching_cost_control_reversed == identity_cost_control_reversed {
        return (1.0, None);
    }

    let g2_green_reversed = glyph2
        .green_vectors
        .iter()
        .map(|v| {
            // Reverse the sign of the first element
            let mut v = v.clone();
            v[0] = -v[0];
            v
        })
        .collect();
    let (_, matching_cost_green_reversed, identity_cost_green_reversed) =
        matching_for_vectors(&glyph1.green_vectors, &g2_green_reversed);
    if matching_cost_green_reversed == identity_cost_green_reversed {
        return (1.0, None);
    }

    // Use the worst of the two matchings
    let (matching, matching_cost, identity_cost) = if matching_cost_control / identity_cost_control
        < matching_cost_green / identity_cost_green
    {
        (
            matching_control,
            matching_cost_control,
            identity_cost_control,
        )
    } else {
        (matching_green, matching_cost_green, identity_cost_green)
    };
    let this_tolerance = if identity_cost != 0.0 {
        matching_cost / identity_cost
    } else {
        1.0
    };
    // log::debug(
    //     "test-contour-order: tolerance %g",
    //     this_tolerance,
    // )
    (this_tolerance, Some(matching))
}

fn matching_for_vectors(m0: &Vec<Vec<f64>>, m1: &Vec<Vec<f64>>) -> (Matching, f64, f64) {
    assert!(m0.len() == m1.len());
    let mut weights = vec![];
    for v0 in m0 {
        for v1 in m1 {
            weights.push(v0.vdiff_hypot2(v1));
        }
    }
    let mut costs = munkres::WeightMatrix::from_row_vec(m0.len(), weights);
    let matching = munkres::solve_assignment(&mut costs).unwrap();
    let matching_cost = matching.iter().map(|pos| costs.element_at(*pos)).sum();
    let identity_cost = (0..m0.len())
        .map(|i| costs.element_at(Position { row: i, column: i }))
        .sum();
    (Matching(matching), matching_cost, identity_cost)
}
