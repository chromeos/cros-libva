# Libva Rust Wrapper

This crate provides lightweight and (hopefully) safe libva abstractions for use
within Rust code with minimal dependencies. It is developed for use in
ChromeOS, but has no ChromeOS specifics or dependencies and should thus be
usable anywhere.

## Dependencies

The native [libva](https://github.com/intel/libva) library is required at link
time, so make sure to have the `libva-dev` or equivalent package for your
distribution installed. The VA-API driver corresponding to your hardware is
also required: for Intel hardware it will be
[intel-media-driver](https://github.com/intel/media-driver), whereas AMD
hardware relies on [Mesa](https://gitlab.freedesktop.org/mesa/mesa).

An easy way to see whether everything is in order is to run the `vainfo`
utility packaged with `libva-utils` or as a standalone package in some
distributions. `vainfo` will print the VA-API version, driver string, and a
list of supported profiles and endpoints, i.e.:

```
vainfo: VA-API version: 1.13 (libva 2.13.0)
vainfo: Driver version: Intel iHD driver for Intel(R) Gen Graphics - 22.2.2 ()
vainfo: Supported profile and entrypoints
      VAProfileNone                   : VAEntrypointVideoProc
      VAProfileNone                   : VAEntrypointStats
      VAProfileMPEG2Simple            : VAEntrypointVLD
      VAProfileMPEG2Simple            : VAEntrypointEncSlice
      VAProfileMPEG2Main              : VAEntrypointVLD
      VAProfileMPEG2Main              : VAEntrypointEncSlice
      VAProfileH264Main               : VAEntrypointVLD
      etc
```

For decoding, the desired profile must be supported under `VAEntrypointVLD`.
For example, in order to decode VP8 media, this line must be present in the
output of `vainfo`:

```
      VAProfileVP8Version0_3          : VAEntrypointVLD
```

Whereas to decode H264 Main profile media, this line must be present:

```
      VAProfileH264Main               : VAEntrypointVLD
```

For more information on VA-API and its usage within ChromeOS, see [this
guide](https://chromium.googlesource.com/chromium/src/+/master/docs/gpu/vaapi.md).

## Using

The name of this crate is `cros-libva` to highlight the fact that it originates
from ChromeOS and it not an official bindings. For ease of use, it is
recommended to rename it to just `libva` in your project by using the following
line in your `Cargo.toml`:

```
libva = { package = "cros-libva", version = "0.0.1" }
```

## Testing

For a brief introduction on how to use this crate, see the
`libva_utils_mpeg2vldemo` test under `src/lib.rs`. You can also quickly test
MPEG2 decoding by running it:

```
cargo test -- --ignored libva_utils_mpeg2vldemo
```

## Credits

The first version of this crate was written by Daniel Almeida and hosted in the
[crosvm repository](https://chromium.googlesource.com/crosvm/crosvm/) before
being split.
