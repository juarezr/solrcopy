// build.rs

#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;

use clap::CommandFactory;
use clap_complete::{generate_to, shells::*};
use std::io::Error;

#[path = "src/helpers.rs"]
mod helpers;
include!("src/args.rs");

fn main() -> Result<(), Error> {
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/args.rs");

    // helpers::print_env_vars();

    build_artifacts()
}

pub(crate) fn build_artifacts() -> Result<(), Error> {
    let pkg_dir = std::env::var("CARGO_MANIFEST_DIR").expect("# Missing $CARGO_MANIFEST_DIR!");
    let target = std::env::var("TARGET").expect("# Missing $TARGET!");

    let out_dir = if cfg!(debug_assertions) {
        format!("{}/target/debug", pkg_dir)
    } else {
        format!("{}/target/{}/release", pkg_dir, target)
    };
    eprintln!("# out_dir: {}", out_dir);

    let pkg_name = std::env::var("CARGO_PKG_NAME").expect("# Missing $CARGO_PKG_NAME!");

    let mut cmd = Cli::command();

    let path = generate_to(Bash, &mut cmd, &pkg_name, &out_dir)?;
    println!("cargo:info=Bash completion file is generated: {:?}", path);
    let path = generate_to(Zsh, &mut cmd, &pkg_name, &out_dir)?;
    println!("cargo:info=Zsh completion file is generated: {:?}", path);
    let path = generate_to(PowerShell, &mut cmd, &pkg_name, &out_dir)?;
    println!("cargo:info=PowerShell completion file is generated: {:?}", path);
    let path = generate_to(Fish, &mut cmd, &pkg_name, &out_dir)?;
    println!("cargo:info=Fish completion file is generated: {:?}", path);
    let path = generate_to(Elvish, &mut cmd, &pkg_name, &out_dir)?;
    println!("cargo:info=Elvish completion file is generated: {:?}", path);

    Ok(())
}

// end of build script \\
