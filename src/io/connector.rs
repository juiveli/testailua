// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2022 Juan Palacios <jpalaciosdev@gmail.com>

//! Utilities to monitor process events using the kernel [process events
//! connector] interface.
//!
//! [process events connector]: https://github.com/torvalds/linux/commit/9f46080c41d5f3f7c00b4e169ba4b0b2865258bf

use std::{
    io,
    mem::{self, offset_of},
    time::Duration,
};

use crate::{
    io::socket::Socket,
    solver::{PEvent, PID},
};

#[macro_use]
mod cnproc;

/// A connector to monitor process events.
pub struct ProcessEventsConnector(Socket);

impl ProcessEventsConnector {
    /// Attempts to create a new `ProcessEventsConnector` instance.
    ///
    /// # Errors
    ///
    /// If this function encounters any form of I/O error, an error variant will
    /// be returned.
    pub fn try_new() -> Result<Self, io::Error> {
        let socket = Socket::try_new(
            libc::PF_NETLINK,
            libc::SOCK_DGRAM,
            cnproc::NETLINK_CONNECTOR as libc::c_int,
        )?;

        let listener = ProcessEventsConnector(socket)
            .timeout(Duration::from_secs(3))?
            .install_filter()?
            .bind()?;
        listener.subscribe_to_proc_events(true)?;

        Ok(listener)
    }

    /// Setups the socket bindings.
    fn bind(self) -> Result<ProcessEventsConnector, io::Error> {
        // Safety: `libc::sockaddr_nl` is a C structure, so it's safe to
        // initialize it with zeros.
        let mut address = unsafe { mem::zeroed::<libc::sockaddr_nl>() };
        address.nl_pid = 0; // let the kernel handle its value
        address.nl_family = libc::AF_NETLINK as _;
        address.nl_groups = cnproc::CN_IDX_PROC;

        // Safety: Calling `Socket::bind` ffi method with a pointer to address
        // is safe at this point, now that the structure has been allocated and
        // properly initialized.
        unsafe {
            self.0.bind(
                &address as *const _ as *const _,
                mem::size_of_val(&address) as _,
            )?
        };

        Ok(self)
    }

    /// Setups the socket data receiving timeout.
    fn timeout(self, duration: Duration) -> Result<ProcessEventsConnector, io::Error> {
        let duration = libc::timeval {
            tv_sec: duration.as_secs().clamp(0, i64::MAX as u64) as i64,
            tv_usec: 0,
        };

        // Safety: Calling `Socket::set_option` ffi method with a pointer to
        // duration is safe at this point, now that the structure has been
        // allocated properly initialized.
        unsafe {
            self.0.set_option(
                libc::SOL_SOCKET,
                libc::SO_RCVTIMEO,
                &duration as *const _ as *const _,
                mem::size_of_val(&duration) as _,
            )?
        };

        Ok(self)
    }

    /// Setups the socket filter.
    fn install_filter(self) -> Result<ProcessEventsConnector, io::Error> {
        use cnproc::*;
        use libc::{c_uint, c_ushort};

        type ExecProcEvent = proc_event__bindgen_ty_1_exec_proc_event;
        type ExitProcEvent = proc_event__bindgen_ty_1_exit_proc_event;

        #[rustfmt::skip]
        let mut filter = [
            // Check message from kernel.
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, offset_of!(nlmsghdr, nlmsg_pid)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_K, 0, 1, 0),
            bpf_stmt!(BPF_RET | BPF_K, 0x0),

            // Check message type NLMSG_DONE.
            bpf_stmt!(BPF_LD | BPF_H | BPF_ABS, offset_of!(nlmsghdr, nlmsg_type)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_K, c_ushort::to_be(NLMSG_DONE as c_ushort), 1, 0),
            bpf_stmt!(BPF_RET | BPF_K, 0x0),

            // Check proc connector event CN_IDX_PROC.
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, id) +
                                                offset_of!(cb_id, idx)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_K, c_uint::to_be(CN_IDX_PROC), 1, 0),
            bpf_stmt!(BPF_RET | BPF_K, 0x0),

            // Check proc connector event CN_VAL_PROC.
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, id) +
                                                offset_of!(cb_id, val)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_K, c_uint::to_be(CN_VAL_PROC), 1, 0),
            bpf_stmt!(BPF_RET | BPF_K, 0x0),

            // Accept exec messages from processes.
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, data) +
                                                offset_of!(proc_event, what)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_K, c_uint::to_be(PROCESS_EVENT_EXEC), 0, 6),

            // Processes have process_pid == process_tgid (thread group leaders).
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, data) +
                                                offset_of!(proc_event, event_data) +
                                                offset_of!(ExecProcEvent, process_pid)),
            bpf_stmt!(BPF_ST, 0),
            bpf_stmt!(BPF_LDX | BPF_W | BPF_MEM, 0),
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, data) +
                                                offset_of!(proc_event, event_data) +
                                                offset_of!(ExecProcEvent, process_tgid)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_X, 0, 0, 9),
            bpf_stmt!(BPF_RET | BPF_K, 0xffffffff),

            // Accept exit messages from processes.
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, data) +
                                                offset_of!(proc_event, what)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_K, c_uint::to_be(PROCESS_EVENT_EXIT), 0, 6),

            // Processes have process_pid == process_tgid
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, data) +
                                                offset_of!(proc_event, event_data) +
                                                offset_of!(ExitProcEvent, process_pid)),
            bpf_stmt!(BPF_ST, 0),
            bpf_stmt!(BPF_LDX | BPF_W | BPF_MEM, 0),
            bpf_stmt!(BPF_LD | BPF_W | BPF_ABS, nlmsg_length(0) +
                                                offset_of!(cn_msg, data) +
                                                offset_of!(proc_event, event_data) +
                                                offset_of!(ExitProcEvent, process_tgid)),
            bpf_jump!(BPF_JMP | BPF_JEQ | BPF_X, 0, 0, 1),
            bpf_stmt!(BPF_RET | BPF_K, 0xffffffff),

            // Drop any other messages.
            bpf_stmt!(BPF_RET | BPF_K, 0x0),
        ];
        // Safety: `cnproc::sock_fprog` is a C structure, so it's safe to
        // initialize it with zeros.
        let mut fprog = unsafe { mem::zeroed::<sock_fprog>() };
        fprog.filter = filter.as_mut_ptr();
        fprog.len = filter.len() as _;

        // Safety: Calling `Socket::set_option` ffi method with a pointer to
        // fprog is safe at this point, now that the structure and all the
        // related data have been allocated and properly initialized.
        unsafe {
            self.0.set_option(
                libc::SOL_SOCKET,
                libc::SO_ATTACH_FILTER,
                &fprog as *const _ as *const _,
                mem::size_of_val(&fprog) as _,
            )?
        };

        Ok(self)
    }

    /// Subscribe and unsubscribe to proc events.
    fn subscribe_to_proc_events(&self, subscribe: bool) -> io::Result<()> {
        use cnproc::*;

        const MSG_SIZE: usize =
            nlmsg_length(mem::size_of::<cn_msg>() + mem::size_of::<proc_cn_mcast_op>());
        let mut msg_buffer = [0u8; MSG_SIZE];
        let msg = msg_buffer.as_mut_ptr() as *mut nlcn_msg<cn_msg>;

        // Safety: Dereferencing msg is safe in this context as it doesn't
        // outlives msg_buffer and, being the later a fixed size array, there is
        // no re-allocations that could invalidate the pointer.
        unsafe {
            (*msg).nl_hdr.0.nlmsg_pid = 0; // the kernel is handling its value
            (*msg).nl_hdr.0.nlmsg_type = libc::NLMSG_DONE as _;
            (*msg).nl_hdr.0.nlmsg_len = MSG_SIZE as _;

            (*msg).cn_msg.id.idx = cnproc::CN_IDX_PROC;
            (*msg).cn_msg.id.val = cnproc::CN_VAL_PROC;
            (*msg).cn_msg.len = mem::size_of::<proc_cn_mcast_op>() as _;

            // Safety: Dereferencing (*msg).cn_msg.data as a proc_cn_mcast_op
            // type is safe in this context as it points to memory allocated on
            // the msg_buffer.
            let data = (*msg).cn_msg.data.as_mut_ptr() as *mut proc_cn_mcast_op;
            *data = if subscribe {
                proc_cn_mcast_op_PROC_CN_MCAST_LISTEN
            } else {
                proc_cn_mcast_op_PROC_CN_MCAST_IGNORE
            };
        }

        // Safety: Calling `Socket::send` ffi method with a pointer to msg is
        // safe at this point as the needed data has been allocated and properly
        // initialized.
        unsafe { self.0.send(msg as *const _, MSG_SIZE, 0)? };

        Ok(())
    }
}

impl Drop for ProcessEventsConnector {
    fn drop(&mut self) {
        if let Err(e) = self.subscribe_to_proc_events(false) {
            log::error!("An error occur while unsubscribing from proc events: {}", e);
        }
    }
}

pub struct Iter<'a>(&'a ProcessEventsConnector);

impl<'a> Iterator for Iter<'a> {
    type Item = io::Result<PEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        use cnproc::*;

        const MSG_SIZE: usize =
            nlmsg_length(mem::size_of::<cn_msg>() + mem::size_of::<proc_event>());
        let mut msg_buffer = [0u8; MSG_SIZE];

        // Safety: Calling `Socket::receive` ffi method with a pointer to
        // msg_buffer is safe at this point as the buffer has enough memory to
        // hold the message.
        if let Err(error) = unsafe {
            self.0
                 .0
                .receive(msg_buffer.as_mut_ptr() as *mut _, MSG_SIZE, 0)
        } {
            let result = match error.kind() {
                io::ErrorKind::WouldBlock => None,
                _ => Some(Err(error)),
            };
            return result;
        }

        let msg = msg_buffer.as_ptr() as *const nlcn_msg<cn_msg>;

        // Safety: Dereferencing msg is safe in this context as it doesn't
        // outlives msg_buffer and, being the later a fixed size array, there is
        // no re-allocations that could invalidate the pointer. Dereferencing
        // (*msg).cn_msg.data as a proc_cn_mcast_op type is also safe as it
        // points to memory allocated on the msg_buffer.
        unsafe {
            let event = (*msg).cn_msg.data.as_ptr() as *const proc_event;

            match (*event).what {
                PROCESS_EVENT_EXEC => Some(Ok(PEvent::Exec(PID::from(
                    (*event).event_data.exec.process_pid,
                )))),
                PROCESS_EVENT_EXIT => Some(Ok(PEvent::Exit(PID::from(
                    (*event).event_data.exit.process_pid,
                )))),
                _ => None,
            }
        }
    }
}

impl<'a> IntoIterator for &'a ProcessEventsConnector {
    type Item = <Iter<'a> as Iterator>::Item;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}
