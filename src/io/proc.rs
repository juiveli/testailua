// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

//! Utilities to read `/proc` files.

use std::{
    ffi::OsString,
    fs::File,
    io::{self, BufRead},
    os::unix::prelude::OsStringExt,
    path::PathBuf,
};

use crate::solver::{PCmdLine, PExe, PID};

/// Attempts to get the process executable name for the given `pid`.
///
/// # Errors
///
/// If this function encounters any form of I/O error, an error variant will be
/// returned.
pub fn exe_reader(pid: PID) -> io::Result<PExe> {
    match PathBuf::from(pid).join("exe").read_link()?.file_name() {
        Some(exe) => Ok(exe.to_os_string().into()),
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No executable file name",
        )),
    }
}

/// Attempts to get the process command line for the given `pid`.
///
/// # Errors
///
/// If this function encounters any form of I/O error, an error variant will be
/// returned.
pub fn cmdline_reader(pid: PID) -> io::Result<PCmdLine> {
    let cmdline = io::BufReader::new(File::open(PathBuf::from(pid).join("cmdline"))?)
        .split(b'\0')
        .filter_map(|v| match v {
            Err(e) => Some(Err(e)),
            Ok(data) => {
                if data.is_empty() {
                    None
                } else {
                    Some(Ok(OsString::from_vec(data)))
                }
            }
        })
        .collect::<io::Result<Vec<_>>>()?;

    Ok(cmdline.into())
}
