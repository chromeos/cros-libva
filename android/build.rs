// Copyright 2024 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! cros-libva bindings generator in Android.

include!("../lib/bindgen.rs");

fn main() {
    bindgen_cmd::build(vaapi_gen_builder);
}
