#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

//! Rust bindings for [libpd](https://github.com/libpd/libpd).
//!
//! This crate is not meant to be used directly, but rather as a dependency of [libpd-rs](https://github.com/alisomay/libpd-rs).
//!
//! ## Support
//!
//! - Desktop
//!   - macOS:
//!     - `x86_64` ✅
//!     - `aarch64` ✅
//!   - linux:
//!     - `x86_64` ✅
//!     - `aarch64` ✅
//!   - windows:
//!     - msvc
//!       - `x86_64` ✅
//!       - `aarch64` (not tested but should work)
//!     - gnu
//!       - `x86_64` (not tested but should work)
//!       - `aarch64` (not tested but should work)
//! - Mobile
//!   - iOS (not yet but will be addressed)
//!   - Android (not yet but will be addressed)
//!
//! - Web (not yet but will be addressed)
//!
//!
//! ## Contribute
//!
//! There is always room for more testing and improvement on `build.rs`. If you're interested PRs are open.
//!
//! Or if you wish you can add support for the unsupported platforms.

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
