// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

use anyhow::{Context, Result};
use clap::{Arg, ArgAction, ArgMatches, Command};
use copes::{
    io::{self, connector::ProcessEventsConnector},
    solver::{self, ExecutedFileName, PEvent, PID},
};
use core::fmt;
use std::{
    collections::HashMap,
    io::Write,
    sync::{self, atomic},
};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const ARG_CMDLINE_NAME: &str = "cmdline";
const ARG_NOCOLOR_NAME: &str = "nocolor";

fn main() -> Result<()> {
    simple_logger::init_with_env().context("Couldn't setup logger")?;

    let args = cmdline_args();
    let stop = sync::Arc::new(atomic::AtomicBool::new(false));

    let stop_handle = stop.clone();
    ctrlc::set_handler(move || {
        stop_handle.store(true, atomic::Ordering::Relaxed);
    })
    .context("Couldn't set Ctrl-C handler")?;

    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    let mut line_color = ColorSpec::new();

    let mut process_registry = HashMap::new();
    let data_source = create_events_source()?;
    let mut event = data_source.into_iter();
    loop {
        if let Some(event) = event.next() {
            if let Err(e) = event
                .and_then(|event| handle_event(event, &args, &mut process_registry))
                .and_then(|line| print_output_line(line, &args, &mut stdout, &mut line_color))
            {
                log::error!("{}", e);
            }
        }

        if stop.load(atomic::Ordering::Relaxed) {
            break;
        }
    }

    Ok(())
}

fn create_events_source() -> Result<ProcessEventsConnector> {
    ProcessEventsConnector::try_new()
        .map_err(|error| match &error.kind() {
            std::io::ErrorKind::PermissionDenied => {
                anyhow::Error::new(error).context("The program was started without root privileges")
            }
            _ => anyhow::Error::new(error),
        })
        .context("Couldn't create process events source")
}

fn cmdline_args() -> ArgMatches {
    Command::new(env!("CARGO_CRATE_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .arg(
            Arg::new(ARG_CMDLINE_NAME)
                .short('c')
                .action(ArgAction::SetTrue)
                .help("Print the process command line"),
        )
        .arg(
            Arg::new(ARG_NOCOLOR_NAME)
                .long("no-color")
                .action(ArgAction::SetTrue)
                .help("Do not colorize output"),
        )
        .get_matches()
}

enum OutputLine {
    Exec(String),
    Exit(String),
}

impl fmt::Display for OutputLine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputLine::Exec(line) | OutputLine::Exit(line) => write!(f, "{}", line),
        }
    }
}

fn handle_event(
    event: PEvent,
    args: &ArgMatches,
    process_registry: &mut HashMap<PID, ExecutedFileName>,
) -> std::io::Result<Option<OutputLine>> {
    let output_line = match event {
        PEvent::Exec(pid) => handle_exec_event(pid, args, process_registry),
        PEvent::Exit(pid) => handle_exit_event(pid, process_registry),
    }?
    .map(|event_line| {
        let line = format!("{} {}", event, event_line);
        match event {
            PEvent::Exec(_) => OutputLine::Exec(line),
            PEvent::Exit(_) => OutputLine::Exit(line),
        }
    });

    Ok(output_line)
}

fn handle_exec_event(
    pid: PID,
    args: &ArgMatches,
    process_registry: &mut HashMap<PID, ExecutedFileName>,
) -> std::io::Result<Option<String>> {
    let mut line_elements = Vec::new();

    let cmdline = io::proc::cmdline_reader(pid)?;
    let exe = solver::get_process_executed_file(io::proc::exe_reader(pid)?, &cmdline);

    line_elements.push(exe.to_string());
    process_registry.insert(pid, exe);

    if args.get_flag(ARG_CMDLINE_NAME) {
        line_elements.push(cmdline.to_string());
    }

    Ok(Some(line_elements.join(" ")))
}

fn handle_exit_event(
    pid: PID,
    process_registry: &mut HashMap<PID, ExecutedFileName>,
) -> std::io::Result<Option<String>> {
    Ok(process_registry.remove(&pid).map(|exe| exe.to_string()))
}

fn print_output_line(
    line: Option<OutputLine>,
    args: &ArgMatches,
    stdout: &mut StandardStream,
    line_color: &mut ColorSpec,
) -> std::io::Result<()> {
    match line {
        Some(line) => {
            if !args.get_flag(ARG_NOCOLOR_NAME) {
                if let Err(e) = match line {
                    OutputLine::Exec(_) => stdout.reset(),
                    OutputLine::Exit(_) => stdout.set_color(line_color.set_fg(Some(Color::Red))),
                } {
                    log::error!("Couldn't setup output color: {}", e);
                }
            }

            writeln!(stdout, "{}", line)
        }
        None => Ok(()),
    }
}
