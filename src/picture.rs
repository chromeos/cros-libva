// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::cell::Ref;
use std::cell::RefCell;
use std::cell::RefMut;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::bindings;
use crate::buffer::Buffer;
use crate::context::Context;
use crate::display::Display;
use crate::status::VaStatus;
use crate::surface::Surface;
use crate::Image;
use crate::VaError;

// Use the sealed trait pattern to make sure that new states are not created in caller code. More
// information about the sealed trait pattern can be found at
// <https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed>
mod private {
    pub trait Sealed {}
}

/// A `Picture` will only have valid YUV data after a sequence of operations are performed in a
/// particular order. This order correspond to the following VA-API calls: `vaBeginPicture`,
/// `vaRenderPicture`, `vaEndPicture` and `vaSyncSurface`. This trait enforces this ordering by
/// implementing the Typestate pattern to constrain what operations are available in what particular
/// states.
///
/// The states for the state machine are:
///
/// * PictureNew -> PictureBegin
/// * PictureBegin -> PictureRender
/// * PictureRender ->PictureEnd
/// * PictureEnd -> PictureSync
///
/// Where the surface can be reclaimed in both `PictureNew` and `PictureSync`, as either no
/// operation took place (as in `PictureNew`), or it is guaranteed that the operation has already
/// completed (as in `PictureSync`).
///
/// More information about the Typestate pattern can be found at
/// <http://cliffle.com/blog/rust-typestate/>
pub trait PictureState: private::Sealed {}

/// Represents a `Picture` that has just been created.
pub enum PictureNew {}
impl PictureState for PictureNew {}
impl private::Sealed for PictureNew {}

/// Represents a `Picture` after `vaBeginPicture` has been called.
pub enum PictureBegin {}
impl PictureState for PictureBegin {}
impl private::Sealed for PictureBegin {}

/// Represents a `Picture` after `vaRenderPicture` has been called.
pub enum PictureRender {}
impl PictureState for PictureRender {}
impl private::Sealed for PictureRender {}

/// Represents a `Picture` after `vaEndPicture` has been called.
pub enum PictureEnd {}
impl PictureState for PictureEnd {}
impl private::Sealed for PictureEnd {}

/// Represents a `Picture` after `vaSyncSurface` has been called on the underlying surface.
pub enum PictureSync {}
impl PictureState for PictureSync {}
impl private::Sealed for PictureSync {}

/// Represents a state where one can reclaim the underlying `Surface` for this `Picture`. This is
/// true when either no decoding has been initiated or, alternatively, when the decoding operation
/// has completed for the underlying `vaSurface`
pub trait PictureReclaimableSurface: PictureState + private::Sealed {}
impl PictureReclaimableSurface for PictureNew {}
impl PictureReclaimableSurface for PictureSync {}

struct PictureInner {
    /// Timestamp of the picture.
    timestamp: u64,
    /// A context associated with this picture.
    context: Rc<Context>,
    /// Contains the buffers used to decode the data.
    buffers: Vec<Buffer>,
    /// Contains the actual decoded data. Note that the surface may be shared in
    /// interlaced decoding.
    surface: Rc<RefCell<Surface>>,
}

/// A `Surface` that is being rendered into.
///
/// This struct abstracts the decoding flow using `vaBeginPicture`, `vaRenderPicture`,
/// `vaEndPicture` and `vaSyncSurface` in a type-safe way.
///
/// The surface will have valid picture data after all the stages of decoding are called.
pub struct Picture<S: PictureState> {
    inner: Box<PictureInner>,
    phantom: std::marker::PhantomData<S>,
}

impl Picture<PictureNew> {
    /// Creates a new Picture with a given `timestamp`. `surface` is the underlying surface that
    /// libva will render to.
    pub fn new(timestamp: u64, context: Rc<Context>, surface: Surface) -> Self {
        Self {
            inner: Box::new(PictureInner {
                timestamp,
                context,
                buffers: Default::default(),
                surface: Rc::new(RefCell::new(surface)),
            }),

            phantom: PhantomData,
        }
    }

    /// Creates a new Picture with a given `frame_number` to identify it,
    /// reusing the Surface from `picture`. This is useful for interlaced
    /// decoding as one can render both fields to the same underlying surface.
    pub fn new_from_same_surface<T: PictureReclaimableSurface, S: PictureReclaimableSurface>(
        timestamp: u64,
        picture: &Picture<S>,
    ) -> Picture<T> {
        let context = Rc::clone(&picture.inner.context);
        Picture {
            inner: Box::new(PictureInner {
                timestamp,
                context,
                buffers: Default::default(),
                surface: Rc::clone(&picture.inner.surface),
            }),

            phantom: PhantomData,
        }
    }

    /// Add `buffer` to the picture.
    pub fn add_buffer(&mut self, buffer: Buffer) {
        self.inner.buffers.push(buffer);
    }

    /// Wrapper around `vaBeginPicture`.
    pub fn begin(self) -> Result<Picture<PictureBegin>, VaError> {
        // Safe because `self.inner.context` represents a valid VAContext and
        // `self.inner.surface` represents a valid VASurface.
        let res = VaStatus(unsafe {
            bindings::vaBeginPicture(
                self.inner.context.display().handle(),
                self.inner.context.id(),
                self.inner.surface.borrow().id(),
            )
        });

        res.check().map(|()| Picture {
            inner: self.inner,
            phantom: PhantomData,
        })
    }
}

impl Picture<PictureBegin> {
    /// Wrapper around `vaRenderPicture`.
    pub fn render(self) -> Result<Picture<PictureRender>, VaError> {
        // Safe because `self.inner.context` represents a valid `VAContext` and `self.inner.surface`
        // represents a valid `VASurface`. `buffers` point to a Rust struct and the vector length is
        // passed to the C function, so it is impossible to write past the end of the vector's
        // storage by mistake.
        VaStatus(unsafe {
            bindings::vaRenderPicture(
                self.inner.context.display().handle(),
                self.inner.context.id(),
                Buffer::as_id_vec(&self.inner.buffers).as_mut_ptr(),
                self.inner.buffers.len() as i32,
            )
        })
        .check()
        .map(|()| Picture {
            inner: self.inner,
            phantom: PhantomData,
        })
    }
}

impl Picture<PictureRender> {
    /// Wrapper around `vaEndPicture`.
    pub fn end(self) -> Result<Picture<PictureEnd>, VaError> {
        // Safe because `self.inner.context` represents a valid `VAContext`.
        VaStatus(unsafe {
            bindings::vaEndPicture(
                self.inner.context.display().handle(),
                self.inner.context.id(),
            )
        })
        .check()
        .map(|()| Picture {
            inner: self.inner,
            phantom: PhantomData,
        })
    }
}

impl Picture<PictureEnd> {
    /// Syncs the picture, ensuring that all pending operations are complete when this call returns.
    pub fn sync(self) -> Result<Picture<PictureSync>, (VaError, Self)> {
        let res = self.inner.surface.borrow().sync();

        match res {
            Ok(()) => Ok(Picture {
                inner: self.inner,
                phantom: PhantomData,
            }),
            Err(e) => Err((e, self)),
        }
    }

    /// Queries the status of the underlying surface.
    ///
    /// This call can be used to implement a non-blocking path, wherein a decoder queries the status
    /// of the surface after each decode operation instead of blocking on it.
    pub fn query_status(&self) -> Result<bindings::VASurfaceStatus::Type, VaError> {
        self.inner.surface.borrow_mut().query_status()
    }
}

impl Picture<PictureSync> {
    /// Create a new derived image from this `Picture`.
    ///
    /// Derived images are a direct view (i.e. without any copy) on the buffer content of the
    /// `Picture`. On the other hand, not all `Pictures` can be derived.
    pub fn derive_image(&self) -> Result<Image, VaError> {
        // An all-zero byte-pattern is a valid initial value for `VAImage`.
        let mut image: bindings::VAImage = Default::default();

        // Safe because `self` has a valid display handle and ID.
        VaStatus(unsafe {
            bindings::vaDeriveImage(self.display().handle(), self.surface().id(), &mut image)
        })
        .check()?;

        Image::new(self, image, true)
    }

    /// Create new image from the `Picture`.
    ///
    /// The image will contain a copy of the `Picture` in the desired `format`, `width` and `height`.
    pub fn create_image(
        &self,
        mut format: bindings::VAImageFormat,
        width: u32,
        height: u32,
    ) -> Result<Image, VaError> {
        let dpy = self.display().handle();
        // An all-zero byte-pattern is a valid initial value for `VAImage`.
        let mut image: bindings::VAImage = Default::default();

        // Safe because `dpy` is a valid display handle.
        VaStatus(unsafe {
            bindings::vaCreateImage(dpy, &mut format, width as i32, height as i32, &mut image)
        })
        .check()?;

        // Safe because `dpy` is a valid display handle, `picture.surface` is a valid VASurface and
        // `image` is a valid `VAImage`.
        match VaStatus(unsafe {
            bindings::vaGetImage(
                dpy,
                self.surface().id(),
                0,
                0,
                width,
                height,
                image.image_id,
            )
        })
        .check()
        {
            Ok(()) => Image::new(self, image, false),
            Err(e) => {
                // Safe because `image` is a valid `VAImage`.
                unsafe {
                    bindings::vaDestroyImage(dpy, image.image_id);
                }

                Err(e)
            }
        }
    }
}

impl<S: PictureState> Picture<S> {
    /// Returns the timestamp of this picture.
    pub fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    /// Returns the ID of the underlying surface.
    pub fn surface_id(&self) -> bindings::VASurfaceID {
        self.inner.surface.borrow().id()
    }

    /// Returns a reference to the display owning this `Picture`.
    pub(crate) fn display(&self) -> &Rc<Display> {
        self.inner.context.display()
    }

    /// Returns the size of the surface being rendered to by this `Picture`.
    pub fn surface_size(&self) -> (u32, u32) {
        self.inner.surface.borrow().size()
    }
}

impl<S: PictureReclaimableSurface> Picture<S> {
    /// Reclaim ownership of the Surface this picture has been created from, consuming the picture
    /// in the process. Useful if the Surface is part of a pool.
    ///
    /// This will fail and return the passed object if there are more than one reference to the
    /// underlying surface.
    pub fn take_surface(self) -> Result<Surface, Self> {
        let inner = self.inner;
        match Rc::try_unwrap(inner.surface) {
            Ok(surface) => Ok(surface.into_inner()),
            Err(surface) => Err(Self {
                inner: Box::new(PictureInner {
                    surface,
                    context: inner.context,
                    buffers: inner.buffers,
                    timestamp: inner.timestamp,
                }),
                phantom: PhantomData,
            }),
        }
    }

    /// Returns a reference to the underlying `Surface` for this `Picture`
    pub fn surface(&self) -> Ref<Surface> {
        self.inner.surface.borrow()
    }

    /// Returns a mutable reference to the underlying `Surface` for this `Picture`
    pub fn surface_mut(&mut self) -> RefMut<Surface> {
        self.inner.surface.borrow_mut()
    }
}
