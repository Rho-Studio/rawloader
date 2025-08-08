use serde::{Deserialize, Serialize};

// This is the root structure that will hold the entire TOML file content.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "cameras")]
    pub cameras: Vec<Camera>,
}

// Represents a single [[cameras]] table from the TOML file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    // Required fields present in every camera entry
    pub make: String,
    pub model: String,
    pub clean_make: String,
    pub clean_model: String,
    pub color_matrix: Vec<i64>,
    pub color_pattern: String,

    // Optional fields that may not be present for every camera.
    pub blackpoint: Option<i64>,
    pub whitepoint: Option<i64>,
    pub crops: Option<Vec<i64>>,
    // We use `#[serde(default)]` for Vecs so that if the key is missing,
    // we get an empty vector instead of an error.
    #[serde(default)]
    pub model_aliases: Vec<Vec<String>>,
    pub blackareav: Option<Vec<i64>>,
    pub blackareah: Option<Vec<i64>>,
    pub wb_offset: Option<i64>,
    pub hints: Option<Vec<String>>,
    pub filesize: Option<i64>,
    pub raw_width: Option<i64>,
    pub raw_height: Option<i64>,
    pub bps: Option<i64>,

    // This captures the nested [[cameras.modes]] tables.
    #[serde(default)]
    pub modes: Vec<Mode>,
}

// Represents a single [[cameras.modes]] table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    pub mode: String,
    // These fields are optional within a mode definition.
    pub blackpoint: Option<i64>,
    pub whitepoint: Option<i64>,
    pub color_pattern: Option<String>,
    pub crops: Option<Vec<i64>>,
    pub highres_width: Option<i64>,
}
