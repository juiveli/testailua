// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

//! Process executable solver workflows.

use std::{
    ffi::{OsStr, OsString},
    path::Path,
    sync::OnceLock,
};

use super::{ExecutedFileName, PCmdLine, PExe};

/// Returns the file name of the executable that started a process.
pub fn get_process_executed_file(pexe: PExe, cmdline: &PCmdLine) -> ExecutedFileName {
    if wine_executables().contains(&pexe.as_ref()) {
        if let Some(name) = wine_executed_file_name(cmdline.as_ref()) {
            return PExe::from(name).into();
        }
    }

    pexe.into()
}

fn wine_executables() -> &'static Vec<&'static OsStr> {
    static WINE_EXECUTABLES: OnceLock<Vec<&'static OsStr>> = OnceLock::new();
    WINE_EXECUTABLES.get_or_init(|| {
        vec![
            OsStr::new("wine-preloader"),
            OsStr::new("wine64-preloader"),
            OsStr::new("wine"),
            OsStr::new("wine64"),
            OsStr::new("wineloader"),
            OsStr::new("wineloader64"),
        ]
    })
}

fn wine_executed_file_name(cmdline: &[OsString]) -> Option<OsString> {
    cmdline
        .iter()
        .skip_while(|cmd| is_wine_executable(cmd))
        .take(1)
        .flat_map(|cmd| get_wine_exe_from_path(cmd))
        .last()
}

fn is_wine_executable(cmd: &OsStr) -> bool {
    let path = Path::new(cmd);
    let file_name = path.file_name();

    path.is_absolute()
        && file_name.is_some()
        && wine_executables().contains(file_name.as_ref().unwrap())
}

fn get_wine_exe_from_path(cmd: &OsStr) -> Option<OsString> {
    // Try to get the last path component, which should be the app .exe
    match cmd
        .to_string_lossy()
        .split(|c| c == '\\' || c == '/')
        .last()
    {
        Some(app_name) => {
            // Look for the .exe extension
            match app_name.split('.').last() {
                Some(extension) if extension.eq_ignore_ascii_case("exe") => Some(app_name.into()),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_wine_executable_returns_true_for_wine_executables_on_absolute_paths() {
        assert!(is_wine_executable(OsStr::new("/some/path/wine")));
        assert!(is_wine_executable(OsStr::new("/some/path/wine64")));
        assert!(is_wine_executable(OsStr::new("/some/path/wine-preloader")));
        assert!(is_wine_executable(OsStr::new(
            "/some/path/wine64-preloader"
        )));
        assert!(is_wine_executable(OsStr::new("/some/path/wineloader")));
        assert!(is_wine_executable(OsStr::new("/some/path/wineloader64")));
    }

    #[test]
    fn is_wine_executable_returns_false_for_wine_executables_on_relative_paths() {
        assert!(!is_wine_executable(OsStr::new("wineloader64")));
    }

    #[test]
    fn is_wine_executable_returns_false_for_non_wine_executables() {
        assert!(!is_wine_executable(OsStr::new("/some/path/executable")));
        assert!(!is_wine_executable(OsStr::new("executable")));
    }

    #[test]
    fn get_wine_exe_from_path_returns_none_from_non_valid_wine_app_paths() {
        let wine_cmd = OsStr::new("\\");
        assert_eq!(None, get_wine_exe_from_path(wine_cmd));

        let wine_cmd = OsStr::new("C:\\");
        assert_eq!(None, get_wine_exe_from_path(wine_cmd));

        let wine_cmd = OsStr::new("C:\\no_extension");
        assert_eq!(None, get_wine_exe_from_path(wine_cmd));

        let wine_cmd = OsStr::new("C:\\no_exe_extension.txt");
        assert_eq!(None, get_wine_exe_from_path(wine_cmd));
    }

    #[test]
    fn get_wine_exe_from_path_returns_exe_from_valid_wine_app_paths() {
        // back slash
        let wine_cmd = OsStr::new("C:\\Program Files (x86)\\App\\App.exe");
        assert_eq!(Some("App.exe".into()), get_wine_exe_from_path(wine_cmd));

        // slash
        let wine_cmd = OsStr::new("C:/Program Files (x86)/App/App.exe");
        assert_eq!(Some("App.exe".into()), get_wine_exe_from_path(wine_cmd));

        // slash + back slash
        let wine_cmd = OsStr::new("C:\\Program Files (x86)\\App/Binaries/App.exe");
        assert_eq!(Some("App.exe".into()), get_wine_exe_from_path(wine_cmd));

        // unix path
        let wine_cmd = OsStr::new("/Program Files (x86)/App/Binaries/App.exe");
        assert_eq!(Some("App.exe".into()), get_wine_exe_from_path(wine_cmd));
    }

    #[test]
    fn wine_executed_file_name_returns_none_from_non_wine_launch_cmdline() {
        let cmdline = vec![OsString::from("/usr/bin/cat")];
        assert_eq!(None, wine_executed_file_name(&cmdline));
    }

    #[test]
    fn wine_executed_file_name_returns_executed_windows_exe_from_wine_launch_cmdline() {
        // traditional launch
        let cmdline = vec![
            OsString::from("/usr/bin/wine-preloader"),
            OsString::from("/usr/bin/wine"),
            OsString::from("C:\\Program Files (x86)\\App\\App.exe"),
        ];
        assert_eq!(Some("App.exe".into()), wine_executed_file_name(&cmdline));

        // without preloader
        let cmdline = vec![
            OsString::from("/usr/bin/wine"),
            OsString::from("C:\\Program Files (x86)\\App\\App.exe"),
        ];
        assert_eq!(Some("App.exe".into()), wine_executed_file_name(&cmdline));

        // wine64 launch
        let cmdline = vec![
            OsString::from("/usr/bin/wine64-preloader"),
            OsString::from("/usr/bin/wine64"),
            OsString::from("C:\\Program Files (x86)\\App\\App.exe"),
        ];
        assert_eq!(Some("App.exe".into()), wine_executed_file_name(&cmdline));

        // unix path launch app argument
        let cmdline = vec![
            OsString::from("/usr/bin/wine-preloader"),
            OsString::from("/usr/bin/wine"),
            OsString::from("/Program Files (x86)/App/Binaries/App.exe"),
        ];
        assert_eq!(Some("App.exe".into()), wine_executed_file_name(&cmdline));
    }

    #[test]
    fn get_process_executed_file_returns_the_process_executable_name_for_regular_processes() {
        let exe = PExe::from(OsString::from("cat"));
        let cmdline = PCmdLine::from(vec![
            OsString::from("/usr/bin/cat"),
            OsString::from("test.log"),
        ]);
        assert_eq!(
            ExecutedFileName::from(exe.clone()),
            get_process_executed_file(exe, &cmdline)
        );
    }

    #[test]
    fn get_process_executed_file_returns_executed_windows_exe_for_wine_processes_with_preloader() {
        let exe = PExe::from(OsString::from("wine-preloader"));
        let cmdline = PCmdLine::from(vec![
            OsString::from("/usr/bin/wine-preloader"),
            OsString::from("/usr/bin/wine"),
            OsString::from("C:\\Program Files (x86)\\App\\App.exe"),
        ]);
        assert_eq!(
            ExecutedFileName::from(PExe::from(OsString::from("App.exe"))),
            get_process_executed_file(exe, &cmdline)
        );
    }

    #[test]
    fn get_process_executed_file_returns_executed_windows_exe_for_wine_processes_without_preloader()
    {
        let exe = PExe::from(OsString::from("wine"));
        let cmdline = PCmdLine::from(vec![
            OsString::from("/usr/lib/wine/wine64"),
            OsString::from("C:\\Program Files (x86)\\App\\App.exe"),
        ]);
        assert_eq!(
            ExecutedFileName::from(PExe::from(OsString::from("App.exe"))),
            get_process_executed_file(exe, &cmdline)
        );
    }
}
