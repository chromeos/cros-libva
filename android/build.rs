// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! cros-libva bindings generator in Android.

#[path = "../lib/bindgen_gen.rs"]
mod bindgen_gen;

fn main() {
    bindgen_cmd::build(bindgen_gen::vaapi_gen_builder);
}
