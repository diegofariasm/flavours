use anyhow::{anyhow, Context, Result};
use base16_color_scheme::{
    scheme::{BaseIndex, RgbColor},
    Scheme,
};
use dirs::{data_dir, preference_dir};
use flavours::find::find_schemes;
use flavours::find::find_template;
use palette::Srgb;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use flavours::operations::{apply, build, current, generate, info, list, list_templates, update};
use flavours::{cli, completions};

use std::fs::{create_dir_all, write};
fn main() -> Result<()> {
    let matches = cli::build_cli().get_matches();

    // Completetions flag
    if matches.is_present("completions") {
        return completions::completions(matches.value_of("completions"));
    };

    // Flavours data directory
    let flavours_dir = match matches.value_of("directory") {
        // User supplied
        Some(argument) => Path::new(argument)
            .canonicalize()
            .with_context(|| "Invalid data directory supplied on argument")?,
        // If not supplied
        None => {
            // Try to get from env var
            match env::var("FLAVOURS_DATA_DIRECTORY") {
                Ok(path) => Path::new(&path)
                    .canonicalize()
                    .with_context(|| "Invalid data directory supplied on env var")?,
                // Use default instead
                Err(_) => data_dir()
                    .ok_or_else(|| anyhow!("Error getting default data directory"))?
                    .join("flavours"),
            }
        }
    };

    let flavours_config_dir = preference_dir()
        .ok_or_else(|| anyhow!("Error getting default config directory"))?
        .join("flavours");

    // Flavours config file
    let flavours_config = match matches.value_of("config") {
        // User supplied
        Some(path) => Path::new(path)
            .canonicalize()
            .with_context(|| "Invalid config file supplied on argument")?,
        // If not supplied
        None => {
            // Try to get from env var
            match env::var("FLAVOURS_CONFIG_FILE") {
                Ok(path) => Path::new(&path)
                    .canonicalize()
                    .with_context(|| "Invalid config file supplied on env var")?,
                // Use default instead
                Err(_) => flavours_config_dir.join("config.toml"),
            }
        }
    };

    // Should we be verbose?
    let verbose = matches.is_present("verbose");

    if verbose {
        println!("Using directory: {:?}", flavours_dir);
        println!("Using config file: {:?}", flavours_config);
    };

    // Check which subcommand was used
    match matches.subcommand() {
        Some(("current", _)) => current::current(&flavours_dir, verbose),

        Some(("apply", sub_matches)) => {
            //Get search patterns
            let patterns = match sub_matches.values_of("pattern") {
                Some(content) => content.collect(),
                //Defaults to wildcard
                None => vec!["*"],
            };
            let lightweight = sub_matches.is_present("lightweight");
            let from_stdin = sub_matches.is_present("stdin");
            let theme = sub_matches.value_of("theme").unwrap();

            apply::apply(
                patterns,
                &theme,
                &flavours_dir,
                &flavours_config_dir,
                &flavours_config,
                lightweight,
                from_stdin,
                verbose,
            )
        }

        Some(("build", sub_matches)) => {
            // let scheme = sub_matches.value_of("scheme");
            let scheme_file_str = sub_matches.value_of("scheme").ok_or_else(|| {
                anyhow!(
                    "You must specify a scheme or file, the latter taking precedence if it exists"
                )
            })?;

            let scheme_file_path = if Path::new(scheme_file_str).exists() {
                PathBuf::from(scheme_file_str)
            } else {
                find_schemes(scheme_file_str, &flavours_dir, &flavours_config_dir)?
                    .first() // Get the first PathBuf from the vector
                    .ok_or_else(|| anyhow!("Could not find a scheme for {}", scheme_file_str))? // Handle None case
                    .clone() // Clone the PathBuf to create a new instance
            };

            if verbose {
                println!("Scheme is at: {:#?}", scheme_file_path);
            }

            let scheme_slug = scheme_file_path
                .file_stem()
                .ok_or_else(|| anyhow!("The scheme path must contain a valid filename"))?
                .to_string_lossy()
                .to_string();

            // Get template file path
            let template_file_str = sub_matches.value_of("template").ok_or_else(|| {
                 anyhow!("You must specify a template or file, the latter taking precedence if it exists")
             })?;

            let subtemplate = sub_matches.value_of("subtemplate").unwrap_or("default");

            let template_file_path = if Path::new(template_file_str).exists() {
                if subtemplate != "default" {
                    return Err(anyhow!(
                        "Using subtemplates is not supported incase you are using a scheme file."
                    ));
                }

                PathBuf::from(template_file_str) // Create a PathBuf from the existing path
            } else {
                find_template(
                    template_file_str,
                    subtemplate,
                    &flavours_dir,
                    &flavours_config_dir,
                )
                .with_context(|| {
                    format!(
                        "Failed to locate subtemplate file {}/{}",
                        template_file_str, subtemplate
                    )
                })?
            };

            if verbose {
                println!("Template is at: {:#?}", template_file_path);
            }

            let scheme_contents = &fs::read_to_string(&scheme_file_path)
                .with_context(|| format!("Couldn't read scheme file at {:?}.", scheme_file_path))?;

            let template_contents =
                &fs::read_to_string(&template_file_path).with_context(|| {
                    format!("Couldn't read template file at {:?}.", template_file_path)
                })?;

            build::build(scheme_slug, scheme_contents, template_contents)
        }

        Some(("list", sub_matches)) => {
            let patterns = match sub_matches.values_of("pattern") {
                Some(content) => content.collect(),
                //Defaults to wildcard
                None => vec!["*"],
            };
            let theme = sub_matches.value_of("theme").unwrap();
            let lines = sub_matches.is_present("lines");

            if sub_matches.is_present("templates") {
                list_templates::list(
                    patterns,
                    &flavours_dir,
                    &flavours_config_dir,
                    verbose,
                    lines,
                )
            } else {
                list::list(
                    patterns,
                    &theme,
                    &flavours_dir,
                    &flavours_config_dir,
                    verbose,
                    lines,
                )
            }
        }

        Some(("update", sub_matches)) => {
            let operation = sub_matches
                .value_of("operation")
                .ok_or_else(|| anyhow!("Invalid operation"))?;
            update::update(operation, &flavours_dir, verbose, &flavours_config)
        }

        Some(("info", sub_matches)) => {
            let patterns = match sub_matches.values_of("pattern") {
                Some(content) => content.collect(),
                //Defaults to wildcard
                None => vec!["*"],
            };
            let raw = sub_matches.is_present("raw");
            info::info(patterns, &flavours_dir, &flavours_config_dir, raw)
        }

        Some(("generate", sub_matches)) => {
            let slug = sub_matches.value_of("slug").unwrap_or("generated").into();
            let name = sub_matches.value_of("name").unwrap_or("Generated").into();
            let author = sub_matches.value_of("author").unwrap_or("Flavours").into();

            let image = match sub_matches.value_of("file") {
                Some(content) => Path::new(content)
                    .canonicalize()
                    .with_context(|| "Invalid image file supplied"),
                None => Err(anyhow!("No image file specified")),
            }?;

            let mode = match sub_matches.value_of("mode") {
                Some("dark") => Ok(generate::Mode::Dark),
                Some("light") => Ok(generate::Mode::Light),
                Some("auto") => {
                    let img_buffer = image::open(&image)?;
                    let img_pixels = img_buffer.to_rgba8().into_raw();

                    // Use color thief to get a palette
                    let palette = color_thief::get_palette(
                        img_pixels.as_slice(),
                        color_thief::ColorFormat::Rgba,
                        1,
                        15,
                    )?;

                    // Calculate the average luminance of the colors in the palette
                    let total_luminance: f32 = palette
                        .iter()
                        .map(|&color| {
                            let srgb_color: Srgb = Srgb::new(
                                f32::from(color.r) / 255.0,
                                f32::from(color.g) / 255.0,
                                f32::from(color.b) / 255.0,
                            );

                            let red = srgb_color.red as f32 * 0.222;
                            let green = srgb_color.green as f32 * 0.707;
                            let blue = srgb_color.blue as f32 * 0.071;

                            red + green + blue
                        })
                        .sum();

                    let average_luminance = total_luminance / palette.len() as f32;

                    if verbose {
                        println!("The average luminance is: {}", average_luminance);
                    };

                    if average_luminance > 0.6 {
                        Ok(generate::Mode::Light)
                    } else {
                        Ok(generate::Mode::Dark)
                    }
                }
                _ => Err(anyhow!("No valid mode specified")),
            }?;

            let to_stdout = sub_matches.is_present("stdout");

            let colors = generate::generate(&image, mode, verbose)?;
            let scheme = Scheme {
                scheme: name,
                slug,
                author,
                colors: colors
                    .into_iter()
                    .enumerate()
                    .map(|(index, color)| {
                        let mut rgb_color = [0u8; 3];
                        hex::decode_to_slice(color, &mut rgb_color)?;
                        Ok((BaseIndex(index.try_into()?), RgbColor(rgb_color)))
                    })
                    .collect::<Result<BTreeMap<_, _>>>()?,
            };

            if to_stdout {
                print!("{}", serde_yaml::to_string(&scheme)?);
            } else {
                let path = flavours_dir
                    .join("base16")
                    .join("schemes")
                    .join("generated");
                if !path.exists() {
                    create_dir_all(&path)
                        .with_context(|| format!("Couldn't create directory {:?}", &path))?;
                }
                let file_path = &path.join(format!("{}.yaml", &scheme.slug));
                write(file_path, serde_yaml::to_string(&scheme)?)
                    .with_context(|| format!("Couldn't write scheme file at {:?}", path))?;
            }
            Ok(())
        }
        _ => Err(anyhow!("No valid subcommand specified")),
    }
}
