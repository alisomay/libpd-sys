# libpd-sys

[![Build Status](https://github.com/alisomay/libpd-sys/workflows/Build/badge.svg)](https://github.com/alisomay/libpd-sys/actions?query=workflow%3ABuild)
[![License: BSD-3-Clause](https://img.shields.io/badge/License-BSD_3--Clause-green.svg)](https://opensource.org/licenses/BSD-3-Clause)

Rust bindings for [libpd](https://github.com/libpd/libpd).

This crate is not meant to be used directly, but rather as a dependency of [libpd-rs](https://github.com/alisomay/libpd-rs).

## List of bundled externals

This will be a growing list of bundled externals.

The way to add externals to [libpd](https://github.com/libpd/libpd) is to compile and statically link them.

In the future, some external packs will be made features.

- `moog~`
- `freeverb~`

## Contribute

There is always room for more testing and improvement on `build.rs`. If you're interested PRs are open.

Or if you wish you can add support for the unsupported platforms.

## Support

- Desktop
  - macOS:
    - `x86_64` ✅
    - `aarch64` ✅
  - linux:
    - `x86_64` ✅
    - `aarch64` ✅
  - windows:
    - msvc
      - `x86_64` ✅
      - `aarch64` ✅
    - gnu
      - `x86_64` (not tested but should work)
      - `aarch64` (not tested but should work)
- Mobile

  - iOS (not yet but will be addressed)
  - Android (not yet but will be addressed)

- Web (not yet but will be addressed)

## Change log

- 0.2.0
  - Add preparation and support for bundling externals
  - Change `libpd` submodule to track <https://github.com/alisomay/libpd> `libpdrs` branch.
