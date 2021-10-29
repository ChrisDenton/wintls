//! Register thread local destructors
//!
//! This is super simple. It just allows adding to a thread-local list of
//! destructors to run. Once the thread exits, the list is drained and all
//! destructors are run, from last to first.
//!
//! # Example
//!
//! ```
//! wintls::dtor::register_dtor(|| println!("Goodbye Thread!"));
//! ```
//!
//! See also, the examples directory.
//!
//! # Registering Destructors in Destructors
//!
//! Destructors can be registered while a destructor is being run. It is
//! currently up to the users of this library to prevent an infinite loop in
//! this situation (e.g. by only allowing a thread local to be initialized and
//! destroyed once, or by checking [`state`]).
//!
//! # Limitations
//!
//! If this is used in a DLL and the DLL is unloaded then destructors will only
//! be run for the thread that does the unloading. I cannot think of a generic
//! way round this. For example, leaking memory is always safe but arbitrarily
//! freeing memory that is possibly being used is obviously not safe.
//!
//! Ideally the drop code would be delayed until the thread exits but if the
//! DLL has already been unloaded then there's no code left to run.

crate::unsafe_local!(
	static DESTRUCTORS: Vec<fn()> = Vec::new();
);

#[derive(Clone, Copy)]
pub enum DtorState {
	/// Passively waiting to run destructors, either when the thread exits or
	/// [`drop_locals`] is called.
	Passive,
	/// Destructors are currently being run.
	Dropping,
}
crate::static_thread_local! {
	static STATE: DtorState = DtorState::Passive;
}
/// Returns the current [`DtorState`].
pub fn state() -> DtorState {
	STATE.get()
}

/// Register a destructor for a thread local on this thread only.
///
/// Every thread that initializes a value will need to call this otherwise the
/// drop function will not be run. Here are a few strategies for registering
/// per-thread drops:
///
/// * Lazily register a drop function the first time the relevant TLS value is
///   accessed.
/// * Register all drop functions the first time any TLS value is first accessed.
/// * Register all drops when the thread starts. This can be done using the
///   `CRT$XDC` initializer function. However, if a DLL is lazily loaded, then
///   any threads existing prior to the load will not be initialized (other than
///   the thread that loads the DLL).
/// * Some combination of the above.
///
/// My preference is currently for the first option.
pub fn register_dtor(f: fn()) {
	unsafe { DESTRUCTORS.as_ref_mut().push(f) };
}

#[link_section = ".CRT$XLB"]
#[doc(hidden)]
#[used]
pub static TLS_CALLBACK: unsafe extern "system" fn(*mut i8, u32, *mut i8) = tls_callback;
extern "system" fn tls_callback(_: *mut i8, reason: u32, _: *mut i8) {
	const DLL_THREAD_DETACH: u32 = 3;
	const DLL_PROCESS_DETACH: u32 = 0;

	if reason == DLL_THREAD_DETACH || reason == DLL_PROCESS_DETACH {
		unsafe {
			STATE.set(DtorState::Dropping);
			drop_locals_internal();
			// The thread local memory is never used after this point.
			DESTRUCTORS.drop_value();
		}
	}
}
unsafe fn drop_locals_internal() {
	// As noted in the docs, this is potentially an infinite loop.
	// It's currently up to users of this API to prevent that.
	while let Some(dtor) = DESTRUCTORS.as_ref_mut().pop() {
		(dtor)();
	}
}

/// Runs the thread local drops.
///
/// # SAFETY
///
/// This is incredibly unsafe. For example, it's not safe to call this if there
/// are any live references or pointers to memory that would be freed by drop
/// functions.
///
/// I'm not actually sure if this is useful. It'd be easier and safer to simply
/// exit the thread and start a new one. Or else use a [fiber][1] for local
/// storage.
///
/// [1]: https://docs.microsoft.com/en-us/windows/win32/procthread/fibers
pub unsafe fn drop_locals() {
	STATE.set(DtorState::Dropping);
	drop_locals_internal();
	STATE.set(DtorState::Passive);
}
