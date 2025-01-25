// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

//! This module contains a simple `Socket` implementation built around
//! [`libc::socket`]. `Socket` methods are just direct calls to their ffi
//! counterparts.
//!
//! [`libc::socket`]: https://docs.rs/libc/latest/libc/fn.socket.html

/// Helper macro to call a libc ffi function.
/// The resulting value is returned into a [std::io::Result].
macro_rules! ffi_call {
    ($name: ident( $( $arg: expr ),* )) => {{
        let result = libc::$name($($arg, )*);
        match result {
            -1 => Err(std::io::Error::last_os_error()),
            _ =>  Ok(result),
        }
    }};

    // handle leading comma
    ($name: ident( $( $arg: expr ),+ ,)) => {
        ffi_fn!( $name( $( $arg ),+ ) )
    };
}

pub struct Socket(libc::c_int);

impl Socket {
    /// Attempts to create a new `Socket` in a `domain`, with type `ty` using a
    /// specific `protocol`.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O error, an error variant will be
    /// returned.
    pub fn try_new(
        domain: libc::c_int,
        ty: libc::c_int,
        protocol: libc::c_int,
    ) -> std::io::Result<Socket> {
        // Safety: It's safe to call the ffi function in this context as it
        // won't produce undefined behaviour on the Rust side upon a failure.
        unsafe { ffi_call!(socket(domain, ty, protocol)).map(Socket) }
    }

    /// Assign an `address` with of a specific `length` to the `Socket`.
    ///
    /// Calls ffi `bind` on the `Socket` with `address` and `length` as
    /// arguments.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory pointed by `address` has been
    /// allocated and properly initialized and its size is passed as `length`.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O error, an error variant will be
    /// returned.
    pub unsafe fn bind(
        &self,
        address: *const libc::sockaddr,
        length: libc::socklen_t,
    ) -> std::io::Result<()> {
        ffi_call!(bind(self.0, address, length)).map(|_| ())
    }

    /// Manipulates the options of the `Socket`.
    ///
    /// Calls ffi `setsockopt` on the `Socket` with `level`, `name`, `value` and
    /// `length` as arguments.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory pointed by `value` has been
    /// allocated and properly initialized and its size is passed as `length`.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O error, an error variant will be
    /// returned.
    pub unsafe fn set_option(
        &self,
        level: libc::c_int,
        name: libc::c_int,
        value: *const libc::c_void,
        length: libc::socklen_t,
    ) -> std::io::Result<()> {
        ffi_call!(setsockopt(self.0, level, name, value, length)).map(|_| ())
    }

    /// Transmit a message through the `Socket`.
    ///
    /// Calls ffi `send` on the `Socket` with `data`, `length` and `flags` as
    /// arguments.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory pointed by `data` has been
    /// allocated and properly initialized and its size is passed as `length`.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O error, an error variant will be
    /// returned.
    pub unsafe fn send(
        &self,
        data: *const libc::c_void,
        length: libc::size_t,
        flags: libc::c_int,
    ) -> std::io::Result<()> {
        ffi_call!(send(self.0, data, length, flags)).map(|_| ())
    }

    /// Receive a message from the `Socket`.
    ///
    /// Calls ffi `recv` on the `Socket` with `buffer`, `length` and `flags` as
    /// arguments.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the memory pointed by `buffer` has been
    /// allocated and its size is passed as `length`.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O error, an error variant will be
    /// returned.
    pub unsafe fn receive(
        &self,
        buffer: *mut libc::c_void,
        length: libc::size_t,
        flags: libc::c_int,
    ) -> std::io::Result<()> {
        ffi_call!(recv(self.0, buffer, length, flags)).map(|_| ())
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        // Safety: A Socket instance always have a valid open file descriptor.
        if let Err(e) = unsafe { ffi_call!(close(self.0)) } {
            log::error!("An error occur while closing the socket: {}", e);
        }
    }
}
