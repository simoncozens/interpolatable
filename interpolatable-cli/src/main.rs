mod cairopen;
mod plot;

use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use indexmap::IndexMap;
use indicatif::{ProgressBar, ProgressIterator};
use interpolatable::{run_tests, Problem};
use plot::InterpolatablePlot;
use read_fonts::{
    tables::{fvar::VariationAxisRecord, post::DEFAULT_GLYPH_NAMES},
    types::Version16Dot16,
    ReadError, TableProvider,
};
use skrifa::{setting::VariationSetting, FontRef, GlyphId};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Output JSON
    #[clap(short, long)]
    json: bool,

    /// Output to PDF files
    #[clap(short, long)]
    pdf: Option<String>,

    /// The font file to test
    pub font: PathBuf,
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
fn poor_mans_denormalize(peak: f32, axis: &VariationAxisRecord) -> f32 {
    // Insert avar here
    if peak > 0.0 {
        lerp(
            axis.default_value().to_f32(),
            axis.max_value().to_f32(),
            peak,
        )
    } else {
        lerp(
            axis.default_value().to_f32(),
            axis.min_value().to_f32(),
            -peak,
        )
    }
}

pub trait DenormalizeLocation {
    /// Given a normalized location tuple, turn it back into a friendly representation in userspace
    fn denormalize_location(&self, tuple: &[f32]) -> Result<Vec<VariationSetting>, ReadError>;
}

impl DenormalizeLocation for FontRef<'_> {
    fn denormalize_location(&self, tuple: &[f32]) -> Result<Vec<VariationSetting>, ReadError> {
        let all_axes = self.fvar()?.axes()?;
        Ok(all_axes
            .iter()
            .zip(tuple)
            .filter(|&(_axis, peak)| *peak != 0.0)
            .map(|(axis, peak)| {
                let value = poor_mans_denormalize(*peak, axis);
                (axis.axis_tag().to_string().as_str(), value).into()
            })
            .collect())
    }
}

fn glyph_variations(font: &FontRef, gid: GlyphId) -> Result<Vec<Vec<VariationSetting>>, ReadError> {
    font.gvar()
        .expect("Can't open gvar table")
        .glyph_variation_data(gid)
        .map(|data| {
            data.tuples()
                .map(|t| {
                    let tuple: Vec<f32> =
                        t.peak().values.iter().map(|v| v.get().to_f32()).collect();
                    font.denormalize_location(&tuple)
                        .expect("Can't denormalize location")
                })
                .collect()
        })
}

pub fn glyph_name_for_id(fontref: &FontRef, gid: usize) -> String {
    if let Ok(post) = fontref.post() {
        match post.version() {
            Version16Dot16::VERSION_1_0 => {
                if let Some(name) = DEFAULT_GLYPH_NAMES.get(gid) {
                    return name.to_string();
                }
            }
            Version16Dot16::VERSION_2_0 => {
                let strings: Vec<Option<read_fonts::tables::post::PString>> =
                    post.string_data().unwrap().iter().map(|x| x.ok()).collect();
                if let Some(index) = post.glyph_name_index() {
                    let idx = index.get(gid).unwrap().get() as usize;
                    if idx < 258 {
                        return DEFAULT_GLYPH_NAMES[idx].to_string();
                    } else {
                        let entry = strings.get(idx - 258).unwrap();
                        if let Some(name) = entry.map(|x| x.to_string()) {
                            return name;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    format!("gid{:}", gid)
}

fn main() {
    let args = Args::parse();
    let fontdata = std::fs::read(&args.font).expect("Can't read font file");
    let font = FontRef::new(&fontdata).expect("Can't parse font");
    let mut report: IndexMap<String, Vec<Problem>> = IndexMap::new();
    let mut glyphname_to_id: HashMap<String, GlyphId> = HashMap::new();
    let mut locations: Vec<Vec<VariationSetting>> = vec![vec![]];
    for gid in (0..font.maxp().expect("Can't open maxp table").num_glyphs()).progress() {
        let mut default_glyph = interpolatable::Glyph::new_from_font(&font, gid.into(), &[])
            .expect("Can't convert glyph");
        default_glyph.master_name = "default".to_string();
        default_glyph.master_index = 0;
        if let Ok(variations) = glyph_variations(&font, gid.into()) {
            for variation in variations {
                let mut glyph = interpolatable::Glyph::new_from_font(&font, gid.into(), &variation)
                    .expect("Can't convert glyph");
                glyph.master_name = variation
                    .iter()
                    .map(|v| format!("{}={}", v.selector, v.value))
                    .collect::<Vec<_>>()
                    .join(",");
                if !locations.contains(&variation) {
                    locations.push(variation.clone());
                }
                glyph.master_index = locations.iter().position(|x| x == &variation).unwrap();
                let problems = run_tests(
                    &default_glyph,
                    &glyph,
                    None,
                    None,
                    Some(font.head().unwrap().units_per_em()),
                );
                if !problems.is_empty() {
                    let glyphname = glyph_name_for_id(&font, gid.into());
                    if !args.json {
                        println!("Problems with glyph {}:", &glyphname);
                        for problem in problems.iter() {
                            println!("  {:#?}", problem);
                        }
                    }
                    glyphname_to_id.insert(glyphname.clone(), gid.into());
                    report.insert(glyphname.clone(), problems);
                }
            }
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    }

    if let Some(pdf) = args.pdf {
        let surface =
            cairo::PdfSurface::new(InterpolatablePlot::WIDTH, InterpolatablePlot::HEIGHT, &pdf)
                .expect("Can't create PDF");
        let mut plot =
            InterpolatablePlot::new(&surface, font, &locations, glyphname_to_id, None, None);
        plot.add_title_page(&[args.font], None, None, None)
            .expect("Can't add title page");
        if !report.is_empty() {
            plot.add_summary(&report).expect("Can't add summary");
        }
        plot.add_problems(&report).expect("Couldn't add problems");
        if report.is_empty() {
            plot.draw_cupcake().expect("No cupcake for you!");
        } else {
            plot.add_index().expect("Can't add index");
            plot.add_table_of_contents()
                .expect("Can't add table of contents");
        }
    }
}
