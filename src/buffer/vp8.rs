// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Wrappers around VP8 `VABuffer` types.

use crate::bindings;

/// Wrapper over the `pic_fields` bindgen field in `VAPictureParameterBufferVP8`.
pub struct VP8PicFields(bindings::_VAPictureParameterBufferVP8__bindgen_ty_1);

impl VP8PicFields {
    /// Creates the bindgen field
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key_frame: u32,
        version: u32,
        segmentation_enabled: u32,
        update_mb_segmentation_map: u32,
        update_segment_feature_data: u32,
        filter_type: u32,
        sharpness_level: u32,
        loop_filter_adj_enable: u32,
        mode_ref_lf_delta_update: u32,
        sign_bias_golden: u32,
        sign_bias_alternate: u32,
        mb_no_coeff_skip: u32,
        loop_filter_disable: u32,
    ) -> Self {
        let _bitfield_1 =
            bindings::_VAPictureParameterBufferVP8__bindgen_ty_1__bindgen_ty_1::new_bitfield_1(
                key_frame,
                version,
                segmentation_enabled,
                update_mb_segmentation_map,
                update_segment_feature_data,
                filter_type,
                sharpness_level,
                loop_filter_adj_enable,
                mode_ref_lf_delta_update,
                sign_bias_golden,
                sign_bias_alternate,
                mb_no_coeff_skip,
                loop_filter_disable,
            );

        Self(bindings::_VAPictureParameterBufferVP8__bindgen_ty_1 {
            bits: bindings::_VAPictureParameterBufferVP8__bindgen_ty_1__bindgen_ty_1 {
                _bitfield_align_1: Default::default(),
                _bitfield_1,
                __bindgen_padding_0: Default::default(),
            },
        })
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::_VAPictureParameterBufferVP8__bindgen_ty_1 {
        &self.0
    }
}

/// Wrapper over the `VABoolCoderContextVPX` FFI type.
pub struct BoolCoderContextVPX(bindings::VABoolCoderContextVPX);

impl BoolCoderContextVPX {
    /// Creates the wrapper
    pub fn new(range: u8, value: u8, count: u8) -> Self {
        Self(bindings::VABoolCoderContextVPX {
            range,
            value,
            count,
        })
    }
}

/// Wrapper over the `PictureParameterBufferVP8` FFI type.
pub struct PictureParameterBufferVP8(Box<bindings::VAPictureParameterBufferVP8>);

impl PictureParameterBufferVP8 {
    /// Creates the wrapper
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        frame_width: u32,
        frame_height: u32,
        last_ref_frame: bindings::VASurfaceID,
        golden_ref_frame: bindings::VASurfaceID,
        alt_ref_frame: bindings::VASurfaceID,
        pic_fields: &VP8PicFields,
        mb_segment_tree_probs: [u8; 3usize],
        loop_filter_level: [u8; 4usize],
        loop_filter_deltas_ref_frame: [i8; 4usize],
        loop_filter_deltas_mode: [i8; 4usize],
        prob_skip_false: u8,
        prob_intra: u8,
        prob_last: u8,
        prob_gf: u8,
        y_mode_probs: [u8; 4usize],
        uv_mode_probs: [u8; 3usize],
        mv_probs: [[u8; 19usize]; 2usize],
        bool_coder_ctx: &BoolCoderContextVPX,
    ) -> Self {
        let pic_fields = pic_fields.0;
        let bool_coder_ctx = bool_coder_ctx.0;

        Self(Box::new(bindings::VAPictureParameterBufferVP8 {
            frame_width,
            frame_height,
            last_ref_frame,
            golden_ref_frame,
            alt_ref_frame,
            out_of_loop_frame: bindings::constants::VA_INVALID_SURFACE,
            pic_fields,
            mb_segment_tree_probs,
            loop_filter_level,
            loop_filter_deltas_ref_frame,
            loop_filter_deltas_mode,
            prob_skip_false,
            prob_intra,
            prob_last,
            prob_gf,
            y_mode_probs,
            uv_mode_probs,
            mv_probs,
            bool_coder_ctx,
            va_reserved: Default::default(),
        }))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut bindings::VAPictureParameterBufferVP8 {
        self.0.as_mut()
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::VAPictureParameterBufferVP8 {
        self.0.as_ref()
    }
}

/// Wrapper over the `VASliceParameterBufferVP8` FFI type.
pub struct SliceParameterBufferVP8(Box<bindings::VASliceParameterBufferVP8>);

impl SliceParameterBufferVP8 {
    /// Creates the wrapper.
    pub fn new(
        slice_data_size: u32,
        slice_data_offset: u32,
        slice_data_flag: u32,
        macroblock_offset: u32,
        num_of_partitions: u8,
        partition_size: [u32; 9usize],
    ) -> Self {
        Self(Box::new(bindings::VASliceParameterBufferVP8 {
            slice_data_size,
            slice_data_offset,
            slice_data_flag,
            macroblock_offset,
            num_of_partitions,
            partition_size,
            va_reserved: Default::default(),
        }))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut bindings::VASliceParameterBufferVP8 {
        self.0.as_mut()
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::VASliceParameterBufferVP8 {
        self.0.as_ref()
    }
}

/// Wrapper over the `VAIQMatrixBufferVP8` FFI type.
pub struct IQMatrixBufferVP8(Box<bindings::VAIQMatrixBufferVP8>);

impl IQMatrixBufferVP8 {
    /// Creates the wrapper.
    pub fn new(quantization_index: [[u16; 6usize]; 4usize]) -> Self {
        Self(Box::new(bindings::VAIQMatrixBufferVP8 {
            quantization_index,
            va_reserved: Default::default(),
        }))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut bindings::VAIQMatrixBufferVP8 {
        self.0.as_mut()
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::VAIQMatrixBufferVP8 {
        self.0.as_ref()
    }
}

/// Wrapper over the VAProbabilityDataBufferVP8 FFI type.
pub struct ProbabilityDataBufferVP8(Box<bindings::VAProbabilityDataBufferVP8>);

impl ProbabilityDataBufferVP8 {
    /// Creates the wrapper.
    pub fn new(dct_coeff_probs: [[[[u8; 11usize]; 3usize]; 8usize]; 4usize]) -> Self {
        Self(Box::new(bindings::VAProbabilityDataBufferVP8 {
            dct_coeff_probs,
            va_reserved: Default::default(),
        }))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut bindings::VAProbabilityDataBufferVP8 {
        self.0.as_mut()
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::VAProbabilityDataBufferVP8 {
        self.0.as_ref()
    }
}
