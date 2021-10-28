#![feature(asm)]
#![feature(thread_local)]

#[thread_local]
static TEST: u32 = 0xfeedface;

#[inline(always)]
pub fn inline_the_value() -> u32 {
	TEST
}

#[inline(never)]
pub fn get_the_value() -> u32 {
	TEST
}

// Use a module handle to get the right thread-local.
// This works because statics themselves aren't inlined.
static MODULE_STATIC_DATA: (&u32, fn() -> u32) = {
	unsafe { (&wintls::raw::_tls_index, || wintls::raw::static_key!(TEST)) }
};
#[inline(always)]
pub fn get_module_value() -> u32 {
	let (&module, key) = MODULE_STATIC_DATA;
	unsafe { wintls::raw::get_static_from_module(module, key()) }
}
