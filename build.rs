use std::{env, process::Command};

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=Cargo.toml");

    Command::new("cargo").args(["about", "init"]).spawn()?;

    let out_dir = env::var("OUT_DIR").unwrap();
    Command::new("cargo")
        .args([
            "about",
            "generate",
            "about.hbs",
            "-o",
            &format!("{out_dir}/license.html"),
        ])
        .spawn()?;

    Ok(())
}
