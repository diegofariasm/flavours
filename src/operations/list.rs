use anyhow::{anyhow, Result};
use std::path::Path;

use crate::find::{filter_schemes_by_theme, find_schemes, find_templates};

/// List schemes subcommand
///
/// * `patterns` - Vector with patterns
/// * `base_dir` - flavours' base data dir
/// * `config_dir` - flavours' config dir
/// * `verbose` - Should we be verbose? (unused)
/// * `lines` - Should we print each scheme on its own line?
pub fn schemes(
    patterns: Vec<&str>,
    theme: &str,
    base_dir: &Path,
    config_dir: &Path,
) -> Result<Vec<String>> {
    let mut schemes = Vec::new();
    for pattern in patterns {
        let mut found_schemes = find_schemes(pattern, base_dir, config_dir)?;

        // Filter the  schemes based on the theme mode the user wants.
        found_schemes = match theme {
            "dark" => filter_schemes_by_theme(found_schemes, "dark")?,
            "light" => filter_schemes_by_theme(found_schemes, "light")?,
            _ => found_schemes,
        };

        for found_scheme in found_schemes {
            schemes.push(String::from(
                found_scheme
                    .file_stem()
                    .ok_or_else(|| anyhow!("Couldn't get scheme name"))?
                    .to_str()
                    .ok_or_else(|| anyhow!("Couldn't convert name"))?,
            ));
        }
    }

    schemes.sort();
    schemes.dedup();

    if schemes.is_empty() {
        return Err(anyhow!("No matching scheme found"));
    };

    Ok(schemes)
}

/// List templates subcommand
///
/// * `patterns` - Vector with patterns
/// * `base_dir` - flavours' base data dir
/// * `config_dir` - flavours' config dir
/// * `verbose` - Should we be verbose? (unused)
/// * `lines` - Should we print each scheme on its own line?
pub fn templates(patterns: Vec<&str>, base_dir: &Path, config_dir: &Path) -> Result<Vec<String>> {
    let mut templates = Vec::new();
    for pattern in patterns {
        let found_templates = find_templates(pattern, base_dir, config_dir)?;
        for found_template in found_templates {
            templates.push(
                found_template
                    .strip_prefix(base_dir)
                    .map_or_else(
                        |_| found_template.strip_prefix(config_dir),
                        |path| path.strip_prefix("base16/"),
                    )
                    .map_err(|_| anyhow!("Couldn't get template name"))?
                    .to_str()
                    .ok_or_else(|| anyhow!("Couldn't convert name"))?
                    .replacen("templates/", "", 2)
                    .replace(".mustache", ""),
            );
        }
    }

    templates.sort();
    templates.dedup();

    if templates.is_empty() {
        return Err(anyhow!("No matching template found"));
    };

    Ok(templates)
}
