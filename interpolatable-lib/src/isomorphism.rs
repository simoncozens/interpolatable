use kurbo::Vec2;

use crate::GlyfPoint;

#[derive(Debug, Clone)]
pub(crate) struct Characteristic {
    pub rotated_list: Vec<Vec2>,
    pub rotation: usize,
    pub reverse: bool,
}
#[derive(Default, Debug, Clone)]
pub(crate) struct Isomorphisms(pub(crate) Vec<Characteristic>);

fn points_complex_vector(points: Vec<GlyfPoint>) -> Vec<Vec2> {
    let mut vector = Vec::with_capacity(points.len() * 4);
    let len = points.len();
    let cycle_index = |x| x % len;
    for i in 0..points.len() {
        let pt0 = points[i].point;
        let pt1 = points[cycle_index(i + 1)].point;
        let pt2 = points[cycle_index(i + 2)].point;

        // The point itself
        vector.push(pt0.to_vec2());
        // The vector to the next point
        let d0 = pt1 - pt0;
        vector.push(d0 * 3.0);
        // The turn vector
        let d1 = pt2 - pt1;
        vector.push(d1 - d0);
        //  The angle to the next point, as a cross product
        let cross = d0.x * d1.y - d0.y * d1.x;
        vector.push(Vec2::new(cross.abs().sqrt().copysign(cross) * 4.0, 0.0)); // This is a plain float in Python;
    }
    vector
}

fn points_characteristic_bits<'a>(
    points: impl DoubleEndedIterator<Item = &'a GlyfPoint>,
) -> Vec<bool> {
    points.rev().map(|pt| pt.is_control).collect()
}

impl Isomorphisms {
    pub(crate) fn new(points: &[GlyfPoint]) -> Self {
        let points = if points.len() > 1
            && points.first() == points.last()
            && points.first().map(|x| x.is_control) == Some(true)
        {
            &points[0..points.len() - 1]
        } else {
            &points[0..points.len()]
        };
        let mut isomorphism = Self::default();
        isomorphism.add(points, false);
        isomorphism.add(points, true);
        isomorphism
    }

    fn add(&mut self, points: &[GlyfPoint], reverse: bool) {
        let reference_bits = points_characteristic_bits(points.iter());
        let n = points.len();
        let points: Vec<GlyfPoint> = if reverse {
            points.iter().rev().cloned().collect()
        } else {
            points.to_vec()
        };
        let mut bits = if reverse {
            points_characteristic_bits(points.iter())
        } else {
            reference_bits.clone()
        };
        let vector = points_complex_vector(points);
        assert_eq!(vector.len() % n, 0);
        let mult: usize = vector.len() / n;

        for i in 0..n {
            if bits == reference_bits {
                let rotations = i * mult;
                let mut rotated = vector.clone();
                rotated.rotate_left(rotations);
                self.0.push(Characteristic {
                    rotated_list: rotated,
                    rotation: if reverse { n - 1 - (i) } else { i },
                    reverse,
                });
            }
            bits.rotate_right(1);
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Characteristic> {
        self.0.iter()
    }

    pub fn get(&self, index: usize) -> Option<&Characteristic> {
        self.0.get(index)
    }
}
