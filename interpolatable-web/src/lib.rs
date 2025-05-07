use std::collections::HashMap;

use indexmap::IndexMap;
use interpolatable::run_tests;
// use js_sys::{Reflect, Uint8Array};
use read_fonts::{
    tables::{fvar::VariationAxisRecord, post::DEFAULT_GLYPH_NAMES},
    types::Version16Dot16,
    ReadError, TableProvider,
};
use serde_json::{json, Value};
use skrifa::{setting::VariationSetting, FontRef, GlyphId};
use wasm_bindgen::prelude::*;
extern crate console_error_panic_hook;

#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
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

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn check_font(font_data: &[u8]) -> Result<String, JsValue> {
    let font = skrifa::FontRef::new(font_data).map_err(|e| e.to_string())?;

    let mut report: IndexMap<String, Vec<Value>> = IndexMap::new();
    let mut glyphname_to_id: HashMap<String, GlyphId> = HashMap::new();
    let mut locations: Vec<Vec<VariationSetting>> = vec![vec![]];
    let default_location = font
        .denormalize_location(&vec![0.0; font.fvar().unwrap().axes().unwrap().len()])
        .unwrap();
    for gid in 0..font.maxp().expect("Can't open maxp table").num_glyphs() {
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
                    glyphname_to_id.insert(glyphname.clone(), gid.into());
                    let default_outline: Vec<String> =
                        default_glyph.curves.iter().map(|c| c.to_svg()).collect();
                    let outline: Vec<String> = glyph.curves.iter().map(|c| c.to_svg()).collect();
                    let serialized_problems = problems
                        .iter()
                        .map(|p| serde_json::to_value(p).unwrap())
                        .collect::<Vec<_>>();
                    let midway_location = lerp_location(&default_location, &variation, 0.5);
                    let midway_glyph =
                        interpolatable::Glyph::new_from_font(&font, gid.into(), &midway_location)
                            .ok_or("Can't convert glyph")?;
                    let midway_name = midway_location
                        .iter()
                        .map(|v| format!("{}={}", v.selector, v.value))
                        .collect::<Vec<_>>()
                        .join(",");
                    let midway_outline = midway_glyph
                        .curves
                        .iter()
                        .map(|v| v.to_svg())
                        .collect::<Vec<_>>();
                    report.entry(glyphname.clone()).or_default().push(json!({
                        "default_outline": default_outline,
                        "outline": outline,
                        "midway_location": midway_name,
                        "midway_outline": midway_outline,
                        "problems": serialized_problems,
                        "default_name": default_glyph.master_name,
                        "master_name": glyph.master_name,
                        "master_index": glyph.master_index,
                    }));
                }
            }
        }
    }

    serde_json::to_string(&report).map_err(|e| e.to_string().into())
}
