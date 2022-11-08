use minify_html::{minify, Cfg};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let minify_cfg = Cfg::new();
    let templates = fs::read_dir("templates/src")?;
    for template in templates {
        let template = template?;
        fs::write(
            format!("templates/{}", template.file_name().to_string_lossy()),
            minify(&fs::read(template.path().as_path())?, &minify_cfg),
        )?;
    }

    println!("cargo:rerun-if-changed=templates/src");

    Ok(())
}
