#![feature(asm)]

wintls::static_thread_local! {
	static DATA: u32 = 0xfeedface;
	static HELLO: [u8; 11] = *b"Hello World";
}

fn main() {
	println!("{:x}", DATA.get());
	println!("{}", std::str::from_utf8(&HELLO.get()).unwrap());
	HELLO.set(*b"Goodbye TLS");
	println!("{}", std::str::from_utf8(&HELLO.get()).unwrap());
}
