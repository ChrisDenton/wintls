/// Initialize a `static` as a thread-local.
/// 
/// # Example
/// ```
/// wintls::raw::init_static!(
///     static DATA: u32 = 0xfeedface;
/// );
/// ```
#[macro_export]
macro_rules! init_static {
	($vis:vis static $name:ident: $ty:ty = $value:expr;) => {
		// This doesn't need to be `Cell` or anything. The trick is that we
		// don't ever touch this memory. Instead thread-local copies are used.
		#[link_section = ".tls$"]
		#[used]
		$vis static $name: $ty = $value;
	};
}

/// Returns a mutable pointer to a tls value.
/// 
/// Generally it should not be stored as this pointer may point to old data when
/// a library is loaded that causes the [`tls_array`] to be reallocated.
/// 
/// # Safety
/// * The key must be a valid key returned by [`static_key`]
/// * The type should be the same as when it was created.
/// 
/// # Example
/// 
/// ```
/// #![feature(asm)]
/// wintls::raw::init_static!(
///     static DATA: u32 = 0xfeedface;
/// );
/// unsafe {
///     let key: u32 = wintls::raw::static_key!(DATA);
///     let value: *mut u32 = wintls::raw::static_ptr(key);
/// }
/// ```
#[inline(always)]
pub unsafe fn static_ptr<T>(key: u32) -> *mut T {
	let mut ptr: *mut T = tls_array().cast();
	let key = key as usize;
	let index = _tls_index as usize;
	asm!(
		"mov {ptr}, [{ptr} + {index} * {multiplier}]",
		"lea {ptr}, [{key} + {ptr}]",
		ptr = inout(reg) ptr,
		index = in(reg) index,
		key = in(reg) key,
		multiplier = const INDEX_MULTIPLIER,
	);
	ptr
}

/// Sets a static thread-local value.
/// 
/// # Safety
/// * The key must be a valid key returned by [`static_key`]
/// * The type should be the same as when it was created.
/// 
/// # Example
/// 
/// ```
/// #![feature(asm)]
/// wintls::raw::init_static!(
///     static DATA: u32 = 0xfeedface;
/// );
/// unsafe {
///     let key: u32 = wintls::raw::static_key!(DATA);
///     wintls::raw::set_static(key, 5_u32);
/// }
/// ```
#[inline(always)]
pub unsafe fn set_static<T>(key: u32, value: T) {
	*static_ptr(key) = value
}

/// Returns the value of a static thread-local.
/// 
/// # Safety
/// The key must be a valid key returned by [`static_key`]
/// 
/// # Example
/// 
/// ```
/// #![feature(asm)]
/// wintls::raw::init_static!(
///     static DATA: u32 = 0xfeedface;
/// );
/// unsafe {
///     let key: u32 = wintls::raw::static_key!(DATA);
///     let value: u32 = wintls::raw::get_static(key);
/// }
/// ```
#[inline(always)]
pub unsafe fn get_static<T: Copy>(key: u32) -> T {
	*static_ptr(key)
}

/// Convenience macro for setting the static thread-local value by its
/// identifier.
#[macro_export]
macro_rules! set_static {
	($name:ident, $value:expr) => {
		$crate::raw_internal::set_static($crate::static_key!($name), $value)
	};
}
/// Convenience macro for getting the static thread-local value by its
/// identifier.
#[macro_export]
macro_rules! get_static {
	($name:ident) => {
		$crate::raw_internal::get_static($crate::static_key!($name))
	};
}

/// Convenience macro for getting a static thread-local pointer by its
/// identifier.
#[macro_export]
macro_rules! static_ptr {
    ($name:ident) => {
        $crate::static_ptr($crate::static_key!($name))
    };
}

/// Returns a key that identifies the thread local.
/// 
/// # Safety
/// Must only be used with static thread locals.
/// 
/// Normal pointer safety rules apply. The memory may be deallocated or reused
/// when the thread exits.
/// 
/// # Example
/// 
/// ```
/// #![feature(asm)]
/// wintls::raw::init_static!(
///     static DATA: u32 = 0xfeedface;
/// );
/// unsafe {
///     let key: u32 = wintls::raw::static_key!(DATA);
/// }
/// ```
#[macro_export]
macro_rules! static_key {
	($name:ident) => {{
		let offset: u32;
		#[cfg(any(target_arch="x86_64", target_arch="x86"))]
		asm!(
			// The quotes around "{name}" avoids potential ambiguity.
			// The `@SECREL32` is LLVM magic that marks `{name}` as being
			// section relative (i.e. relative to the start of the TLS section).
			// This uses `IMAGE_REL_I386_SECREL` (x86) or `IMAGE_REL_AMD64_SECREL` (x64).
			// See: https://docs.microsoft.com/en-us/windows/win32/debug/pe-format#x64-processors
			r#"mov {offset:e}, DWORD PTR OFFSET "{name}"@SECREL32"#,
			name = sym $name,
			offset = out(reg) offset,
			// FIXME: Check if these options are correct.
			options(pure, readonly, preserves_flags, nostack),
		);
		offset
	}}
}

/// Returns a pointer to thread-local memory for the current thread.
/// 
/// Calling this twice on the same thread usually returns the same value.
/// However, if a new library is loaded and if that library uses static thread
/// locals, it may cause a new TLS array to be allocated for each thread.
/// 
/// Despite this, a pointer returned here will never be freed as long as the
/// thread is still running.
/// 
/// # Safety
/// 
/// This is particularly unsafe to use because:
/// * it does not return the size of the array.
/// * it gives access to all the thread's locals regardless of the module they
///   were loaded in.
/// 
/// Note also that the memory may be deallocated or reused when the thread exits.
#[inline(always)]
pub fn tls_array() -> *mut *mut u8 {
	tls_array_()
}

// TODO: aarch64
// x18 + 0x58

#[cfg(target_arch="x86_64")]
#[inline(always)]
pub fn tls_array_() -> *mut *mut u8 {
	unsafe {
		let tls_array: *mut *mut u8;
		asm!(
			"mov {}, gs:[0x58]",
			out(reg) tls_array,
			options(pure, readonly, preserves_flags, nostack),
		);
		tls_array
	}
}
#[cfg(target_arch="x86")]
#[inline(always)]
fn tls_array_() -> *mut *mut u8 {
	unsafe {
		let tls_array: *mut *mut u8;
		asm!(
			"mov {}, fs:[0x2c]",
			out(reg) tls_array,
			options(pure, readonly, preserves_flags, nostack),
		);
		tls_array
	}
}
#[cfg(target_arch="x86")]
const INDEX_MULTIPLIER: usize = 4;
#[cfg(target_arch="x86_64")]
const INDEX_MULTIPLIER: usize = 8;

extern "C" {
	/// The offset (divided by 8) into the static thread local array where this module's locals begin.
	pub static _tls_index: u32;
}

#[doc(inline)]
pub use crate::init_static;
#[doc(inline)]
pub use crate::static_key;
#[doc(inline)]
pub use crate::static_ptr;
#[doc(inline)]
pub use crate::get_static;
#[doc(inline)]
pub use crate::set_static;
