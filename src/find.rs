use anyhow::{anyhow, Context, Result};
use base16_color_scheme::scheme::BaseIndex;
use base16_color_scheme::scheme::RgbColor;
use base16_color_scheme::Scheme;
use glob::glob;
use path::{Path, PathBuf};
use std::path;
use std::{fs, vec};

use crate::scheme::Luminance;

/// Find color schemes matching pattern in either the config dir or the data dir.
///
/// * `pattern` - Which pattern to use
/// * `base_dir` - flavours' base data dir
/// * `config_dir` - flavours' config dir
pub fn find_schemes(pattern: &str, base_dir: &Path, config_dir: &Path) -> Result<Vec<PathBuf>> {
    let config_scheme_dir = config_dir.join("schemes");
    let data_scheme_dir = base_dir.join("base16").join("schemes");

    let dirs = [config_scheme_dir, data_scheme_dir];
    let dirs = dirs.iter().filter_map(|dir| dir.to_str());

    let mut found = Vec::new();
    for dir in dirs {
        let glob_pattern = format!("{}/*/{}.y*ml", dir, pattern);
        let matches = glob(&glob_pattern)?;
        for element in matches {
            // Check if scheme is valid
            let current_element = element?;
            let scheme_contents = fs::read_to_string(&current_element)
                .with_context(|| format!("Couldn't read scheme file at {:?}.", current_element))?;

            let scheme_result: Result<Scheme, _> = serde_yaml::from_str(&scheme_contents);
            match scheme_result {
                Ok(_) => {
                    found.push(current_element);
                }
                Err(_) => {
                    // The "scheme" might just not be a scheme at all.
                    // So instead of polluting the output of the tool with it, we just skip.
                    // eprintln!("A invalid scheme was found: {:#?}", current_element);
                }
            }
        }
    }

    Ok(found)
}

pub fn get_luminance(scheme: &Scheme) -> Luminance {
    let rgb2luminance = |rgb: &RgbColor| {
        let [r, g, b] = rgb.0;

        // there are exacter ways, this turns out to be good enough
        // https://www.w3.org/TR/AERT/#color-contrast
        let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
        luminance / 255.0
    };

    // Take into account the main background colors as per the styling guide
    // https://github.com/tinted-theming/home/blob/main/styling.md
    let background_indices = vec![0, 1];

    let luminances: Vec<_> = background_indices
        .into_iter()
        .map(|idx| rgb2luminance(scheme.colors.get(&BaseIndex(idx)).unwrap()))
        .collect();

    let avg_luminance: f32 = luminances.iter().sum::<f32>() / luminances.len() as f32;

    if avg_luminance < 0.5 {
        Luminance::Dark
    } else {
        Luminance::Light
    }
}

pub fn filter_schemes_by_theme(schemes: Vec<PathBuf>, theme: &str) -> Result<Vec<PathBuf>> {
    let mut filtered_schemes = Vec::new();

    for scheme_file in schemes {
        let scheme_contents = fs::read_to_string(&scheme_file)
            .with_context(|| format!("Couldn't read scheme file at {:?}.", scheme_file))?;

        let scheme: Scheme = serde_yaml::from_str(&scheme_contents)?;

        let light_mode = match get_luminance(&scheme) {
            Luminance::Light => true,
            Luminance::Dark => false,
        };

        if theme == "light" {
            if light_mode {
                filtered_schemes.push(scheme_file)
            }
        } else if !light_mode {
            filtered_schemes.push(scheme_file)
        }
    }

    Ok(filtered_schemes)
}

/// Find templates matching pattern in either the config dir or the data dir.
///
/// * `pattern` - Which pattern to use
/// * `base_dir` - flavours' base data dir
/// * `config_dir` - flavours' config dir
pub fn find_templates(pattern: &str, base_dir: &Path, config_dir: &Path) -> Result<Vec<PathBuf>> {
    let config_scheme_dir = config_dir.join("templates");
    let data_scheme_dir = base_dir.join("base16").join("templates");
    let dir_vec = vec![config_scheme_dir, data_scheme_dir];
    let dir_vec: Vec<&str> = dir_vec.iter().filter_map(|dir| dir.to_str()).collect();
    let pattern = match pattern
        // remove extension if it was included
        .trim_end_matches(".mustache")
        // split on '/' if present
        .split_once('/')
        // only replace '/' if there it was only one
        // and was not '/templates/' already
        .filter(|(_, post)| !post.contains('/') || post.starts_with("templates/"))
    {
        // automatically expand single '/' to '/templates/'
        Some((template_pattern, subtemplate_pattern)) => {
            let subtemplate_pattern = subtemplate_pattern.replace("templates/", "");
            format!(
                "{}/templates/{}.mustache",
                template_pattern, subtemplate_pattern
            )
        }
        // otherwise leave pattern untouched
        None => pattern.to_string(),
    };
    let mut found = Vec::new();
    for dir in dir_vec {
        let pattern_glob = format!("{}/{}", dir, pattern);
        let matches = glob(&pattern_glob)?;
        for element in matches {
            found.push(element?);
        }
    }
    Ok(found)
}

/// Find template file in either the config dir or the data dir.
///
/// * `template` - template
/// * `subtemplate` - subtemplate
/// * `base_dir` - flavours' base data dir
/// * `config_dir` - flavours' config dir
pub fn find_template(
    template: &str,
    subtemplate: &str,
    base_dir: &Path,
    config_dir: &Path,
) -> Result<PathBuf> {
    let template_config_file = config_dir
        .join("templates")
        .join(template)
        .join("templates")
        .join(format!("{}.mustache", subtemplate));

    let template_data_file = base_dir
        .join("base16")
        .join("templates")
        .join(template)
        .join("templates")
        .join(format!("{}.mustache", subtemplate));

    if template_config_file.is_file() {
        Ok(template_config_file)
    } else if template_data_file.is_file() {
        Ok(template_data_file)
    } else {
        return Err(anyhow!(
            "Neither {:?} or {:?} exist",
            template_config_file,
            template_data_file
        ));
    }
}
