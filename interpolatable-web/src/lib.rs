use std::collections::HashMap;

use indexmap::IndexMap;
use interpolatable::{
    run_tests,
    utils::{glyph_name_for_id, glyph_variations, DenormalizeLocation},
};
use read_fonts::TableProvider;
use serde_json::{json, Value};
use skrifa::{setting::VariationSetting, GlyphId};
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
    log(&format!("{:?}", default_location));
    for gid in 0..font.maxp().expect("Can't open maxp table").num_glyphs() {
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
                (loc, glyph)
            });
            let to_test = std::iter::once((&default_location, default_glyph))
                .chain(variation_glyphs)
                .collect::<Vec<_>>();
            for pair in to_test.windows(2) {
                if let [(before_loc, before), (after_loc, after)] = pair {
                    // println!("Testing {} vs {}", after.master_name, before.master_name);
                    let problems = run_tests(
                        before,
                        after,
                        None,
                        None,
                        Some(font.head().unwrap().units_per_em()),
                    );
                    if !problems.is_empty() {
                        let glyphname = glyph_name_for_id(&font, gid.into())
                            .unwrap_or_else(|_| format!("gid{}", gid));
                        glyphname_to_id.insert(glyphname.clone(), gid.into());
                        let default_outline: Vec<String> =
                            before.curves.iter().map(|c| c.to_svg()).collect();
                        let outline: Vec<String> =
                            after.curves.iter().map(|c| c.to_svg()).collect();
                        let serialized_problems = problems
                            .iter()
                            .map(|p| serde_json::to_value(p).unwrap())
                            .collect::<Vec<_>>();
                        let midway_location = lerp_location(before_loc, after_loc, 0.5);
                        let midway_glyph = interpolatable::Glyph::new_from_font(
                            &font,
                            gid.into(),
                            &midway_location,
                        )
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
                            "default_name": before.master_name,
                            "master_name": after.master_name,
                            "master_index": after.master_index,
                        }));
                    }
                }
            }
        }
    }

    serde_json::to_string(&report).map_err(|e| e.to_string().into())
}
