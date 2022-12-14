use askama::Template;
use std::{env::current_dir, fs, path::Path, process::Command, str};

use clap::{arg, command};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([name] "debug or release?").required(true))
        .get_matches();

    let user_input = matches
        .get_one::<String>("name")
        .expect("Need to specify debug or release");

    let target_dir = match user_input.as_str() {
        "debug" => "../target/wasm32-unknown-unknown/debug/content.wasm",
        "release" => "../target/wasm32-unknown-unknown/release/content.wasm",
        _ => {
            println!("Need to specify debug or release");
            panic!()
        }
    };

    #[derive(Template)]
    #[template(path = "index.html", escape = "none")]
    struct IndexTemplate {
        js: String,
        wasm64: String,
        favicon64: String,
    }

    let pwd = current_dir().unwrap();

    // Create a folder for temp assets
    let new_dir = pwd.join("templates").join("temp_assets");
    match fs::create_dir(&new_dir) {
        Ok(_) => {}
        Err(err) => {
            println!(
                "The temp_assets directory may already exist, but by all rights it
                 shouldn't. Going to try to delete it and recreate it, but it may
                 be worth taking a moment to invesitage this oddity. The error
                 alerting us to this is: {:?}",
                err
            );
            fs::remove_dir_all(&new_dir).unwrap();
            fs::create_dir(&new_dir).unwrap();
        }
    };

    // Generate wasm and js
    let wasm_bindgen = Command::new("wasm-bindgen")
        .arg("--target")
        .arg("no-modules")
        .arg("--no-typescript")
        .arg("--out-dir")
        .arg("templates/temp_assets/")
        .arg(target_dir)
        .output()
        .expect("wasm-bindgen")
        .status;
    assert!(wasm_bindgen.success());

    // TODO Bring base64 in house
    let wasm64 = Command::new("base64")
        .arg("templates/temp_assets/content_bg.wasm")
        .output()
        .expect("base64")
        .stdout;
    let wasm64 = str::from_utf8(&wasm64).expect("wasm64").into();

    // Grab js
    let js_path = Path::new("templates")
        .join("temp_assets")
        .join("content.js");
    let js = fs::read_to_string(js_path).expect("we opened content.js");

    // Grab favicon
    let favicon_path = Path::new("templates").join("favicon.base64");
    let favicon64 = fs::read_to_string(favicon_path).expect("we opened favicon.base64");

    // Pack wasm & js into template
    let index_template = IndexTemplate {
        js,
        wasm64,
        favicon64,
    };
    let contents = index_template.render().unwrap();

    // Delete old index.html
    Command::new("rm")
        .arg("cartridge/index.html")
        .output()
        .expect("delete index.html");

    // Delete temporary assets directory
    Command::new("rm")
        .arg("-rf")
        .arg("templates/temp_assets")
        .output()
        .expect("delete index.html");

    // Create index.html from template
    let index_path = Path::new("cartridge").join("index.html");
    fs::write(index_path, contents).expect("wrote to index.html");
}
