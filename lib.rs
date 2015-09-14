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
