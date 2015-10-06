mod x86;
use x86::*;

const MAX_BDR       : seL4_Word = 199;                  /* CONFIG_MAX_NUM_BOOTINFO_DEVICE_REGIONS */
const MAX_CAPS      : seL4_Word = 167;                  /* CONFIG_MAX_NUM_BOOTINFO_UNTYPED_CAPS */

enum seL4_Cap {
    seL4_CapNull                =  0,                   /* null cap */
    seL4_CapInitThreadTCB       =  1,                   /* initial thread's TCB cap */
    seL4_CapInitThreadCNode     =  2,                   /* initial thread's root CNode cap */
    seL4_CapInitThreadVSpace    =  3,                   /* initial thread's VSpace cap */
    seL4_CapIRQControl          =  4,                   /* global IRQ controller cap */
    seL4_CapASIDControl         =  5,                   /* global ASID controller cap */
    seL4_CapInitThreadASIDPool  =  6,                   /* initial thread's ASID pool cap */
    seL4_CapIOPort              =  7,                   /* global IO port cap (null cap if not supported) */
    seL4_CapIOSpace             =  8,                   /* global IO space cap (null cap if no IOMMU support) */
    seL4_CapBootInfoFrame       =  9,                   /* bootinfo frame cap */
    seL4_CapInitThreadIPCBuffer = 10,                   /* initial thread's IPC buffer frame cap */
    seL4_CapDomain              = 11,                   /* global domain controller cap */
}

struct seL4_SlotRegion {
    start                   : seL4_Word,                /* first CNode slot position OF region */
    end                     : seL4_Word,                /* first CNode slot position AFTER region */
}

struct seL4_DeviceRegion {
    basePaddr               : seL4_Word,                /* base physical address of device region */
    frameSizeBits           : seL4_Word,                /* size (2^n bytes) of a device-region frame */
    frames                  : seL4_SlotRegion,          /* device-region frame caps */
}

struct seL4_BootInfo {
    nodeID                  : seL4_Word,                /* ID [0..numNodes-1] of the seL4 node (0 if uniprocessor) */
    numNodes                : seL4_Word,                /* number of seL4 nodes (1 if uniprocessor) */
    numIOPTLevels           : seL4_Word,                /* number of IOMMU PT levels (0 if no IOMMU support) */
    ipcBuffer               : &'a seL4_IPCBuffer,       /* pointer to initial thread's IPC buffer */
    empty                   : seL4_SlotRegion,          /* empty slots (null caps) */
    sharedFrames            : seL4_SlotRegion,          /* shared-frame caps (shared between seL4 nodes) */
    userImageFrames         : seL4_SlotRegion,          /* userland-image frame caps */
    userImagePTs            : seL4_SlotRegion,          /* userland-image PT caps */
    untyped                 : seL4_SlotRegion,          /* untyped-object caps (untyped caps) */
    untypedPaddrList        : [seL4_Word; MAX_CAPS],    /* physical address of each untyped cap */
    untypedSizeBitsList     : [u8; MAX_CAPS],           /* size (2^n) bytes of each untyped cap */
    initThreadCNodeSizeBits : u8,                       /* initial thread's root CNode size (2^n slots) */
    numDeviceRegions        : seL4_Word,                /* number of device regions */
    deviceRegions           : [seL4_DeviceRegion; MAX_BDR],  /* device regions */
    initThreadDomain        : seL4_Word,                /* Initial thread's domain ID */
}

static mut boot_info : *const BootInfo<'static> = 0 as *const BootInfo<'static>;

#[no_mangle]
pub extern "C" fn init_boot_info(bi: *const BootInfo<'static>) {
    unsafe { boot_info = bi };
}

pub fn get_boot_info() -> &'static BootInfo<'static> {
    assert!(unsafe { boot_info != (0 as *const BootInfo<'static>) });
    unsafe { &*boot_info }
}
