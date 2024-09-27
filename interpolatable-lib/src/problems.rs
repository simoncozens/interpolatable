use serde::Serialize;

use crate::Glyph;

#[derive(Debug, Serialize)]
pub struct Problem {
    pub master_1_name: String,
    pub master_2_name: String,
    pub master_1_index: usize,
    pub master_2_index: usize,
    #[serde(flatten)]
    pub details: ProblemDetails,
    pub tolerance: Option<f64>,
    pub contour: Option<usize>,
    pub node: Option<usize>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ProblemDetails {
    PathCount {
        count_1: usize,
        count_2: usize,
    },
    NodeCount {
        count_1: usize,
        count_2: usize,
    },
    NodeIncompatibility {
        is_control_1: bool,
        is_control_2: bool,
    },
    ContourOrder {
        order_1: Vec<usize>,
        order_2: Vec<usize>,
    },
    WrongStartPoint {
        proposed_point: usize,
        reverse: bool,
    },
    Overweight {
        value_1: f64,
        value_2: f64,
    },
    Underweight {
        value_1: f64,
        value_2: f64,
    },
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
