// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

//! Process executable solver domain model.

use std::{
    ffi::{OsStr, OsString},
    fmt,
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// Process ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PID(i32);

/// Monitored process events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PEvent {
    Exec(PID),
    Exit(PID),
}

/// Process executable name.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PExe(OsString);

/// Process command line.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PCmdLine(Vec<OsString>);

/// Name of the file that was executed.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct ExecutedFileName(OsString);

// --- Implementations

fn proc_path() -> &'static Path {
    static PROC_PATH: OnceLock<&Path> = OnceLock::new();
    PROC_PATH.get_or_init(|| Path::new("/proc"))
}

impl From<i32> for PID {
    fn from(value: i32) -> Self {
        PID(value)
    }
}

impl AsRef<i32> for PID {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl From<PID> for PathBuf {
    fn from(pid: PID) -> Self {
        proc_path().join(pid.0.to_string())
    }
}

impl fmt::Display for PID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Display for PEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PEvent::Exec(pid) => write!(f, "Exec({})", pid),
            PEvent::Exit(pid) => write!(f, "Exit({})", pid),
        }
    }
}

impl From<OsString> for PExe {
    fn from(value: OsString) -> Self {
        PExe(value)
    }
}

impl AsRef<OsStr> for PExe {
    fn as_ref(&self) -> &OsStr {
        &self.0
    }
}

impl From<Vec<OsString>> for PCmdLine {
    fn from(value: Vec<OsString>) -> Self {
        PCmdLine(value)
    }
}

impl AsRef<Vec<OsString>> for PCmdLine {
    fn as_ref(&self) -> &Vec<OsString> {
        &self.0
    }
}

impl fmt::Display for PCmdLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self
            .0
            .iter()
            .map(|s| s.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ");
        write!(f, "[{}]", data)
    }
}

impl From<PExe> for ExecutedFileName {
    fn from(value: PExe) -> Self {
        ExecutedFileName(value.0)
    }
}

impl fmt::Display for ExecutedFileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string_lossy())
    }
}
