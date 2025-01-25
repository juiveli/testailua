// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

//! Process events connector kernel bindings and utilities.

#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    unused,
    deref_nullptr
)]

include!(concat!(env!("OUT_DIR"), "/cnproc_bindings.rs"));

#[repr(C, align(4))] // NLMSG_ALIGNTO == 4
pub struct nlmsghdr_aligned<T>(pub T);

#[repr(C)]
pub struct nlcn_msg<T> {
    pub nl_hdr: nlmsghdr_aligned<libc::nlmsghdr>,
    pub cn_msg: T,
}

/// Creates a [sock_filter] jump rule.
///
/// [sock_filter]: https://www.kernel.org/doc/Documentation/networking/filter.txt
macro_rules! bpf_jump {
    ($code: expr, $k: expr, $jt: expr, $jf: expr) => {
        sock_filter {
            code: $code as libc::c_ushort,
            jt: $jt,
            jf: $jf,
            k: $k as libc::c_uint,
        }
    };
}

/// Creates a [sock_filter] statement rule.
///
/// [sock_filter]: https://www.kernel.org/doc/Documentation/networking/filter.txt
macro_rules! bpf_stmt {
    ($code: expr, $k: expr) => {
        sock_filter {
            code: $code as libc::c_ushort,
            jt: 0,
            jf: 0,
            k: $k as libc::c_uint,
        }
    };
}

#[inline]
const fn nlmsg_align(len: usize) -> usize {
    (len + NLMSG_ALIGNTO as usize - 1) & !(NLMSG_ALIGNTO as usize - 1)
}

#[inline]
const fn nlmsg_hdrlen() -> usize {
    nlmsg_align(std::mem::size_of::<nlmsghdr>())
}

#[inline]
/// Computes a netlink message length taking into account its header
/// size.
pub const fn nlmsg_length(len: usize) -> usize {
    len + nlmsg_hdrlen()
}

#[cfg(test)]
mod tests {
    use std::mem::offset_of;

    use super::*;

    // for testing convenience
    impl PartialEq for sock_filter {
        fn eq(&self, other: &Self) -> bool {
            self.code == other.code
                && self.jt == other.jt
                && self.jf == other.jf
                && self.k == other.k
        }
    }

    #[test]
    fn bpf_jump_expansion() {
        assert_eq!(
            sock_filter {
                code: 1 | 2,
                jt: 3,
                jf: 4,
                k: 5
            },
            bpf_jump!(1 | 2, 5, 3, 4)
        );
    }

    #[test]
    fn bpf_stmt_expansion() {
        assert_eq!(
            sock_filter {
                code: 1 | 2,
                jt: 0,
                jf: 0,
                k: 3
            },
            bpf_stmt!(1 | 2, 3)
        );
    }

    #[test]
    fn nlmsg_length_value() {
        assert_eq!(16, nlmsg_length(0));
        assert_eq!(24, nlmsg_length(8));
    }

    #[test]
    fn filter_array_macros() {
        let data = [
            sock_filter {
                code: (BPF_JMP | BPF_JEQ | BPF_K) as libc::c_ushort,
                jt: 0,
                jf: 0,
                k: offset_of!(nlmsghdr, nlmsg_pid) as libc::c_uint,
            },
            sock_filter {
                code: (BPF_JMP | BPF_JEQ | BPF_K) as libc::c_ushort,
                jt: 1,
                jf: 0,
                k: c_uint::to_be(CN_IDX_PROC),
            },
        ];

        // Common functions used on filters:
        //  offsetof -> offset_of! macro
        //  htons -> c_ushort::to_be
        //  htonl -> c_uint::to_be

        let filter = [
            bpf_stmt!(BPF_JMP | BPF_JEQ | BPF_K, offset_of!(nlmsghdr, nlmsg_pid)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_K, c_uint::to_be(CN_IDX_PROC), 1, 0),
        ];
        assert_eq!(data, filter);
    }
}
