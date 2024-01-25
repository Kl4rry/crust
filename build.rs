fn main() -> std::io::Result<()> {
    {
        use std::{env, process::Command};
        println!("cargo:rerun-if-changed=Cargo.toml");

        Command::new("cargo").args(["about", "init"]).spawn()?;

        let out_dir = env::var("OUT_DIR").unwrap();
        let child = Command::new("cargo")
            .args([
                "about",
                "generate",
                "about.hbs",
                "-o",
                &format!("{out_dir}/license.html"),
            ])
            .spawn()?;
        let exit_status = child.wait_with_output()?.status;
        assert!(exit_status.success());
    }

    Ok(())
}
