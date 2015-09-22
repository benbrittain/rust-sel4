/* Copyright (c) 2015 The Robigalia Project Developers
 * Licensed under the Apache License, Version 2.0
 * <LICENSE-APACHE or
 * http://www.apache.org/licenses/LICENSE-2.0> or the MIT
 * license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
 * at your option. All files in the project carrying such
 * notice may not be copied, modified, or distributed except
 * according to those terms.
 */

use std::fs::File;
use std::process::{Command, Stdio};
use std::os::unix::prelude::*;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let arches = ["x86", "arm"];
    for arch in &arches {
        assert!(Command::new("/usr/bin/env")
            .arg("python")
            .args(&["tools/syscall_stub_gen.py", "-a", arch,
                  "seL4/libsel4/include/interfaces/sel4.xml",
                  &*format!("seL4/libsel4/arch_include/{}/interfaces/sel4arch.xml", arch),
                  "-o", &*format!("{}/{}_syscall_stub.rs", out_dir, arch)])
            .status().unwrap().success());
    }

    for arch in &arches {
        assert!(Command::new("/usr/bin/env")
            .arg("python")
            .args(&["tools/invocation_header_gen.py",
                  "--dest", &*format!("{}/{}_invocation.rs", out_dir, arch),
                  "seL4/libsel4/include/interfaces/sel4.xml",
                  &*format!("seL4/libsel4/arch_include/{}/interfaces/sel4arch.xml", arch)])
            .status().unwrap().success());
    }

    assert!(Command::new("/usr/bin/env")
        .arg("python")
        .args(&["tools/syscall_header_gen.py",
              "--xml", "seL4/include/api/syscall.xml",
              "--dest", &*format!("{}/syscalls.rs", out_dir)])
        .status().unwrap().success());

    let bfin = File::open("seL4/libsel4/include/sel4/types.bf").unwrap();
    let bfout = File::create(&*format!("{}/types.rs", out_dir)).unwrap();
    assert!(Command::new("/usr/bin/env")
        .arg("python")
        .arg("tools/bitfield_gen.py")
        .stdin(unsafe { Stdio::from_raw_fd(bfin.as_raw_fd()) })
        .stdout(unsafe { Stdio::from_raw_fd(bfout.as_raw_fd()) })
        .status().unwrap().success());
    std::mem::forget(bfin);
    std::mem::forget(bfout);
}
