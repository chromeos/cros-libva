// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use crate::bindings;
use crate::picture::Picture;
use crate::picture::PictureSync;
use crate::va_check;
use crate::VaError;

/// Wrapper around `VAImage` that is tied to the lifetime of a given `Picture`.
///
/// An image is used to either get the surface data to client memory, or to copy image data in
/// client memory to a surface.
pub struct Image<'a> {
    /// The picture whose `Surface` we use as the source of pixel data.
    picture: &'a Picture<PictureSync>,
    /// The `VAImage` returned by libva.
    image: bindings::VAImage,
    /// The mapped surface data.
    data: &'a [u8],
    /// Whether the image was derived using the `vaDeriveImage` API or created using the
    /// `vaCreateImage` API.
    derived: bool,
    /// The display resolution requested by the client. The implementation is
    /// free to enlarge this value as needed. In any case, we guarantee that an
    /// image at least as large is returned.
    display_resolution: (u32, u32),
}

impl<'a> Image<'a> {
    /// Helper method to map a `VAImage` using `vaMapBuffer` and return an `Image`.
    ///
    /// Returns an error if the mapping failed.
    pub(crate) fn new(
        picture: &'a Picture<PictureSync>,
        image: bindings::VAImage,
        derived: bool,
        display_resolution: (u32, u32),
    ) -> Result<Self, VaError> {
        let mut addr = std::ptr::null_mut();

        // Safe since `picture.inner.context` represents a valid `VAContext` and `image` has been
        // successfully created at this point.
        match va_check(unsafe {
            bindings::vaMapBuffer(picture.display().handle(), image.buf, &mut addr)
        }) {
            Ok(_) => {
                // Assert that libva provided us with a coded resolution that is
                // at least as large as `display_resolution`.
                assert!(u32::from(image.width) >= display_resolution.0);
                assert!(u32::from(image.height) >= display_resolution.1);

                // Safe since `addr` points to data mapped onto our address space since we called
                // `vaMapBuffer` above, which also guarantees that the data is valid for
                // `image.data_size`.
                let data =
                    unsafe { std::slice::from_raw_parts_mut(addr as _, image.data_size as usize) };
                Ok(Image {
                    picture,
                    image,
                    data,
                    derived,
                    display_resolution,
                })
            }
            Err(e) => {
                // Safe because `picture.inner.context` represents a valid `VAContext` and `image`
                // represents a valid `VAImage`.
                unsafe {
                    bindings::vaDestroyImage(picture.display().handle(), image.image_id);
                }

                Err(e)
            }
        }
    }

    /// Get a reference to the underlying `VAImage` that describes this image.
    pub fn image(&self) -> &bindings::VAImage {
        &self.image
    }

    /// Returns whether this image is directly derived from its underlying `Picture`, as opposed to
    /// being a view/copy of said `Picture` in a guaranteed pixel format.
    pub fn is_derived(&self) -> bool {
        self.derived
    }

    /// Returns the display resolution as passed in by the client. This is a
    /// value that is less than or equal to the coded resolution.
    pub fn display_resolution(&self) -> (u32, u32) {
        self.display_resolution
    }

    /// Returns the coded resolution. This value can be larger than the value
    /// passed in when the image was created if the driver needs to.
    pub fn coded_resolution(&self) -> (u32, u32) {
        (self.image.width.into(), self.image.height.into())
    }
}

impl<'a> AsRef<[u8]> for Image<'a> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'a> Drop for Image<'a> {
    fn drop(&mut self) {
        unsafe {
            // Safe since the buffer is mapped in `Image::new`, so `self.image.buf` points to a
            // valid `VABufferID`.
            bindings::vaUnmapBuffer(self.picture.display().handle(), self.image.buf);
            // Safe since `self.image` represents a valid `VAImage`.
            bindings::vaDestroyImage(self.picture.display().handle(), self.image.image_id);
        }
    }
}
