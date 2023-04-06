// Copyright 2023 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Wrappers around H264 `VABuffer` types.

use crate::bindings;

/// Wrapper over the `VAPictureH264` FFI type.
pub struct PictureH264(bindings::VAPictureH264);

impl PictureH264 {
    /// Creates the wrapper
    pub fn new(
        picture_id: bindings::VASurfaceID,
        frame_idx: u32,
        flags: u32,
        top_field_order_cnt: i32,
        bottom_field_order_cnt: i32,
    ) -> Self {
        Self(bindings::VAPictureH264 {
            picture_id,
            frame_idx,
            flags,
            TopFieldOrderCnt: top_field_order_cnt,
            BottomFieldOrderCnt: bottom_field_order_cnt,
            va_reserved: Default::default(),
        })
    }
}

/// Wrapper over the `seq_fields` bindgen field in `VAPictureParameterBufferH264`.
pub struct H264SeqFields(bindings::_VAPictureParameterBufferH264__bindgen_ty_1);

impl H264SeqFields {
    /// Creates the bindgen field
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chroma_format_idc: u32,
        residual_colour_transform_flag: u32,
        gaps_in_frame_num_value_allowed_flag: u32,
        frame_mbs_only_flag: u32,
        mb_adaptive_frame_field_flag: u32,
        direct_8x8_inference_flag: u32,
        min_luma_bi_pred_size8x8: u32,
        log2_max_frame_num_minus4: u32,
        pic_order_cnt_type: u32,
        log2_max_pic_order_cnt_lsb_minus4: u32,
        delta_pic_order_always_zero_flag: u32,
    ) -> Self {
        let _bitfield_1 =
            bindings::_VAPictureParameterBufferH264__bindgen_ty_1__bindgen_ty_1::new_bitfield_1(
                chroma_format_idc,
                residual_colour_transform_flag,
                gaps_in_frame_num_value_allowed_flag,
                frame_mbs_only_flag,
                mb_adaptive_frame_field_flag,
                direct_8x8_inference_flag,
                min_luma_bi_pred_size8x8,
                log2_max_frame_num_minus4,
                pic_order_cnt_type,
                log2_max_pic_order_cnt_lsb_minus4,
                delta_pic_order_always_zero_flag,
            );

        Self(bindings::_VAPictureParameterBufferH264__bindgen_ty_1 {
            bits: bindings::_VAPictureParameterBufferH264__bindgen_ty_1__bindgen_ty_1 {
                _bitfield_align_1: Default::default(),
                _bitfield_1,
                __bindgen_padding_0: Default::default(),
            },
        })
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&mut self) -> &bindings::_VAPictureParameterBufferH264__bindgen_ty_1 {
        &self.0
    }
}

/// Wrapper over the `pic_fields` bindgen field in `VAPictureParameterBufferH264`.
pub struct H264PicFields(bindings::_VAPictureParameterBufferH264__bindgen_ty_2);

impl H264PicFields {
    /// Creates the bindgen field
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entropy_coding_mode_flag: u32,
        weighted_pred_flag: u32,
        weighted_bipred_idc: u32,
        transform_8x8_mode_flag: u32,
        field_pic_flag: u32,
        constrained_intra_pred_flag: u32,
        pic_order_present_flag: u32,
        deblocking_filter_control_present_flag: u32,
        redundant_pic_cnt_present_flag: u32,
        reference_pic_flag: u32,
    ) -> Self {
        let _bitfield_1 =
            bindings::_VAPictureParameterBufferH264__bindgen_ty_2__bindgen_ty_1::new_bitfield_1(
                entropy_coding_mode_flag,
                weighted_pred_flag,
                weighted_bipred_idc,
                transform_8x8_mode_flag,
                field_pic_flag,
                constrained_intra_pred_flag,
                pic_order_present_flag,
                deblocking_filter_control_present_flag,
                redundant_pic_cnt_present_flag,
                reference_pic_flag,
            );

        Self(bindings::_VAPictureParameterBufferH264__bindgen_ty_2 {
            bits: bindings::_VAPictureParameterBufferH264__bindgen_ty_2__bindgen_ty_1 {
                _bitfield_align_1: Default::default(),
                _bitfield_1,
                __bindgen_padding_0: Default::default(),
            },
        })
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&mut self) -> &bindings::_VAPictureParameterBufferH264__bindgen_ty_2 {
        &self.0
    }
}

/// A wrapper over `VAPictureParameterBufferH264` FFI type
pub struct PictureParameterBufferH264(Box<bindings::VAPictureParameterBufferH264>);

impl PictureParameterBufferH264 {
    /// Creates the wrapper
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        curr_pic: PictureH264,
        reference_frames: [PictureH264; 16],
        picture_width_in_mbs_minus1: u16,
        picture_height_in_mbs_minus1: u16,
        bit_depth_luma_minus8: u8,
        bit_depth_chroma_minus8: u8,
        num_ref_frames: u8,
        seq_fields: &H264SeqFields,
        num_slice_groups_minus1: u8,
        slice_group_map_type: u8,
        slice_group_change_rate_minus1: u16,
        pic_init_qp_minus26: i8,
        pic_init_qs_minus26: i8,
        chroma_qp_index_offset: i8,
        second_chroma_qp_index_offset: i8,
        pic_fields: &H264PicFields,
        frame_num: u16,
    ) -> Self {
        let reference_frames = (0..16usize)
            .map(|i| reference_frames[i].0)
            .collect::<Vec<_>>()
            .try_into()
            // try_into is guaranteed to work because the iterator and target array have the same
            // size.
            .unwrap();

        let seq_fields = seq_fields.0;
        let pic_fields = pic_fields.0;

        Self(Box::new(bindings::VAPictureParameterBufferH264 {
            CurrPic: curr_pic.0,
            ReferenceFrames: reference_frames,
            picture_width_in_mbs_minus1,
            picture_height_in_mbs_minus1,
            bit_depth_luma_minus8,
            bit_depth_chroma_minus8,
            num_ref_frames,
            seq_fields,
            num_slice_groups_minus1,
            slice_group_map_type,
            slice_group_change_rate_minus1,
            pic_init_qp_minus26,
            pic_init_qs_minus26,
            chroma_qp_index_offset,
            second_chroma_qp_index_offset,
            pic_fields,
            frame_num,
            va_reserved: Default::default(),
        }))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut bindings::VAPictureParameterBufferH264 {
        self.0.as_mut()
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::VAPictureParameterBufferH264 {
        self.0.as_ref()
    }
}

/// Wrapper over the `VASliceParameterBufferH264` FFI type.
pub struct SliceParameterBufferH264(Box<bindings::VASliceParameterBufferH264>);

impl SliceParameterBufferH264 {
    /// Creates the wrapper
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        slice_data_size: u32,
        slice_data_offset: u32,
        slice_data_flag: u32,
        slice_data_bit_offset: u16,
        first_mb_in_slice: u16,
        slice_type: u8,
        direct_spatial_mv_pred_flag: u8,
        num_ref_idx_l0_active_minus1: u8,
        num_ref_idx_l1_active_minus1: u8,
        cabac_init_idc: u8,
        slice_qp_delta: i8,
        disable_deblocking_filter_idc: u8,
        slice_alpha_c0_offset_div2: i8,
        slice_beta_offset_div2: i8,
        ref_pic_list_0: [PictureH264; 32usize],
        ref_pic_list_1: [PictureH264; 32usize],
        luma_log2_weight_denom: u8,
        chroma_log2_weight_denom: u8,
        luma_weight_l0_flag: u8,
        luma_weight_l0: [i16; 32usize],
        luma_offset_l0: [i16; 32usize],
        chroma_weight_l0_flag: u8,
        chroma_weight_l0: [[i16; 2usize]; 32usize],
        chroma_offset_l0: [[i16; 2usize]; 32usize],
        luma_weight_l1_flag: u8,
        luma_weight_l1: [i16; 32usize],
        luma_offset_l1: [i16; 32usize],
        chroma_weight_l1_flag: u8,
        chroma_weight_l1: [[i16; 2usize]; 32usize],
        chroma_offset_l1: [[i16; 2usize]; 32usize],
    ) -> Self {
        let ref_pic_list_0 = ref_pic_list_0.map(|pic| pic.0);
        let ref_pic_list_1 = ref_pic_list_1.map(|pic| pic.0);

        Self(Box::new(bindings::VASliceParameterBufferH264 {
            slice_data_size,
            slice_data_offset,
            slice_data_flag,
            slice_data_bit_offset,
            first_mb_in_slice,
            slice_type,
            direct_spatial_mv_pred_flag,
            num_ref_idx_l0_active_minus1,
            num_ref_idx_l1_active_minus1,
            cabac_init_idc,
            slice_qp_delta,
            disable_deblocking_filter_idc,
            slice_alpha_c0_offset_div2,
            slice_beta_offset_div2,
            RefPicList0: ref_pic_list_0,
            RefPicList1: ref_pic_list_1,
            luma_log2_weight_denom,
            chroma_log2_weight_denom,
            luma_weight_l0_flag,
            luma_weight_l0,
            luma_offset_l0,
            chroma_weight_l0_flag,
            chroma_weight_l0,
            chroma_offset_l0,
            luma_weight_l1_flag,
            luma_weight_l1,
            luma_offset_l1,
            chroma_weight_l1_flag,
            chroma_weight_l1,
            chroma_offset_l1,
            va_reserved: Default::default(),
        }))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut bindings::VASliceParameterBufferH264 {
        self.0.as_mut()
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::VASliceParameterBufferH264 {
        self.0.as_ref()
    }
}

/// Wrapper over the `VAIQMatrixBufferH264` FFI type
pub struct IQMatrixBufferH264(Box<bindings::VAIQMatrixBufferH264>);

impl IQMatrixBufferH264 {
    /// Creates the wrapper.
    pub fn new(
        scaling_list4x4: [[u8; 16usize]; 6usize],
        scaling_list8x8: [[u8; 64usize]; 2usize],
    ) -> Self {
        Self(Box::new(bindings::VAIQMatrixBufferH264 {
            ScalingList4x4: scaling_list4x4,
            ScalingList8x8: scaling_list8x8,
            va_reserved: Default::default(),
        }))
    }

    pub(crate) fn inner_mut(&mut self) -> &mut bindings::VAIQMatrixBufferH264 {
        self.0.as_mut()
    }

    /// Returns the inner FFI type. Useful for testing purposes.
    pub fn inner(&self) -> &bindings::VAIQMatrixBufferH264 {
        self.0.as_ref()
    }
}
