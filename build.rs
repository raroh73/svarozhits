use minify_html::{minify, Cfg};
use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = Cfg::new();

    fs::write(
        "templates/base.html",
        minify(&fs::read("templates/src/base.html")?, &cfg),
    )?;
    fs::write(
        "templates/index.html",
        minify(&fs::read("templates/src/index.html")?, &cfg),
    )?;

    println!("cargo:rerun-if-changed=templates/src");

    Ok(())
}
