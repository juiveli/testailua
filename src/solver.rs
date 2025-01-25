// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

//! Process executable solver bounded context.

pub mod domain;
pub mod workflow;

pub use domain::{ExecutedFileName, PCmdLine, PEvent, PExe, PID};
pub use workflow::get_process_executed_file;
