#![feature(asm)]

use wintls::raw::{init_static, static_key, static_ptr, get_static, set_static};

init_static!(
	static TEST: u32 = 0xfeedface;
);

// TODO: Should also test get/set
#[test]
fn raw_get_set() {
	unsafe {
		// Get the key
		let key: u32 = static_key!(TEST);

		// Get a mutable pointer to the value.
		let value: u32 = get_static(key);
        assert_eq!(value, 0xfeedface);
		
		// Set the value.
		set_static::<u32>(key, 5);

		// Get the value again.
		let value: u32 = get_static(key);
        assert_eq!(value, 5);

		// Spawn a thread and make sure the value has reverted to default...
		let _ = std::thread::spawn(move || {
			let value: u32 = get_static(key);
            assert_eq!(value, 0xfeedface);
		})
		.join();
		
		// ...but the first thread's value is the same.
        let value: u32 = get_static(key);
		assert_eq!(value, 5);
	}
}

#[test]
fn raw_ptr() {
	unsafe {
		// Get the key
		let key: u32 = static_key!(TEST);

		// Get a mutable pointer to the value.
		let value = static_ptr::<u32>(key);
        assert_eq!(*value, 0xfeedface);
		
		// Set the value.
		*value = 5;

		// Get the value again.
		let value = static_ptr::<u32>(key);
        assert_eq!(*value, 5);

		// Spawn a thread and make sure the value has reverted to default...
		let _ = std::thread::spawn(move || {
			let value = static_ptr::<u32>(key);
            assert_eq!(*value, 0xfeedface);
		})
		.join();
		
		// ...but the first thread's value is the same.
        let value = static_ptr::<u32>(key);
		assert_eq!(*value, 5);
	}
}
