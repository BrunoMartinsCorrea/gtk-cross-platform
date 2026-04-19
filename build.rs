// SPDX-License-Identifier: GPL-3.0-or-later
use std::env;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let app_id = env::var("APP_ID")
        .unwrap_or_else(|_| "com.example.GtkCrossPlatform".to_string());
    let profile = env::var("PROFILE")
        .unwrap_or_else(|_| "default".to_string());
    let localedir = env::var("LOCALEDIR")
        .unwrap_or_else(|_| "/usr/local/share/locale".to_string());
    let pkgdatadir = env::var("PKGDATADIR")
        .unwrap_or_else(|_| "/usr/local/share/gtk-cross-platform".to_string());
    let source_datadir = env::var("SOURCE_DATADIR")
        .unwrap_or_else(|_| format!("{}/data", manifest_dir));

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

    glib_build_tools::compile_resources(
        &["data/resources"],
        "data/resources/resources.gresource.xml",
        "compiled.gresource",
    );
}
