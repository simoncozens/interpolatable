use std::ops::Add;

use crate::{ComputeControlStatistics, CurveStatistics};
use itertools::Itertools;
use kurbo::{PathEl, Point, Vec2};

#[derive(Debug, Default, Clone)]
pub struct ControlStatistics {
    points: Vec<Point>,
    total: Point, // A cache
}

impl Add<Self> for ControlStatistics {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut points = self.points;
        points.extend(rhs.points);
        let total = self.total + rhs.total.to_vec2();
        ControlStatistics { points, total }
    }
}

impl CurveStatistics for ControlStatistics {
    fn area(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        // Use the triangle formula
        self.points
            .iter()
            .circular_tuple_windows()
            .map(|(p0, p1)| p0.x * p1.y - p1.x * p0.y)
            .sum::<f64>()
            / 2.0
    }
    /// Find the center of mass of the path
    fn center_of_mass(&self) -> Point {
        Point::new(
            self.total.x / self.points.len() as f64,
            self.total.y / self.points.len() as f64,
        )
    }

    /// Find the variance of the path
    fn variance(&self) -> Vec2 {
        let len = self.points.len() as f64;
        if len <= 1.0 {
            return Vec2::ZERO;
        }

        let sum_squares = self.points.iter().fold(Point::ZERO, |total, p| {
            Point::new(total.x + p.x * p.x, total.y + p.y * p.y)
        });
        Vec2::new(
            (sum_squares.x - (self.total.x * self.total.x) / len) / (len - 1.0),
            (sum_squares.y - (self.total.y * self.total.y) / len) / (len - 1.0),
        )
    }

    /// Find the covariance of the path
    fn covariance(&self) -> f64 {
        let sum_xy = self.points.iter().fold(0.0, |total, p| total + p.x * p.y);
        let len = self.points.len() as f64;
        (sum_xy - self.total.x * self.total.y / len) / (len - 1.0)
    }
}

impl ControlStatistics {
    #[allow(dead_code)]
    pub fn new(points: Vec<Point>) -> Self {
        let total = points.iter().fold(Point::ZERO, |total, p| {
            Point::new(total.x + p.x, total.y + p.y)
        });
        ControlStatistics { points, total }
    }
}

impl<'a, T: 'a> ComputeControlStatistics<'a> for T
where
    &'a T: IntoIterator<Item = PathEl>,
{
    fn control_statistics(&'a self) -> ControlStatistics {
        let mut statistics = ControlStatistics::default();
        for el in self {
            match el {
                PathEl::MoveTo(p) => {
                    statistics.points.push(p);
                }
                PathEl::LineTo(p) => {
                    statistics.points.push(p);
                }
                PathEl::QuadTo(p1, p2) => {
                    statistics.points.push(p1);
                    statistics.points.push(p2);
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    statistics.points.push(p1);
                    statistics.points.push(p2);
                    statistics.points.push(p3);
                }
                PathEl::ClosePath => {}
            }
        }
        statistics.total = statistics.points.iter().fold(Point::ZERO, |total, p| {
            Point::new(total.x + p.x, total.y + p.y)
        });
        statistics
    }
}
