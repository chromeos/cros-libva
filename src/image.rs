// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::borrow::Borrow;
use std::rc::Rc;

use crate::bindings;
use crate::picture::Picture;
use crate::va_check;
use crate::Display;
use crate::PictureReclaimableSurface;
use crate::Surface;
use crate::SurfaceMemoryDescriptor;
use crate::VaError;

/// Wrapper around `VAImage` that is tied to the lifetime of a given `Picture`.
///
/// An image is used to either get the surface data to client memory, or to copy image data in
/// client memory to a surface.
pub struct Image<'a, D: SurfaceMemoryDescriptor> {
    /// The display from which the image was created, so we can unmap it upon destruction.
    display: Rc<Display>,
    /// The `VAImage` returned by libva.
    image: bindings::VAImage,
    /// The mapped surface data.
    data: &'a mut [u8],
    /// Whether the image was derived using the `vaDeriveImage` API or created using the
    /// `vaCreateImage` API.
    derived: bool,
    /// The display resolution requested by the client. The implementation is
    /// free to enlarge this value as needed. In any case, we guarantee that an
    /// image at least as large is returned.
    display_resolution: (u32, u32),
    /// Tracks whether the underlying data has possibly been written to, i.e. an encoder will create
    /// an image and map its buffer in order to write to it, so we must writeback later.
    dirty: bool,
    /// We need the surface to writeback the image. Also, from a logical POV,
    /// the surface, which is the backing of the data, should not drop while we
    /// live.
    surface: &'a Surface<D>,
}

impl<'a, D: SurfaceMemoryDescriptor> Image<'a, D> {
    /// Helper method to map a `VAImage` using `vaMapBuffer` and return an `Image`.
    ///
    /// Returns an error if the mapping failed.
    pub(crate) fn new<T: Borrow<Surface<D>>, S: PictureReclaimableSurface>(
        picture: &'a Picture<S, T>,
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
                    display: Rc::clone(picture.display()),
                    image,
                    data,
                    derived,
                    display_resolution,
                    dirty: false,
                    surface: picture.surface(),
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

impl<'a, D: SurfaceMemoryDescriptor> AsRef<[u8]> for Image<'a, D> {
    fn as_ref(&self) -> &[u8] {
        self.data
    }
}

impl<'a, D: SurfaceMemoryDescriptor> AsMut<[u8]> for Image<'a, D> {
    fn as_mut(&mut self) -> &mut [u8] {
        self.dirty = true;
        self.data
    }
}

impl<'a, D: SurfaceMemoryDescriptor> Drop for Image<'a, D> {
    fn drop(&mut self) {
        if !self.derived && self.dirty {
            // Safe because `picture.inner.context` represents a valid `VAContext`,
            // `picture.surface` represents a valid `VASurface` and `image` represents a valid
            // `VAImage`.
            unsafe {
                bindings::vaPutImage(
                    self.display.handle(),
                    self.surface.id(),
                    self.image.image_id,
                    0,
                    0,
                    self.image.width as u32,
                    self.image.height as u32,
                    0,
                    0,
                    self.image.width as u32,
                    self.image.height as u32,
                );
            }
        }

        unsafe {
            // Safe since the buffer is mapped in `Image::new`, so `self.image.buf` points to a
            // valid `VABufferID`.
            bindings::vaUnmapBuffer(self.display.handle(), self.image.buf);
            // Safe since `self.image` represents a valid `VAImage`.
            bindings::vaDestroyImage(self.display.handle(), self.image.image_id);
        }
    }
}
