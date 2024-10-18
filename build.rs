// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::env::{self, VarError};
use std::path::{Path, PathBuf};

/// Environment variable that can be set to point to the directory containing the `va.h`, `va_drm.h` and `va_drmcommon.h`
/// files to use to generate the bindings.
const CROS_LIBVA_H_PATH_ENV: &str = "CROS_LIBVA_H_PATH";

/// Default header file to parse if the `CROS_LIBVA_H_PATH` environment variable is not set.
const DEFAULT_CROS_LIBVA_H_PATH: &str = "/usr/include";

/// Wrapper file to use as input of bindgen.
const WRAPPER_PATH: &str = "libva-wrapper.h";

/// Allow list
const ALLOW_LIST_TYPE : &str = ".*ExternalBuffers.*|.*PRIME.*|.*MPEG2.*|.*VP8.*|.*VP9.*|.*H264.*|.*HEVC.*|VACodedBufferSegment|.*AV1.*|VAEncMisc.*|VASurfaceDecodeMBErrors|VADecodeErrorType";

fn main() {
    // Do not require dependencies when generating docs.
    if std::env::var("CARGO_DOC").is_ok() || std::env::var("DOCS_RS").is_ok() {
        return;
    }

    let va_h_path = env::var(CROS_LIBVA_H_PATH_ENV)
        .or_else(|e| {
            if let VarError::NotPresent = e {
                Ok(DEFAULT_CROS_LIBVA_H_PATH.to_string())
            } else {
                Err(e)
            }
        })
        .expect("invalid `CROS_LIBVA_H_PATH` environment variable");

    // Check the path exists.
    assert!(
        Path::new(&va_h_path).exists(),
        "{} does'nt exist",
        va_h_path
    );

    // Tell cargo to link va and va-drm objects dynamically.
    println!("cargo:rustc-link-lib=dylib=va");
    println!("cargo:rustc-link-lib=dylib=va-drm"); // for the vaGetDisplayDRM entrypoint

    let bindings = bindgen::builder()
        .header(WRAPPER_PATH)
        .clang_arg(format!("-I{}", va_h_path))
        .raw_line("pub mod constants;")
        .derive_default(true)
        .derive_eq(true)
        .layout_tests(false)
        .constified_enum_module("VA.*")
        .allowlist_function("va.*")
        .allowlist_type(ALLOW_LIST_TYPE)
        .generate()
        .expect("unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("`OUT_DIR` is not set"));

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    let bindings = bindgen::builder()
        .header(WRAPPER_PATH)
        .clang_arg(format!("-I{}", va_h_path))
        .allowlist_var("VA.*")
        .generate()
        .expect("unable to generate bindings");
    bindings
        .write_to_file(out_path.join("constants.rs"))
        .expect("Couldn't write bindings!");
}
