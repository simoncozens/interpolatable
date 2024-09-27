use crate::{problems::Problem, Glyph};

pub(crate) fn test_compatibility<'a>(glyph1: &'a Glyph, glyph2: &'a Glyph) -> Vec<Problem> {
    let mut problems = vec![];
    if glyph1.curves.len() != glyph2.curves.len() {
        problems.push(Problem::path_count(
            glyph1,
            glyph2,
            glyph1.curves.len(),
            glyph2.curves.len(),
        ));
    }
    for (path_index, (p1, p2)) in glyph1.points.iter().zip(glyph2.points.iter()).enumerate() {
        if p1.len() != p2.len() {
            problems.push(Problem::node_count(
                glyph1,
                glyph2,
                path_index,
                p1.len(),
                p2.len(),
            ));
        }
        for (node_index, (point1, point2)) in p1.iter().zip(p2.iter()).enumerate() {
            if point1.is_control != point2.is_control {
                problems.push(Problem::node_incompatibility(
                    glyph1,
                    glyph2,
                    path_index,
                    node_index,
                    point1.is_control,
                    point2.is_control,
                ));
            }
        }
    }
    problems
}
