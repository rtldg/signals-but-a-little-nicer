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
