use riscv::register::mstatus;

use super::BootInfo;

pub fn get_boot_info(_nonstandard_a2: usize) -> BootInfo {
    BootInfo {
        next_address: get_image_address(),
        mpp: mstatus::MPP::Supervisor,
    }
}

const PAYLOAD_PTR: *const u8 = payload_image.0.as_ptr();

#[inline]
fn get_image_address() -> usize {
    let address = PAYLOAD_PTR as usize;
    // Optimization barrier: prevent LLVM from constant-folding the address of
    // the linker-script-placed `.payload` section, so that the runtime
    // (post-relocation) address is used.
    unsafe { core::arch::asm!("", options(nomem, nostack, preserves_flags)) };
    address
}

include!(concat!(env!("OUT_DIR"), "/generated_alignment.rs"));
include!(concat!(env!("OUT_DIR"), "/generated_payload.rs"));
#[cfg(feature = "fdt")]
include!(concat!(env!("OUT_DIR"), "/generated_fdt.rs"));
