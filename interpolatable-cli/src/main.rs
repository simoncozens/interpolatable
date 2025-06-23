mod cairopen;
mod plot;

use std::{collections::HashMap, path::PathBuf};

use clap::Parser;
use fontations::read::TableProvider;
use fontations::skrifa::{setting::VariationSetting, FontRef, GlyphId, MetadataProvider};
use indexmap::IndexMap;
use indicatif::ProgressIterator;
use interpolatable::{run_tests, utils::glyph_variations, Problem};
use plot::InterpolatablePlot;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Output JSON
    #[clap(short, long)]
    json: bool,

    /// Output to PDF files
    #[clap(short, long)]
    pdf: Option<String>,

    #[clap(long)]
    glyphs: Vec<String>,

    /// The font file to test
    pub font: PathBuf,
}

fn main() {
    let args = Args::parse();
    let fontdata = std::fs::read(&args.font).expect("Can't read font file");
    let font = FontRef::new(&fontdata).expect("Can't parse font");
    let mut report: IndexMap<String, Vec<Problem>> = IndexMap::new();
    let mut locations: Vec<Vec<VariationSetting>> = vec![vec![]];
    let glyphnames = font.glyph_names();
    for gid in (0..font.maxp().expect("Can't open maxp table").num_glyphs()).progress() {
        let glyph_name = glyphnames
            .get(gid.into())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("gid{}", gid));
        if !args.glyphs.is_empty() && !args.glyphs.contains(&glyph_name) {
            continue;
        }
        let mut default_glyph = interpolatable::Glyph::new_from_font(&font, gid.into(), &[])
            .expect("Can't convert glyph");
        default_glyph.master_name = "default".to_string();
        default_glyph.master_index = 0;
        if let Ok(variations) = glyph_variations(&font, gid.into()) {
            let variation_glyphs = variations.iter().map(|loc| {
                let mut glyph = interpolatable::Glyph::new_from_font(&font, gid.into(), loc)
                    .expect("Couldn't convert glyph");
                glyph.master_name = loc
                    .iter()
                    .map(|v| format!("{}={}", v.selector, v.value))
                    .collect::<Vec<_>>()
                    .join(",");
                if !locations.contains(loc) {
                    locations.push(loc.clone());
                }
                glyph.master_index = locations.iter().position(|x| x == loc).unwrap();
                glyph
            });

            let to_test = std::iter::once(default_glyph)
                .chain(variation_glyphs)
                .collect::<Vec<_>>();
            for pair in to_test.windows(2) {
                if let [before, after] = pair {
                    // println!("Testing {} vs {}", after.master_name, before.master_name);
                    let problems = run_tests(
                        before,
                        after,
                        None,
                        None,
                        Some(font.head().unwrap().units_per_em()),
                    );
                    if !problems.is_empty() {
                        let glyphname = glyphnames
                            .get(gid.into())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("gid{}", gid));
                        if !args.json {
                            println!("Problems with glyph {}:", &glyphname);
                            for problem in problems.iter() {
                                println!("  {:#?}", problem);
                            }
                        }
                        report.insert(glyphname.clone(), problems);
                    }
                }
            }
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&report).unwrap());
    }

    let glyphname_to_id: HashMap<String, GlyphId> = glyphnames
        .iter()
        .map(|(i, name)| (name.to_string(), i))
        .collect();

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
