#![feature(exit_status_error)]

use std::fs;
use std::io::{BufReader, Write};
use std::path::Path;

use glob::glob;
use procss::BuildCss;

fn get_version_from_package() -> Option<String> {
    let file = fs::File::open("./package.json").ok()?;
    let reader = BufReader::new(file);
    let value: serde_json::Value = serde_json::from_reader(reader).ok()?;
    let version = value.as_object().unwrap().get("version")?.as_str()?;
    Some(version.to_owned())
}

fn with_wd<T>(indir: &str, f: impl FnOnce() -> T) -> T {
    let current_dir = std::env::current_dir().unwrap();
    std::env::set_current_dir(indir).unwrap();
    let res = f();
    std::env::set_current_dir(current_dir).unwrap();
    res
}

fn glob_with_wd(indir: &str, input: &str) -> Vec<String> {
    with_wd(indir, || {
        glob(input)
            .unwrap()
            .map(|x| x.unwrap().to_string_lossy().to_string())
            .collect()
    })
}

fn main() -> Result<(), anyhow::Error> {
    println!("cargo:rerun-if-changed=build.rs");
    let mut build = BuildCss::new("./src/less");
    let files = glob_with_wd("./src/less", "**/*.less");
    for src in files.iter() {
        build.add_file(src);
    }

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let out_path = Path::new(&out_dir);
    // let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").unwrap();
    // let manifest_path = Path::new(&manifest_dir);

    let mut file = fs::File::create(out_path.join("out_dir.rs"))?;
    write!(
        &mut file,
        r##"

pub mod out_dir {{

#[macro_export]
macro_rules! css {{
    ($name:expr) => {{ {{
        (
            $name,
            include_str!(concat!(
                {:?},
                "/css/",
                $name,
                ".css"
            )),
        )
    }} }};
    ($path:expr, $name:expr) => {{ {{
        (
            $name,
            include_str!(concat!(
                {:?},
                "/",
                $path,
                "/",
                $name,
                ".css"
            )),
        )
    }} }};
}}

pub use css;
}}

"##,
        out_path,
        out_path
    )?;

    build.compile()?.write(out_path.join("./css"))?;

    let mut build = BuildCss::new("./src/themes");
    build.add_file("variables.less");
    build.add_file("fonts.less");
    build.add_file("pro.less");
    build.add_file("pro-dark.less");
    build.add_file("monokai.less");
    build.add_file("solarized.less");
    build.add_file("solarized-dark.less");
    build.add_file("vaporwave.less");
    build.add_file("themes.less");
    build.compile()?.write(out_path.join("./themes"))?;

    println!(
        "cargo:rustc-env=PKG_VERSION={}",
        get_version_from_package().expect("Version not detected")
    );

    Ok(())
}
