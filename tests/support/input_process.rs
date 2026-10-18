// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Bounded, continuously drained process transport for the public input client.

use super::native_input_report::InputTestReport;

use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const INPUT_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_LINE: u64 = 8192;
const MAX_STDOUT: usize = 1024 * 1024;
const MAX_STDERR: usize = 64 * 1024;

pub(crate) struct ClientProcess {
    child: Child,
    stdin: ChildStdin,
    output: mpsc::Receiver<Result<String, String>>,
    diagnostics: Arc<Mutex<Vec<u8>>>,
    readers: Vec<JoinHandle<()>>,
    sequence: u64,
}

impl ClientProcess {
    pub(crate) fn spawn(command: &mut Command) -> Result<Self, String> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("could not start VMNL input client: {error}"))?;
        // Piped handles are guaranteed by the Command configuration above.
        let stdin = child.stdin.take().ok_or("missing client stdin")?;
        let stdout = child.stdout.take().ok_or("missing client stdout")?;
        let mut stderr = child.stderr.take().ok_or("missing client stderr")?;
        let (sender, output) = mpsc::channel();
        let stdout_reader = thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut total = 0;
            loop {
                let mut bytes = Vec::new();
                match reader
                    .by_ref()
                    .take(MAX_LINE + 1)
                    .read_until(b'\n', &mut bytes)
                {
                    Ok(0) => break,
                    Ok(_) => {
                        total += bytes.len();
                        if bytes.len() as u64 > MAX_LINE
                            || total > MAX_STDOUT
                            || bytes.last() != Some(&b'\n')
                        {
                            let _ = sender.send(Err(
                                "client stdout exceeded protocol limits or ended mid-record".into(),
                            ));
                            break;
                        }
                        let record = String::from_utf8(bytes)
                            .map(|line| line.trim_end_matches('\n').to_owned())
                            .map_err(|error| format!("client stdout is not UTF-8: {error}"));
                        if sender.send(record).is_err() {
                            break;
                        }
                    }
                    Err(error) => {
                        let _ = sender.send(Err(format!("could not read client stdout: {error}")));
                        break;
                    }
                }
            }
        });
        let diagnostics = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&diagnostics);
        let stderr_reader = thread::spawn(move || {
            let mut chunk = [0; 4096];
            while let Ok(count) = stderr.read(&mut chunk) {
                if count == 0 {
                    break;
                }
                if let Ok(mut buffer) = captured.lock() {
                    let available = MAX_STDERR.saturating_sub(buffer.len());
                    buffer.extend_from_slice(&chunk[..count.min(available)]);
                }
                // Always drain the pipe, including bytes beyond the diagnostic cap.
            }
        });
        Ok(Self {
            child,
            stdin,
            output,
            diagnostics,
            readers: vec![stdout_reader, stderr_reader],
            sequence: 0,
        })
    }

    pub(crate) fn observe(
        &mut self,
        report: &mut InputTestReport,
        section: &str,
        input: &str,
        checks: &[&str],
        action: &str,
        inject: impl FnOnce() -> Result<(), String>,
    ) -> Result<(), String> {
        self.sequence += 1;
        let sequence = self.sequence;
        let mut injection = Some(inject);
        for (index, check) in checks.iter().enumerate() {
            report.run(section, input, check, || -> Result<(), String> {
                if let Some(inject) = injection.take() {
                    self.send(&format!("ARM\t{sequence}\t{action}"))?;
                    self.expect(&format!("READY\t{sequence}"), INPUT_TIMEOUT)?;
                    inject()?;
                    self.send(&format!("INJECTED\t{sequence}"))?;
                }
                self.expect(
                    &format!("PASS\t{sequence}\t{check}"),
                    INPUT_TIMEOUT + Duration::from_secs(1),
                )?;
                if index + 1 == checks.len() {
                    self.expect(&format!("OBSERVED\t{sequence}"), INPUT_TIMEOUT)?;
                }
                Ok(())
            })?;
        }
        Ok(())
    }

    pub(crate) fn id(&self) -> u32 {
        self.child.id()
    }

    pub(crate) fn send(&mut self, command: &str) -> Result<(), String> {
        writeln!(self.stdin, "{command}")
            .and_then(|()| self.stdin.flush())
            .map_err(|error| format!("could not send command to input client: {error}"))
    }

    pub(crate) fn receive(&mut self, timeout: Duration) -> Result<String, String> {
        match self.output.recv_timeout(timeout) {
            Ok(record) => record,
            Err(error) => {
                let status = self.child.try_wait().map_err(|error| error.to_string())?;
                let stderr = self.diagnostics.lock().map_or_else(
                    |_| "diagnostic buffer poisoned".into(),
                    |bytes| String::from_utf8_lossy(&bytes).into_owned(),
                );
                Err(format!(
                    "input client did not reply: {error}; status={status:?}; stderr={stderr}"
                ))
            }
        }
    }

    pub(crate) fn expect(&mut self, expected: &str, timeout: Duration) -> Result<(), String> {
        expect_record(&self.receive(timeout)?, expected)
    }

    pub(crate) fn finish(&mut self, timeout: Duration) -> Result<(), String> {
        self.send("FINISH")?;
        self.expect("DONE", timeout)?;
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(status) = self.child.try_wait().map_err(|error| error.to_string())? {
                if !status.success() {
                    return Err(format!("input client exited with {status}"));
                }
                // DONE must be the final record, even if the child exits successfully.
                return match self.output.recv_timeout(timeout) {
                    Err(mpsc::RecvTimeoutError::Disconnected) => Ok(()),
                    record => Err(format!("unexpected output after DONE: {record:?}")),
                };
            }
            if Instant::now() >= deadline {
                return Err("input client did not exit after DONE".into());
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Drop for ClientProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        for reader in self.readers.drain(..) {
            let _ = reader.join();
        }
    }
}

pub(crate) fn expect_record(actual: &str, expected: &str) -> Result<(), String> {
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "expected client record {expected:?}, received {actual:?}"
        ))
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "linux")]
    fn injection_requires_ready_and_release_requires_complete_press_observation() {
        for (response, expect_press) in [
            ("print('READY\\t2', flush=True)", false),
            ("print('READY\\t1', flush=True)\nassert input() == 'INJECTED\\t1'\nprint('PASS\\t1\\tPress\\nPASS\\t1\\tPress input states\\nOBSERVED\\t2', flush=True)", true),
        ] {
            let mut command = Command::new("python3");
            command.args(["-c", &format!("input()\n{response}")]);
            let mut client = ClientProcess::spawn(&mut command).expect("fixture starts");
            let mut report = InputTestReport::default();
            let checks = ["Press", "Press input states"];
            report.plan("Keyboard", "A", &checks);
            report.plan("Keyboard", "A", &["Release"]);
            let (mut pressed, mut released) = (false, false);
            let result = client.observe(&mut report, "Keyboard", "A", &checks, "KEY_PRESS\tA", || {
                pressed = true;
                Ok(())
            }).and_then(|()| client.observe(&mut report, "Keyboard", "A", &["Release"], "KEY_RELEASE\tA", || {
                released = true;
                Ok(())
            }));
            let error = result.expect_err("fixture must reject the wrong acknowledgement");
            let expected = if expect_press { "OBSERVED\\t1" } else { "READY\\t1" };
            assert!(error.contains(expected), "{error}");
            assert_eq!(pressed, expect_press);
            assert!(!released, "release must wait for complete press acknowledgement");
            assert!(!report.is_complete());
            assert!(report.render().contains("FAIL"));
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn shutdown_rejects_extra_records_and_unsuccessful_exit() {
        for ending in ["print('EXTRA', flush=True)", "raise SystemExit(17)"] {
            let mut command = Command::new("python3");
            command.args([
                "-c",
                &format!("input()\nprint('DONE', flush=True)\n{ending}"),
            ]);
            let mut client = ClientProcess::spawn(&mut command).expect("fixture starts");
            assert!(client.finish(Duration::from_secs(5)).is_err());
        }
    }

    #[test]
    fn acknowledgements_reject_failures_wrong_sequence_and_wrong_check() {
        assert!(expect_record("PASS\t7\tPress", "PASS\t7\tPress").is_ok());
        for actual in [
            "PASS\t6\tPress",
            "PASS\t7\tScancode",
            "FAIL\t7\tPress\tmissing event",
            "OBSERVED\t7",
            "PASS",
        ] {
            assert!(expect_record(actual, "PASS\t7\tPress").is_err());
        }
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn transport_drains_large_stderr_and_reaps_successful_client() {
        let mut command = Command::new("python3");
        command.args(["-c", "import sys\nsys.stdout.write('RECORD\\n' * 20000)\nsys.stdout.flush()\nsys.stderr.write('x' * (2 * 1024 * 1024))\nprint('READY', flush=True)\nassert input() == 'FINISH'\nprint('DONE', flush=True)"]);
        let mut client = ClientProcess::spawn(&mut command).expect("fixture starts");
        assert!(client.id() > 0);
        for _ in 0..20000 {
            client
                .expect("RECORD", Duration::from_secs(5))
                .expect("stdout drains");
        }
        client
            .expect("READY", Duration::from_secs(5))
            .expect("stderr cannot block stdout");
        client
            .finish(Duration::from_secs(5))
            .expect("client exits and both pipes drain");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn transport_fails_on_crash_timeout_and_oversized_output() {
        for script in [
            "raise SystemExit(17)",
            "import time; time.sleep(10)",
            "print('x' * 9000)",
            "import sys; sys.stdout.write('unterminated')",
        ] {
            let mut command = Command::new("python3");
            command.args(["-c", script]);
            let mut client = ClientProcess::spawn(&mut command).expect("fixture starts");
            assert!(
                client.receive(Duration::from_millis(100)).is_err(),
                "{script}"
            );
            // Drop kills a timed-out child and reaps it before joining its readers.
        }
    }
}
