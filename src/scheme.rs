use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

/// Luminosity of a theme
#[derive(Deserialize, Serialize, Debug)]
pub enum Luminance {
    Dark,
    Light,
}

#[derive(Deserialize, Serialize)]
pub struct Scheme {
    /// Displayed name for the scheme
    pub name: String,

    /// Scheme luminance
    pub luminance: Luminance,

    /// File path for the scheme
    file: PathBuf,
}

impl Scheme {
    pub fn slug(self) -> String {
        self.file
            .file_stem()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.name)
    }
}

impl fmt::Display for Luminance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Luminance::Dark => write!(f, "dark"),
            Luminance::Light => write!(f, "light"),
        }
    }
}
