// SPDX-License-Identifier: MIT
// Copyright 2025 rtldg <rtldg@protonmail.com>

// https://man7.org/linux/man-pages/man7/signal.7.html

use nix::sys::signal::{SigSet, Signal};
use std::sync::Mutex;
use tokio::sync::broadcast::{self, Sender};

use crate::{SignalError, SignalInfo, SignalReceiver};

static SENDER: Mutex<Option<Sender<SignalInfo>>> = Mutex::new(None);

const SIGNALS: [Signal; 6] = [
	Signal::SIGHUP,
	Signal::SIGINT,
	Signal::SIGTERM,
	Signal::SIGQUIT,
	Signal::SIGUSR1,
	Signal::SIGUSR2,
];

fn signal_thread(sender: Sender<SignalInfo>) {
	let sigset = SigSet::from_iter(SIGNALS);
	loop {
		if let Ok(signal) = sigset.wait() {
			let signal = match signal {
				Signal::SIGINT => SignalInfo::Int,
				Signal::SIGHUP => SignalInfo::Hup,
				Signal::SIGTERM => SignalInfo::Term,
				Signal::SIGQUIT => SignalInfo::Quit,
				Signal::SIGUSR1 => SignalInfo::Usr1,
				Signal::SIGUSR2 => SignalInfo::Usr2,
				_ => unreachable!(),
			};
			let _ = sender.send(signal);
		}
	}
}

pub(crate) fn get_or_init_receiver() -> Result<SignalReceiver, SignalError> {
	let mut lock = SENDER.lock().unwrap();
	if let Some(x) = &*lock {
		Ok(x.subscribe())
	} else {
		let (sender, receiver) = broadcast::channel(64);

		let sigset = SigSet::from_iter(SIGNALS);
		if let Err(errno) = sigset.thread_block() {
			return Err(SignalError::Errno(errno as i32));
		}

		// just unwrap/panic if we fail to create the thread... so much easier this way...
		let _ = std::thread::Builder::new().name("signal-handler".into()).spawn({
			let sender = sender.clone();
			move || signal_thread(sender)
		})?;

		*lock = Some(sender);
		Ok(receiver)
	}
}
