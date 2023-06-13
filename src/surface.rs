// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::rc::Rc;

use crate::bindings;
use crate::display::Display;
use crate::va_check;
use crate::UsageHint;
use crate::VaError;

/// Trait describing a memory origin for surfaces.
pub trait SurfaceMemoryDescriptor {
    /// Add the required attributes to `attr` in order to attach the memory of this descriptor to
    /// the surface when it is created.
    fn add_attrs(&self, attrs: &mut Vec<bindings::VASurfaceAttrib>);
}

/// Used when we want the VA driver to allocate surface memory for us. In this case we don't need
/// to add any specific attribute for surface creation.
impl SurfaceMemoryDescriptor for () {
    fn add_attrs(&self, _attrs: &mut Vec<bindings::VASurfaceAttrib>) {}
}

/// An owned VA surface that is tied to a particular `Display`.
pub struct Surface {
    display: Rc<Display>,
    id: bindings::VASurfaceID,
    width: u32,
    height: u32,
}

impl Surface {
    /// Create `Surfaces` by wrapping around a `vaCreateSurfaces` call. This is just a helper for
    /// [`Display::create_surfaces`].
    pub(crate) fn new<D: SurfaceMemoryDescriptor>(
        display: Rc<Display>,
        rt_format: u32,
        va_fourcc: Option<u32>,
        width: u32,
        height: u32,
        usage_hint: Option<UsageHint>,
        descriptors: &[D],
    ) -> Result<Vec<Self>, VaError> {
        let mut attrs = vec![];

        if let Some(usage_hint) = usage_hint {
            let attr = bindings::VASurfaceAttrib {
                type_: bindings::VASurfaceAttribType::VASurfaceAttribUsageHint,
                flags: bindings::constants::VA_SURFACE_ATTRIB_SETTABLE,
                value: bindings::VAGenericValue {
                    type_: bindings::VAGenericValueType::VAGenericValueTypeInteger,
                    value: bindings::_VAGenericValue__bindgen_ty_1 {
                        i: usage_hint.bits() as i32,
                    },
                },
            };

            attrs.push(attr);
        }

        if let Some(fourcc) = va_fourcc {
            let attr = bindings::VASurfaceAttrib {
                type_: bindings::VASurfaceAttribType::VASurfaceAttribPixelFormat,
                flags: bindings::constants::VA_DISPLAY_ATTRIB_SETTABLE,
                value: bindings::VAGenericValue {
                    type_: bindings::VAGenericValueType::VAGenericValueTypeInteger,
                    value: bindings::_VAGenericValue__bindgen_ty_1 { i: fourcc as i32 },
                },
            };

            attrs.push(attr);
        }

        for desc in descriptors.iter() {
            desc.add_attrs(&mut attrs);
        }

        let mut surface_ids = Vec::with_capacity(descriptors.len());

        // Safe because `self` represents a valid VADisplay. The `surface` and `attrs` vectors are
        // properly initialized and valid sizes are passed to the C function, so it is impossible to
        // write past the end of their storage by mistake.
        va_check(unsafe {
            bindings::vaCreateSurfaces(
                display.handle(),
                rt_format,
                width,
                height,
                surface_ids.as_mut_ptr(),
                descriptors.len() as u32,
                attrs.as_mut_ptr(),
                attrs.len() as u32,
            )
        })?;

        // Safe because the C function will have written to exactly `num_surfaces` entries, which is
        // known to be within the vector's capacity.
        unsafe { surface_ids.set_len(descriptors.len()) };

        let va_surfaces = surface_ids
            .into_iter()
            .map(|id| Self {
                display: Rc::clone(&display),
                id,
                width,
                height,
            })
            .collect();

        Ok(va_surfaces)
    }

    /// Wrapper around `vaSyncSurface` that blocks until all pending operations on the render
    /// target have been completed.
    ///
    /// Upon return it
    /// is safe to use the render target for a different picture.
    pub fn sync(&self) -> Result<(), VaError> {
        // Safe because `self` represents a valid VASurface.
        va_check(unsafe { bindings::vaSyncSurface(self.display.handle(), self.id) })
    }

    /// Convenience function to return a VASurfaceID vector. Useful to interface with the C API
    /// where a surface array might be needed.
    pub fn as_id_vec(surfaces: &[Self]) -> Vec<bindings::VASurfaceID> {
        surfaces.iter().map(|surface| surface.id).collect()
    }

    /// Wrapper over `vaQuerySurfaceStatus` to find out any pending ops on the render target.
    pub fn query_status(&self) -> Result<bindings::VASurfaceStatus::Type, VaError> {
        let mut status: bindings::VASurfaceStatus::Type = 0;
        // Safe because `self` represents a valid VASurface.
        va_check(unsafe {
            bindings::vaQuerySurfaceStatus(self.display.handle(), self.id, &mut status)
        })?;

        Ok(status)
    }

    /// Returns the ID of this surface.
    pub fn id(&self) -> bindings::VASurfaceID {
        self.id
    }

    /// Returns the dimensions of this surface.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        // Safe because `self` represents a valid VASurface.
        unsafe { bindings::vaDestroySurfaces(self.display.handle(), &mut self.id, 1) };
    }
}
