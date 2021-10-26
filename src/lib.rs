//! Creates static thread-locals for Windows binaries.
//! 
//! Note this crate is more a sketch than a finished design.
//! 
//! # Use
//! 
//! This requires a nightly compiler. You will also need to add `#![feature(asm)]`
//! to your `main.rs` or `lib.rs` file.
//! 
//! # Example
//! 
//! ```
//! #![feature(asm)]
//! 
//! wintls::static_thread_local!{
//!     static TEST: u32 = 0xfeedface;
//! }
//! 
//! fn main() {
//!     println!("{:x}", TEST.get()); // feedface
//!
//!     TEST.set(5);
//!     println!("{:x}", TEST.get()); // 5
//! 
//!     std::thread::spawn(|| {
//!         println!("{:x}", TEST.get()); // feedface
//!     }).join();
//! }
//! ```
//! 
//! <style>#macros + * > *:not(:last-child) { display:none } </style>


// TODO: aarch64 support
#![cfg(all(windows, any(target_arch="x86_64", target_arch="x86")))]
#![feature(asm)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Some module jiggery pokery for the sake of macros.
// TODO: move to a separate crate.
#[doc(hidden)]
pub mod raw_internal;
#[cfg(feature="raw")]
pub mod raw;

/// Statically initialize a thread local.
/// 
/// Note that no [`Drop`] implementations will be run.
/// 
/// # Example
/// 
/// ```
/// #![feature(asm)]
/// 
/// wintls::static_thread_local!{
///     static DATA: u32 = 0xfeedface;
///     static HELLO: [u8; 11] = *b"Hello World";
/// }
/// ```
/// 
/// See [`StaticThreadLocal`] for more information.
#[macro_export]
macro_rules! static_thread_local {
	(static $name:ident: $ty:ty = $value:expr;) => {
		static $name: $crate::StaticThreadLocal<$ty> = {
			$crate::init_static!(static $name: $ty = $value;);
			unsafe {
				$crate::StaticThreadLocal {
					get: || $crate::get_static!($name),
					set: |v| $crate::set_static!($name, v)
				}
			}
		};
	};
	($(static $name:ident: $ty:ty = $value:expr;)+) => {
		$(
			$crate::static_thread_local!{static $name: $ty = $value;}
		)+
	}
}

/// Enables setting or getting a static thread local value.
/// 
/// # Initialization and Destruction
/// 
/// The thread local can be statically initialized using [`static_thread_local`].
/// No [`Drop`] implementations will be run.
/// 
/// # Example
/// ```
/// #![feature(asm)]
/// 
/// wintls::static_thread_local!{
///     static DATA: u32 = 0xfeedface;
/// }
/// 
/// fn main() {
///     println!("{:x}", DATA.get());
/// }
/// ```
pub struct StaticThreadLocal<T> {
	#[doc(hidden)]
	pub get: fn() -> T,
	#[doc(hidden)]
	pub set: fn(T),
}
impl<T: Copy> StaticThreadLocal<T> {
	/// Returns the value of the the thread local.
	/// 
	/// # Example
	/// 
	/// ```
	/// # #![feature(asm)]
	/// # use wintls::static_thread_local;
	/// #
	/// # static_thread_local!{
	/// #     static DATA: u32 = 0xfeedface;
	/// # }
	/// # fn main() {
	/// let value = DATA.get();
	/// # }
	/// ```
	#[inline]
	pub fn get(&self) -> T {
		(self.get)()
	}

	/// Sets the value of the the thread local.
	/// 
	/// # Example
	/// 
	/// ```
	/// # #![feature(asm)]
	/// # use wintls::static_thread_local;
	/// #
	/// # static_thread_local!{
	/// #     static DATA: u32 = 0xfeedface;
	/// # }
	/// # fn main() {
	/// DATA.set(5);
	/// # }
	/// ```
	#[inline]
	pub fn set(&self, value: T) {
		(self.set)(value)
	}
}

// `_tls_used` is where the TLS directory information is stored.
// It must be included by the linker.
#[cfg(not(target_arch = "x86"))]
#[link_section = ".drectve"]
#[used]
static DIRECTIVE: [u8; 19] = *b"/INCLUDE:_tls_used ";
// On x86 the name is mangled by prefixing another underscore.
#[cfg(target_arch = "x86")]
#[link_section = ".drectve"]
#[used]
static DIRECTIVE: [u8; 20] = *b"/INCLUDE:__tls_used ";
