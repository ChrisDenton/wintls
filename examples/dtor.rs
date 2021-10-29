use wintls::dtor::register_dtor;
fn main() {
	register_dtor(|| println!("GoodbyeA!"));
	register_dtor(|| println!("GoodbyeB!"));
	register_dtor(|| println!("GoodbyeC!"));
	std::thread::spawn(|| {
		register_dtor(|| println!("Goodbye1!"));
		register_dtor(|| {
			register_dtor(|| println!("huh??"));
			println!("Goodbye2!")
		});
	})
	.join()
	.unwrap();
	println!("Hello!");
}
