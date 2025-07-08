// SPDX-License-Identifier: MIT
// Copyright 2025 rtldg <rtldg@protonmail.com>

//! A wrapper around signal-handlers for Windows [console applications] and Unix programs.
//!
//! Signals in Unix are a bad API.
//! A clean way to handle them is to block signals on all threads and create a dedicated thread to wait on them.
//! Which is what this does...
//!
//! #### Example
//! ```
//! fn main() {
//! 	let mut signal_recv = signals_but_a_little_nicer::get_or_init_receiver().unwrap();
//! 	println!("hit CTRL+C...");
//! 	std::thread::sleep(std::time::Duration::from_secs(5));
//!
//! 	if signal_recv.len() == 0 {
//! 		println!("you didn't hit CTRL+C :(");
//! 	} else {
//! 		while let Ok(signal) = signal_recv.try_recv() {
//! 			println!(" received {signal:?}!");
//! 		}
//! 	}
//! }
//! ```
//!
//! Also note that you'll want to register the handler before creating threads (or say a tokio runtime):
//! ```
//! use anyhow::Context;
//!
//! fn main() -> anyhow::Result<()> {
//! 	let signal_recv = signals_but_a_little_nicer::get_or_init_receiver().context("failed to setup signal handler")?;
//! 	let rt = tokio::runtime::Runtime::new()?;
//! 	rt.block_on(async { tokio::spawn(async_main(signal_recv)).await? })
//! }
//!
//! async fn async_main(mut signal_recv: signals_but_a_little_nicer::SignalReceiver) -> anyhow::Result<()> {
//! 	tokio::spawn(async {
//! 		tokio::time::sleep(std::time::Duration::from_secs(3)).await;
//! 		std::process::exit(0);
//! 	});
//! 	while let Ok(signal) = signal_recv.recv().await {
//! 		println!("signal received: {signal:?}");
//! 	}
//! 	Ok(())
//! }
//! ```

#[cfg(unix)]
mod unix;
#[cfg(unix)]
use unix as os;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
use windows as os;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SignalInfo {
	/// SIGINT or ctrl+c.
	Int,
	/// SIGTERM (on Windows: logoff, shutdown, or window closed).
	/// On Windows, you'll want to cleanup & exit fast because the process will be terminated soon.
	Term,
	/// SIGHUP (on Windows: ctrl+break).
	Hup,
	/// SIGQUIT (unix only)
	Quit,
	/// SIGUSR1 (unix only)
	Usr1,
	/// SIGUSR2 (unix only)
	Usr2,
}

#[derive(thiserror::Error, Debug)]
pub enum SignalError {
	/// Unix only
	#[error("failed to create signal handler thread")]
	Thread(#[from] std::io::Error),
	/// The inner value comes from [errno](https://docs.rs/nix/latest/nix/errno/enum.Errno.html) on Unix and [GetLastError](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-getlasterror) on Windows.
	#[error("failed to setup signal handler")]
	Errno(i32),
}

pub type SignalReceiver = tokio::sync::broadcast::Receiver<SignalInfo>;

/// If this if returns an Err() then the process is expected to exit soon as the signal-handler state might be unstable.
pub fn get_or_init_receiver() -> Result<SignalReceiver, SignalError> {
	os::get_or_init_receiver()
}
