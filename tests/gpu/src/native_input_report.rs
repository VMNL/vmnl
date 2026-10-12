// SPDX-FileCopyrightText: 2026 Hugo Duda
// SPDX-License-Identifier: MIT

//! Headless per-check reporting for the public native-input scenario.

#![allow(clippy::panic)]

use std::fmt::{Display, Write as _};
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

#[derive(Debug, PartialEq)]
enum Status {
    NotRun,
    Pass,
    Fail(String),
    Excluded(String),
}

struct Check {
    section: String,
    input: String,
    name: String,
    status: Status,
}

#[derive(Default)]
pub(crate) struct InputTestReport {
    checks: Vec<Check>,
}

impl InputTestReport {
    pub(crate) fn plan(&mut self, section: &str, input: &str, names: &[&str]) {
        for name in names {
            assert!(!self.checks.iter().any(|check| {
                check.section == section && check.input == input && check.name == *name
            }));
            self.checks.push(Check {
                section: section.into(),
                input: input.into(),
                name: (*name).into(),
                status: Status::NotRun,
            });
        }
    }

    pub(crate) fn run<T, E: Display>(
        &mut self,
        section: &str,
        input: &str,
        name: &str,
        operation: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let check = self.pending_check(section, input, name);
        match catch_unwind(AssertUnwindSafe(operation)) {
            Ok(Ok(value)) => {
                check.status = Status::Pass;
                Ok(value)
            }
            Ok(Err(error)) => {
                check.status = Status::Fail(error.to_string());
                Err(error)
            }
            Err(payload) => {
                let reason = payload
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| payload.downcast_ref::<&str>().copied())
                    .unwrap_or("assertion panicked with a non-string payload");
                check.status = Status::Fail(reason.into());
                resume_unwind(payload)
            }
        }
    }

    pub(crate) fn exclude(&mut self, section: &str, input: &str, name: &str, reason: &str) {
        assert!(!reason.trim().is_empty());
        self.pending_check(section, input, name).status = Status::Excluded(reason.into());
    }

    fn pending_check(&mut self, section: &str, input: &str, name: &str) -> &mut Check {
        let Some(check) = self
            .checks
            .iter_mut()
            .find(|check| check.section == section && check.input == input && check.name == name)
        else {
            panic!("unplanned input check: {section}/{input}/{name}");
        };
        assert_eq!(
            check.status,
            Status::NotRun,
            "input check already completed"
        );
        check
    }

    pub(crate) fn is_complete(&self) -> bool {
        self.checks.iter().any(|check| check.status == Status::Pass)
            && self
                .checks
                .iter()
                .all(|check| matches!(check.status, Status::Pass | Status::Excluded(_)))
    }

    pub(crate) fn render(&self) -> String {
        let mut output =
            String::from("\nVMNL INPUT TEST — X11\nPath: XTEST → GLFW → VMNL Event / Input\n");
        let mut section = "";
        let mut input = "";
        let (mut passed, mut failed, mut excluded, mut not_run) = (0, 0, 0, 0);
        for check in &self.checks {
            if section != check.section {
                section = &check.section;
                input = "";
                let _ = write!(output, "\n{section}:\n");
            }
            if input != check.input {
                input = &check.input;
                let _ = writeln!(output, "  {input}:");
            }
            let (status, reason) = match &check.status {
                Status::Pass => {
                    passed += 1;
                    ("PASS", None)
                }
                Status::Fail(reason) => {
                    failed += 1;
                    ("FAIL", Some(reason.as_str()))
                }
                Status::Excluded(reason) => {
                    excluded += 1;
                    ("EXCLUDED", Some(reason.as_str()))
                }
                Status::NotRun => {
                    not_run += 1;
                    ("NOT RUN", Some("scenario stopped before this check"))
                }
            };
            let _ = write!(output, "    {}: {status}", check.name);
            if let Some(reason) = reason {
                output.push_str(" — ");
                output.push_str(&reason.replace('\n', "\n      "));
            }
            output.push('\n');
        }
        let result = if failed != 0 {
            "FAIL"
        } else if !self.is_complete() {
            "INCOMPLETE"
        } else {
            "PASS"
        };
        let _ = write!(output,
            "\nSummary:\n  Checks:   {}\n  PASS:     {passed}\n  FAIL:     {failed}\n  EXCLUDED: {excluded}\n  NOT RUN:  {not_run}\n  Result:   {result}\n",
            self.checks.len(),
        );
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_checks_and_counts_exclusions_separately() {
        let mut report = InputTestReport::default();
        report.plan("Keyboard", "A", &["Press", "Release"]);
        report.plan("Keyboard", "B", &["Press"]);
        report.plan("Mouse", "Button7", &["Press"]);
        assert_eq!(
            report.run("Keyboard", "A", "Press", || Ok::<_, &str>(38)),
            Ok(38)
        );
        assert!(report
            .run("Keyboard", "A", "Release", || Ok::<_, &str>(()))
            .is_ok());
        assert!(report
            .run("Keyboard", "B", "Press", || Ok::<_, &str>(()))
            .is_ok());
        report.exclude(
            "Mouse",
            "Button7",
            "Press",
            "server button 11 exceeds limit 10",
        );
        assert!(report.is_complete());
        assert_eq!(report.render(), concat!(
            "\nVMNL INPUT TEST — X11\nPath: XTEST → GLFW → VMNL Event / Input\n",
            "\nKeyboard:\n  A:\n    Press: PASS\n    Release: PASS\n  B:\n    Press: PASS\n",
            "\nMouse:\n  Button7:\n    Press: EXCLUDED — server button 11 exceeds limit 10\n",
            "\nSummary:\n  Checks:   4\n  PASS:     3\n  FAIL:     0\n  EXCLUDED: 1\n  NOT RUN:  0\n  Result:   PASS\n",
        ));
    }

    #[test]
    fn preserves_errors_and_reports_remaining_checks_as_not_run() {
        let mut report = InputTestReport::default();
        report.plan("Keyboard", "A", &["Press", "Release"]);
        assert_eq!(
            report.run("Keyboard", "A", "Press", || Err::<(), _>("timeout")),
            Err("timeout")
        );
        let output = report.render();
        assert!(!report.is_complete());
        assert!(output.contains("Press: FAIL — timeout"));
        assert!(output.contains("Release: NOT RUN — scenario stopped before this check"));
        assert!(output.contains(
            "PASS:     0\n  FAIL:     1\n  EXCLUDED: 0\n  NOT RUN:  1\n  Result:   FAIL"
        ));
    }

    #[test]
    fn preserves_panics_and_reports_the_failed_assertion() {
        let mut report = InputTestReport::default();
        report.plan("Mouse", "Left", &["Press", "Release"]);
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            report.run("Mouse", "Left", "Press", || -> Result<(), &str> {
                panic!("wrong state")
            })
        }));
        assert!(outcome.is_err());
        let output = report.render();
        assert!(output.contains("Press: FAIL — wrong state"));
        assert!(output.contains("Release: NOT RUN"));
        assert!(output.contains("Result:   FAIL"));
    }

    #[test]
    fn incomplete_or_entirely_excluded_reports_do_not_claim_a_pass() {
        let mut report = InputTestReport::default();
        report.plan("Keyboard", "A", &["Press"]);
        assert!(!report.is_complete());
        assert!(report.render().contains("Result:   INCOMPLETE"));
        report.exclude("Keyboard", "A", "Press", "unavailable mapping");
        assert!(!report.is_complete());
        assert!(report.render().contains("Result:   INCOMPLETE"));
        assert!(!InputTestReport::default().is_complete());
        assert!(InputTestReport::default()
            .render()
            .contains("Result:   INCOMPLETE"));
    }
}
