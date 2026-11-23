//! A library for computing statistics on paths
//!
//! This library provides methods for computing statistics on paths, such as the area, center of mass, variance, covariance, correlation, and slant.
//!
//! It implements two mechanisms for computing statistics, one based on Green's theorem, and
//! the other using the control only. The library is a straight port of the Python library
//! `fontTools.pens.statisticsPen`.
//!
//! While it is expected to be used on [kurbo::BezPath] objects, it can be used on any object that
//! can iterate over [kurbo::PathEl] objects.
//!
//! # Example
//!
//! ```
//! use kurbo::BezPath;
//! use greencurves::{ComputeGreenStatistics, CurveStatistics};
//!
//! let b = BezPath::from_svg("M300 -10Q229 -10 173.5 19.0Q118 48 86.5 109.0Q55 170 55 265Q55 364 88.0 426.0Q121 488 177.5 517.0Q234 546 306 546Q347 546 385.0 537.5Q423 529 447 517L420 444Q396 453 364.0 461.0Q332 469 304 469Q146 469 146 266Q146 169 184.5 117.5Q223 66 299 66Q343 66 376.5 75.0Q410 84 438 97V19Q411 5 378.5 -2.5Q346 -10 300 -10Z").expect("Failed to parse path");
//! let stats = b.green_statistics();
//!
//! use approx::assert_relative_eq;
//!
//! assert_relative_eq!(stats.center_of_mass().x, 214.4132814627106, epsilon = f64::EPSILON);
//! assert_relative_eq!(stats.center_of_mass().y, 267.5738980976807, epsilon = f64::EPSILON);
//! assert_relative_eq!(stats.variance().x, 11909.914244819694, epsilon = f64::EPSILON);
//! assert_relative_eq!(stats.variance().y, 34930.81282036622, epsilon = f64::EPSILON);
//! assert_relative_eq!(stats.covariance(), 123.24645984253584, epsilon = f64::EPSILON);
//! assert_relative_eq!(stats.correlation(), 0.006042487913362581, epsilon = f64::EPSILON);
//! assert_relative_eq!(stats.slant(), 0.0035283020889418774, epsilon = f64::EPSILON);
//! ```
pub use control::ControlStatistics;
pub use green::GreenStatistics;
use kurbo::{Point, Vec2};
mod control;
mod green;

/// Compute statistics on a path using the Green's theorem method
pub trait ComputeGreenStatistics<'a> {
    /// Compute statistics for the curve using the Green's theorem method
    fn green_statistics(&'a self) -> GreenStatistics;
}

/// Compute statistics on a path using the control polygon method
pub trait ComputeControlStatistics<'a> {
    /// Compute statistics for the curve using the control polygon method
    fn control_statistics(&'a self) -> ControlStatistics;
}

/// Statistics for a curve returned by either of the two methods
pub trait CurveStatistics {
    /// Calculate the signed area of a path
    fn area(&self) -> f64;
    /// Find the center of mass of the path
    fn center_of_mass(&self) -> Point;
    /// Find the variance of the path
    fn variance(&self) -> Vec2;
    /// Find the covariance of the path
    fn covariance(&self) -> f64;

    /// Find the standard deviation of the path
    fn stddev(&self) -> Vec2 {
        let variance = self.variance();
        Vec2::new(variance.x.sqrt(), variance.y.sqrt())
    }

    /// Find the correlation of the path
    ///
    /// Uses the Pearson product-moment correlation coefficient
    /// from <https://en.wikipedia.org/wiki/Pearson_product-moment_correlation_coefficient>
    fn correlation(&self) -> f64 {
        let stddev = self.stddev();
        let correlation = (self.covariance() / (stddev.x * stddev.y)).clamp(-1.0, 1.0);
        if correlation.abs() > 0.001 {
            correlation
        } else {
            0.0
        }
    }

    #[allow(dead_code)]
    /// Find the slant of the path
    fn slant(&self) -> f64 {
        let slant = self.covariance() / self.variance().y;
        if slant.abs() > 0.001 {
            slant
        } else {
            0.0
        }
    }
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use kurbo::{BezPath, Shape};

    fn approx_eq_point(found: Point, x: f64, y: f64) {
        assert_relative_eq!(found.x, x, epsilon = f64::EPSILON);
        assert_relative_eq!(found.y, y, epsilon = f64::EPSILON);
    }

    #[test]
    fn test_green_slash() {
        /* Noto Sans Regular 'slash', i.e. all lines */
        let b = BezPath::from_svg("M362 714 96 0H10L276 714Z").expect("Failed to parse path");
        let stats = b.green_statistics();
        assert_relative_eq!(stats.area(), b.area(), epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_x, -11421144.0, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_y, -21921228.0, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_xx, -2524236568.0, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_xy, -5049189516.0, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_yy, -10434504528.0, epsilon = f64::EPSILON);
        approx_eq_point(stats.center_of_mass(), 186.0, 357.0);
        approx_eq_point(stats.variance().to_point(), 6512.666666666664, 42483.0);
        approx_eq_point(
            stats.stddev().to_point(),
            80.70109458158956,
            206.11404610069638,
        );
        assert_relative_eq!(stats.covariance(), 15827.0, epsilon = f64::EPSILON);
        assert_relative_eq!(
            stats.correlation(),
            0.9515061251689377,
            epsilon = f64::EPSILON
        );
        assert_relative_eq!(stats.slant(), 0.37254901960784315, epsilon = f64::EPSILON);
    }

    #[test]
    fn test_green_c() {
        /* Noto Sans Regular 'c', i.e. a single quad path */
        let b = BezPath::from_svg("M300 -10Q229 -10 173.5 19.0Q118 48 86.5 109.0Q55 170 55 265Q55 364 88.0 426.0Q121 488 177.5 517.0Q234 546 306 546Q347 546 385.0 537.5Q423 529 447 517L420 444Q396 453 364.0 461.0Q332 469 304 469Q146 469 146 266Q146 169 184.5 117.5Q223 66 299 66Q343 66 376.5 75.0Q410 84 438 97V19Q411 5 378.5 -2.5Q346 -10 300 -10Z").expect("Failed to parse path");
        let stats = b.green_statistics();
        assert_relative_eq!(stats.moment_x, -17521942.69999999, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_y, -21866250.44166668, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_xx, -4730220386.45952, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_xy, -4698486262.534222, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_yy, -8705398445.642557, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.area(), -81720.4166666667, epsilon = f64::EPSILON);
        approx_eq_point(stats.center_of_mass(), 214.4132814627106, 267.5738980976807);
        approx_eq_point(
            stats.variance().to_point(),
            11909.914244819694,
            34930.81282036622,
        );
        assert_relative_eq!(
            stats.covariance(),
            123.24645984253584,
            epsilon = f64::EPSILON
        );
        assert_relative_eq!(
            stats.correlation(),
            0.006042487913362581,
            epsilon = f64::EPSILON
        );
        assert_relative_eq!(stats.slant(), 0.0035283020889418774, epsilon = f64::EPSILON);
    }

    #[test]
    fn test_green_b() {
        /* Noto Sans Regular 'b', two paths, with a hole in the middle */
        let b = BezPath::from_svg("M173 575Q173 541 171.5 511.5Q170 482 168 465H173Q196 499 236.0 522.0Q276 545 339 545Q439 545 499.5 475.5Q560 406 560 268Q560 130 499.0 60.0Q438 -10 339 -10Q276 -10 236.0 13.0Q196 36 173 68H166L148 0H85V760H173ZM324 472Q239 472 206.0 423.0Q173 374 173 271V267Q173 168 205.5 115.5Q238 63 326 63Q398 63 433.5 116.0Q469 169 469 269Q469 472 324 472Z").expect("Failed to parse path");
        let stats = b.green_statistics();
        assert_relative_eq!(stats.moment_x, -41623081.73333333, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_y, -47608259.06666666, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_xx, -15411808308.351183, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_xy, -12141640687.237495, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.moment_yy, -21553901545.110718, epsilon = f64::EPSILON);
        assert_relative_eq!(stats.area(), b.area(), epsilon = f64::EPSILON);
    }

    #[test]
    fn test_control_c() {
        /* Noto Sans Regular 'c', i.e. a single quad path */
        let b = BezPath::from_svg("M300 -10Q229 -10 173.5 19.0Q118 48 86.5 109.0Q55 170 55 265Q55 364 88.0 426.0Q121 488 177.5 517.0Q234 546 306 546Q347 546 385.0 537.5Q423 529 447 517L420 444Q396 453 364.0 461.0Q332 469 304 469Q146 469 146 266Q146 169 184.5 117.5Q223 66 299 66Q343 66 376.5 75.0Q410 84 438 97V19Q411 5 378.5 -2.5Q346 -10 300 -10Z").expect("Failed to parse path");
        let stats = b.control_statistics();
        assert_relative_eq!(stats.area(), -77720.5, epsilon = f64::EPSILON);
        approx_eq_point(
            stats.center_of_mass(),
            270.3243243243243,
            253.52702702702703,
        );
        approx_eq_point(
            stats.variance().to_point(),
            16318.183558558554,
            47474.235360360355,
        );
        assert_relative_eq!(
            stats.covariance(),
            181.07432432432608,
            epsilon = f64::EPSILON
        );
        approx_eq_point(
            stats.stddev().to_point(),
            127.7426458100761,
            217.88583102248836,
        );
        assert_relative_eq!(
            stats.correlation(),
            0.006505669207299498,
            epsilon = f64::EPSILON
        );
        assert_relative_eq!(stats.slant(), 0.0038141598900931013, epsilon = f64::EPSILON);
    }
}
