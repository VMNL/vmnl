// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Collect both pipes while the bounded child is alive, including long repeat traces.

use std::{
    io::Read,
    process::{Child, Output},
    thread,
    time::{Duration, Instant},
};

pub(crate) fn collect(mut child: Child) -> Result<Output, String> {
    let stdout = child.stdout.take().ok_or("probe stdout pipe missing")?;
    let stderr = child.stderr.take().ok_or("probe stderr pipe missing")?;
    let out = thread::spawn(move || read_pipe(stdout));
    let err = thread::spawn(move || read_pipe(stderr));
    let deadline = Instant::now() + Duration::from_secs(7);
    let mut expired = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            Ok(None) => {
                expired = true;
                let _ = child.kill();
                break child.wait().map_err(|error| error.to_string());
            }
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                break Err(error.to_string());
            }
        }
    };
    let stdout = out.join().map_err(|_| "stdout reader panicked")??;
    let mut stderr = err.join().map_err(|_| "stderr reader panicked")??;
    if expired {
        stderr.extend_from_slice(b"\nprobe exceeded seven-second exit watchdog\n");
    }
    Ok(Output {
        status: status?,
        stdout,
        stderr,
    })
}

fn read_pipe(mut pipe: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    Ok(bytes)
}
