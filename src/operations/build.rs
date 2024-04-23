use anyhow::Result;
use base16_color_scheme::{Scheme, Template};
/// Build a template
///
/// Given template base and scheme, builds the template and returns it
///
/// * `template_base` - Template base string
/// * `scheme` - Scheme structure
pub fn build_template(template_base: &str, scheme: &Scheme) -> Result<String> {
    let template = Template::new(template_base)?;
    Ok(template.render(scheme))
}

/// Build function
///
/// * `scheme_file` - Path to scheme file
/// * `template_file` - Path to template
pub fn build(scheme_slug: String, scheme_contents: &str, template_contents: &str) -> Result<()> {
    let mut scheme: Scheme = serde_yaml::from_str(scheme_contents)?;
    scheme.slug = scheme_slug;

    let template = Template::new(template_contents)?;

    //Template with correct colors
    println!("{}", template.render(&scheme));
    Ok(())
}
