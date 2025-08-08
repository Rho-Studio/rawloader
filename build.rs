#[path = "./src/toml.rs"]
mod toml;

use crate::toml::Config;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Cameras {
    #[serde(rename = "Camera", default)]
    pub cameras: Vec<Camera>,
    #[serde(rename = "@version")]
    pub version: Option<String>,
}

impl Cameras {
    fn group_by_make_and_model(self) -> HashMap<(String, String), Vec<Camera>> {
        let mut result = vec![];

        for camera in self.cameras {
            let key = (camera.make.clone(), camera.model.clone());
            result.push((key, camera));
        }

        result
            .into_iter()
            .fold(HashMap::new(), |mut map, (key, value)| {
                if let Some(values) = map.get_mut(&key) {
                    values.push(value);
                } else {
                    let values = vec![value];
                    map.insert(key, values);
                }

                map
            })
    }
}

#[derive(Debug, Deserialize)]
pub struct Camera {
    #[serde(rename = "@make")]
    pub make: String,
    #[serde(rename = "@model")]
    pub model: String,
    #[serde(rename = "@supported")]
    pub supported: Option<String>,
    #[serde(rename = "@mode")]
    pub mode: Option<String>,
    #[serde(rename = "@decoder_version")]
    pub decoder_version: Option<u32>,
    #[serde(rename = "ID")]
    pub id: Option<Id>,
    #[serde(rename = "CFA")]
    pub cfa: Option<Cfa>,
    #[serde(rename = "CFA2")]
    pub cfa2: Option<Cfa2>,
    #[serde(rename = "Crop")]
    pub crop: Option<Crop>,
    #[serde(rename = "Sensor", default)]
    pub sensors: Vec<Sensor>,
    #[serde(rename = "BlackAreas")]
    pub black_areas: Option<BlackAreas>,
    #[serde(rename = "Aliases")]
    pub aliases: Option<Aliases>,
    #[serde(rename = "Hints")]
    pub hints: Option<Hints>,
    #[serde(rename = "ColorMatrices")]
    pub color_matrices: Option<ColorMatrices>,
}

#[derive(Debug, Deserialize)]
pub struct Id {
    #[serde(rename = "@make")]
    pub make: String,
    #[serde(rename = "@model")]
    pub model: String,
    #[serde(rename = "#text")]
    pub value: Option<String>, // Changed to Option<String>
}

#[derive(Debug, Deserialize)]
pub struct Cfa {
    #[serde(rename = "@width")]
    pub width: u32,
    #[serde(rename = "@height")]
    pub height: u32,
    #[serde(rename = "Color", default)]
    pub colors: Vec<Color>,
}

#[derive(Debug, Deserialize)]
pub struct Cfa2 {
    #[serde(rename = "@width")]
    pub width: u32,
    #[serde(rename = "@height")]
    pub height: u32,
    #[serde(rename = "Color", default)]
    pub colors: Vec<Color>,
    #[serde(rename = "ColorRow", default)]
    pub color_rows: Vec<ColorRow>,
}

#[derive(Debug, Deserialize)]
pub struct Color {
    #[serde(rename = "@x")]
    pub x: u32,
    #[serde(rename = "@y")]
    pub y: u32,
    #[serde(rename = "#text")]
    pub value: Option<String>, // Changed to Option<String>
}

#[derive(Debug, Deserialize)]
pub struct ColorRow {
    #[serde(rename = "@y")]
    pub y: u32,
    #[serde(rename = "#text")]
    pub value: Option<String>, // Changed to Option<String>
}

#[derive(Debug, Deserialize)]
pub struct Crop {
    #[serde(rename = "@x")]
    pub x: i32,
    #[serde(rename = "@y")]
    pub y: i32,
    #[serde(rename = "@width")]
    pub width: i32,
    #[serde(rename = "@height")]
    pub height: i32,
}

#[derive(Debug, Deserialize)]
pub struct Sensor {
    #[serde(rename = "@black")]
    pub black: String,
    #[serde(rename = "@white")]
    pub white: String,
    #[serde(rename = "@black_colors")]
    pub black_colors: Option<String>,
    #[serde(rename = "@iso_list")]
    pub iso_list: Option<String>,
    #[serde(rename = "@iso_min")]
    pub iso_min: Option<String>,
    #[serde(rename = "@iso_max")]
    pub iso_max: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlackAreas {
    #[serde(rename = "Vertical", default)]
    pub vertical: Vec<Vertical>,
    #[serde(rename = "Horizontal", default)]
    pub horizontal: Vec<Horizontal>,
}

#[derive(Debug, Deserialize)]
pub struct Vertical {
    #[serde(rename = "@x")]
    pub x: u32,
    #[serde(rename = "@width")]
    pub width: u32,
}

#[derive(Debug, Deserialize)]
pub struct Horizontal {
    #[serde(rename = "@y")]
    pub y: u32,
    #[serde(rename = "@height")]
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct Aliases {
    #[serde(rename = "Alias", default)]
    pub aliases: Vec<Alias>,
}

#[derive(Debug, Deserialize)]
pub struct Alias {
    #[serde(rename = "@id")]
    pub id: Option<String>,
    #[serde(rename = "#text")]
    pub value: Option<String>, // Changed to Option<String>
}

#[derive(Debug, Deserialize)]
pub struct Hints {
    #[serde(rename = "Hint", default)]
    pub hints: Vec<Hint>,
}

impl Hints {
    fn find_hint(&self, hint: &str) -> Option<&Hint> {
        self.hints.iter().find(|item| item.name == hint)
    }
}

#[derive(Debug, Deserialize)]
pub struct Hint {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct ColorMatrices {
    #[serde(rename = "ColorMatrix")]
    pub color_matrix: ColorMatrix,
}

#[derive(Debug, Deserialize)]
pub struct ColorMatrix {
    #[serde(rename = "@planes")]
    pub planes: u32,
    #[serde(rename = "ColorMatrixRow", default)]
    pub rows: Vec<ColorMatrixRow>,
}

#[derive(Debug, Deserialize)]
pub struct ColorMatrixRow {
    #[serde(rename = "@plane")]
    pub plane: u32,
    #[serde(rename = "#text")]
    pub value: Option<String>, // Changed to Option<String>
}

lazy_static! {
    static ref HINTS_MAP: HashMap<&'static str, &'static str> = {
        vec![
            ("wb_mangle", "wb_mangle"),
            ("no_decompressed_lowbits", "nolowbits"),
            ("easyshare_offset_hack", "easyshare_offset_hack"),
            ("swapped_wb", "swapped_wb"),
            ("coolpixsplit", "coolpixsplit"),
            ("coolpixmangled", "msb32"),
            ("force_uncompressed", "unpacked"),
            ("double_width_unpacked", "double_width"),
            ("jpeg32_bitorder", "jpeg32"),
            ("fuji_rotate", "fuji_rotation"),
        ]
        .into_iter()
        .collect()
    };
}

fn main() {
    let response = reqwest::blocking::get("https://raw.githubusercontent.com/darktable-org/rawspeed/refs/heads/develop/data/cameras.xml").unwrap();
    let parsed_data: Cameras = serde_xml_rs::from_str(response.text().unwrap().as_str()).unwrap();

    let mut cameras = parsed_data
        .group_by_make_and_model()
        .into_iter()
        .filter_map(|((make, model), mut models)| {
            models.retain(|item| item.supported != Some("no".to_string()));

            if models.is_empty() {
                return None;
            }

            let default = models
                .iter()
                .find(|mode| mode.mode.is_none())
                .or(models.first())
                .unwrap();

            let modes = models
                .iter()
                .filter(|mode| mode.mode.is_some())
                .collect::<Vec<_>>();

            let color_matrix = default
                .color_matrices
                .iter()
                .flat_map(|item| &item.color_matrix.rows)
                .filter_map(|item| {
                    if let Some(row) = item.value.as_ref() {
                        Some(
                            row.split_whitespace()
                                .filter_map(|item| item.parse::<i64>().ok())
                                .collect::<Vec<_>>(),
                        )
                    } else {
                        None
                    }
                })
                .flatten()
                .collect();

            let color_pattern = if let Some(cfa) = &default.cfa {
                cfa.colors
                    .iter()
                    .filter_map(|color| {
                        color
                            .value
                            .clone()
                            .and_then(|item| item.chars().nth(0))
                            .map(|char| char.to_string())
                    })
                    .collect::<Vec<_>>()
                    .join("")
            } else if let Some(cfa) = &default.cfa2 {
                cfa.colors
                    .iter()
                    .filter_map(|color| color.value.clone())
                    .collect::<Vec<_>>()
                    .join("")
            } else {
                String::new()
            };

            let color_pattern = if color_pattern == "FMYC" {
                "".to_string()
            } else {
                color_pattern
            };

            let blackpoint = default
                .sensors
                .iter()
                .filter_map(|item| item.black.parse::<i64>().ok())
                .min();

            let whitepoint = default
                .sensors
                .iter()
                .filter_map(|item| item.white.parse::<i64>().ok())
                .max();

            let crops = default.crop.as_ref().map(|item| {
                [
                    i64::from(item.y.abs()),
                    i64::from(item.width.abs()),
                    i64::from(item.height.abs()),
                    i64::from(item.x.abs()),
                ]
                .to_vec()
            });

            let mut model_aliases = default
                .aliases
                .as_ref()
                .map(|item| {
                    item.aliases
                        .iter()
                        .filter_map(|item| item.id.clone().zip(item.value.clone()))
                        .map(|(id, value)| vec![id, value])
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if let Some(id) = &default.id {
                let mut aliases = vec![id.model.clone()];

                if let Some(value) = &id.value {
                    aliases.push(value.clone());
                }

                model_aliases.push(aliases);
            }

            let blackareav = default
                .black_areas
                .as_ref()
                .and_then(|item| item.vertical.last())
                .map(|item| vec![i64::from(item.x), i64::from(item.width)]);

            let blackareah = default
                .black_areas
                .as_ref()
                .and_then(|item| item.horizontal.last())
                .map(|item| vec![i64::from(item.y), i64::from(item.height)]);

            let wb_offset = default
                .hints
                .as_ref()
                .and_then(|item| item.find_hint("wb_offset"))
                .and_then(|item| item.value.parse::<i64>().ok());

            let filesize = default
                .hints
                .as_ref()
                .and_then(|item| item.find_hint("filesize"))
                .and_then(|item| item.value.parse::<i64>().ok());

            let raw_width = default
                .hints
                .as_ref()
                .and_then(|item| item.find_hint("full_width"))
                .and_then(|item| item.value.parse::<i64>().ok());

            let raw_height = default
                .hints
                .as_ref()
                .and_then(|item| item.find_hint("full_height"))
                .and_then(|item| item.value.parse::<i64>().ok());

            let hints = default.hints.as_ref().map(|item| {
                item.hints
                    .iter()
                    .map(|hint| {
                        HINTS_MAP
                            .get(hint.name.as_str())
                            .map(|item| item.to_string())
                            .unwrap_or(hint.name.clone())
                    })
                    .collect()
            });

            let modes = modes
                .iter()
                .map(|item| {
                    let blackpoint = item
                        .sensors
                        .iter()
                        .filter_map(|item| item.black.parse::<i64>().ok())
                        .min();

                    let whitepoint = item
                        .sensors
                        .iter()
                        .filter_map(|item| item.white.parse::<i64>().ok())
                        .max();

                    let crops = item.crop.as_ref().map(|item| {
                        [
                            i64::from(item.y),
                            i64::from(item.width),
                            i64::from(item.height),
                            i64::from(item.x),
                        ]
                        .to_vec()
                    });

                    let color_pattern = if let Some(cfa) = &item.cfa {
                        Some(
                            cfa.colors
                                .iter()
                                .filter_map(|color| {
                                    color
                                        .value
                                        .clone()
                                        .and_then(|item| item.chars().nth(0))
                                        .map(|char| char.to_string())
                                })
                                .collect::<Vec<_>>()
                                .join(""),
                        )
                    } else if let Some(cfa) = &item.cfa2 {
                        Some(
                            cfa.colors
                                .iter()
                                .filter_map(|color| color.value.clone())
                                .collect::<Vec<_>>()
                                .join(""),
                        )
                    } else {
                        None
                    };

                    let color_pattern =
                        color_pattern.and_then(|cfa| if cfa == "FMYC" { None } else { Some(cfa) });

                    toml::Mode {
                        mode: item.mode.clone().unwrap(),
                        blackpoint,
                        whitepoint,
                        crops,
                        color_pattern,
                        highres_width: None,
                    }
                })
                .collect();

            Some(toml::Camera {
                make,
                model,
                clean_make: "".to_string(),
                clean_model: "".to_string(),
                color_matrix,
                color_pattern,
                blackpoint,
                whitepoint,
                crops,
                model_aliases: model_aliases.clone(),
                blackareav,
                blackareah,
                wb_offset,
                hints,
                filesize,
                raw_width,
                raw_height,
                bps: None,
                modes,
            })
        })
        .collect::<Vec<_>>();

    cameras.sort_by(|a, b| a.make.cmp(&b.make).then(a.model.cmp(&b.model)));

    let config = Config { cameras };

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("all.toml");
    let mut out = File::create(dest_path).unwrap();

    let contents = ::toml::to_string(&config).unwrap();
    out.write_all(&contents.as_bytes()).unwrap();
    out.write_all(b"\n").unwrap();
}
