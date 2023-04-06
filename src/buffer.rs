// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Wrappers and helpers around `VABuffer`s.

mod h264;
mod mpeg2;
mod vp8;
mod vp9;

pub use h264::*;
pub use mpeg2::*;
pub use vp8::*;
pub use vp9::*;

use std::rc::Rc;

use anyhow::Result;
use log::error;

use crate::bindings;
use crate::status::Status;
use crate::Context;

/// Wrapper type representing a buffer created with `vaCreateBuffer`.
pub struct Buffer {
    context: Rc<Context>,
    id: bindings::VABufferID,
}

impl Buffer {
    /// Creates a new buffer by wrapping a `vaCreateBuffer` call. This is just a helper for
    /// [`Context::create_buffer`].
    pub(crate) fn new(context: Rc<Context>, mut type_: BufferType) -> Result<Self> {
        let mut buffer_id = 0;

        let (ptr, size) = match type_ {
            BufferType::PictureParameter(ref mut picture_param) => match picture_param {
                PictureParameter::MPEG2(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                PictureParameter::VP8(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                PictureParameter::VP9(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                PictureParameter::H264(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
            },

            BufferType::SliceParameter(ref mut slice_param) => match slice_param {
                SliceParameter::MPEG2(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                SliceParameter::VP8(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                SliceParameter::VP9(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                SliceParameter::H264(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
            },

            BufferType::IQMatrix(ref mut iq_matrix) => match iq_matrix {
                IQMatrix::MPEG2(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                IQMatrix::VP8(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                IQMatrix::H264(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
            },

            BufferType::Probability(ref mut wrapper) => (
                wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                std::mem::size_of_val(wrapper.inner_mut()),
            ),

            BufferType::SliceData(ref mut data) => {
                (data.as_mut_ptr() as *mut std::ffi::c_void, data.len())
            }
        };

        // Safe because `self` represents a valid `VAContext`. `ptr` and `size` are also ensured to
        // be correct, as `ptr` is just a cast to `*c_void` from a Rust struct, and `size` is
        // computed from `std::mem::size_of_val`.
        Status(unsafe {
            bindings::vaCreateBuffer(
                context.display().handle(),
                context.id(),
                type_.inner(),
                size as u32,
                1,
                ptr,
                &mut buffer_id,
            )
        })
        .check()?;

        Ok(Self {
            context,
            id: buffer_id,
        })
    }

    /// Convenience function to return a `VABufferID` vector from a slice of `Buffer`s in order to
    /// easily interface with the C API where a buffer array might be needed.
    pub fn as_id_vec(buffers: &[Self]) -> Vec<bindings::VABufferID> {
        buffers.iter().map(|buffer| buffer.id).collect()
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // Safe because `self` represents a valid buffer, created with
        // vaCreateBuffers.
        let status =
            Status(unsafe { bindings::vaDestroyBuffer(self.context.display().handle(), self.id) })
                .check();
        if status.is_err() {
            error!("vaDestroyBuffer failed: {}", status.unwrap_err());
        }
    }
}

/// Abstraction over `VABufferType`s.
pub enum BufferType {
    /// Abstraction over `VAPictureParameterBufferType`. Needed for MPEG2, VP8, VP9, H264.
    PictureParameter(PictureParameter),
    /// Abstraction over `VASliceParameterBufferType`. Needed for MPEG2, VP8, VP9, H264.
    SliceParameter(SliceParameter),
    /// Abstraction over `VAIQMatrixBufferType`. Needed for VP8, H264.
    IQMatrix(IQMatrix),
    /// Abstraction over `VAProbabilityDataBufferType`. Needed for VP8.
    Probability(vp8::ProbabilityDataBufferVP8),
    /// Abstraction over `VASliceDataBufferType`. Needed for VP9, H264.
    SliceData(Vec<u8>),
}

impl BufferType {
    /// Returns the inner FFI buffer type.
    pub(crate) fn inner(&self) -> bindings::VABufferType::Type {
        match self {
            BufferType::PictureParameter(_) => bindings::VABufferType::VAPictureParameterBufferType,
            BufferType::SliceParameter(_) => bindings::VABufferType::VASliceParameterBufferType,
            BufferType::IQMatrix(_) => bindings::VABufferType::VAIQMatrixBufferType,
            BufferType::Probability(_) => bindings::VABufferType::VAProbabilityBufferType,
            BufferType::SliceData { .. } => bindings::VABufferType::VASliceDataBufferType,
        }
    }
}

/// Abstraction over the `PictureParameterBuffer` types we support.
pub enum PictureParameter {
    /// Wrapper over VAPictureParameterBufferMPEG2.
    MPEG2(mpeg2::PictureParameterBufferMPEG2),
    /// Wrapper over VAPictureParameterBufferVP8.
    VP8(vp8::PictureParameterBufferVP8),
    /// Wrapper over VAPictureParameterBufferVP9.
    VP9(vp9::PictureParameterBufferVP9),
    /// Wrapper over VAPictureParameterBufferH264.
    H264(h264::PictureParameterBufferH264),
}

/// Abstraction over the `SliceParameterBuffer` types we support
pub enum SliceParameter {
    /// Wrapper over VASliceParameterBufferMPEG2
    MPEG2(mpeg2::SliceParameterBufferMPEG2),
    /// Wrapper over VASliceParameterBufferVP8
    VP8(vp8::SliceParameterBufferVP8),
    /// Wrapper over VASliceParameterBufferVP9
    VP9(vp9::SliceParameterBufferVP9),
    /// Wrapper over VASliceParameterBufferH264
    H264(h264::SliceParameterBufferH264),
}

/// Abstraction over the `IQMatrixBuffer` types we support.
pub enum IQMatrix {
    /// Abstraction over `VAIQMatrixBufferMPEG2`
    MPEG2(mpeg2::IQMatrixBufferMPEG2),
    /// Abstraction over `VAIQMatrixBufferVP8`
    VP8(vp8::IQMatrixBufferVP8),
    /// Abstraction over `VAIQMatrixBufferH264`
    H264(h264::IQMatrixBufferH264),
}
