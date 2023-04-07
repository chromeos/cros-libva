#!/usr/bin/env bash
# Copyright 2022 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

# Regenerate libva bindgen bindings.

set -euo pipefail
bindgen \
    --raw-line "pub mod constants;" \
    --with-derive-eq \
    --constified-enum-module "VA.*" \
    --allowlist-function "va.*" \
    --allowlist-type ".*MPEG2.*|.*VP8.*|.*VP9.*|.*H264.*" \
    "libva-wrapper.h" \
    > src/bindings/va.rs

bindgen \
    --allowlist-var "VA.*" \
    "libva-wrapper.h" \
    > src/bindings/va/constants.rs
