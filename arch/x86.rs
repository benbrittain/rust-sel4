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
pub type seL4_CapData_t = seL4_Word;

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
    eip: seL4_Word,
    esp: seL4_Word,
    eflags: seL4_Word,
    eax: seL4_Word,
    ebx: seL4_Word,
    ecx: seL4_Word,
    edx: seL4_Word,
    esi: seL4_Word,
    edi: seL4_Word,
    ebp: seL4_Word,
    tls_base: seL4_Word,
    fs: seL4_Word,
    gs: seL4_Word,
}

const seL4_MsgLengthBits: usize = 7;
const seL4_MsgMaxLength: usize = 120;
const seL4_MsgExtraCapBits: usize = 2;
const seL4_MsgMaxExtraCaps: usize = (1usize << seL4_MsgExtraCapBits) - 1;

pub struct seL4_IPCBuffer {
    tag: seL4_MessageInfo,
    msg: [seL4_Word; seL4_MsgMaxLength],
    userData: seL4_Word,
    caps_or_badges: [seL4_Word; seL4_MsgMaxExtraCaps],
    receiveCNode: seL4_CPtr,
    receiveIndex: seL4_CPtr,
    receiveDepth: seL4_CPtr,
}

const Default_VMAttributes: usize = 0;
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
pub unsafe fn seL4_GetIPCBuffer() -> seL4_IPCBuffer {
    seL4_GetUserData()
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
    asm!("movl %0, %gs:488(,%1,0x4)" : : "r"(cptr), "r"(index) : "memory" : "volatile");
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
		: "a" (SyscallId::Send),
		  "b" (dest),
		  "S" (msgInfo.words[0]),
		  "D" (seL4_GetMR(0)),
		  "c" (seL4_GetMR(1))
		: "%edx"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_SendWithMRs(dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
							   mr0: Option<seL4_Word>, mr1: Option<seL4_Word>) {
	asm!("pushl %ebp
		  movl %ecx, %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::Send),
		  "b" (dest),
		  "S" (msgInfo.words[0]),
		  "D" (match(mr0) { Some(mr0ref) => *mr0ref, None => 0 }),
		  "c" (match(mr1) { Some(mr1ref) => *mr1ref, None => 0 })
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
		: "a" (SyscallId::NBSend),
		  "b" (dest)
		  "S" (msgInfo.words[0]),
		  "D" (seL4_GetMR(0)),
		  "c" (seL4_GetMR(1))
		: "%edx"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_NBSendWithMRs(dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
								 mr0: Option<seL4_Word>, mr1: Option<seL4_Word>) {
	asm!("pushl %ebp
		  movl %ecx, %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::NBSend),
		  "b" (dest),
		  "S" (msgInfo.words[0]),
		  "D" (match(mr0) { Some(mr0ref) => *mr0ref, None => 0 }),
		  "c" (match(mr1) { Some(mr1ref) => *mr1ref, None => 0 })
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
		: "a" (SyscallId::Reply),
		  "S" (msgInfo.words[0])
		  "D" (seL4_GetMR(0)),
		  "c" (seL4_GetMR(1))
		: "%ebx", "%edx"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_ReplyWithMRs(msgInfo: seL4_MessageInfo,
								mr0: Option<seL4_Word>, mr1: Option<seL4_Word>) {
	asm!("pushl %ebp
		  movl %ecx, %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::Reply),
		  "S" (msgInfo.words[0]),
		  "D" (match(mr0) { Some(mr0ref) => *mr0ref, None => 0 })
		  "c" (match(mr1) { Some(mr1ref) => *mr1ref, None => 0 })
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
		: "a" (SyscallId::Send),
		  "b" (dest),
		  "S" (seL4_MessageInfo::new(0, 0, 0, 1).words[0]),
		  "D" (msg)
		: "%ecx", "%edx"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_Wait(src: seL4_CPtr, sender: Option<&mut seL4_Word>) -> seL4_MessageInfo {
	let info: seL4_MessageInfo;
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
		: "=b" (badge),
		  "=S" (info.words[0]),
		  "=D" (mr0),
		  "=c" (mr1)
		: "a" (SyscallId::Wait),
		  "b" (src)
		: "%edx", "memory"
		: "volatile");

	seL4_SetMR(0, mr0);
	seL4_SetMR(1, mr1);

	if let Some(sendref) = sender {
		*sendref = badge;
	}

	info
}

#[inline(always)]
pub unsafe fn seL4_WaitWithMRs(src: seL4_CPtr, sender: Option<&mut seL4_Word>,
							   mr0: Option<&mut seL4_Word>, mr1: Option<&mut seL4_Word>) -> seL4_MessageInfo {
	let info: seL4_MessageInfo;
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
		: "=b" (badge),
		  "=S" (info.words[0]),
		  "=D" (msg0),
		  "=c" (msg1)
		: "a" (SyscallId::Wait),
		  "b" (src)
		: "%edx", "memory"
		: "volatile");

	if let Some(mr0ref) = mr0 {
		*mr0ref = msg0;
	}
	if let Some(mr1ref) = mr1 {
		*mr1ref = msg1;
	}
	if let Some(sendref) = sender {
		*sendref = badge;
	}

	info
}

#[inline(always)]
pub unsafe fn seL4_Call(dest: seL4_CPtr, msgInfo: seL4_MessageInfo) -> seL4_MessageInfo {
	let info: seL4_MessageInfo;
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
		: "=S" (info.words[0]),
		  "=D" (mr0),
		  "=c" (mr1),
		  "=b" (dest)
		: "a" (SyscallId::call),
		  "b" (dest),
		  "S" (msgInfo.words[0]),
		  "D" (mr0),
		  "c" (mr1)
		: "%edx", "memory"
		: "volatile");

	seL4_SetMR(0, mr0);
	seL4_SetMR(1, mr1);

	info
}

#[inline(always)]
pub unsafe fn seL4_CallWithMRs(dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
							   mr0: Option<&mut seL4_Word>, mr1: Option<&mut seL4_Word>) -> seL4_MessageInfo {
	let info: seL4_MessageInfo;
	let mut msg0: seL4_Word = 0;
	let mut msg1: seL4_Word = 0;

	if let Some(mr0ref) = mr0 {
		if msgInfo.get_length() > 0 {
			msg0 = *mr0;
		}
	}
	if let Some(mr1ref) = mr1 {
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
		: "=S" (info.words[0])
		  "=D" (msg0),
		  "=c" (msg1),
		  "=b" (dest)
		: "a" (SyscallId::Call),
		  "b" (dest)
		  "S" (msgInfo.words[0]),
		  "D" (msg0),
		  "c" (msg1)
		: "%edx", "memory"
		: "volatile");

	if let Some(mr0ref) = mr0 {
		*mr0ref = msg0;
	}
	if let Some(mr1ref) = mr1 {
		*mr1ref = msg1;
	}

	info
}

#[inline(always)]
pub unsafe fn seL4_ReplyWait(dest: seL4_CPtr, msgInfo: seL4_MessageInfo,
							 sender: Option<&mut seL4_Word>) {
	let info: seL4_MessageInfo;
	let badge: seL4_Word;
	let mr0 = seL4_GetMR(0);
	let mr1 = seL4_GetMR(1);

	asm!("pushl %ebp
		  movl %ecx, %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  movl %ebp, %ecx
		  popl %ebp"
		: "=b" (badge),
		  "=S" (info.words[0]),
		  "=D" (mr0),
		  "=c" (mr1)
		: "a" (SyscallId::ReplyWait),
		  "b" (dest),
		  "S" (msgInfo.words[0]),
		  "D" (mr0),
		  "c" (mr1)
		: "%edx", "memory"
		: "volatile");

	seL4_SetMR(0, mr0);
	seL4_SetMR(1, mr1);

	if let Some(sendref) = sender {
		*sendref = badge;
	}

	info
}

#[inline(always)]
pub unsafe fn seL4_ReplayWaitWithMRs(dest: seL4_CPtr, msgInfo: seL4_MessageInfo, sender: Option<&mut seL4_Word>,
									 mr0: Option<&mut seL4_Word>, mr1: Option<&mut seL4_Word>) -> seL4_MessageInfo {
	let info: seL4_MessageInfo;
	let badge: seL4_Word;
	let mut msg0: seL4_Word = 0;
	let mut msg1: seL4_Word = 0;

	if let Some(mr0ref) = mr0 {
		if msgInfo.get_length() > 0 {
			msg0 = *mr0ref;
		}
	}
	if let Some(mr1ref) = mr1 {
		if msgInfo.get_length() > 1 {
			msg1 = *mr1ref;
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
		: "=b" (badge),
		  "=S" (info.words[0]),
		  "=D" (msg0),
		  "=c" (msg1)
		: "b" (dest),
		  "S" (msgInfo.words[0]),
		  "D" (msg0),
		  "c" (msg1)
		: "%edx", "memory"
		: "volatile");

	if let Some(mr0ref) = mr0 {
		*mr0ref = msg0;
	}
	if let Some(mr1ref) = mr1 {
		*mr1ref = msg1;
	}
	if let Some(sendref) = sender {
		*sendref = badge;
	}

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
		: "a" (SyscallId::Yield)
		: "%ebx", "%ecx", "%edx", "%esi", "%edi", "memory"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_DebugPutChar(c: u8) {
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::DebugPutChar),
		  "b" (c)
		: "%ecx", "%edx", "%esi", "%edi", "memory"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_DebugHalt() {
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::DebugHalt)
		: "%ebx", "%ecx", "%edx", "%esi", "%edi", "memory"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_DebugSnapshot() {
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::DebugSnapshot)
		: "%ebx", "%ecx", "%edx", "%esi", "%edi", "memory"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_DebugCapIdentify(cap: seL4_CPtr) -> u32 {
	let mut _cap = cap;
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		: "=b" (_cap)
		: "a" (SyscallId::DebugCapIdentify),
		  "b" (_cap)
		: "%ecx", "%edx", "%esi", "%edi", "memory"
		: "volatile");
	_cap
}

// Note: name MUST be NUL-terminated.
#[inline(always)]
pub unsafe fn seL4_DebugNameThread(tcb: seL4_CPtr, name: &[u8]) {
	core::ptr::copy_nonoverlapping(seL4_GetIPCBuffer() as *mut u8, name.as_ptr(), name.len());
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::DebugNameThread),
		  "b" (tcb)
		: "%ecx", "%edx", "%esi", "%edi", "memory"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_DebugRun(userfn: extern fn(*mut u8), userarg: *mut u8) {
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::DebugRun),
		  "b" (userfn),
		  "S" (userarg)
		: "%ecx", "%edx", "%edi", "memory"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_BenchmarkResetLog() {
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		:
		: "a" (SyscallId::BenchmarkResetLog)
		: "%ecx", "%edx", "%edi", "memory"
		: "volatile");
}

#[inline(always)]
pub unsafe fn seL4_BenchmarkDumpLog(start: seL4_Word, size: seL4_Word) -> u32 {
	let dump: u32;
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		: "=b" (dump)
		: "a" (SyscallId::BenchmarkDumpLog),
		  "b" (start),
		  "S" (size)
		: "%ecx", "%edx", "%edi", "memory"
		: "volatile");
	dump
}

#[inline(always)]
pub unsafe fn seL4_BenchmarkLogSize() -> u32 {
	let ret: u32;
	asm!("pushl %ebp
		  movl %esp, %ecx
		  leal 1f, %edx
		  1:
		  sysenter
		  popl %ebp"
		: "=b" (ret)
		: "a" (SyscallId::BenchmarkLogSize)
		: "%ecx", "%edx", "%edi", "memory"
		: "volatile");
	ret
}
