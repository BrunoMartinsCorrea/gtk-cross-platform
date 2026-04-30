// SPDX-License-Identifier: GPL-3.0-or-later
use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let app_id = env::var("APP_ID").unwrap_or_else(|_| "com.example.GtkCrossPlatform".to_string());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "default".to_string());
    let localedir = env::var("LOCALEDIR").unwrap_or_else(|_| {
        // On macOS, detect Homebrew prefix at compile time (arm64=/opt/homebrew, x86=/usr/local).
        if cfg!(target_os = "macos") {
            let arm = "/opt/homebrew/share/locale";
            let x86 = "/usr/local/share/locale";
            if std::path::Path::new(arm).exists() {
                arm
            } else {
                x86
            }
            .to_string()
        } else {
            "/usr/share/locale".to_string()
        }
    });
    let pkgdatadir = env::var("PKGDATADIR")
        .unwrap_or_else(|_| "/usr/local/share/gtk-cross-platform".to_string());
    let source_datadir =
        env::var("SOURCE_DATADIR").unwrap_or_else(|_| format!("{}/data", manifest_dir));

    println!("cargo:rustc-env=APP_ID={}", app_id);
    println!("cargo:rustc-env=PROFILE={}", profile);
    println!("cargo:rustc-env=LOCALEDIR={}", localedir);
    println!("cargo:rustc-env=PKGDATADIR={}", pkgdatadir);
    println!("cargo:rustc-env=SOURCE_DATADIR={}", source_datadir);
    println!("cargo:rustc-env=GETTEXT_PACKAGE=gtk-cross-platform");

    println!("cargo:rerun-if-env-changed=APP_ID");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=LOCALEDIR");
    println!("cargo:rerun-if-env-changed=PKGDATADIR");
    println!("cargo:rerun-if-env-changed=SOURCE_DATADIR");
    println!("cargo:rerun-if-changed=build.rs");

    compile_blueprints();

    glib_build_tools::compile_resources(
        &["data/resources", "data"],
        "data/resources/resources.gresource.xml",
        "compiled.gresource",
    );
}

fn compile_blueprints() {
    let blp_files = ["data/resources/window.blp"];

    for f in &blp_files {
        println!("cargo:rerun-if-changed={f}");
    }

    let mut cmd = std::process::Command::new("blueprint-compiler");
    cmd.arg("batch-compile");

    // blueprint-compiler needs GObject Introspection typelibs to resolve widget types.
    // On macOS the Homebrew girepository path is not in the default search path.
    if cfg!(target_os = "macos") {
        let typelib_path =
            if std::path::Path::new("/opt/homebrew/lib/girepository-1.0").exists() {
                "/opt/homebrew/lib/girepository-1.0"
            } else {
                "/usr/local/lib/girepository-1.0"
            };
        cmd.args(["--typelib-path", typelib_path]);
    }

    // Positional args: output-dir input-dir filenames...
    cmd.args(["data/resources", "data/resources"]);
    cmd.args(&blp_files);

    match cmd.status() {
        Ok(s) if s.success() => {}
        Ok(_) => println!(
            "cargo:warning=blueprint-compiler exited with error; using committed .ui files"
        ),
        Err(_) => println!(
            "cargo:warning=blueprint-compiler not found; using committed .ui files. \
             See https://gnome.pages.gitlab.gnome.org/blueprint-compiler/"
        ),
    }
}
