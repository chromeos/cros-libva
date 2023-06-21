// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::raw::c_void;
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
pub struct Surface<D: SurfaceMemoryDescriptor> {
    display: Rc<Display>,
    id: bindings::VASurfaceID,
    descriptor: D,
    width: u32,
    height: u32,
}

impl From<i32> for bindings::VAGenericValue {
    fn from(i: i32) -> Self {
        Self {
            type_: bindings::VAGenericValueType::VAGenericValueTypeInteger,
            value: bindings::_VAGenericValue__bindgen_ty_1 { i },
        }
    }
}

impl From<f32> for bindings::VAGenericValue {
    fn from(f: f32) -> Self {
        Self {
            type_: bindings::VAGenericValueType::VAGenericValueTypeFloat,
            value: bindings::_VAGenericValue__bindgen_ty_1 { f },
        }
    }
}

impl From<*mut c_void> for bindings::VAGenericValue {
    fn from(p: *mut c_void) -> Self {
        Self {
            type_: bindings::VAGenericValueType::VAGenericValueTypePointer,
            value: bindings::_VAGenericValue__bindgen_ty_1 { p },
        }
    }
}

/// Helpers to build valid `VASurfaceAttrib`s.
impl bindings::VASurfaceAttrib {
    pub fn new_pixel_format(fourcc: u32) -> Self {
        Self {
            type_: bindings::VASurfaceAttribType::VASurfaceAttribPixelFormat,
            flags: bindings::constants::VA_SURFACE_ATTRIB_SETTABLE,
            value: bindings::VAGenericValue::from(fourcc as i32),
        }
    }

    pub fn new_usage_hint(usage_hint: UsageHint) -> Self {
        Self {
            type_: bindings::VASurfaceAttribType::VASurfaceAttribUsageHint,
            flags: bindings::constants::VA_SURFACE_ATTRIB_SETTABLE,
            value: bindings::VAGenericValue::from(usage_hint.bits() as i32),
        }
    }
}

impl<D: SurfaceMemoryDescriptor> Surface<D> {
    /// Create `Surfaces` by wrapping around a `vaCreateSurfaces` call. This is just a helper for
    /// [`Display::create_surfaces`].
    pub(crate) fn new(
        display: Rc<Display>,
        rt_format: u32,
        va_fourcc: Option<u32>,
        width: u32,
        height: u32,
        usage_hint: Option<UsageHint>,
        descriptors: Vec<D>,
    ) -> Result<Vec<Self>, (VaError, Vec<D>)> {
        let mut attrs = vec![];

        if let Some(usage_hint) = usage_hint {
            attrs.push(bindings::VASurfaceAttrib::new_usage_hint(usage_hint));
        }

        if let Some(fourcc) = va_fourcc {
            attrs.push(bindings::VASurfaceAttrib::new_pixel_format(fourcc));
        }

        for desc in descriptors.iter() {
            desc.add_attrs(&mut attrs);
        }

        let mut surface_ids = Vec::with_capacity(descriptors.len());

        // Safe because `self` represents a valid VADisplay. The `surface` and `attrs` vectors are
        // properly initialized and valid sizes are passed to the C function, so it is impossible to
        // write past the end of their storage by mistake.
        match va_check(unsafe {
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
        }) {
            Ok(()) => {
                // Safe because the C function will have written to exactly `num_surfaces` entries, which is
                // known to be within the vector's capacity.
                unsafe { surface_ids.set_len(descriptors.len()) };

                let va_surfaces = surface_ids
                    .into_iter()
                    .zip(descriptors)
                    .map(|(id, descriptor)| Self {
                        display: Rc::clone(&display),
                        id,
                        descriptor,
                        width,
                        height,
                    })
                    .collect();

                Ok(va_surfaces)
            }
            Err(e) => Err((e, descriptors)),
        }
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

    /// Returns a PRIME descriptor for this surface.
    pub fn export_prime(&self) -> Result<DrmPrimeSurfaceDescriptor, VaError> {
        let mut desc: bindings::VADRMPRIMESurfaceDescriptor = Default::default();

        va_check(unsafe {
            bindings::vaExportSurfaceHandle(
                self.display.handle(),
                self.id(),
                bindings::constants::VA_SURFACE_ATTRIB_MEM_TYPE_DRM_PRIME_2,
                bindings::constants::VA_EXPORT_SURFACE_READ_ONLY
                    | bindings::constants::VA_EXPORT_SURFACE_COMPOSED_LAYERS,
                &mut desc as *mut _ as *mut c_void,
            )
        })?;

        // We do not use a `From<VADRMPRIMESurfaceDescriptor>` implementation as this would allow
        // to create "safe" descriptors outside of this method and thus from made up values,
        // violating the safety guarantee that our FDs are legit.

        let objects = (0..desc.num_objects as usize)
            // Make sure we don't go out of bounds.
            .take(4)
            .map(|i| desc.objects[i])
            .map(|o| {
                DrmPrimeSurfaceDescriptorObject {
                    // Safe because `o.fd` is a valid file descriptor returned by
                    // `vaExportSurfaceHandle`.
                    fd: unsafe { OwnedFd::from_raw_fd(o.fd) },
                    size: o.size,
                    drm_format_modifier: o.drm_format_modifier,
                }
            })
            .collect();

        let layers = (0..desc.num_layers as usize)
            // Make sure we don't go out of bounds.
            .take(4)
            .map(|i| desc.layers[i])
            .map(|l| DrmPrimeSurfaceDescriptorLayer {
                drm_format: l.drm_format,
                num_planes: l.num_planes,
                object_index: [
                    l.object_index[0] as u8,
                    l.object_index[1] as u8,
                    l.object_index[2] as u8,
                    l.object_index[3] as u8,
                ],
                offset: l.offset,
                pitch: l.pitch,
            })
            .collect();

        Ok(DrmPrimeSurfaceDescriptor {
            fourcc: desc.fourcc,
            width: desc.width,
            height: desc.height,
            objects,
            layers,
        })
    }
}

impl<D: SurfaceMemoryDescriptor> AsRef<D> for Surface<D> {
    fn as_ref(&self) -> &D {
        &self.descriptor
    }
}

impl<D: SurfaceMemoryDescriptor> AsMut<D> for Surface<D> {
    fn as_mut(&mut self) -> &mut D {
        &mut self.descriptor
    }
}

impl<D: SurfaceMemoryDescriptor> Drop for Surface<D> {
    fn drop(&mut self) {
        // Safe because `self` represents a valid VASurface.
        unsafe { bindings::vaDestroySurfaces(self.display.handle(), &mut self.id, 1) };
    }
}

/// Safe wrapper for the `object` member of `VADRMPRIMESurfaceDescriptor`.
pub struct DrmPrimeSurfaceDescriptorObject {
    pub fd: OwnedFd,
    pub size: u32,
    pub drm_format_modifier: u64,
}

/// Safe wrapper for the `layers` member of `VADRMPRIMESurfaceDescriptor`.
pub struct DrmPrimeSurfaceDescriptorLayer {
    pub drm_format: u32,
    pub num_planes: u32,
    pub object_index: [u8; 4],
    pub offset: [u32; 4],
    pub pitch: [u32; 4],
}

/// Safe wrapper around `VADRMPRIMESurfaceDescriptor`.
pub struct DrmPrimeSurfaceDescriptor {
    pub fourcc: u32,
    pub width: u32,
    pub height: u32,
    pub objects: Vec<DrmPrimeSurfaceDescriptorObject>,
    pub layers: Vec<DrmPrimeSurfaceDescriptorLayer>,
}
