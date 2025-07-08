// SPDX-License-Identifier: MIT
// Copyright 2025 rtldg <rtldg@protonmail.com>
// Copyright Tokio Contributors

use std::sync::Mutex;

use tokio::sync::broadcast::{self, Sender};

use crate::{SignalError, SignalInfo, SignalReceiver};

static SENDER: Mutex<Option<Sender<SignalInfo>>> = Mutex::new(None);

// https://learn.microsoft.com/en-us/windows/console/setconsolectrlhandler
// https://learn.microsoft.com/en-us/windows/console/handlerroutine
// TODO: Setup an invisible window to listen for events because the logoff/shutdown are apparently only received by services?

#[allow(non_camel_case_types)]
type PHANDLER_ROUTINE = Option<unsafe extern "system" fn(ctrltype: u32) -> i32>;
const CTRL_BREAK_EVENT: u32 = 1u32;
const CTRL_CLOSE_EVENT: u32 = 2u32;
const CTRL_C_EVENT: u32 = 0u32;
const CTRL_LOGOFF_EVENT: u32 = 5u32;
const CTRL_SHUTDOWN_EVENT: u32 = 6u32;
windows_targets::link!("kernel32.dll" "system" fn SetConsoleCtrlHandler(handlerroutine: PHANDLER_ROUTINE, add: i32) -> i32);
windows_targets::link!("kernel32.dll" "system" fn GetLastError() -> i32);

unsafe extern "system" fn ctrl_handler(ctrltype: u32) -> i32 {
	// The handler is called in a new thread, so we don't have to deal with reentrancy (like normal Unix signals).

	// going to steal a small amount of code here because I wouldn't have known unless I checked tokio...
	// https://github.com/tokio-rs/tokio/blob/7a6c424f6e07d79e7c029866ed601bb948aba10a/tokio/src/signal/windows/sys.rs#L128-L146

	let event_was_handled = {
		let lock = SENDER.lock().unwrap();
		let broadcast = lock.as_ref().unwrap();
		broadcast
			.send(match ctrltype {
				CTRL_C_EVENT => SignalInfo::Int,
				CTRL_BREAK_EVENT => SignalInfo::Hup,
				_ => SignalInfo::Term,
			})
			.is_ok()
	};

	if event_was_handled {
		if ctrltype == CTRL_CLOSE_EVENT || ctrltype == CTRL_LOGOFF_EVENT || ctrltype == CTRL_SHUTDOWN_EVENT {
			// Returning from the handler function of those events immediately terminates the process.
			// So for async systems, the easiest solution is to simply never return from
			// the handler function.
			loop {
				std::thread::park();
			}
		} else {
			1 // TRUE
		}
	} else {
		0 // FALSE
	}
}

pub(crate) fn get_or_init_receiver() -> Result<SignalReceiver, SignalError> {
	let mut lock = SENDER.lock().unwrap();
	if let Some(x) = &*lock {
		Ok(x.subscribe())
	} else {
		let (sender, receiver) = broadcast::channel(64);

		unsafe {
			if 0 == SetConsoleCtrlHandler(Some(ctrl_handler), 1) {
				return Err(SignalError::Errno(GetLastError()));
			}
		}

		*lock = Some(sender);
		Ok(receiver)
	}
}
