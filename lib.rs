/* Copyright (c) 2015 The Robigalia Project Developers
 * Licensed under the Apache License, Version 2.0
 * <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT
 * license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
 * at your option. All files in the project carrying such
 * notice may not be copied, modified, or distributed except
 * according to those terms.
 */
#![no_std]
#![feature(asm, no_std)]
#![allow(bad_style, unused_parens)]

#[cfg(all(target_arch = "x86", target_pointer_width = "32"))]
include!("arch/x86.rs");

#[cfg(all(target_arch = "arm", target_pointer_width = "32"))]
include!("arch/arm.rs");

#[cfg(all(target_arch = "x86", target_pointer_width = "32"))]
include!(concat!(env!("OUT_DIR"), "/x86_invocation.rs"));

#[cfg(all(target_arch = "arm", target_pointer_width = "32"))]
include!(concat!(env!("OUT_DIR"), "/arm_invocation.rs"));

#[cfg(all(target_arch = "x86", target_pointer_width = "32"))]
include!(concat!(env!("OUT_DIR"), "/x86_syscall_stub.rs"));

#[cfg(all(target_arch = "arm", target_pointer_width = "32"))]
include!(concat!(env!("OUT_DIR"), "/arm_syscall_stub.rs"));

include!(concat!(env!("OUT_DIR"), "/types.rs"));
include!(concat!(env!("OUT_DIR"), "/syscalls.rs"));
