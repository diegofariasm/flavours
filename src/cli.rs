use clap::{builder::PossibleValue, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, subcommand_required(true))]
pub struct Flavours {
    /// Specify a configuration file (Defaults to ~/.config/flavours/config.toml on Linux)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Be more verbose
    #[arg(short, long)]
    pub verbose: bool,

    /// Specify a data directory (Defaults to ~/.local/share/flavours on Linux)
    #[arg(short, long)]
    pub directory: Option<PathBuf>,

    #[command(subcommand)]
    pub commands: FlavoursCommand,
}

#[derive(Parser)]
pub struct LuminanceArg {
    /// Specific theme luminance to filter.
    #[arg(short, long, default_value = "all", value_parser = [PossibleValue::new("all"), PossibleValue::new("dark"), PossibleValue::new("light")])]
    pub luminance: String,
}

#[derive(Parser)]
pub struct PatternArg {
    /// Scheme name or glob pattern to match when showing scheme(s). If ommited, defaults to * (all installed schemes).
    pub pattern: Option<Vec<String>>,
}

#[derive(Subcommand)]
pub enum FlavoursCommand {
    /// Applies scheme, according to user configuration
    Apply {
        /// Whether to run flavours in lightweight mode.
        #[arg(short, long)]
        lightweight: bool,

        /// Reads scheme from stdin instead of from flavours directory.
        #[arg(short, long)]
        stdin: bool,

        #[command(flatten)]
        pattern_arg: PatternArg,

        #[command(flatten)]
        luminance_arg: LuminanceArg,
    },

    /// Builds a template with given scheme
    Build {
        /// Scheme or scheme file to use when building
        scheme: String,

        /// Template or scheme file to use when building
        template: String,

        /// Subtemplate to use when building with user defined templates
        subtemplate: String,
    },

    /// Get information from the last applied scheme
    Current,
    /// Generate schemes based on images
    Generate {
        /// Scheme slug (the name you specify when applying schemes) to output to. If ommited, defaults to 'generated'
        #[arg(short, long, default_value = "generated")]
        slug: String,

        /// In which luminance mode to generate the scheme
        #[arg(short, long, default_value = "auto", value_parser = [PossibleValue::new("auto"), PossibleValue::new("dark"), PossibleValue::new("light")])]
        mode: Option<String>,

        /// Scheme display name (can include spaces and capitalization) to write, defaults to 'generated'
        #[arg(short, long, default_value = "generated")]
        name: String,

        /// Scheme author info (name, email, etc) to write, defaults to 'flavours'
        #[arg(short, long, default_value = "flavours")]
        author: String,

        /// Outputs scheme to stdout instead of writing it to a file.
        #[arg(short, long)]
        stdout: bool,

        /// Image file from where to generate scheme
        image: PathBuf,
    },

    /// Prints a list with all matching schemes
    List {
        /// Print each scheme on its own line
        #[arg(short, long)]
        lines: bool,

        /// List templates instead of schemes
        #[arg(short, long)]
        templates: bool,

        #[command(flatten)]
        luminance_arg: LuminanceArg,

        #[command(flatten)]
        pattern_arg: PatternArg,
    },

    /// Shows scheme colors for all schemes matching pattern. Optionally uses truecolor
    Info {
        /// Don't pretty print the colors.
        #[arg(short, long)]
        raw: bool,

        #[command(flatten)]
        pattern_arg: PatternArg,
    },

    /// Downloads schemes, templates, or updates their lists (from repos specified in sources.yml)
    Update {
        ///  Downloads schemes, templates, or updates their lists (from repos specified in sources.yml)
        #[arg(short, long, default_value = "all", value_parser = ["lists", "schemes", "templates", "all"])]
        operation: String,
    },
}
