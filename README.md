# CoreCtrl Process Executable Solver
`copes` is a command line utility to identity the executable files that must be used in [automatic CoreCtrl profiles](https://gitlab.com/corectrl/corectrl/-/wikis/How-profiles-works).

## Compilation
To compile the application, the following software must be installed in your system:
- A [Rust](https://www.rust-lang.org/) compiler and associated tools (Rust 1.77 or later). You can easily install them using [rustup.rs](https://rustup.rs/).
- `clang` compiler and a linker (`ld`, `lld`... your choice).

To build the program, run `cargo build -r` on the project directory. The program executable will be placed in the `target/release` directory.

## Runtime dependencies
Under the hood, this program uses the [process events connector kernel interface](https://github.com/torvalds/linux/commit/9f46080c41d5f3f7c00b4e169ba4b0b2865258bf). Therefore, a Linux kernel compiled with `CONFIG_PROC_EVENTS` option enabled is required.

## Usage
If you are using Linux 6.5 or earlier versions, you must run this program with root privileges. Otherwise, you can skip the `sudo` part on the following commands if you only want to monitor non-privileged processes.

    sudo target/release/copes

By default, this utility shows the process event (either `Exec` or `Exit`), the process `PID` and the executable file for which the process was started. Executable file names are resolved in a similar way CoreCtrl does.

Use the `c` option to show the process command line. This option can be useful to see how the process was started.

    sudo target/release/copes -c

Press `Control + c` to quit the program.

To get a list with all the available options, run `target/release/copes -h`.

## Finding the right executable file for an automatic CoreCtrl profile
Suppose that you have created an automatic profile, but for some reason, it's not activated when you start the program for which you created the profile.

The following list outlines the common causes of this issue:
- The program only runs for a short period of time. This behaviour is very common on games, where a launcher is used to starts the game. For example, `game.exe` is a launcher that starts a process for `game_dx11.exe` and then exits.
- The program executable name differs from the name you used as the executable name on the profile. Be aware that file names are **case sensitive**.
- The program is written using an interpreted language (like bash script or python), which currently is not handled by CoreCtrl.

Another cause could be the kernel you are running was compiled with the `CONFIG_PROC_EVENTS` option disabled. If you run this utility and see no output after launching some programs, then it's probable that this is the cause of the issue. You can verify it by searching for such option on the kernel configuration file. Please, refer to your distribution documentation to locate such file on your file system.

You can check any of the aforementioned causes by starting this utility **before** launching the application associated to the profile.

### Example
Running `sudo target/release/copes` produces the following output when launching the popular game *Control*:

    ...
    Exec(24138) tabtip.exe
    Exec(24141) Control.exe
    Exec(24147) Control_DX12.exe
    Exit(24141) Control.exe

At this point the game is running. Notice that `Control.exe` is just a launcher whose only purpose is to start the true game executable (`Control_DX12.exe`) and that its process (`24141`) exits just after starting the other one (`24147`).

When the game is quit:

    ...
    Exit(24147) Control_DX12.exe
    Exit(24138) tabtip.exe
    Exit(24133) explorer.exe
    Exit(24062) winedevice.exe
    Exit(24071) winedevice.exe
    Exit(24084) plugplay.exe
    Exit(24090) svchost.exe
    Exit(24107) rpcss.exe
    Exit(24053) steam.exe
    ...

Notice how the game process (`24147`), started for the executable `Control_DX12.exe` exits at this point.

In this case, `Control_DX12.exe` must be used as the executable name on the CoreCtrl profile. Otherwise, the profile won't be active while playing the game.
