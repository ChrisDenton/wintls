// This example creates a type that will return a different value depending on
// which thread uses it.

// Use the std's thread_local for this.
// We still handle drops using this library.
#![feature(thread_local)]

// A module to protect our secrets.
mod string_local {
	use std::{fmt::Display, ptr::addr_of_mut};
	use wintls::dtor::register_dtor;

	// Magically changes when sent across threads.
	// Each thread will drop their local string when a thread exits.
	// This should be more like a RefCell but for simplicity in this example I
	// just expose methods that immediately drop any references.
	pub struct StringLocal {
		get: fn() -> *mut String,
	}
	impl StringLocal {
		pub fn push_str(&self, s: &str) {
			unsafe { self.get_mut().push_str(s) }
		}
		// SAFETY:
		// * The reference must not outlive the current thread.
		// * You must not have another reference at the same time.
		unsafe fn get_mut(&self) -> &mut String {
			&mut *(self.get)()
		}
		// SAFETY:
		// * The reference must not outlive the current thread.
		// * The value must not be mutated while this reference is live.
		unsafe fn get(&self) -> &str {
			&*(self.get)()
		}
	}
	impl Display for StringLocal {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			unsafe { self.get().fmt(f) }
		}
	}
	// This is adapted from the standard library.
	pub fn get_mut() -> StringLocal {
		#[thread_local]
		static mut BUFFER: String = String::new();
		#[thread_local]
		static mut STATE: u8 = 0;

		fn destroy() {
			unsafe {
				println!("dropping thread local");
				debug_assert_eq!(STATE, 1);
				STATE = 2;
				core::ptr::drop_in_place(core::ptr::addr_of_mut!(BUFFER));
			}
		}
		unsafe {
			match STATE {
				0 => {
					register_dtor(destroy);
					STATE = 1;
				}
				1 => {}
				_ => panic!("the thread local was already destroyed!"),
			}
			StringLocal {
				get: || addr_of_mut!(BUFFER),
			}
		}
	}
}
use string_local::*;

fn main() {
	let a = get_mut();
	a.push_str("Hello!");

	let b = std::thread::spawn(|| {
		let b = get_mut();
		b.push_str(" World!");
		println!("Thread2: {}", b); // " World!"
							// return the the StringLocal
		b
	})
	.join()
	.unwrap();
	// `a` and `b` are the same.
	println!("Thread1: {}", a); // "Hello!"
	println!("Thread1: {}", b); // "Hello!"
}
