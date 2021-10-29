//! Creates static thread-locals for Windows binaries.
//!
//! Essentially, this is a manual implementation of the thread_local feature.
//!
//! Note this crate is a playground for my education and not intended for
//! production use. If anything does end up being useful then I'll create a new
//! crate with just the good parts, or else incorporate it into existing code.
//!
//! # Use
//!
//! This requires a nightly compiler. You will also need to add `#![feature(asm)]`
//! to your `main.rs` or `lib.rs` file if you use the TLS macros.
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
//! <style>#macros + * > *:not(:is(:nth-last-child(2), :last-child)) { display:none } </style>

// TODO: aarch64 support
#![cfg(all(windows, any(target_arch = "x86_64", target_arch = "x86")))]
#![feature(asm)]
#![cfg_attr(docsrs, feature(doc_cfg))]

// Some module jiggery pokery for the sake of macros.
// TODO: move to a separate crate.
#[cfg(feature = "raw")]
pub mod raw;
#[doc(hidden)]
pub mod raw_internal;

pub mod dtor;

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
	($vis:vis static $name:ident: $ty:ty = $value:expr;) => {
		$vis static $name: $crate::StaticThreadLocal<$ty> = {
			if ::core::mem::needs_drop::<$ty>() {
				panic!("static thread locals cannot be dropped");
			};

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
	#[inline(always)]
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
	#[inline(always)]
	pub fn set(&self, value: T) {
		(self.set)(value)
	}
}

/// Grants unsafe access to the thread local.
///
/// In general you should make sure that any references to the actual thread
/// local are as short-lived as possible.
///
/// # Stale Pointers
///
/// If a new DLL is lazily loaded and that DLL requires the static TLS array to
/// be expanded then, for each thread, a new TLS array will be created without
/// dropping the old one. The old array will then be copied to the new one.
///
/// So now there are two copies of the thread local data. If a new pointer is
/// made then it'll point to the new data but any old pointers will still point
/// to the "stale" data.
pub struct UnsafeLocal<T> {
	#[doc(hidden)]
	pub get: fn() -> *mut T,
}
impl<T> UnsafeLocal<T> {
	/// Getting a pointer is safe.
	/// Using it should be mostly safe (normal caveats aside) so long as there
	/// aren't any active references. That said, you should almost certainly use
	/// and discard the pointer asap.
	pub fn as_ptr(&self) -> *mut T {
		(self.get)()
	}
	/// There can be many shared references but there must not be a mutable
	/// reference at all. Also no mutation should occur for the lifetime of this
	/// reference.
	pub unsafe fn as_ref(&self) -> &T {
		&*self.as_ptr()
	}
	/// There can only be one mutable reference at a time and there must not be
	/// any shared references. Also mutation should only happen via this
	/// reference and not through any pointer.
	pub unsafe fn as_ref_mut(&self) -> &mut T {
		&mut *self.as_ptr()
	}

	/// Drops the memory. No further use of the memory should occur after
	/// calling this, unless a new value is created in place.
	pub unsafe fn drop_value(&self) {
		core::ptr::drop_in_place(self.as_ptr());
	}
}

/// Create an [`UnsafeLocal`].
#[macro_export]
macro_rules! unsafe_local {
	($vis:vis static $name:ident: $ty:ty = $value:expr;) => {
		static $name: $crate::UnsafeLocal<$ty> = {
			$crate::init_static!(
				static $name: $ty = $value;
			);
			$crate::UnsafeLocal {
				get: || unsafe { $crate::static_ptr!($name) },
			}
		};
	};
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
