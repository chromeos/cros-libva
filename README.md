# Libva Rust wrapper

This crate aims to provide safe libva abstractions for use within Rust code using its own
bindgen-generated bindings. While developed for use in ChromeOS, there are no ChromeOS specifics or
dependencies so this crate should be usable anywhere.

The first version was written by Daniel Almeida and hosted in the [crosvm
repository](https://chromium.googlesource.com/crosvm/crosvm/) before being split out.

Note: This create requires the native [libva](https://github.com/intel/libva) library at link time.
It also requires a VA-API driver to be installed on the system. The VA-API driver to use depends on
the underlying hardware, e.g.: the implementation for Intel hardware is in
[intel-media-driver](https://github.com/intel/media-driver), whereas AMD hardware will depend on
[Mesa](https://gitlab.freedesktop.org/mesa/mesa).

An easy way to see whether everything is in order is to run the `vainfo` utility. This is usually
packaged with `libva-utils` or as a standalone package in some distributions. `vainfo` will print
the VA-API version, the driver string, and a list of supported profiles and endpoints, i.e.:

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

For decoding, the desired profile must be supported under `VAEntrypointVLD`. For example, in order
to decode VP8 media, this line must be present in the output of `vainfo`:

```
      VAProfileVP8Version0_3          : VAEntrypointVLD
```

Whereas to decode H264 Main profile media, this line must be present:

```
      VAProfileH264Main               : VAEntrypointVLD
```

For more information on VA-API and its usage within ChromeOS, see
[this guide](https://chromium.googlesource.com/chromium/src/+/master/docs/gpu/vaapi.md).

For a brief introduction on how to use this crate, see the `libva_utils_mpeg2vldemo` test under
src/lib.rs. You can also quickly test MPEG2 decoding by running it:

```
cargo test -- --ignored libva_utils_mpeg2vldemo
```
