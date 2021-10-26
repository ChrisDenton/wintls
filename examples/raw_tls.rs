#![feature(asm)]

use wintls::raw::{init_static, static_key, static_ptr};

init_static!(
	static TEST: u32 = 0xfeedface;
);

// TODO: Should use get/set

fn main() {
	unsafe {
		// Get the key
		let key: u32 = static_key!(TEST);
		dbg!(key);

		// Get a mutable pointer to the value.
		let value = static_ptr::<u32>(key);
        println!("thread 1: {:x}", *value);
		
		// Set the value.
		*value = 5;
	}
}
