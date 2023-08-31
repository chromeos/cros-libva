// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! Implements a lightweight and safe interface over `libva`.
//!
//! The starting point to using this crate is to open a [`Display`], from which a [`Context`] and
//! [`Surface`]s can be allocated and used for doing actual work.

mod bindings;
pub mod buffer;
mod config;
mod context;
mod display;
mod generic_value;
mod image;
mod picture;
mod surface;
mod usage_hint;

pub use bindings::constants;
pub use bindings::VAConfigAttrib;
pub use bindings::VAConfigAttribType;
pub use bindings::VADRMPRIMESurfaceDescriptor;
pub use bindings::VAEntrypoint;
pub use bindings::VAImageFormat;
pub use bindings::VAProfile;
pub use bindings::VASurfaceAttrib;
pub use bindings::VASurfaceAttribExternalBuffers;
pub use bindings::VASurfaceAttribType;
pub use bindings::VASurfaceID;
pub use bindings::VASurfaceStatus;
pub use bindings::_VADRMPRIMESurfaceDescriptor__bindgen_ty_1 as VADRMPRIMESurfaceDescriptorObject;
pub use bindings::_VADRMPRIMESurfaceDescriptor__bindgen_ty_2 as VADRMPRIMESurfaceDescriptorLayer;
pub use buffer::*;
pub use config::*;
pub use context::*;
pub use display::*;
pub use generic_value::*;
pub use image::*;
pub use picture::*;
pub use surface::*;
pub use usage_hint::*;

use std::num::NonZeroI32;

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

impl std::fmt::Display for VaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::ffi::CStr;

        // Safe because `vaErrorStr` will return a pointer to a statically allocated, null
        // terminated C string. The pointer is guaranteed to never be null.
        let err_str = unsafe { CStr::from_ptr(bindings::vaErrorStr(self.0.get())) }
            .to_str()
            .unwrap();
        f.write_str(err_str)
    }
}

impl std::error::Error for VaError {}

/// Checks a VA return value and returns a `VaError` if it is not `VA_STATUS_SUCCESS`.
///
/// This can be used on the return value of any VA function returning `VAStatus` in order to
/// convert it to a proper Rust `Result`.
fn va_check(code: VAStatus) -> Result<(), VaError> {
    match code as u32 {
        constants::VA_STATUS_SUCCESS => Ok(()),
        _ => Err(VaError(unsafe { NonZeroI32::new_unchecked(code) })),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::rc::Rc;

    use super::*;

    /// Returns a 32-bit CRC for the visible part of `image`, which must be in NV12 format.
    fn crc_nv12_image(image: &Image) -> u32 {
        let data = image.as_ref();
        let va_image = image.image();
        let offsets = &va_image.offsets;
        let pitches = &va_image.pitches;
        let width = va_image.width as usize;
        let height = va_image.height as usize;

        // We only support NV12 images
        assert_eq!(va_image.format.fourcc, u32::from_ne_bytes(*b"NV12"));
        // Consistency check
        assert_eq!(va_image.num_planes, 2);

        let mut hasher = crc32fast::Hasher::new();

        let offset = offsets[0] as usize;
        let pitch = pitches[0] as usize;
        let y_plane = data[offset..(offset + pitch * height)]
            .chunks(pitch)
            .map(|line| &line[0..width]);

        let offset = offsets[1] as usize;
        let pitch = pitches[1] as usize;
        let uv_plane = data[offset..(offset + pitch * ((height + 1) / 2))]
            .chunks(pitch)
            .map(|line| &line[0..width]);

        for line in y_plane.chain(uv_plane) {
            hasher.update(line);
        }

        hasher.finalize()
    }

    #[test]
    // Ignore this test by default as it requires libva-compatible hardware.
    #[ignore]
    fn libva_utils_mpeg2vldemo() {
        // Adapted from <https://github.com/intel/libva-utils/blob/master/decode/mpeg2vldemo.cpp>
        let display = Display::open().unwrap();

        assert!(!display.query_vendor_string().unwrap().is_empty());
        let profiles = display.query_config_profiles().unwrap();
        assert!(!profiles.is_empty());

        let profile = bindings::VAProfile::VAProfileMPEG2Main;
        let entrypoints = display.query_config_entrypoints(profile).unwrap();
        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| *e == bindings::VAEntrypoint::VAEntrypointVLD));

        let format = bindings::constants::VA_RT_FORMAT_YUV420;
        let width = 16u32;
        let height = 16u32;

        let mut attrs = vec![bindings::VAConfigAttrib {
            type_: bindings::VAConfigAttribType::VAConfigAttribRTFormat,
            value: 0,
        }];

        let entrypoint = bindings::VAEntrypoint::VAEntrypointVLD;
        display
            .get_config_attributes(profile, entrypoint, &mut attrs)
            .unwrap();
        assert!(attrs[0].value != bindings::constants::VA_ATTRIB_NOT_SUPPORTED);
        assert!(attrs[0].value & bindings::constants::VA_RT_FORMAT_YUV420 != 0);

        let config = display.create_config(attrs, profile, entrypoint).unwrap();

        let mut surfaces = display
            .create_surfaces(
                format,
                None,
                width,
                height,
                Some(UsageHint::USAGE_HINT_DECODER),
                vec![()],
            )
            .unwrap();
        let context = display
            .create_context(
                &config,
                width,
                ((height + 15) / 16) * 16,
                Some(&surfaces),
                true,
            )
            .unwrap();

        // The picture data is adapted from libva-utils at decode/mpeg2vldemo.cpp
        // Data dump of a 16x16 MPEG2 video clip,it has one I frame
        let mut mpeg2_clip: Vec<u8> = vec![
            0x00, 0x00, 0x01, 0xb3, 0x01, 0x00, 0x10, 0x13, 0xff, 0xff, 0xe0, 0x18, 0x00, 0x00,
            0x01, 0xb5, 0x14, 0x8a, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x01, 0xb8, 0x00, 0x08,
            0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x0f, 0xff, 0xf8, 0x00, 0x00, 0x01, 0xb5,
            0x8f, 0xff, 0xf3, 0x41, 0x80, 0x00, 0x00, 0x01, 0x01, 0x13, 0xe1, 0x00, 0x15, 0x81,
            0x54, 0xe0, 0x2a, 0x05, 0x43, 0x00, 0x2d, 0x60, 0x18, 0x01, 0x4e, 0x82, 0xb9, 0x58,
            0xb1, 0x83, 0x49, 0xa4, 0xa0, 0x2e, 0x05, 0x80, 0x4b, 0x7a, 0x00, 0x01, 0x38, 0x20,
            0x80, 0xe8, 0x05, 0xff, 0x60, 0x18, 0xe0, 0x1d, 0x80, 0x98, 0x01, 0xf8, 0x06, 0x00,
            0x54, 0x02, 0xc0, 0x18, 0x14, 0x03, 0xb2, 0x92, 0x80, 0xc0, 0x18, 0x94, 0x42, 0x2c,
            0xb2, 0x11, 0x64, 0xa0, 0x12, 0x5e, 0x78, 0x03, 0x3c, 0x01, 0x80, 0x0e, 0x80, 0x18,
            0x80, 0x6b, 0xca, 0x4e, 0x01, 0x0f, 0xe4, 0x32, 0xc9, 0xbf, 0x01, 0x42, 0x69, 0x43,
            0x50, 0x4b, 0x01, 0xc9, 0x45, 0x80, 0x50, 0x01, 0x38, 0x65, 0xe8, 0x01, 0x03, 0xf3,
            0xc0, 0x76, 0x00, 0xe0, 0x03, 0x20, 0x28, 0x18, 0x01, 0xa9, 0x34, 0x04, 0xc5, 0xe0,
            0x0b, 0x0b, 0x04, 0x20, 0x06, 0xc0, 0x89, 0xff, 0x60, 0x12, 0x12, 0x8a, 0x2c, 0x34,
            0x11, 0xff, 0xf6, 0xe2, 0x40, 0xc0, 0x30, 0x1b, 0x7a, 0x01, 0xa9, 0x0d, 0x00, 0xac,
            0x64,
        ];

        let picture_coding_extension =
            MPEG2PictureCodingExtension::new(0, 3, 0, 1, 0, 0, 0, 0, 0, 1, 1);
        let pic_param = PictureParameterBufferMPEG2::new(
            16,
            16,
            0xffffffff,
            0xffffffff,
            1,
            0xffff,
            &picture_coding_extension,
        );

        let pic_param = BufferType::PictureParameter(PictureParameter::MPEG2(pic_param));

        let iq_matrix = IQMatrixBufferMPEG2::new(
            1,
            1,
            0,
            0,
            [
                8, 16, 16, 19, 16, 19, 22, 22, 22, 22, 22, 22, 26, 24, 26, 27, 27, 27, 26, 26, 26,
                26, 27, 27, 27, 29, 29, 29, 34, 34, 34, 29, 29, 29, 27, 27, 29, 29, 32, 32, 34, 34,
                37, 38, 37, 35, 35, 34, 35, 38, 38, 40, 40, 40, 48, 48, 46, 46, 56, 56, 58, 69, 69,
                83,
            ],
            [
                16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            [0; 64],
            [0; 64],
        );

        let iq_matrix = BufferType::IQMatrix(IQMatrix::MPEG2(iq_matrix));

        let slice_param = SliceParameterBufferMPEG2::new(150, 0, 0, 38, 0, 0, 2, 0);

        let slice_param = BufferType::SliceParameter(SliceParameter::MPEG2(slice_param));

        let test_data_offset = 47;
        let slice_data = BufferType::SliceData(mpeg2_clip.drain(test_data_offset..).collect());

        let buffers = vec![
            context.create_buffer(pic_param).unwrap(),
            context.create_buffer(slice_param).unwrap(),
            context.create_buffer(iq_matrix).unwrap(),
            context.create_buffer(slice_data).unwrap(),
        ];

        let mut picture = Picture::new(0, Rc::clone(&context), surfaces.remove(0));
        for buffer in buffers {
            picture.add_buffer(buffer);
        }

        // Actual client code can just chain the calls.
        let picture = picture.begin().unwrap();
        let picture = picture.render().unwrap();
        let picture = picture.end().unwrap();
        let picture = picture.sync().map_err(|(e, _)| e).unwrap();

        // Test whether we can map the resulting surface to obtain the raw yuv
        // data
        let image_fmts = display.query_image_formats().unwrap();
        let image_fmt = image_fmts
            .into_iter()
            .find(|f| f.fourcc == bindings::constants::VA_FOURCC_NV12)
            .expect("No valid VAImageFormat found for NV12");

        let resolution = (width, height);
        let image = picture
            .create_image(image_fmt, resolution, resolution)
            .unwrap();

        assert_eq!(crc_nv12_image(&image), 0xa5713e52);
    }

    #[test]
    // Ignore this test by default as it requires libva-compatible hardware.
    #[ignore]
    fn enc_h264_demo() {
        // Based on `gst-launch-1.0 videotestsrc num-buffers=1 ! video/x-raw,width=64,height=64,format=NV12 ! vaapih264enc ! filesink location=frame.h264`
        // Frame created using `gst-launch-1.0 videotestsrc num-buffers=1 ! video/x-raw,width=64,height=64,format=NV12 ! filesink location=src/test_frame.nv12`
        let raw_frame_nv12 = include_bytes!("test_frame.nv12");

        let display = Display::open().unwrap();

        let format = bindings::constants::VA_RT_FORMAT_YUV420;
        let entrypoint = bindings::VAEntrypoint::VAEntrypointEncSliceLP;
        let profile = bindings::VAProfile::VAProfileH264ConstrainedBaseline;
        let width = 64u32;
        let height = 64u32;

        let mut attrs = vec![bindings::VAConfigAttrib {
            type_: bindings::VAConfigAttribType::VAConfigAttribRTFormat,
            value: 0,
        }];

        display
            .get_config_attributes(profile, entrypoint, &mut attrs)
            .unwrap();

        let config = display.create_config(attrs, profile, entrypoint).unwrap();

        let mut surfaces = display
            .create_surfaces(
                format,
                None,
                width,
                height,
                Some(UsageHint::USAGE_HINT_ENCODER),
                vec![()],
            )
            .unwrap();

        let context = display
            .create_context(&config, width, height, Some(&surfaces), true)
            .unwrap();

        let seq_fields = H264EncSeqFields::new(
            1, // 4:2:0
            1, // Only frames
            0, 0, 0, 1, 0, 2, 0,
        );

        let image_fmts = display.query_image_formats().unwrap();
        let image_fmt = image_fmts
            .into_iter()
            .find(|f| f.fourcc == bindings::constants::VA_FOURCC_NV12)
            .expect("No valid VAImageFormat found for NV12");

        let surface = surfaces.pop().unwrap();
        let surface_id = surface.id();

        let coded_buffer = context.create_enc_coded(raw_frame_nv12.len()).unwrap();

        let mut image =
            Image::create_from(&surface, image_fmt, (width, height), (width, height)).unwrap();

        let va_image = *image.image();
        let dest = image.as_mut();
        let data = &raw_frame_nv12[..];
        let width = width as usize;
        let height = height as usize;

        let mut src = data;
        let mut dst = &mut dest[va_image.offsets[0] as usize..];

        // Copy luma
        for _ in 0..height {
            dst[..width].copy_from_slice(&src[..width]);
            dst = &mut dst[va_image.pitches[0] as usize..];
            src = &src[width..];
        }

        // Advance to the offset of the chroma plane
        let mut src = &data[width * height..];
        let mut dst = &mut dest[va_image.offsets[1] as usize..];

        let height = height / 2;

        // Copy chroma
        for _ in 0..height {
            dst[..width].copy_from_slice(&src[..width]);
            dst = &mut dst[va_image.pitches[1] as usize..];
            src = &src[width..];
        }
        drop(image);

        let sps = BufferType::EncSequenceParameter(EncSequenceParameter::H264(
            EncSequenceParameterBufferH264::new(
                0,
                10,
                10,
                30,
                1,
                0,
                1,
                (width / 16) as u16,  // width / 16
                (height / 16) as u16, // height / 16
                &seq_fields,
                0,
                0,
                0,
                0,
                0,
                [0; 256],
                None,
                Some(H264VuiFields::new(1, 1, 0, 0, 0, 1, 0, 0)),
                255,
                1,
                1,
                1,
                60,
            ),
        ));

        let sps = context.create_buffer(sps).unwrap();

        let ref_frames: [PictureH264; 16] = (0..16)
            .map(|_| {
                PictureH264::new(
                    constants::VA_INVALID_ID,
                    0,
                    constants::VA_INVALID_SURFACE,
                    0,
                    0,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| {
                panic!();
            });

        let pps = BufferType::EncPictureParameter(EncPictureParameter::H264(
            EncPictureParameterBufferH264::new(
                PictureH264::new(surface_id, 0, 0, 0, 0),
                ref_frames,
                coded_buffer.id(),
                0,
                0,
                0,
                0,
                26,
                0,
                0,
                0,
                0,
                &H264EncPicFields::new(1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0),
            ),
        ));

        let pps = context.create_buffer(pps).unwrap();

        let ref_pic_list_0: [PictureH264; 32] = (0..32)
            .map(|_| {
                PictureH264::new(
                    constants::VA_INVALID_ID,
                    0,
                    constants::VA_INVALID_SURFACE,
                    0,
                    0,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| {
                panic!();
            });

        let ref_pic_list_1: [PictureH264; 32] = (0..32)
            .map(|_| {
                PictureH264::new(
                    constants::VA_INVALID_ID,
                    0,
                    constants::VA_INVALID_SURFACE,
                    0,
                    0,
                )
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap_or_else(|_| {
                panic!();
            });

        let slice = BufferType::EncSliceParameter(EncSliceParameter::H264(
            EncSliceParameterBufferH264::new(
                0,
                ((width / 16) * (height / 16)) as u32,
                constants::VA_INVALID_ID,
                2, // I
                0,
                1,
                0,
                0,
                [0, 0],
                1,
                0,
                0,
                0,
                ref_pic_list_0,
                ref_pic_list_1,
                0,
                0,
                0,
                [0; 32],
                [0; 32],
                0,
                [[0; 2]; 32],
                [[0; 2]; 32],
                0,
                [0; 32],
                [0; 32],
                0,
                [[0; 2]; 32],
                [[0; 2]; 32],
                0,
                0,
                0,
                2,
                2,
            ),
        ));

        let slice = context.create_buffer(slice).unwrap();

        let mut picture = Picture::new(0, Rc::clone(&context), surface);
        picture.add_buffer(pps);
        picture.add_buffer(sps);
        picture.add_buffer(slice);

        let picture = picture.begin().unwrap();
        let picture = picture.render().unwrap();
        let picture = picture.end().unwrap();
        let _ = picture.sync().map_err(|(e, _)| e).unwrap();

        let coded_buf = MappedCodedBuffer::new(&coded_buffer).unwrap();
        assert_ne!(coded_buf.segments().len(), 0);

        for segment in coded_buf.iter() {
            assert_ne!(segment.buf.len(), 0);
        }

        const WRITE_TO_FILE: bool = false;
        if WRITE_TO_FILE {
            let raw_sps_bitstream: Vec<u8> = vec![
                0x00, 0x00, 0x00, 0x01, 0x67, 0x42, 0xc0, 0x0a, 0xab, 0x42, 0x12, 0x7f, 0xe0, 0x00,
                0x20, 0x00, 0x22, 0x00, 0x00, 0x03, 0x00, 0x02, 0x00, 0x00, 0x03, 0x00, 0x79, 0x28,
            ];

            let raw_pps_bitstream: Vec<u8> = vec![0x00, 0x00, 0x00, 0x01, 0x68, 0xce, 0x3c, 0x80];

            let mut raw_file = std::fs::File::create("libva_utils_enc_h264_demo.h264").unwrap();
            raw_file.write_all(&raw_sps_bitstream).unwrap();
            raw_file.write_all(&raw_pps_bitstream).unwrap();

            for segment in coded_buf.segments() {
                raw_file.write_all(segment.buf).unwrap();
            }

            raw_file.flush().unwrap();
        }
    }
}
