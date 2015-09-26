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

pub type seL4_Word = u32;
pub type seL4_CPtr = seL4_Word;

pub type seL4_IA32_ASIDControl = seL4_CPtr;
pub type seL4_IA32_ASIDPool = seL4_CPtr;
pub type seL4_IA32_IOSpace = seL4_CPtr;
pub type seL4_IA32_IOPort = seL4_CPtr;
pub type seL4_IA32_Page = seL4_CPtr;
pub type seL4_IA32_PageDirectory = seL4_CPtr;
pub type seL4_IA32_PageTable = seL4_CPtr;
pub type seL4_IA32_IOPageTable = seL4_CPtr;

pub type seL4_CNode = seL4_CPtr;
pub type seL4_IRQHandler = seL4_CPtr;
pub type seL4_IRQControl = seL4_CPtr;
pub type seL4_TCB = seL4_CPtr;
pub type seL4_Untyped = seL4_CPtr;
pub type seL4_DomainSet = seL4_CPtr;

pub struct seL4_UserContext {
    pub eip: seL4_Word,
    pub esp: seL4_Word,
    pub eflags: seL4_Word,
    pub eax: seL4_Word,
    pub ebx: seL4_Word,
    pub ecx: seL4_Word,
    pub edx: seL4_Word,
    pub esi: seL4_Word,
    pub edi: seL4_Word,
    pub ebp: seL4_Word,
    pub tls_base: seL4_Word,
    pub fs: seL4_Word,
    pub gs: seL4_Word,
}

pub const seL4_MsgLengthBits: usize = 7;
pub const seL4_MsgMaxLength: usize = 120;
pub const seL4_MsgExtraCapBits: usize = 2;
pub const seL4_MsgMaxExtraCaps: usize = (1usize << seL4_MsgExtraCapBits) - 1;

pub struct seL4_IPCBuffer {
    pub tag: seL4_MessageInfo,
    pub msg: [seL4_Word; seL4_MsgMaxLength],
    pub userData: seL4_Word,
    pub caps_or_badges: [seL4_Word; seL4_MsgMaxExtraCaps],
    pub receiveCNode: seL4_CPtr,
    pub receiveIndex: seL4_CPtr,
    pub receiveDepth: seL4_CPtr,
}

pub const Default_VMAttributes: usize = 0;
pub enum seL4_IA32_VMAttributes {
    WriteBack = 0,
    WriteThrough = 1,
    CacheDisabled = 2,
    Uncacheable = 3,
    WriteCombining = 4,
}

pub enum seL4_CapRights {
    CanWrite = 0x01,
    CanRead = 0x02,
    CanGrant = 0x04,
    AllRights = 0x07,
}

#[inline(always)]
pub unsafe fn seL4_GetTag() -> seL4_MessageInfo {
    let mut tag: seL4_MessageInfo = ::core::mem::uninitialized();
    asm!("movl %gs:0, $0" : "=r"(tag.words[0]) : : : "volatile");
    tag
}

#[inline(always)]
pub unsafe fn seL4_SetTag(tag: seL4_MessageInfo) {
    asm!("movl $0, %gs:0" : : "r"(tag.words[0]) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetMR(regnum: isize) -> seL4_Word {
    let mr;
    asm!("movl %gs:4(,$1,0x4), $0" : "=r"(mr) : "r"(regnum) : : "volatile");
    mr
}

#[inline(always)]
pub unsafe fn seL4_SetMR(regnum: isize, value: seL4_Word) {
    asm!("movl $0, %gs:4(,$1,0x4)" : : "r"(value), "r"(regnum) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetUserData() -> seL4_Word {
    let data;
    asm!("movl %gs:484, $0" : "=r"(data) : : : "volatile");
    data
}

#[inline(always)]
pub unsafe fn seL4_GetIPCBuffer() -> *mut seL4_IPCBuffer {
    seL4_GetUserData() as isize as *mut seL4_IPCBuffer
}

#[inline(always)]
pub unsafe fn seL4_SetUserData(data: seL4_Word) {
    asm!("movl $0, %gs:484" : : "r"(data) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetBadge(index: isize) -> seL4_CapData {
    let mut badge: seL4_CapData = ::core::mem::uninitialized();
    asm!("movl %gs:488(,$1,0x4), $0" : "=r"(badge.words[0]) : "r"(index) : : "volatile");
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
    asm!("movl $0, %gs:488(,$1,0x4)" : : "r"(cptr), "r"(index) : "memory" : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_GetCapReceivePath(receiveCNode: *mut seL4_CPtr,
                                     receiveIndex: *mut seL4_CPtr,
                                     receiveDepth: *mut seL4_Word) {
    if !receiveCNode.is_null() {
        asm!("movl %gs:500, $0" : "=r"(*receiveCNode) : : : "volatile");
    }

    if !receiveIndex.is_null() {
        asm!("movl %gs:504, $0" : "=r"(*receiveIndex) : : : "volatile");
    }

    if !receiveDepth.is_null() {
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

#[inline(always)]
pub unsafe fn seL4_Send(dest: seL4_CPtr, msgInfo: seL4_MessageInfo) {
    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::Send as seL4_Word),
        "{bx}" (dest),
          "{si}" (msgInfo.words[0]),
          "{di}" (seL4_GetMR(0)),
          "{cx}" (seL4_GetMR(1))
          : "%edx"
        : "volatile");
}

macro_rules! opt_deref {
    ($name:expr) => {
        if !$name.is_null() {
            *$name
        } else {
            0
        }
    }
}

macro_rules! opt_assign {
    ($loc:expr, $val:expr) => {
        if !$loc.is_null() {
            *$loc = $val;
        }
    }
}

#[inline(always)]
pub unsafe fn seL4_SendWithMRs(dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
                               mr0: *mut seL4_Word, mr1: *mut seL4_Word) {
    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::Send as seL4_Word),
        "{bx}" (dest),
          "{si}" (msgInfo.words[0]),
          "{di}" (opt_deref!(mr0)),
          "{cx}" (opt_deref!(mr1))
          : "%edx"
        : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_NBSend(dest: seL4_CPtr, msgInfo: seL4_MessageInfo) {
    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::NBSend as seL4_Word),
        "{bx}" (dest)
          "{si}" (msgInfo.words[0]),
          "{di}" (seL4_GetMR(0)),
          "{cx}" (seL4_GetMR(1))
          : "%edx"
        : "volatile");
}
#[inline(always)]
pub unsafe fn seL4_NBSendWithMRs(dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
                                 mr0: *mut seL4_Word, mr1: *mut seL4_Word) {
    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::NBSend as seL4_Word),
        "{bx}" (dest),
          "{si}" (msgInfo.words[0]),
          "{di}" (opt_deref!(mr0)),
          "{cx}" (opt_deref!(mr1))
          : "%edx"
        : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_Reply(msgInfo: seL4_MessageInfo) {
    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::Reply as seL4_Word),
        "{si}" (msgInfo.words[0])
          "{di}" (seL4_GetMR(0)),
          "{cx}" (seL4_GetMR(1))
        : "%ebx", "%edx"
        : "volatile");
}
#[inline(always)]
pub unsafe fn seL4_ReplyWithMRs(msgInfo: seL4_MessageInfo,
                                mr0: *mut seL4_Word, mr1: *mut seL4_Word) {
    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::Reply as seL4_Word),
        "{si}" (msgInfo.words[0]),
          "{di}" (opt_deref!(mr0))
          "{cx}" (opt_deref!(mr1))
        : "%ebx", "%edx"
        : "volatile");
}


#[inline(always)]
pub unsafe fn seL4_Notify(dest: seL4_CPtr, msg: seL4_Word) {
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::Send as seL4_Word),
        "{bx}" (dest),
          "{si}" (seL4_MessageInfo::new(0, 0, 0, 1).words[0]),
          "{di}" (msg)
        : "%ecx", "%edx"
        : "volatile");
}

#[inline(always)]
pub unsafe fn seL4_Wait(src: seL4_CPtr, sender: *mut seL4_Word) -> seL4_MessageInfo {
    let mut info = seL4_MessageInfo { words: [0] };
    let badge: seL4_Word;
    let mr0: seL4_Word;
    let mr1: seL4_Word;

    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          movl %ebp, %ecx
          popl %ebp"
        : "={bx}" (badge),
          "={si}" (info.words[0]),
          "={dx}" (mr0),
          "={cx}" (mr1)
        : "{ax}" (SyscallId::Wait as seL4_Word),
        "{bx}" (src)
        : "%edx", "memory"
        : "volatile");

    seL4_SetMR(0, mr0);
    seL4_SetMR(1, mr1);

    opt_assign!(sender, badge);

    ::core::mem::uninitialized()
}

#[inline(always)]
pub unsafe fn seL4_WaitWithMRs(src: seL4_CPtr, sender: *mut seL4_Word,
                               mr0: *mut seL4_Word, mr1: *mut seL4_Word) -> seL4_MessageInfo {
    let mut info: seL4_MessageInfo = ::core::mem::uninitialized();
    let badge: seL4_Word;
    let msg0: seL4_Word;
    let msg1: seL4_Word;

    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          movl %ebp, %ecx
          popl %ebp"
        : "={bx}" (badge),
        "={si}" (info.words[0]),
          "={di}" (msg0),
          "={cx}" (msg1)
        : "{ax}" (SyscallId::Wait as seL4_Word),
        "{bx}" (src)
        : "%edx", "memory"
        : "volatile");

    opt_assign!(mr0, msg0);
    opt_assign!(mr1, msg1);
    opt_assign!(sender, badge);

    info
}

#[inline(always)]
pub unsafe fn seL4_Call(mut dest: seL4_CPtr, msgInfo: seL4_MessageInfo) -> seL4_MessageInfo {
    let mut info: seL4_MessageInfo = ::core::mem::uninitialized();
    let mut mr0 = seL4_GetMR(0);
    let mut mr1 = seL4_GetMR(1);

    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          movl %ebp, %ecx
          popl %ebp"
        : "={si}" (info.words[0]),
        "={di}" (mr0),
          "={cx}" (mr1),
          "={bx}" (dest) /* dummy, tells GCC that ebx is clobbered (check if necessary) */
        : "{ax}" (SyscallId::Call as seL4_Word),
        "{bx}" (dest),
          "{si}" (msgInfo.words[0]),
          "{di}" (mr0),
          "{cx}" (mr1)
          : "%edx", "memory"
        : "volatile");

    seL4_SetMR(0, mr0);
    seL4_SetMR(1, mr1);

    info
}

#[inline(always)]
pub unsafe fn seL4_CallWithMRs(mut dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
                               mr0: *mut seL4_Word, mr1: *mut seL4_Word) -> seL4_MessageInfo {
    let mut info: seL4_MessageInfo = ::core::mem::uninitialized();
    let mut msg0: seL4_Word = 0;
    let mut msg1: seL4_Word = 0;

    if !mr0.is_null() {
        if msgInfo.get_length() > 0 {
            msg0 = *mr0;
        }
    }
    if !mr1.is_null() {
        if msgInfo.get_length() > 1 {
            msg1 = *mr1;
        }
    }

    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          movl %ebp, %ecx
          popl %ebp"
        : "={si}" (info.words[0])
        "={di}" (msg0),
          "={cx}" (msg1),
          "={bx}" (dest) /* dummy, tells GCC that ebx is clobbered (check if still necessary) */
        : "{ax}" (SyscallId::Call as seL4_Word),
        "{bx}" (dest)
          "{si}" (msgInfo.words[0]),
          "{di}" (msg0),
          "{cx}" (msg1)
          : "%edx", "memory"
        : "volatile");

    opt_assign!(mr0, msg0);
    opt_assign!(mr1, msg1);

    info
}

#[inline(always)]
pub unsafe fn seL4_ReplyWait(dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
                             sender: *mut seL4_Word) -> seL4_MessageInfo {
    let mut info: seL4_MessageInfo = ::core::mem::uninitialized();
    let badge: seL4_Word;
    let mut mr0 = seL4_GetMR(0);
    let mut mr1 = seL4_GetMR(1);

    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          movl %ebp, %ecx
          popl %ebp"
        : "={bx}" (badge),
        "={si}" (info.words[0]),
          "={di}" (mr0),
          "={cx}" (mr1)
        : "{ax}" (SyscallId::ReplyWait as seL4_Word),
        "{bx}" (dest),
          "{si}" (msgInfo.words[0]),
          "{di}" (mr0),
          "{cx}" (mr1)
          : "%edx", "memory"
        : "volatile");

    seL4_SetMR(0, mr0);
    seL4_SetMR(1, mr1);

    opt_assign!(sender, badge);

    info
}

#[inline(always)]
pub unsafe fn seL4_ReplayWaitWithMRs(dest: seL4_CPtr, msgInfo: seL4_MessageInfo, sender: *mut seL4_Word,
                                     mr0: *mut seL4_Word, mr1: *mut seL4_Word) -> seL4_MessageInfo {
    let mut info: seL4_MessageInfo = ::core::mem::uninitialized();
    let badge: seL4_Word;
    let mut msg0: seL4_Word = 0;
    let mut msg1: seL4_Word = 0;

    if !mr0.is_null() {
        if msgInfo.get_length() > 0 {
            msg0 = *mr0;
        }
    }
    if !mr1.is_null() {
        if msgInfo.get_length() > 1 {
            msg1 = *mr1;
        }
    }

    asm!("pushl %ebp
          movl %ecx, %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          movl %ebp, %ecx
          popl %ebp"
        : "={bx}" (badge),
        "={si}" (info.words[0]),
          "={di}" (msg0),
          "={cx}" (msg1)
        : "{bx}" (dest),
        "{si}" (msgInfo.words[0]),
          "{di}" (msg0),
          "{cx}" (msg1)
        : "%edx", "memory"
        : "volatile");

    opt_assign!(mr0, msg0);
    opt_assign!(mr1, msg1);
    opt_assign!(sender, badge);

    info
}

#[inline(always)]
pub unsafe fn seL4_Yield() {
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::Yield as seL4_Word)
        : "%ebx", "%ecx", "%edx", "%esi", "%edi", "memory"
        : "volatile");
}

#[inline(always)]
#[cfg(feature = "SEL4_DEBUG")]
pub unsafe fn seL4_DebugPutChar(c: u8) {
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::DebugPutChar as seL4_Word),
        "{bx}" (c)
        : "%ecx", "%edx", "%esi", "%edi", "memory"
        : "volatile");
}

#[inline(always)]
#[cfg(feature = "SEL4_DEBUG")]
pub unsafe fn seL4_DebugHalt() {
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::DebugHalt as seL4_Word)
        : "%ebx", "%ecx", "%edx", "%esi", "%edi", "memory"
        : "volatile");
}

#[inline(always)]
#[cfg(feature = "SEL4_DEBUG")]
pub unsafe fn seL4_DebugSnapshot() {
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::DebugSnapshot as seL4_Word)
        : "%ebx", "%ecx", "%edx", "%esi", "%edi", "memory"
        : "volatile");
}

#[inline(always)]
#[cfg(feature = "SEL4_DEBUG")]
pub unsafe fn seL4_DebugCapIdentify(cap: seL4_CPtr) -> u32 {
    let mut _cap = cap;
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        : "={bx}" (_cap)
        : "{ax}" (SyscallId::DebugCapIdentify as seL4_Word),
          "{bx}" (_cap)
          : "%ecx", "%edx", "%esi", "%edi", "memory"
        : "volatile");
    _cap
}

// Note: name MUST be NUL-terminated.
#[inline(always)]
#[cfg(feature = "SEL4_DEBUG")]
pub unsafe fn seL4_DebugNameThread(tcb: seL4_CPtr, name: &[u8]) {
    core::ptr::copy_nonoverlapping(seL4_GetIPCBuffer() as *mut u8, name.as_ptr(), name.len());
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::DebugNameThread as seL4_Word),
        "{bx}" (tcb)
        : "%ecx", "%edx", "%esi", "%edi", "memory"
        : "volatile");
}

#[inline(always)]
#[cfg(feature = "SEL4_DANGEROUS_CODE_INJECTION")]
pub unsafe fn seL4_DebugRun(userfn: extern fn(*mut u8), userarg: *mut u8) {
    let userfnptr = userfn as *mut ();
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::DebugRun as seL4_Word),
        "{bx}" (userfnptr),
          "{si}" (userarg)
          : "%ecx", "%edx", "%edi", "memory"
        : "volatile");
}

#[inline(always)]
#[cfg(feature = "SEL4_CONFIG_BENCHMARK")]
pub unsafe fn seL4_BenchmarkResetLog() {
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        :
        : "{ax}" (SyscallId::BenchmarkResetLog as seL4_Word)
        : "%ecx", "%edx", "%edi", "memory"
        : "volatile");
}

#[inline(always)]
#[cfg(feature = "SEL4_CONFIG_BENCHMARK")]
pub unsafe fn seL4_BenchmarkDumpLog(start: seL4_Word, size: seL4_Word) -> u32 {
    let dump: u32;
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        : "={bx}" (dump)
        : "{ax}" (SyscallId::BenchmarkDumpLog as seL4_Word),
          "{bx}" (start),
          "{si}" (size)
        : "%ecx", "%edx", "%edi", "memory"
        : "volatile");
    dump
}

#[inline(always)]
#[cfg(feature = "SEL4_CONFIG_BENCHMARK")]
pub unsafe fn seL4_BenchmarkLogSize() -> u32 {
    let ret: u32;
    asm!("pushl %ebp
          movl %esp, %ecx
          leal 1f, %edx
          1:
          sysenter
          popl %ebp"
        : "={bx}" (ret)
        : "{ax}" (SyscallId::BenchmarkLogSize as seL4_Word)
        : "%ecx", "%edx", "%edi", "memory"
        : "volatile");
    ret
}
