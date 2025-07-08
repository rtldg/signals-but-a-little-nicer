# signals-but-a-little-nicer
[![Crates](https://img.shields.io/crates/v/signals-but-a-little-nicer.svg)](
https://crates.io/crates/signals-but-a-little-nicer)
[![Docs](https://docs.rs/signals-but-a-little-nicer/badge.svg)](
https://docs.rs/signals-but-a-little-nicer)

A wrapper around signal-handlers for Windows [console applications] and Unix programs.

Signals in Unix are a bad API.
A clean way to handle them is to block signals on all threads and create a dedicated thread to wait on them.
Which is what this does...

Only a few signals are caught: SIGINT, SIGTERM, SIGHUP, SIGQUIT, SIGUSR1, and SIGUSR2.

### Why?
You can do everything and more with [signal-hook](https://crates.io/crates/signal-hook) or [async-signal](https://crates.io/crates/async-signal) but I explicitly want to block signals on all threads on Unix/Linux (and then use sigwait(3)).

### Example
```rust
fn main() {
	let signal_recv = signals_but_a_little_nicer::get_or_init_recv().unwrap();
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
```

Also note that you'll want to register the handler before creating threads (or say a tokio runtime):
```rust
use anyhow::Context;

fn main() -> anyhow::Result<()> {
	let signal_recv = signals_but_a_little_nicer::get_or_init_receiver().context("failed to setup signal handler")?;
	let rt = tokio::runtime::Runtime::new()?;
	rt.block_on(async { tokio::spawn(async_main(signal_recv)).await? })
}

async fn async_main(mut signal_recv: signals_but_a_little_nicer::SignalReceiver) -> anyhow::Result<()> {
	tokio::spawn(async {
		tokio::time::sleep(std::time::Duration::from_secs(3)).await;
		std::process::exit(0);
	});
	while let Ok(signal) = signal_recv.recv().await {
		println!("signal received: {signal:?}");
	}
	Ok(())
}

```
