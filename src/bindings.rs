// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! This module implements the bindgen C FFI bindings for use within this crate

#[allow(missing_docs)]
#[allow(clippy::useless_transmute)]
#[allow(clippy::too_many_arguments)]
#[allow(non_upper_case_globals)]
#[allow(non_snake_case)]
#[allow(dead_code)]
pub mod va;

pub use va::*;
