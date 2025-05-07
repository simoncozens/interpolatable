use serde::Serialize;

use crate::Glyph;

#[derive(Debug, Serialize)]
/// An interpolation problem between two masters.
pub struct Problem {
    /// The name of the first master.
    pub master_1_name: String,
    /// The name of the second master.
    pub master_2_name: String,
    /// The index of the first master.
    pub master_1_index: usize,
    /// The index of the second master.
    pub master_2_index: usize,
    /// Describes the problem in detail.
    #[serde(flatten)]
    pub details: ProblemDetails,
    /// The tolerance for the problem, if applicable.
    pub tolerance: Option<f64>,
    /// The index of the contour in the glyph, if applicable.
    pub contour: Option<usize>,
    /// The index of the node in the contour, if applicable.
    pub node: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
/// The particular problem found
pub enum ProblemDetails {
    /// The number of paths in the two masters is different.
    PathCount {
        /// The number of paths in the first master.
        count_1: usize,
        /// The number of paths in the second master.
        count_2: usize,
    },
    /// The number of nodes in the two masters is different.
    NodeCount {
        /// The number of nodes in the first master.
        count_1: usize,
        /// The number of nodes in the second master.
        count_2: usize,
    },
    /// The nodes in the two masters are incompatible.
    NodeIncompatibility {
        /// Whether the node in the first master is a control point.
        is_control_1: bool,
        /// Whether the node in the second master is a control point.
        is_control_2: bool,
    },
    /// The order of the contours in the two masters is different.
    ContourOrder {
        /// The order of the contours in the first master.
        order_1: Vec<usize>,
        /// The order of the contours in the second master.
        order_2: Vec<usize>,
    },
    /// The start point of the contour in the two masters is different.
    WrongStartPoint {
        /// What the start point in the second master should be.
        proposed_point: usize,
        /// Whether the contour in the second master is reversed.
        reverse: bool,
    },
    /// The contour in the second master overweight compared to the first master.
    Overweight {
        /// The perceptual weight in the first master.
        value_1: f64,
        /// The perceptual weight in the second master.
        value_2: f64,
    },
    /// The contour in the second master underweight compared to the first master.
    Underweight {
        /// The perceptual weight in the first master.
        value_1: f64,
        /// The perceptual weight in the second master.
        value_2: f64,
    },
    /// The contour in the second master has a kink compared to the first master.
    Kink,
}

impl Problem {
    pub(crate) fn path_count(g1: &Glyph, g2: &Glyph, count_1: usize, count_2: usize) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            tolerance: None,
            contour: None,
            node: None,
            details: ProblemDetails::PathCount { count_1, count_2 },
        }
    }

    pub(crate) fn node_count(
        g1: &Glyph,
        g2: &Glyph,
        path_index: usize,
        count_1: usize,
        count_2: usize,
    ) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            tolerance: None,
            contour: Some(path_index),
            node: None,
            details: ProblemDetails::NodeCount { count_1, count_2 },
        }
    }

    pub(crate) fn node_incompatibility(
        g1: &Glyph,
        g2: &Glyph,
        contour: usize,
        node: usize,
        is_control_1: bool,
        is_control_2: bool,
    ) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            contour: Some(contour),
            node: Some(node),
            tolerance: None,
            details: ProblemDetails::NodeIncompatibility {
                is_control_1,
                is_control_2,
            },
        }
    }

    pub(crate) fn contour_order(
        g1: &Glyph,
        g2: &Glyph,
        tolerance: f64,
        order_1: Vec<usize>,
        order_2: Vec<usize>,
    ) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            tolerance: Some(tolerance),
            contour: None,
            node: None,
            details: ProblemDetails::ContourOrder { order_1, order_2 },
        }
    }

    pub(crate) fn wrong_start_point(
        g1: &Glyph,
        g2: &Glyph,
        tolerance: f64,
        contour: usize,
        proposed_point: usize,
        reverse: bool,
    ) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            tolerance: Some(tolerance),
            contour: Some(contour),
            node: None,
            details: ProblemDetails::WrongStartPoint {
                proposed_point,
                reverse,
            },
        }
    }

    pub(crate) fn overweight(
        g1: &Glyph,
        g2: &Glyph,
        contour: usize,
        tolerance: f64,
        value_1: f64,
        value_2: f64,
    ) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            contour: Some(contour),
            tolerance: Some(tolerance),
            node: None,
            details: ProblemDetails::Overweight { value_1, value_2 },
        }
    }

    pub(crate) fn underweight(
        g1: &Glyph,
        g2: &Glyph,
        contour: usize,
        tolerance: f64,
        value_1: f64,
        value_2: f64,
    ) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            contour: Some(contour),
            tolerance: Some(tolerance),
            details: ProblemDetails::Underweight { value_1, value_2 },
            node: None,
        }
    }

    pub(crate) fn kink(
        g1: &Glyph,
        g2: &Glyph,
        contour: usize,
        node: usize,
        tolerance: f64,
    ) -> Problem {
        Problem {
            master_1_name: g1.master_name.to_string(),
            master_2_name: g2.master_name.to_string(),
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            contour: Some(contour),
            node: Some(node),
            tolerance: Some(tolerance),
            details: ProblemDetails::Kink,
        }
    }

    /// Returns the type of problem as a string.
    pub fn problem_type(&self) -> String {
        match self.details {
            ProblemDetails::PathCount { .. } => "PathCount".to_string(),
            ProblemDetails::NodeCount { .. } => "NodeCount".to_string(),
            ProblemDetails::NodeIncompatibility { .. } => "NodeIncompatibility".to_string(),
            ProblemDetails::ContourOrder { .. } => "ContourOrder".to_string(),
            ProblemDetails::WrongStartPoint { .. } => "WrongStartPoint".to_string(),
            ProblemDetails::Overweight { .. } => "Overweight".to_string(),
            ProblemDetails::Underweight { .. } => "Underweight".to_string(),
            ProblemDetails::Kink => "Kink".to_string(),
        }
    }
}
