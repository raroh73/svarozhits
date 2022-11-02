use minify_html::{minify, Cfg};
use std::fs;

fn main() {
    let cfg = Cfg::new();

    fs::write(
        "templates/base.html",
        minify(&fs::read("templates/src/base.html").unwrap(), &cfg),
    )
    .unwrap();
    fs::write(
        "templates/index.html",
        minify(&fs::read("templates/src/index.html").unwrap(), &cfg),
    )
    .unwrap();
}
