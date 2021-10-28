#![feature(asm)]

fn main() {
	// Get the actual value (no inline)
	println!("{:x}", libfoo::get_the_value());
	// Get a nonsense value (inline)
	println!("{:x}", libfoo::inline_the_value());
	// Get the module handle using the module's index
	println!("{:x}", libfoo::get_module_value());
}
