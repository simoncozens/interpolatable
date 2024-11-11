use std::f64::consts::PI;

use kurbo::Affine;

use crate::{
    isomorphism::{Characteristic, Isomorphisms},
    utils::VdiffHypo2,
    Glyph,
};

pub(crate) fn test_starting_point(
    glyph_b: &Glyph,
    m0_isomorphisms: &Isomorphisms,
    m1_isomorphisms: &Isomorphisms,
    m0_vectors: &[Vec<f64>],
    m1_vectors: &[Vec<f64>],
    ix: usize,
    tolerance: f64,
) -> Option<(f64, usize, bool)> {
    let c0 = m0_isomorphisms.get(0)?;
    let costs: Vec<f64> = m1_isomorphisms
        .iter()
        .map(|c1| c0.rotated_list.vdiff_hypot2(&c1.rotated_list))
        .collect();
    let (mut min_index, mut min_cost) = costs
        .iter()
        .copied()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))?;
    let mut first_cost = *costs.first()?;
    let proposed_point = m1_isomorphisms.get(min_index)?.rotation;
    let reverse = m1_isomorphisms.get(min_index)?.reverse;
    if min_cost < first_cost * tolerance {
        // c0 is the first isomorphism of the m0 master
        // m1_isomorphisms is list of all isomorphisms of the m1 master
        //
        // If the two shapes are both circle-ish and slightly
        // rotated, we detect wrong start point. This is for
        // example the case hundreds of times in
        // RobotoSerif-Italic[GRAD,opsz,wdth,wght].ttf
        //
        // If the proposed point is only one off from the first
        // point (and not reversed), try harder:
        //
        // Find the major eigenvector of the covariance matrix,
        // and rotate the contours by that angle. Then find the
        // closest point again.  If it matches this time, let it
        // pass.
        let num_points = glyph_b.points.get(ix)?.len();
        let leeway = 3usize;
        if !reverse && (proposed_point <= leeway || proposed_point >= num_points - leeway) {
            // Recover the covariance matrix from the GreenVectors.
            let mut transforms = vec![];
            for vector in [m0_vectors.get(ix)?, m1_vectors.get(ix)?].iter() {
                let stddev_x = vector[3] * 0.5;
                let stddev_y = vector[4] * 0.5;
                let mut correlation = vector[5];
                if correlation != 0.0 {
                    correlation /= vector[0].abs();
                }
                // https://cookierobotics.com/007/
                let a = stddev_x * stddev_x;
                let c = stddev_y * stddev_y;
                let b = correlation * stddev_x * stddev_y;
                let delta = (((a - c) * 0.5).powi(2) + b * b).powf(0.5);
                let lambda1 = (a + c) * 0.5 + delta;
                let lambda2 = (a + c) * 0.5 - delta;
                let theta = if b != 0.0 {
                    (lambda1 - a).atan2(b)
                } else if a < c {
                    PI * 0.5
                } else {
                    0.0
                };
                let transform =
                    Affine::rotate(theta).then_scale_non_uniform(lambda1.sqrt(), lambda2.sqrt());
                transforms.push(transform);
            }
            let mut new_c0 = vec![];
            new_c0.push((transforms[0] * c0.rotated_list[0].to_point()).to_vec2());
            new_c0.extend(c0.rotated_list.iter().skip(1).copied());
            let new_contour1: Isomorphisms = Isomorphisms(
                m1_isomorphisms
                    .iter()
                    .map(|c1| {
                        let new_list = c1
                            .rotated_list
                            .iter()
                            .map(|p| (transforms[1] * p.to_point()).to_vec2())
                            .collect();
                        Characteristic {
                            rotated_list: new_list,
                            rotation: c1.rotation,
                            reverse: c1.reverse,
                        }
                    })
                    .collect(),
            );
            // Next few lines duplicate from above.
            let costs: Vec<f64> = new_contour1
                .iter()
                .map(|c1| new_c0.vdiff_hypot2(&c1.rotated_list))
                .collect();
            first_cost = *costs.first()?;
            (min_index, min_cost) = costs
                .iter()
                .copied()
                .enumerate()
                .min_by(|(_, a), (_, b)| a.total_cmp(b))?;
        }
    }
    let this_tolerance = if first_cost != 0.0 {
        min_cost / first_cost
    } else {
        1.0
    };
    Some((this_tolerance, min_index, reverse))
}
