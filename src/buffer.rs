// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Wrappers and helpers around `VABuffer`s.

mod h264;
mod hevc;
mod mpeg2;
mod vp8;
mod vp9;

pub use h264::*;
pub use hevc::*;
pub use mpeg2::*;
pub use vp8::*;
pub use vp9::*;

use std::rc::Rc;

use log::error;

use crate::bindings;
use crate::va_check;
use crate::Context;
use crate::VaError;

/// Wrapper type representing a buffer created with `vaCreateBuffer`.
pub struct Buffer {
    context: Rc<Context>,
    id: bindings::VABufferID,
}

impl Buffer {
    /// Creates a new buffer by wrapping a `vaCreateBuffer` call. This is just a helper for
    /// [`Context::create_buffer`].
    pub(crate) fn new(context: Rc<Context>, mut type_: BufferType) -> Result<Self, VaError> {
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
                PictureParameter::HEVC(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                PictureParameter::HEVCRext(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                PictureParameter::HEVCScc(ref mut wrapper) => (
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
                SliceParameter::HEVC(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
                SliceParameter::HEVCRext(ref mut wrapper) => (
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
                IQMatrix::HEVC(ref mut wrapper) => (
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

            BufferType::EncSequenceParameter(ref mut seq_param) => match seq_param {
                EncSequenceParameter::H264(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
            },

            BufferType::EncPictureParameter(ref mut picture_param) => match picture_param {
                EncPictureParameter::H264(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
            },

            BufferType::EncSliceParameter(ref mut slice_param) => match slice_param {
                EncSliceParameter::H264(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
            },

            BufferType::EncMacroblockParameterBuffer(ref mut mb_param) => match mb_param {
                EncMacroblockParameterBuffer::H264(ref mut wrapper) => (
                    wrapper.inner_mut() as *mut _ as *mut std::ffi::c_void,
                    std::mem::size_of_val(wrapper.inner_mut()),
                ),
            },
        };

        // Safe because `self` represents a valid `VAContext`. `ptr` and `size` are also ensured to
        // be correct, as `ptr` is just a cast to `*c_void` from a Rust struct, and `size` is
        // computed from `std::mem::size_of_val`.
        va_check(unsafe {
            bindings::vaCreateBuffer(
                context.display().handle(),
                context.id(),
                type_.inner(),
                size as u32,
                1,
                ptr,
                &mut buffer_id,
            )
        })?;

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
        let status = va_check(unsafe {
            bindings::vaDestroyBuffer(self.context.display().handle(), self.id)
        });

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
    /// Abstraction over `VAEncSequenceParameterBufferType`. Needed for MPEG2, VP8, VP9, H264, HEVC.
    EncSequenceParameter(EncSequenceParameter),
    /// Abstraction over `VAEncPictureParameterBufferType`. Needed for MPEG2, VP8, VP9, H264, HEVC.
    EncPictureParameter(EncPictureParameter),
    /// Abstraction over `VAEncSliceParameterBufferType`. Needed for MPEG2, VP8, VP9, H264, HEVC.
    EncSliceParameter(EncSliceParameter),
    /// Abstraction over `VAEncMacroblockMapBufferType`. Needed for H264.
    EncMacroblockParameterBuffer(EncMacroblockParameterBuffer),
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

            BufferType::EncSequenceParameter(_) => {
                bindings::VABufferType::VAEncSequenceParameterBufferType
            }

            BufferType::EncPictureParameter(_) => {
                bindings::VABufferType::VAEncPictureParameterBufferType
            }

            BufferType::EncSliceParameter(_) => {
                bindings::VABufferType::VAEncSliceParameterBufferType
            }

            BufferType::EncMacroblockParameterBuffer(_) => {
                bindings::VABufferType::VAEncMacroblockMapBufferType
            }
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
    /// Wrapper over VAPictureParameterBufferHEVC
    HEVC(hevc::PictureParameterBufferHEVC),
    /// Wrapper over VAPictureParameterBufferHEVCRext
    HEVCRext(hevc::PictureParameterBufferHEVCRext),
    /// Wrapper over VAPictureParameterBufferHEVCScc
    HEVCScc(hevc::PictureParameterBufferHEVCScc),
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
    /// Wrapper over VASliceParameterBufferHEVC
    HEVC(hevc::SliceParameterBufferHEVC),
    /// Wrapper over VASliceParameterBufferHEVCRext
    HEVCRext(hevc::SliceParameterBufferHEVCRext),
}

/// Abstraction over the `IQMatrixBuffer` types we support.
pub enum IQMatrix {
    /// Abstraction over `VAIQMatrixBufferMPEG2`
    MPEG2(mpeg2::IQMatrixBufferMPEG2),
    /// Abstraction over `VAIQMatrixBufferVP8`
    VP8(vp8::IQMatrixBufferVP8),
    /// Abstraction over `VAIQMatrixBufferH264`
    H264(h264::IQMatrixBufferH264),
    /// Abstraction over `VAIQMatrixBufferHEVC`
    HEVC(hevc::IQMatrixBufferHEVC),
}

/// Abstraction over the `EncSequenceParameter` types we support.
pub enum EncSequenceParameter {
    /// Abstraction over `VAEncSequenceParameterBufferH264`
    H264(h264::EncSequenceParameterBufferH264),
}

/// Abstraction over the `EncPictureParameter` types we support.
pub enum EncPictureParameter {
    /// Abstraction over `VAEncPictureParameterBufferH264`
    H264(h264::EncPictureParameterBufferH264),
}

/// Abstraction over the `EncSliceParameter` types we support.
pub enum EncSliceParameter {
    /// Abstraction over `VAEncSliceParameterBufferH264`
    H264(h264::EncSliceParameterBufferH264),
}

/// Abstraction over the `EncMacroblockParameterBuffer` types we support.
pub enum EncMacroblockParameterBuffer {
    /// Abstraction over `VAEncMacroblockParameterBufferH264`
    H264(h264::EncMacroblockParameterBufferH264),
}
