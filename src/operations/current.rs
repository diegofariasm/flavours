use anyhow::{anyhow, Context, Result};
use base16_color_scheme::Scheme;
use std::fs;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use crate::find::{find_schemes, get_luminance, Luminance};

/// Get scheme by name
///
/// * `scheme_name` - Name of the scheme to get
/// * `base_dir` - flavours data directory
/// * `config_dir` - flavours config directory
fn get_scheme(scheme_name: String, base_dir: &Path, config_dir: &Path) -> Result<Scheme> {
    let schemes = find_schemes(&scheme_name, base_dir, config_dir)?;
    let scheme_file: &PathBuf = schemes
        .first()
        .with_context(|| "Could not find any schemes")?;

    let scheme_slug = scheme_file
        .file_stem()
        .ok_or_else(|| anyhow!("The scheme path must contain a valid filename"))?
        .to_string_lossy()
        .to_string();

    let scheme_contents = fs::read_to_string(scheme_file)
        .with_context(|| format!("Couldn't read scheme file at {:?}.", scheme_file))?;

    let mut scheme: Scheme = serde_yaml::from_str(&scheme_contents)?;
    scheme.slug = scheme_slug;

    Ok(scheme)
}

/// Get the name of the current scheme
///
/// * `dir` - flavours data directory
pub fn get_current_scheme_name(dir: &Path) -> Result<String> {
    // File that stores last used scheme
    let file_path = &dir.join("lastscheme");

    // Try to open it
    let scheme: String = read_to_string(file_path)
        .with_context(|| "Failed to read last scheme file. Try applying first.")?
        .split_whitespace()
        .collect();

    if scheme.is_empty() {
        Err(anyhow!(
            "Failed to read last scheme from file. Try applying first."
        ))
    } else {
        Ok(scheme)
    }
}

pub fn get_current_scheme_luminance(base_dir: &Path, config_dir: &Path) -> Result<Luminance> {
    let scheme_luminance = get_luminance(&get_scheme(
        get_current_scheme_name(base_dir)?,
        base_dir,
        config_dir,
    )?);

    Ok(scheme_luminance)
}
