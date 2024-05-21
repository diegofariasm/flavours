use anyhow::{anyhow, Context, Result};
use base16_color_scheme::{
    scheme::{BaseIndex, RgbColor},
    Scheme,
};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use dirs::{data_dir, preference_dir};
use flavours::{cli::Output, find::find_template};
use flavours::{find::find_schemes, operations::list};
use palette::Srgb;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use flavours::cli::{Flavours, FlavoursCommand};
use flavours::operations::{apply, build, current, generate, info, update};

use std::fs::{create_dir_all, write};

// use serde::{Serialize, Deserialize};
// #[derive(Serialize, Deserialize)]
// pub struct Template {
//     name: String,
// }

fn main() -> Result<()> {
    let matches = Flavours::parse();

    // Flavours data directory
    let flavours_dir = match matches.directory {
        // User supplied
        Some(argument) => argument
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
    let flavours_config = match matches.config {
        // User supplied
        Some(path) => path
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
    let verbose = matches.verbose;

    if verbose {
        println!("Using directory: {:?}", flavours_dir);
        println!("Using config file: {:?}", flavours_config);
    };

    // Check which subcommand was used
    match matches.commands {
        FlavoursCommand::Completions { generator } => {
            let cmd = Flavours::command();

            generate(
                generator,
                &mut cmd.clone(),
                cmd.get_name().to_string(),
                &mut io::stdout(),
            );

            Ok(())
        }

        FlavoursCommand::Current => {
            let scheme_name = current::get_current_scheme_name(&flavours_dir)
                .expect("Failed to get current scheme name");

            println!("{}", scheme_name);

            Ok(())
        }

        FlavoursCommand::Apply {
            pattern_arg,
            lightweight,
            luminance_arg,
            stdin,
        } => {
            //Get search patterns
            let patterns = match pattern_arg.pattern {
                Some(content) => content,
                //Defaults to wildcard
                None => vec!["*".to_string()],
            };
            let luminance = luminance_arg.luminance;

            let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();

            apply::apply(
                pattern_refs,
                &luminance,
                &flavours_dir,
                &flavours_config_dir,
                &flavours_config,
                lightweight,
                stdin,
                verbose,
            )
        }

        FlavoursCommand::Build {
            scheme,
            template,
            subtemplate,
        } => {
            let scheme_file_path = if Path::new(&scheme).exists() {
                PathBuf::from(scheme)
            } else {
                find_schemes(&scheme, &flavours_dir, &flavours_config_dir)?
                    .first() // Get the first PathBuf from the vector
                    .ok_or_else(|| anyhow!("Could not find a scheme for {}", scheme))? // Handle None case
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

            let template_file_path = if Path::new(&template).exists() {
                if subtemplate != "default" {
                    return Err(anyhow!(
                        "Using subtemplates is not supported incase you are using a scheme file."
                    ));
                }

                PathBuf::from(template) // Create a PathBuf from the existing path
            } else {
                find_template(&template, &subtemplate, &flavours_dir, &flavours_config_dir)
                    .with_context(|| {
                        format!(
                            "Failed to locate subtemplate file {}/{}",
                            template, subtemplate
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

        FlavoursCommand::List(list_matches) => {
            match list_matches {
                flavours::cli::ListCommand::Schemes {
                    luminance_arg,
                    lines,
                    output_arg,
                    pattern_arg,
                } => {
                    //Get search patterns
                    let patterns = match pattern_arg.pattern {
                        Some(content) => content,
                        //Defaults to wildcard
                        None => vec!["*".to_string()],
                    };

                    let luminance = luminance_arg.luminance;

                    let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();

                    let schemes = list::schemes(
                        pattern_refs,
                        &luminance,
                        &flavours_dir,
                        &flavours_config_dir,
                    )?;

                    if let Some(output_arg) = output_arg.output {
                        match output_arg {
                            Output::Json => {
                                let json_object = serde_json::json!({ "schemes": schemes });

                                let json_string = serde_json::to_string(&json_object)?;
                                println!("{}", json_string);
                            }
                        }
                    } else {
                        for scheme in schemes {
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
                    }

                    Ok(())
                }
                flavours::cli::ListCommand::Templates {
                    lines,
                    output_arg,
                    pattern_arg,
                } => {
                    //Get search patterns
                    let patterns = match pattern_arg.pattern {
                        Some(content) => content,
                        //Defaults to wildcard
                        None => vec!["*".to_string()],
                    };

                    let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();

                    let templates =
                        list::templates(pattern_refs, &flavours_dir, &flavours_config_dir)?;

                    if let Some(output_arg) = output_arg.output {
                        match output_arg {
                            Output::Json => {
                                let json_object = serde_json::json!({ "templates": templates });

                                let json_string = serde_json::to_string(&json_object)?;
                                println!("{}", json_string);
                            }
                        }
                    } else {
                        for template in templates {
                            // Print scheme
                            print!("{}", template);
                            if lines {
                                // Print newline
                                println!();
                            } else {
                                // Print space
                                print!(" ");
                            }
                        }
                    }
                    Ok(())
                }
            }
        }
        FlavoursCommand::Update { operation } => {
            update::update(&operation, &flavours_dir, verbose, &flavours_config)
        }

        FlavoursCommand::Info { raw, pattern_arg } => {
            let patterns = match pattern_arg.pattern {
                Some(content) => content,
                //Defaults to wildcard
                None => vec!["*".to_string()],
            };
            let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();

            info::info(pattern_refs, &flavours_dir, &flavours_config_dir, raw)
        }

        FlavoursCommand::Generate {
            slug,
            name,
            author,
            stdout,
            image,
            mode,
        } => {
            let image_file = image
                .canonicalize()
                .with_context(|| "Image file invalid or not found")?;

            let mode = match mode.as_deref() {
                Some("dark") => Ok(generate::Mode::Dark),
                Some("light") => Ok(generate::Mode::Light),
                Some("auto") => {
                    let img_buffer = image::open(image_file)?;
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

                    if average_luminance > 0.5 {
                        Ok(generate::Mode::Light)
                    } else {
                        Ok(generate::Mode::Dark)
                    }
                }
                _ => Err(anyhow!("No valid mode specified")),
            }?;

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

            if stdout {
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
    }
}
