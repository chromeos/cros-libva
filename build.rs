// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

fn main() {
    // Do not require dependencies when generating docs.
    if std::env::var("CARGO_DOC").is_ok() || std::env::var("DOCS_RS").is_ok() {
        return;
    }

    match pkg_config::probe_library("libva") {
        Ok(_) => (),
        Err(e) => panic!("libva not found: {}", e),
    };

    println!("cargo:rustc-link-lib=dylib=va");
    println!("cargo:rustc-link-lib=dylib=va-drm"); // for the vaGetDisplayDRM entrypoint
}
