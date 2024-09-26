use kurbo::{BezPath, Vec2};
use munkres::Position;

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

pub struct Matching(pub(crate) Vec<Position>);

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
    pub fn iter(&self) -> std::slice::Iter<Position> {
        self.0.iter()
    }
}
