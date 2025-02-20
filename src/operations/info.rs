use anyhow::Result;
use base16_color_scheme::scheme::{RgbColor, RgbColorFormatter};
use calm_io::stdoutln;

fn true_color(hex_color: &str, background: bool) -> Result<String> {
    let rgb = hex::decode(hex_color)?;

    let code = if background { 48 } else { 38 };

    Ok(format!("\x1b[{};2;{};{};{}m", code, rgb[0], rgb[1], rgb[2]))
}

pub fn print_color(color: &str) -> Result<()> {
    const RESETCOLOR: &str = "\x1b[0m";
    match stdoutln!(
        "{} #{} {}  {}#{}{}",
        true_color(color, true)?,
        color,
        RESETCOLOR,
        true_color(color, false)?,
        color,
        RESETCOLOR
    ) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::BrokenPipe => Ok(()),
            _ => Err(e),
        },
    }?;
    Ok(())
}

pub fn print_color_rgb(color: RgbColor) -> Result<()> {
    use base16_color_scheme::template::color_field::{Format, Hex};
    use std::fmt::{self, Display, Formatter};

    struct TrueColor(RgbColor, bool);

    impl Display for TrueColor {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            let RgbColor([r, g, b]) = self.0;
            let code = if self.1 { 48 } else { 38 };
            write!(f, "\x1b[{code};2;{r};{g};{b}m")
        }
    }

    const RESETCOLOR: &str = "\x1b[0m";

    let true_color_fg = TrueColor(color, true);
    let true_color_bg = TrueColor(color, false);

    let color = RgbColorFormatter {
        color,
        format: Format::Hex(Hex::Rgb),
    };
    match stdoutln!("{true_color_fg} #{color} {RESETCOLOR}  {true_color_bg}#{color}{RESETCOLOR}",) {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            std::io::ErrorKind::BrokenPipe => Ok(()),
            _ => Err(e),
        },
    }?;
    Ok(())
}
