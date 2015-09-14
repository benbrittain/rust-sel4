/*
 * Copyright 2015, Corey Richardson
 * Copyright 2014, NICTA
 *
 * This software may be distributed and modified according to the terms of
 * the BSD 2-Clause license. Note that NO WARRANTY is provided.
 * See "LICENSE_BSD2.txt" for details.
 *
 * @TAG(NICTA_BSD)
 */

#[inline(always)]
pub unsafe fn seL4_GetTag() -> seL4_MessageInfo {
    let tag;
    asm!("movl %gs:0, $0" : "=r"(tag) : : : "volatile");
    tag
}

#[inline(always)]
pub unsafe fn seL4_SetTag(tag: seL4_MessageInfo) {
    asm!("movl $0, %gs:0" : : "r"(tag) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetMR(regnum: isize) -> seL4_Word {
    let mr;
    asm!("movl %gs:4(,$1,0x4), $0" : "=r"(mr) : "r"(regnum) : : "volatile");
    mr
}

#[inline(always)]
pub unsafe fn seL4_SetMR(regnum: i, value: seL4_Word) {
    asm!("movl $0, %gs:4(,$1,0x4)" : : "r"(value), "r"(regnum) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetUserData() -> seL4_Word {
    let data;
    asm!("movl %gs:484, $0" : "=r"(data) : : : "volatile");
    data
}

#[inline(always)]
pub unsafe fn seL4_SetUserData(data: seL4_Word) {
    asm!("movl $0, %gs:484" : : "r"(data) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetBadge(index: isize) -> seL4_CapData {
    let badge;
    asm!("movl %gs:488(,$1,0x4), $0" : "=r"(badge) : "r"(index) : : "volatile");
    badge
}

#[inline(always)]
pub unsafe fn seL4_GetCap(index: isize) -> seL4_CPtr {
    let cptr;
    asm!("movl %gs:488(,$1,0x4), $0" : "=r"(cptr) : "r"(index) : : "volatile");
    cptr
}

#[inline(always)]
pub unsafe fn seL4_SetCap(index: isize, cptr: seL4_CPtr) {
    asm!("movl %0, %gs:488(,%1,0x4)" : : "r"(cptr), "r"(i) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetCapReceivePath(receiveCNode: *mut seL4_CPtr,
                                     receiveIndex: *mut seL4_CPtr,
                                     receiveDepth: *mut seL4_Word) {
    if (receiveCNode != ::std::ptr::null_mut()) {
        asm!("movl %gs:500, $0" : "=r"(*receiveCNode) : : : "volatile");
    }

    if (receiveIndex != ::std::ptr::null_mut()) {
        asm!("movl %gs:504, $0" : "=r"(*receiveIndex) : : : "volatile");
    }

    if (receiveDepth != ::std::ptr::null_mut()) {
        asm!("movl %gs:508, $0" : "=r"(*receiveDepth) : : : "volatile");
    }
}

#[inline(always)]
pub unsafe fn seL4_SetCapReceivePath(receiveCNode: seL4_CPtr,
                       receiveIndex: seL4_CPtr,
                       receiveDepth: seL4_Word) {
    asm!("movl $0, %gs:500" : : "r"(receiveCNode) : "memory" : "volatile");
    asm!("movl $0, %gs:504" : : "r"(receiveIndex) : "memory" : "volatile");
    asm!("movl $0, %gs:508" : : "r"(receiveDepth) : "memory" : "volatile");
}
