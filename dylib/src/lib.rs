#![feature(asm)]

wintls::raw::init_static!(
	static TEST: u32 = 0xfeedface;
);

#[inline(always)]
pub fn inline_the_value() -> u32 {
    unsafe { wintls::raw::get_static!(TEST) }
}

#[inline(never)]
pub fn get_the_value() -> u32 {
    unsafe { wintls::raw::get_static!(TEST) }
}
