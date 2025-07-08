fn main() {
	let mut signal_recv = signals_but_a_little_nicer::get_or_init_receiver().unwrap();
	println!("hit CTRL+C...");
	std::thread::sleep(std::time::Duration::from_secs(3));

	if signal_recv.len() == 0 {
		println!("you didn't hit CTRL+C :(");
	} else {
		while let Ok(signal) = signal_recv.try_recv() {
			println!(" received {signal:?}!");
		}
	}
}
