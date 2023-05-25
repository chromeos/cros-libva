// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::error::Error;
use std::ffi::CStr;
use std::fmt::Display;
use std::num::NonZeroI32;

use crate::bindings;
use crate::bindings::constants;
use crate::bindings::VAStatus;

/// A `VAStatus` that is guaranteed to not be `VA_STATUS_SUCCESS`.
#[derive(Debug)]
pub struct VaError(NonZeroI32);

impl VaError {
    /// Returns the `VAStatus` of this error.
    pub fn va_status(&self) -> VAStatus {
        self.0.get() as VAStatus
    }
}

impl Display for VaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Safe because `vaErrorStr` will return a pointer to a statically allocated, null
        // terminated C string. The pointer is guaranteed to never be null.
        let err_str = unsafe { CStr::from_ptr(bindings::vaErrorStr(self.0.get())) }
            .to_str()
            .unwrap();
        f.write_str(err_str)
    }
}

impl Error for VaError {}

/// Wrapper over `VAStatus`, calling check() returns a Error if the status is not VA_STATUS_SUCCESS.
#[must_use = "VAStatus might not be VA_STATUS_SUCCESS."]
pub(crate) struct VaStatus(pub VAStatus);

impl VaStatus {
    /// Returns `Ok(())` if this status is successful, and an error otherwise.
    pub(crate) fn check(&self) -> Result<(), VaError> {
        match self.0 as u32 {
            constants::VA_STATUS_SUCCESS => Ok(()),
            _ => Err(VaError(unsafe { NonZeroI32::new_unchecked(self.0) })),
        }
    }
}
