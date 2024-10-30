// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::env::{self};
use std::path::{Path, PathBuf};

mod bindgen_gen;
use bindgen_gen::vaapi_gen_builder;

/// Environment variable that can be set to point to the directory containing the `va.h`, `va_drm.h` and `va_drmcommon.h`
/// files to use to generate the bindings.
const CROS_LIBVA_H_PATH_ENV: &str = "CROS_LIBVA_H_PATH";
const CROS_LIBVA_LIB_PATH_ENV: &str = "CROS_LIBVA_LIB_PATH";

/// Wrapper file to use as input of bindgen.
const WRAPPER_PATH: &str = "libva-wrapper.h";

fn main() {
    // Do not require dependencies when generating docs.
    if std::env::var("CARGO_DOC").is_ok() || std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let va_h_path = env::var(CROS_LIBVA_H_PATH_ENV).unwrap_or_default();
    let va_lib_path = env::var(CROS_LIBVA_LIB_PATH_ENV).unwrap_or_default();
    // Check the path exists.
    if !va_h_path.is_empty() {
        assert!(
            Path::new(&va_h_path).exists(),
            "{} doesn't exist",
            va_h_path
        );
    }

    if !va_lib_path.is_empty() {
        assert!(
            Path::new(&va_h_path).exists(),
            "{} doesn't exist",
            va_h_path
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
