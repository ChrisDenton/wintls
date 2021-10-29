//! Low level primitives for building static thread-local abstractions.
//!
//! # Example
//!
//! ```
//! #![feature(asm)]
//! wintls::raw::init_static!(
//!     static DATA: u32 = 0xfeedface;
//! );
//! unsafe {
//!     let key: u32 = wintls::raw::static_key!(DATA);
//!     let value: u32 = wintls::raw::get_static(key);
//!     wintls::raw::set_static(key, value + 1);
//! }
//! ```

#[doc(inline)]
pub use super::raw_internal::*;
