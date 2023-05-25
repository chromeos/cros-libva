// Copyright 2022 The ChromiumOS Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

use std::rc::Rc;

use log::error;

use crate::bindings;
use crate::buffer::Buffer;
use crate::buffer::BufferType;
use crate::display::Display;
use crate::status::VaStatus;
use crate::Config;
use crate::Surface;
use crate::VaError;

/// A VA context for a particular [`Display`].
pub struct Context {
    display: Rc<Display>,
    id: bindings::VAContextID,
}

impl Context {
    /// Creates a Context by wrapping around a `vaCreateContext` call. This is just a helper for
    /// [`Display::create_context`].
    pub(crate) fn new(
        display: Rc<Display>,
        config: &Config,
        coded_width: u32,
        coded_height: u32,
        surfaces: Option<&Vec<Surface>>,
        progressive: bool,
    ) -> Result<Rc<Self>, VaError> {
        let mut context_id = 0;
        let flags = if progressive {
            bindings::constants::VA_PROGRESSIVE as i32
        } else {
            0
        };

        let mut render_targets = match surfaces {
            Some(surfaces) => Surface::as_id_vec(surfaces),
            None => Default::default(),
        };

        // Safe because `self` represents a valid VADisplay and render_targets
        // and ntargets are properly initialized. Note that render_targets==NULL
        // is valid so long as ntargets==0.
        VaStatus(unsafe {
            bindings::vaCreateContext(
                display.handle(),
                config.id(),
                coded_width as i32,
                coded_height as i32,
                flags,
                render_targets.as_mut_ptr(),
                render_targets.len() as i32,
                &mut context_id,
            )
        })
        .check()?;

        Ok(Rc::new(Self {
            display,
            id: context_id,
        }))
    }

    /// Returns a shared reference to the [`Display`] used by this context.
    pub fn display(&self) -> &Rc<Display> {
        &self.display
    }

    /// Returns the ID of this context.
    pub(crate) fn id(&self) -> bindings::VAContextID {
        self.id
    }

    /// Create a new buffer of type `type_`.
    pub fn create_buffer(self: &Rc<Self>, type_: BufferType) -> Result<Buffer, VaError> {
        Buffer::new(Rc::clone(self), type_)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Safe because `self` represents a valid VAContext.
        let status =
            VaStatus(unsafe { bindings::vaDestroyContext(self.display.handle(), self.id) }).check();
        if status.is_err() {
            error!("vaDestroyContext failed: {}", status.unwrap_err());
        }
    }
}
