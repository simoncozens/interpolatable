use serde::Serialize;

use crate::Glyph;

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Problem<'a> {
    PathCount {
        master_1_name: &'a str,
        master_2_name: &'a str,
        master_1_index: usize,
        master_2_index: usize,
        count_1: usize,
        count_2: usize,
    },
    NodeCount {
        master_1_name: &'a str,
        master_2_name: &'a str,
        master_1_index: usize,
        master_2_index: usize,
        path_index: usize,
        count_1: usize,
        count_2: usize,
    },
    NodeIncompatibility {
        master_1_name: &'a str,
        master_2_name: &'a str,
        master_1_index: usize,
        master_2_index: usize,
        path_index: usize,
        node_index: usize,
        is_control_1: bool,
        is_control_2: bool,
    },
    ContourOrder {
        master_1_name: &'a str,
        master_2_name: &'a str,
        master_1_index: usize,
        master_2_index: usize,
        tolerance: f64,
        order_1: Vec<usize>,
        order_2: Vec<usize>,
    },
    WrongStartPoint {
        master_1_name: &'a str,
        master_2_name: &'a str,
        master_1_index: usize,
        master_2_index: usize,
        tolerance: f64,
        contour: usize,
        proposed_point: usize,
        reverse: bool,
    },
    Overweight {
        master_1_name: &'a str,
        master_2_name: &'a str,
        contour: usize,
        tolerance: f64,
        value_1: f64,
        value_2: f64,
    },
    Underweight {
        master_1_name: &'a str,
        master_2_name: &'a str,
        contour: usize,
        tolerance: f64,
        value_1: f64,
        value_2: f64,
    },
    Kink {
        master_1_name: &'a str,
        master_2_name: &'a str,
        master_1_index: usize,
        master_2_index: usize,
        contour: usize,
        node: usize,
        tolerance: f64,
    },
}

impl Problem<'_> {
    pub(crate) fn path_count<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        count_1: usize,
        count_2: usize,
    ) -> Problem<'a> {
        Problem::PathCount {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            count_1,
            count_2,
        }
    }

    pub(crate) fn node_count<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        path_index: usize,
        count_1: usize,
        count_2: usize,
    ) -> Problem<'a> {
        Problem::NodeCount {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            path_index,
            count_1,
            count_2,
        }
    }

    pub(crate) fn node_incompatibility<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        path_index: usize,
        node_index: usize,
        is_control_1: bool,
        is_control_2: bool,
    ) -> Problem<'a> {
        Problem::NodeIncompatibility {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            path_index,
            node_index,
            is_control_1,
            is_control_2,
        }
    }

    pub(crate) fn contour_order<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        tolerance: f64,
        order_1: Vec<usize>,
        order_2: Vec<usize>,
    ) -> Problem<'a> {
        Problem::ContourOrder {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            tolerance,
            order_1,
            order_2,
        }
    }

    pub(crate) fn wrong_start_point<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        tolerance: f64,
        contour: usize,
        proposed_point: usize,
        reverse: bool,
    ) -> Problem<'a> {
        Problem::WrongStartPoint {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            tolerance,
            contour,
            proposed_point,
            reverse,
        }
    }

    pub(crate) fn overweight<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        contour: usize,
        tolerance: f64,
        value_1: f64,
        value_2: f64,
    ) -> Problem<'a> {
        Problem::Overweight {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            contour,
            tolerance,
            value_1,
            value_2,
        }
    }

    pub(crate) fn underweight<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        contour: usize,
        tolerance: f64,
        value_1: f64,
        value_2: f64,
    ) -> Problem<'a> {
        Problem::Underweight {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            contour,
            tolerance,
            value_1,
            value_2,
        }
    }

    pub(crate) fn kink<'a>(
        g1: &'a Glyph,
        g2: &'a Glyph,
        contour: usize,
        node: usize,
        tolerance: f64,
    ) -> Problem<'a> {
        Problem::Kink {
            master_1_name: &g1.master_name,
            master_2_name: &g2.master_name,
            master_1_index: g1.master_index,
            master_2_index: g2.master_index,
            contour,
            node,
            tolerance,
        }
    }
}
