use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use cairo::{Context, Error, FontSlant, FontWeight, Surface};
use fontations::skrifa::{
    self, setting::VariationSetting, string::StringId, FontRef, GlyphId, MetadataProvider,
};
use indexmap::IndexMap;
use interpolatable::{BezGlyph, Glyph, Problem};
use itertools::Itertools;
use kurbo::{Rect, Shape};

use crate::cairopen::CairoPen;

pub(crate) struct InterpolatablePlot<'a> {
    surface: &'a Surface,
    font: FontRef<'a>,
    locations: &'a [Vec<VariationSetting>],
    toc: HashMap<usize, String>,
    glyphname_to_id: HashMap<String, GlyphId>,
    width: f64,
    height: f64,
    page_number: usize,
}

impl<'a> InterpolatablePlot<'a> {
    pub fn new(
        surface: &'a Surface,
        font: FontRef<'a>,
        locations: &'a [Vec<VariationSetting>],
        glyphname_to_id: HashMap<String, GlyphId>,
        width: Option<f64>,
        height: Option<f64>,
    ) -> Self {
        let width = width.unwrap_or(InterpolatablePlot::WIDTH);
        let height = height.unwrap_or(InterpolatablePlot::HEIGHT);
        let page_number = 0;
        InterpolatablePlot {
            surface,
            font,
            locations,
            toc: HashMap::new(),
            glyphname_to_id,
            width,
            height,
            page_number,
        }
    }

    fn panel_width(&self) -> f64 {
        self.width / 2.0 - 3.0 * InterpolatablePlot::PAD
    }
    fn panel_height(&self) -> f64 {
        self.height / 2.0 - 6.0 * InterpolatablePlot::PAD
    }
    pub const WIDTH: f64 = 8.5 * 72.0;
    pub const HEIGHT: f64 = 11.0 * 72.0;
    const PAD: f64 = 0.1 * 72.0;
    const TITLE_FONT_SIZE: f64 = 24.0;
    const FONT_SIZE: f64 = 16.0;
    // const PAGE_NUMBER: f64 = 1.0;
    const HEAD_COLOR: (f64, f64, f64) = (0.3, 0.3, 0.3);
    const LABEL_COLOR: (f64, f64, f64) = (0.2, 0.2, 0.2);
    const BORDER_COLOR: (f64, f64, f64) = (0.9, 0.9, 0.9);
    // const BORDER_WIDTH: f64 = 0.5;
    const FILL_COLOR: (f64, f64, f64) = (0.8, 0.8, 0.8);
    const STROKE_COLOR: (f64, f64, f64) = (0.1, 0.1, 0.1);
    const STROKE_WIDTH: f64 = 1.0;
    // const ONCURVE_NODE_COLOR: (f64, f64, f64, f64) = (0.0, 0.8, 0.0, 0.7);
    // const ONCURVE_NODE_DIAMETER: f64 = 6.0;
    // const OFFCURVE_NODE_COLOR: (f64, f64, f64, f64) = (0.0, 0.5, 0.0, 0.7);
    // const OFFCURVE_NODE_DIAMETER: f64 = 4.0;
    // const HANDLE_COLOR: (f64, f64, f64, f64) = (0.0, 0.5, 0.0, 0.7);
    // const HANDLE_WIDTH: f64 = 0.5;
    const CORRECTED_START_POINT_COLOR: (f64, f64, f64, f64) = (0.0, 0.9, 0.0, 0.7);
    const CORRECTED_START_POINT_SIZE: f64 = 7.0;
    const WRONG_START_POINT_COLOR: (f64, f64, f64, f64) = (1.0, 0.0, 0.0, 0.7);
    const START_POINT_COLOR: (f64, f64, f64, f64) = (0.0, 0.0, 1.0, 0.7);
    const START_ARROW_LENGTH: f64 = 9.0;
    const KINK_POINT_SIZE: f64 = 7.0;
    const KINK_POINT_COLOR: (f64, f64, f64, f64) = (1.0, 0.0, 1.0, 0.7);
    const KINK_CIRCLE_SIZE: f64 = 15.0;
    const KINK_CIRCLE_STROKE_WIDTH: f64 = 1.0;
    const KINK_CIRCLE_COLOR: (f64, f64, f64, f64) = (1.0, 0.0, 1.0, 0.7);
    const CONTOUR_COLORS: [(f64, f64, f64, f64); 6] = [
        (1.0, 0.0, 0.0, 1.0),
        (0.0, 0.0, 1.0, 1.0),
        (0.0, 1.0, 0.0, 1.0),
        (1.0, 1.0, 0.0, 1.0),
        (1.0, 0.0, 1.0, 1.0),
        (0.0, 1.0, 1.0, 1.0),
    ];
    // const CONTOUR_ALPHA: f64 = 0.5;
    const WEIGHT_ISSUE_CONTOUR_COLOR: (f64, f64, f64, f64) = (0.0, 0.0, 0.0, 0.4);
    const NO_ISSUES_LABEL: &'static str = "Your font's good! Have a cupcake...";
    const NO_ISSUES_LABEL_COLOR: (f64, f64, f64) = (0.0, 0.5, 0.0);
    const CUPCAKE_COLOR: (f64, f64, f64) = (0.3, 0.0, 0.3);
    const CUPCAKE: &'static str = r"
                          ,@.
                        ,@.@@,.
                  ,@@,.@@@.  @.@@@,.
                ,@@. @@@.     @@. @@,.
        ,@@@.@,.@.              @.  @@@@,.@.@@,.
   ,@@.@.     @@.@@.            @,.    .@' @'  @@,
 ,@@. @.          .@@.@@@.  @@'                  @,
,@.  @@.                                          @,
@.     @,@@,.     ,                             .@@,
@,.       .@,@@,.         .@@,.  ,       .@@,  @, @,
@.                             .@. @ @@,.    ,      @
 @,.@@.     @,.      @@,.      @.           @,.    @'
  @@||@,.  @'@,.       @@,.  @@ @,.        @'@@,  @'
     \\@@@@'  @,.      @'@@@@'   @@,.   @@@' //@@@'
      |||||||| @@,.  @@' |||||||  |@@@|@||  ||
       \\\\\\\  ||@@@||  |||||||  |||||||  //
        |||||||  ||||||  ||||||   ||||||  ||
         \\\\\\  ||||||  ||||||  ||||||  //
          ||||||  |||||  |||||   |||||  ||
           \\\\\  |||||  |||||  |||||  //
            |||||  ||||  |||||  ||||  ||
             \\\\  ||||  ||||  ||||  //
              ||||||||||||||||||||||||
";
    const EMOTICON_COLOR: (f64, f64, f64) = (0.0, 0.3, 0.3);
    const SHRUG: &'static str = r#"\_(")_/"#;
    //     const UNDERWEIGHT: &'static str = r"
    //  o
    // /|\
    // / \
    // ";
    //     const OVERWEIGHT: &'static str = r"
    //  o
    // /O\
    // / \
    // ";
    // const YAY: &'static str = r" \o/ ";
}

impl Drop for InterpolatablePlot<'_> {
    fn drop(&mut self) {
        let _ = self.show_page();
        self.surface.finish();
    }
}
impl InterpolatablePlot<'_> {
    pub fn show_page(&mut self) -> Result<(), Error> {
        self.page_number += 1;
        cairo::Context::new(self.surface).unwrap().show_page()
    }

    pub fn add_title_page(
        &mut self,
        files: &[PathBuf],
        show_tolerance: Option<bool>,
        tolerance: Option<f64>,
        kinkiness: Option<f64>,
    ) -> Result<(), Error> {
        let pad = InterpolatablePlot::PAD;
        let width = self.width - 3.0 * pad;
        // let height = self.height - 2.0 * pad;
        let x = pad;
        let mut y = pad;

        self.draw_label(
            "Problem report for",
            x,
            y,
            None,
            0.5,
            true,
            Some(width),
            InterpolatablePlot::TITLE_FONT_SIZE,
        )?;
        y += InterpolatablePlot::TITLE_FONT_SIZE;

        for file in files {
            let basename = file.file_name().unwrap().to_str().unwrap();
            y += InterpolatablePlot::TITLE_FONT_SIZE + pad;
            self.draw_label(
                basename,
                x,
                y,
                None,
                0.5,
                true,
                Some(width),
                InterpolatablePlot::TITLE_FONT_SIZE,
            )?;
            y += InterpolatablePlot::TITLE_FONT_SIZE + pad;

            y = self.draw_font_sha(file, x, y, width)?;
            y = self.draw_font_family_name(file, x, y, width)?;
        }

        self.draw_legend(show_tolerance, tolerance, kinkiness)?;
        self.show_page()
    }

    fn draw_font_sha(&self, file: &Path, x: f64, y: f64, width: f64) -> Result<f64, Error> {
        let data = std::fs::read(file).unwrap();
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(&data);
        self.draw_label(
            &format!("SHA1: {}", hasher.digest()),
            x + InterpolatablePlot::PAD,
            y,
            None,
            0.5,
            false,
            Some(width),
            InterpolatablePlot::FONT_SIZE,
        )?;
        Ok(y + InterpolatablePlot::FONT_SIZE)
    }

    fn draw_font_family_name(&self, file: &Path, x: f64, y: f64, width: f64) -> Result<f64, Error> {
        let mut y = y;
        let data = std::fs::read(file).unwrap();
        let font = FontRef::new(&data).unwrap();
        let family_name = font
            .localized_strings(StringId::WWS_FAMILY_NAME)
            .english_or_first()
            .or_else(|| {
                font.localized_strings(StringId::TYPOGRAPHIC_FAMILY_NAME)
                    .english_or_first()
            })
            .or_else(|| {
                font.localized_strings(StringId::FAMILY_NAME)
                    .english_or_first()
            });
        let version = font
            .localized_strings(StringId::VERSION_STRING)
            .english_or_first();
        for (name, label) in [("Family", family_name), ("Version", version)] {
            if let Some(label) = label {
                self.draw_label(
                    &format!("{}: {}", name, label),
                    x + InterpolatablePlot::PAD,
                    y,
                    None,
                    0.5,
                    false,
                    Some(width),
                    InterpolatablePlot::FONT_SIZE,
                )?;
                y += InterpolatablePlot::FONT_SIZE + InterpolatablePlot::PAD;
            }
        }
        Ok(y)
    }

    fn draw_legend(
        &self,
        show_tolerance: Option<bool>,
        tolerance: Option<f64>,
        kinkiness: Option<f64>,
    ) -> Result<(), Error> {
        let pad = InterpolatablePlot::PAD;
        let font_size = InterpolatablePlot::FONT_SIZE;
        let x = pad;
        let mut y = self.height - pad - font_size * 2.0;
        let width = self.width - 2.0 * pad;

        let xx = x + pad * 2.0;
        let xxx = x + pad * 4.0;

        if let Some(true) = show_tolerance {
            self.draw_label(
                "Tolerance: badness; closer to zero the worse",
                xxx,
                y,
                None,
                0.0,
                false,
                Some(width),
                font_size,
            )?;
            y -= pad + font_size;
        }

        let labelled = |my_y, label, draw: &dyn Fn() -> Result<(), Error>| {
            self.draw_label(
                label,
                xxx,
                my_y,
                Some((0.0, 0.0, 0.0)),
                0.0,
                false,
                Some(width),
                font_size,
            )?;
            draw()
        };

        let cr = cairo::Context::new(self.surface)?;

        labelled(y, "Underweight contours", &|| {
            cr.rectangle(xx - pad * 0.7, y, 1.5 * pad, font_size);
            self.set_fill_stroke_source(
                &cr,
                Some(InterpolatablePlot::FILL_COLOR),
                Some(InterpolatablePlot::STROKE_COLOR),
                InterpolatablePlot::WEIGHT_ISSUE_CONTOUR_COLOR,
            )?;
            cr.fill()
        })?;
        y -= pad + font_size;

        labelled(
            y,
            "Colored contours: contours with the wrong order",
            &|| {
                cr.rectangle(xx - pad * 0.7, y, 1.5 * pad, font_size);
                self.set_fill_stroke_source(
                    &cr,
                    Some(InterpolatablePlot::FILL_COLOR),
                    Some(InterpolatablePlot::STROKE_COLOR),
                    InterpolatablePlot::CONTOUR_COLORS[0],
                )?;
                cr.fill()
            },
        )?;
        y -= pad + font_size;

        labelled(y, "Kink artifact", &|| {
            self.draw_circle(
                &cr,
                xx,
                y + font_size * 0.5,
                Some(InterpolatablePlot::KINK_CIRCLE_COLOR),
                InterpolatablePlot::KINK_CIRCLE_SIZE,
                InterpolatablePlot::KINK_CIRCLE_STROKE_WIDTH,
            )
        })?;
        y -= pad + font_size;

        labelled(y, "Point causing kink in the contour", &|| {
            self.draw_dot(
                &cr,
                xx,
                y + font_size * 0.5,
                Some(InterpolatablePlot::KINK_POINT_COLOR),
                InterpolatablePlot::KINK_POINT_SIZE,
            )
        })?;
        y -= pad + font_size;

        labelled(y, "Suggested new contour start point", &|| {
            self.draw_dot(
                &cr,
                xx,
                y + font_size * 0.5,
                Some(InterpolatablePlot::CORRECTED_START_POINT_COLOR),
                InterpolatablePlot::CORRECTED_START_POINT_SIZE,
            )
        })?;
        y -= pad + font_size;

        labelled(
            y,
            "Contour start point in contours with wrong direction",
            &|| {
                self.draw_arrow(
                    &cr,
                    xx - InterpolatablePlot::START_ARROW_LENGTH * 0.3,
                    y + font_size * 0.5,
                    Some(InterpolatablePlot::WRONG_START_POINT_COLOR),
                )
            },
        )?;
        y -= pad + font_size;

        labelled(
            y,
            "Contour start point when the first two points overlap",
            &|| {
                self.draw_dot(
                    &cr,
                    xx,
                    y + font_size * 0.5,
                    Some(InterpolatablePlot::START_POINT_COLOR),
                    InterpolatablePlot::CORRECTED_START_POINT_SIZE,
                )
            },
        )?;
        y -= pad + font_size;

        labelled(y, "Contour start point and direction", &|| {
            self.draw_arrow(
                &cr,
                xx - InterpolatablePlot::START_ARROW_LENGTH * 0.3,
                y + font_size * 0.5,
                Some(InterpolatablePlot::START_POINT_COLOR),
            )
        })?;
        y -= pad + font_size;

        self.draw_label("Legend:", x, y, None, 0.0, true, Some(width), font_size)?;
        y -= pad + font_size;

        if let Some(k) = kinkiness {
            self.draw_label(
                &format!("Kink-reporting aggressiveness: {}", k),
                xxx,
                y,
                None,
                0.0,
                false,
                Some(width),
                font_size,
            )?;
        }
        if let Some(k) = tolerance {
            self.draw_label(
                &format!("Error tolerance: {}", k),
                xxx,
                y,
                None,
                0.0,
                false,
                Some(width),
                font_size,
            )?;
        }
        self.draw_label("Parameters:", x, y, None, 0.0, true, Some(width), font_size)?;
        Ok(())
    }

    pub fn add_summary(&mut self, problems: &IndexMap<String, Vec<Problem>>) -> Result<(), Error> {
        let pad = InterpolatablePlot::PAD;
        let width = self.width - 3.0 * pad;
        let height = self.height - 2.0 * pad;
        let x = pad;
        let mut y = pad;
        let font_size = InterpolatablePlot::FONT_SIZE;
        self.draw_label(
            "Summary of problems",
            x,
            y,
            None,
            0.0,
            true,
            Some(width),
            InterpolatablePlot::TITLE_FONT_SIZE,
        )?;
        y += InterpolatablePlot::TITLE_FONT_SIZE;
        let mut glyph_per_problem = HashMap::new();
        for (glyph, problems) in problems {
            for problem in problems {
                let entry = glyph_per_problem
                    .entry(problem.problem_type())
                    .or_insert_with(Vec::new);
                entry.push(glyph);
            }
        }

        for (problem_type, glyphs) in glyph_per_problem.iter() {
            y += font_size;
            self.draw_label(
                &format!("{}: {}", problem_type, glyphs.len()),
                x,
                y,
                None,
                0.0,
                true,
                Some(width),
                font_size,
            )?;
            y += font_size;
            let mut glyphs = glyphs.clone();
            glyphs.sort();
            for glyphname in glyphs {
                if y + font_size > height {
                    self.show_page()?;
                    y = font_size + pad;
                }
                self.draw_label(
                    glyphname,
                    x + 2.0 * pad,
                    y,
                    None,
                    0.0,
                    false,
                    Some(width - 2.0 * pad),
                    font_size,
                )?;
                y += font_size;
            }
        }

        self.show_page()
    }

    fn add_listing(&mut self, title: &str, items: &[(usize, String)]) -> Result<(), Error> {
        let pad = InterpolatablePlot::PAD;
        let width = self.width - 2.0 * pad;
        let height = self.height - 2.0 * pad;
        let x = pad;
        let mut y = pad;
        self.draw_label(
            title,
            x,
            y,
            None,
            0.0,
            true,
            Some(width),
            InterpolatablePlot::TITLE_FONT_SIZE,
        )?;
        y += InterpolatablePlot::TITLE_FONT_SIZE + pad;
        let mut last_glyphname = None;
        for (pageno, glyphname) in items.iter() {
            if Some(glyphname) == last_glyphname {
                continue;
            }
            last_glyphname = Some(glyphname);
            if y + InterpolatablePlot::FONT_SIZE > height {
                self.show_page()?;
                y = InterpolatablePlot::FONT_SIZE + pad;
            }
            self.draw_label(
                glyphname,
                x + 5.0 * pad,
                y,
                None,
                0.0,
                true,
                Some(width - 2.0 * pad),
                InterpolatablePlot::FONT_SIZE,
            )?;
            self.draw_label(
                &format!("{}", pageno),
                x,
                y,
                None,
                1.0,
                false,
                Some(4.0 * pad),
                InterpolatablePlot::FONT_SIZE,
            )?;
            y += InterpolatablePlot::FONT_SIZE;
        }
        self.show_page()
    }

    pub fn add_table_of_contents(&mut self) -> Result<(), Error> {
        let mut toc = self
            .toc
            .iter()
            .map(|(page, title)| (*page, title.to_string()))
            .collect::<Vec<_>>();
        toc.sort();
        self.add_listing("Table of contents", &toc)
    }

    pub fn add_index(&mut self) -> Result<(), Error> {
        let mut index = self
            .toc
            .iter()
            .map(|(page, title)| (*page, title.to_string()))
            .collect::<Vec<_>>();
        index.sort_by_key(|(_, title)| title.to_lowercase());
        self.add_listing("Index", &index)
    }

    pub fn add_problems(&mut self, problems: &IndexMap<String, Vec<Problem>>) -> Result<(), Error> {
        for (glyph, problems) in problems {
            let mut last_masters = None;
            let mut current_glyph_problems = vec![];
            for problem in problems {
                let masters = vec![problem.master_1_index, problem.master_2_index];
                if Some(&masters) == last_masters.as_ref() {
                    current_glyph_problems.push(problem);
                    continue;
                }
                // Flush
                if !current_glyph_problems.is_empty() {
                    self.add_problem(glyph, &mut current_glyph_problems)?;
                    self.show_page()?;
                    current_glyph_problems.clear();
                }
                last_masters = Some(masters.clone());
                current_glyph_problems.push(problem);
            }
            if !current_glyph_problems.is_empty() {
                self.add_problem(glyph, &mut current_glyph_problems)?;
                self.show_page()?;
            }
        }
        Ok(())
    }

    fn add_problem(&mut self, glyphname: &str, problems: &mut Vec<&Problem>) -> Result<(), Error> {
        if problems.is_empty() {
            return Ok(());
        }
        self.toc.insert(self.page_number, glyphname.to_string());
        // let first_problem_type = problems[0].problem_type();
        let problem_types = problems
            .iter()
            .map(|problem| problem.problem_type())
            .collect::<HashSet<_>>();
        let title: String = problem_types.iter().join(", ");
        let pad = InterpolatablePlot::PAD;
        let mut x = pad;
        let mut y = pad;

        self.draw_label(
            &format!("Glyph name: {}", glyphname),
            x,
            y,
            Some(InterpolatablePlot::HEAD_COLOR),
            0.0,
            true,
            None,
            InterpolatablePlot::TITLE_FONT_SIZE,
        )?;

        let tolerance = problems
            .iter()
            .map(|p| p.tolerance.unwrap_or(1.0))
            .fold(1.0f64, |a, b| a.min(b));
        if tolerance < 1.0 {
            self.draw_label(
                &format!("Tolerance: {}", tolerance),
                x,
                y,
                None,
                1.0,
                true,
                None,
                InterpolatablePlot::FONT_SIZE,
            )?;
        }
        y += InterpolatablePlot::TITLE_FONT_SIZE + pad;
        self.draw_label(
            &format!("Problems: {}", title),
            x,
            y,
            None,
            0.0,
            true,
            Some(self.width - 2.0 * pad),
            InterpolatablePlot::FONT_SIZE,
        )?;
        y += InterpolatablePlot::FONT_SIZE + pad * 2.0;

        let mut scales = vec![];
        for (which, &master_idx) in [problems[0].master_1_index, problems[0].master_2_index]
            .iter()
            .enumerate()
        {
            let name = if which == 0 {
                &problems[0].master_1_name
            } else {
                &problems[0].master_2_name
            };
            self.draw_label(
                name,
                x,
                y,
                Some(InterpolatablePlot::LABEL_COLOR),
                0.5,
                false,
                Some(self.panel_width()),
                InterpolatablePlot::FONT_SIZE,
            )?;
            y += InterpolatablePlot::FONT_SIZE + pad;
            if let Some(location) = &self.locations.get(master_idx) {
                scales
                    .push(self.draw_glyph(location, glyphname, problems, which, x, y, None, false)?)
            } else {
                self.draw_emoticon(InterpolatablePlot::SHRUG, x, y)?;
            }
            y += self.panel_height() + InterpolatablePlot::FONT_SIZE + pad;
        }
        // XXX More here

        x = pad + self.panel_width() + pad;
        y = pad;
        y += InterpolatablePlot::TITLE_FONT_SIZE + 2.0 * pad;
        y += InterpolatablePlot::FONT_SIZE + pad;

        let midway_location = lerp_location(
            self.locations.get(problems[0].master_1_index).unwrap(),
            self.locations.get(problems[0].master_2_index).unwrap(),
            0.5,
        );
        self.draw_label(
            "midway interpolation",
            x,
            y,
            Some(InterpolatablePlot::HEAD_COLOR),
            0.5,
            false,
            Some(self.panel_width()),
            InterpolatablePlot::FONT_SIZE,
        )?;
        y += InterpolatablePlot::FONT_SIZE + pad;
        self.draw_glyph(
            &midway_location,
            glyphname,
            problems,
            0,
            x,
            y,
            Some(scales.iter().fold(f64::INFINITY, |a, &b| a.min(b))),
            true,
        )?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_glyph(
        &self,
        location: &Vec<VariationSetting>,
        glyphname: &str,
        problems: &[&Problem],
        _which: usize,
        x: f64,
        y: f64,
        scale: Option<f64>,
        midway: bool,
    ) -> Result<f64, Error> {
        let mut scale = scale;
        let glyph_id = self.glyphname_to_id.get(glyphname).unwrap();
        let outline = self.font.outline_glyphs().get(*glyph_id).unwrap();
        let loc = self.font.axes().location(location);
        // Make a bezglyph so we can find the bounds/scale
        let settings =
            skrifa::outline::DrawSettings::unhinted(skrifa::prelude::Size::unscaled(), &loc);
        let mut bezglyph = BezGlyph::default();
        outline.draw(settings, &mut bezglyph).unwrap(); // We made one before, so we know this works.
        let bounds = bezglyph
            .iter()
            .fold(None, |acc: Option<Rect>, curve| {
                let bounds = curve.bounding_box();
                if let Some(acc) = acc {
                    Some(acc.union(bounds))
                } else {
                    Some(bounds)
                }
            })
            .unwrap_or(Rect::ZERO);
        if bounds.width() > 0.0 {
            scale = if let Some(scale) = scale {
                Some(scale.min(self.panel_width() / bounds.width()))
            } else {
                Some(self.panel_width() / bounds.width())
            };
        }
        if bounds.height() > 0.0 {
            scale = if let Some(scale) = scale {
                Some(scale.min(self.panel_height() / bounds.height()))
            } else {
                Some(self.panel_height() / bounds.height())
            };
        }
        let scale = scale.unwrap_or(1.0);

        let cr = cairo::Context::new(self.surface)?;
        cr.translate(x, y);
        cr.translate(
            (self.panel_width() - bounds.width() * scale) / 2.0,
            self.panel_height() - bounds.height() * scale / 2.0,
        );
        cr.scale(scale, -scale);
        cr.translate(-bounds.min_x(), -bounds.min_y());

        let (r, g, b) = InterpolatablePlot::BORDER_COLOR;
        cr.set_source_rgb(r, g, b);
        cr.rectangle(
            bounds.min_x(),
            bounds.min_y(),
            bounds.width(),
            bounds.height(),
        );
        cr.stroke()?;
        let mut cairopen = CairoPen::new(&cr);
        let settings =
            skrifa::outline::DrawSettings::unhinted(skrifa::prelude::Size::unscaled(), &loc);

        outline.draw(settings, &mut cairopen).unwrap();
        let (r, g, b) = InterpolatablePlot::FILL_COLOR;
        cr.set_source_rgb(r, g, b);
        cr.fill_preserve()?;
        let (r, g, b) = InterpolatablePlot::STROKE_COLOR;
        cr.set_source_rgb(r, g, b);
        cr.set_line_width(InterpolatablePlot::STROKE_WIDTH / scale);
        cr.stroke()?;
        cr.new_path();

        // XX
        let glyph: Glyph = bezglyph.into();

        for problem in problems {
            // Just for kink
            if problem.problem_type() != "Kink" {
                continue;
            }
            let contour = problem.contour.unwrap();
            let point = problem.node.unwrap();
            let target = &glyph.points[contour][point].point;
            cr.save()?;
            cr.translate(target.x, target.y);
            cr.scale(1.0 / scale, 1.0 / scale);
            if midway {
                self.draw_circle(
                    &cr,
                    0.0,
                    0.0,
                    Some(InterpolatablePlot::KINK_CIRCLE_COLOR),
                    InterpolatablePlot::KINK_CIRCLE_SIZE,
                    InterpolatablePlot::KINK_CIRCLE_STROKE_WIDTH,
                )?;
            } else {
                self.draw_dot(
                    &cr,
                    0.0,
                    0.0,
                    Some(InterpolatablePlot::KINK_POINT_COLOR),
                    InterpolatablePlot::KINK_POINT_SIZE,
                )?;
            }
            cr.restore()?;
        }

        Ok(scale)
    }

    fn draw_dot(
        &self,
        cr: &Context,
        x: f64,
        y: f64,
        color: Option<(f64, f64, f64, f64)>,
        diameter: f64,
    ) -> Result<(), Error> {
        cr.save()?;
        cr.set_line_width(diameter);
        cr.set_line_cap(cairo::LineCap::Round);
        cr.move_to(x, y);
        cr.line_to(x, y);
        if let Some((red, green, blue, alpha)) = color {
            cr.set_source_rgba(red, green, blue, alpha);
        }
        cr.stroke()?;
        cr.restore()?;
        Ok(())
    }

    fn set_fill_stroke_source(
        &self,
        cr: &Context,
        fill_color: Option<(f64, f64, f64)>,
        stroke_color: Option<(f64, f64, f64)>,
        source_color: (f64, f64, f64, f64),
    ) -> Result<(), Error> {
        if let Some((red, green, blue)) = fill_color {
            cr.set_source_rgb(red, green, blue);
            cr.fill_preserve()?;
        }
        if let Some((red, green, blue)) = stroke_color {
            cr.set_source_rgb(red, green, blue);
            cr.stroke_preserve()?;
        }
        let (red, green, blue, alpha) = source_color;
        cr.set_source_rgba(red, green, blue, alpha);
        Ok(())
    }

    fn draw_circle(
        &self,
        cr: &Context,
        x: f64,
        y: f64,
        color: Option<(f64, f64, f64, f64)>,
        diameter: f64,
        stroke_width: f64,
    ) -> Result<(), Error> {
        cr.save()?;
        cr.set_line_width(stroke_width);
        cr.set_line_cap(cairo::LineCap::Square);
        cr.arc(x, y, diameter / 2.0, 0.0, 2.0 * std::f64::consts::PI);
        if let Some((red, green, blue, alpha)) = color {
            cr.set_source_rgba(red, green, blue, alpha);
        }
        cr.stroke()?;
        cr.restore()?;
        Ok(())
    }

    fn draw_arrow(
        &self,
        cr: &Context,
        x: f64,
        y: f64,
        color: Option<(f64, f64, f64, f64)>,
    ) -> Result<(), Error> {
        cr.save()?;
        if let Some((red, green, blue, alpha)) = color {
            cr.set_source_rgba(red, green, blue, alpha);
        }
        cr.translate(InterpolatablePlot::START_ARROW_LENGTH + x, y);
        cr.move_to(0.0, 0.0);
        cr.line_to(
            -InterpolatablePlot::START_ARROW_LENGTH,
            -InterpolatablePlot::START_ARROW_LENGTH * 0.4,
        );
        cr.line_to(
            -InterpolatablePlot::START_ARROW_LENGTH,
            InterpolatablePlot::START_ARROW_LENGTH * 0.4,
        );
        cr.close_path();
        cr.fill()?;
        cr.restore()?;
        Ok(())
    }

    pub fn draw_cupcake(&self) -> Result<(), Error> {
        self.draw_label(
            InterpolatablePlot::NO_ISSUES_LABEL,
            InterpolatablePlot::PAD,
            InterpolatablePlot::PAD,
            Some(InterpolatablePlot::NO_ISSUES_LABEL_COLOR),
            0.5,
            true,
            Some(InterpolatablePlot::WIDTH - 2.0 * InterpolatablePlot::PAD),
            InterpolatablePlot::TITLE_FONT_SIZE,
        )?;
        self.draw_text(
            InterpolatablePlot::CUPCAKE,
            InterpolatablePlot::PAD,
            InterpolatablePlot::PAD + InterpolatablePlot::FONT_SIZE,
            Some(InterpolatablePlot::CUPCAKE_COLOR),
            Some(InterpolatablePlot::WIDTH - 2.0 * InterpolatablePlot::PAD),
            Some(
                InterpolatablePlot::HEIGHT
                    - 2.0 * InterpolatablePlot::PAD
                    - InterpolatablePlot::FONT_SIZE,
            ),
        )
    }

    fn draw_text(
        &self,
        text: &str,
        x: f64,
        y: f64,
        color: Option<(f64, f64, f64)>,
        width: Option<f64>,
        height: Option<f64>,
    ) -> Result<(), Error> {
        let width = width.unwrap_or(InterpolatablePlot::WIDTH);
        let height = height.unwrap_or(InterpolatablePlot::HEIGHT);
        let cr = cairo::Context::new(self.surface)?;
        if let Some((red, green, blue)) = color {
            cr.set_source_rgb(red, green, blue);
        }
        cr.set_font_size(InterpolatablePlot::FONT_SIZE);
        cr.select_font_face("@cairo:monospace", FontSlant::Normal, FontWeight::Normal);
        let mut text_width = 0.0;
        let mut text_height = 0.0;
        let font_extents = cr.font_extents()?;
        let font_font_size = font_extents.height();
        let font_ascent = font_extents.ascent();
        for line in text.split("\n") {
            let extents = cr.text_extents(line)?;
            text_width = f64::max(text_width, extents.width());
            text_height += font_font_size;
        }
        if text_width == 0.0 {
            return Ok(());
        }
        cr.translate(x, y);
        let scale = (width / text_width).min(height / text_height);
        cr.translate(
            (width - text_width * scale) / 2.0,
            (height - text_height * scale) / 2.0,
        );
        cr.scale(scale, scale);

        cr.translate(0.0, font_ascent);
        for line in text.split("\n") {
            cr.move_to(0.0, 0.0);
            cr.show_text(line)?;
            cr.translate(0.0, font_font_size);
        }

        Ok(())
    }

    fn draw_emoticon(&self, emoticon: &str, x: f64, y: f64) -> Result<(), Error> {
        self.draw_text(
            emoticon,
            x,
            y,
            Some(InterpolatablePlot::EMOTICON_COLOR),
            Some(InterpolatablePlot::WIDTH),
            Some(InterpolatablePlot::HEIGHT),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_label(
        &self,
        label: &str,
        x: f64,
        y: f64,
        color: Option<(f64, f64, f64)>,
        align: f64,
        bold: bool,
        width: Option<f64>,
        font_size: f64,
    ) -> Result<(), Error> {
        let width = width.unwrap_or(InterpolatablePlot::WIDTH);
        let cr = cairo::Context::new(self.surface)?;
        cr.select_font_face(
            "@cairo:",
            FontSlant::Normal,
            if bold {
                FontWeight::Bold
            } else {
                FontWeight::Normal
            },
        );
        cr.set_font_size(font_size);
        let font_extents = cr.font_extents()?;
        let mut font_size = font_size * font_size / font_extents.max_x_advance();
        cr.set_font_size(font_size);
        let mut font_extents = cr.font_extents()?;
        if let Some((red, green, blue)) = color {
            cr.set_source_rgb(red, green, blue);
        } else {
            cr.set_source_rgb(0.0, 0.0, 0.0);
        }
        let mut extents = cr.text_extents(label)?;
        if extents.width() > width {
            font_size = font_size * width / extents.width();
            cr.set_font_size(font_size);
            font_extents = cr.font_extents()?;
            extents = cr.text_extents(label)?;
        }
        let label_x = x + (width - extents.width()) * align;
        let label_y = y + font_extents.ascent();
        cr.move_to(label_x, label_y);
        cr.show_text(label)?;
        Ok(())
    }
}

fn lerp_location(a: &[VariationSetting], b: &[VariationSetting], t: f32) -> Vec<VariationSetting> {
    a.iter()
        .zip(b.iter())
        .map(|(a, b)| {
            let mut a = *a;
            a.value = a.value + (b.value - a.value) * t;
            a
        })
        .collect()
}
