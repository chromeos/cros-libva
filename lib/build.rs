// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use regex::Regex;
use std::env::VarError;
use std::env::{self};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

mod bindgen_gen;
use bindgen_gen::vaapi_gen_builder;

/// Environment variable that can be set to point to the directory containing the `va.h`, `va_drm.h` and `va_drmcommon.h`
/// files to use to generate the bindings.
const CROS_LIBVA_H_PATH_ENV: &str = "CROS_LIBVA_H_PATH";
const CROS_LIBVA_LIB_PATH_ENV: &str = "CROS_LIBVA_LIB_PATH";

/// Wrapper file to use as input of bindgen.
const WRAPPER_PATH: &str = "libva-wrapper.h";

// Return VA_MAJOR_VERSION and VA_MINOR_VERSION from va_version.h.
fn get_va_version(va_h_path: &str) -> (u32, u32) {
    let va_version_h_path = Path::new(va_h_path).join("va/va_version.h");
    assert!(
        va_version_h_path.exists(),
        "{} doesn't exist",
        va_version_h_path.display()
    );
    let header_content = read_to_string(va_version_h_path).unwrap();
    let lines = header_content.lines();

    const VERSION_REGEX_STRINGS: [&str; 2] = [
        r"#define VA_MAJOR_VERSION\s*[0-9]+",
        r"#define VA_MINOR_VERSION\s*[0-9]+",
    ];
    let mut numbers: [u32; 2] = [0; 2];
    for i in 0..2 {
        let re = Regex::new(VERSION_REGEX_STRINGS[i]).unwrap();
        let match_line = lines
            .clone()
            .filter(|&s| re.is_match(s))
            .collect::<Vec<_>>();
        assert!(
            match_line.len() == 1,
            "unexpected match for {}: {:?}",
            VERSION_REGEX_STRINGS[i],
            match_line
        );
        let number_str = Regex::new(r"[0-9]+")
            .unwrap()
            .find(match_line[0])
            .unwrap()
            .as_str();
        numbers[i] = number_str.parse::<u32>().unwrap();
    }

    (numbers[0], numbers[1])
}

fn main() {
    // Do not require dependencies when generating docs.
    if std::env::var("CARGO_DOC").is_ok() || std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let va_h_path = env::var(CROS_LIBVA_H_PATH_ENV)
        .or_else(|e| {
            if let VarError::NotPresent = e {
                let libva_library = pkg_config::probe_library("libva");
                match libva_library {
                    Ok(_) => Ok(libva_library.unwrap().include_paths[0]
                        .clone()
                        .into_os_string()
                        .into_string()
                        .unwrap()),
                    Err(e) => panic!("libva is not found in system: {}", e),
                }
            } else {
                Err(e)
            }
        })
        .expect("libva header location is unknown");

    let va_lib_path = env::var(CROS_LIBVA_LIB_PATH_ENV).unwrap_or_default();
    // Check the path exists.
    if !va_h_path.is_empty() {
        assert!(
            Path::new(&va_h_path).exists(),
            "{} doesn't exist",
            va_h_path
        );
    }

    let (major, minor) = get_va_version(&va_h_path);
    println!("libva {}.{} is used to generate bindings", major, minor);
    let va_check_version = |desired_major: u32, desired_minor: u32| {
        major > desired_major || (major == desired_major && minor >= desired_minor)
    };

    if va_check_version(1, 21) {
        println!("cargo::rustc-cfg=libva_1_21_or_higher");
    }

    if !va_lib_path.is_empty() {
        assert!(
            Path::new(&va_lib_path).exists(),
            "{} doesn't exist",
            va_lib_path
        );
        println!("cargo:rustc-link-arg=-Wl,-rpath={}", va_lib_path);
    }

    // Tell cargo to link va and va-drm objects dynamically.
    println!("cargo:rustc-link-lib=dylib=va");
    println!("cargo:rustc-link-lib=dylib=va-drm"); // for the vaGetDisplayDRM entrypoint

    let mut bindings_builder = vaapi_gen_builder(bindgen::builder()).header(WRAPPER_PATH);
    if !va_h_path.is_empty() {
        bindings_builder = bindings_builder.clang_arg(format!("-I{}", va_h_path));
    }
    let bindings = bindings_builder
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("`OUT_DIR` is not set"));

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
