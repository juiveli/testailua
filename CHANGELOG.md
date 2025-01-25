# Changelog
All notable changes to this project will be documented in this file.

## Unreleased

### Changed
- Require Rust 1.77.
- Replace memoffset crate with standard offset_off! macro.
- Replace lazy_static crate with standard OnceLock.


## copes 1.0.5 (2024-03-08)

### Removed
- Workaround for application not working with Linux 6.7 and Linux 6.6.13. The change that triggers this regression was reverted in Linux 6.7.6 and 6.6.18.


## copes 1.0.4 (2024-01-25)

### Added
- Workaround for application not working with Linux 6.7 and Linux 6.6.13.


## copes 1.0.3 (2023-11-04)

### Fixed
- Compilation with Linux 6.6 API headers.


## copes 1.0.2 (2023-09-19)

### Fixed
- Compilation error with latest kernels.


## copes 1.0.1 (2022-11-07)

### Fixed
- Missing log messages. Print all messages to `stdout`. Use `RUST_LOG` variable to configure the logger output.

### Changed
- Report user friendly error messages.


## copes 1.0.0 (2022-11-04)

### Added
- Process executable solver. Displays process events along with their PIDs and the executable files for which the processes were started. Executable files run through wine are also supported.
- Additional options:
  - `-c` will also show the process command line. Useful to see the how the process was started.
  - `--no-color` to disable output colorization.
