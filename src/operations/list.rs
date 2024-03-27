use anyhow::{anyhow, Result};
use std::path::Path;

use crate::find::filter_schemes_by_theme;
use crate::find::find_schemes;

/// List subcommand
///
/// * `patterns` - Vector with patterns
/// * `base_dir` - flavours' base data dir
/// * `config_dir` - flavours' config dir
/// * `verbose` - Should we be verbose? (unused)
/// * `lines` - Should we print each scheme on its own line?
pub fn list(
    patterns: Vec<&str>,
    theme: &str,
    base_dir: &Path,
    config_dir: &Path,
    _verbose: bool,
    lines: bool,
) -> Result<()> {
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

    for scheme in &schemes {
        // Print scheme
        print!("{}", scheme);
        if lines {
            // Print newline
            println!();
        } else {
            // Print space
            print!(" ");
        }
    }
    // If we separated by spaces, print an ending newline
    if !lines {
        println!();
    }

    Ok(())
}
