//! Reset entry and multi-hart startup protocol.
//!
//! Harts claim stack slots in arrival order. Slot zero is the boot hart; it
//! clears BSS and then publishes readiness with release ordering. Every other
//! admitted hart waits with acquire ordering before entering Rust code that may
//! access BSS. Hart IDs do not need to be dense or start at zero.

use core::cell::UnsafeCell;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use core::sync::atomic::AtomicUsize;

/// Maximum number of harts admitted by the early runtime.
pub const MAX_HARTS: usize = 8;

/// Bytes reserved for each hart's early boot stack.
pub const STACK_SIZE: usize = 16 * 1024;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
const STACK_SHIFT: usize = STACK_SIZE.trailing_zeros() as usize;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
const BOOT_INITIALIZING: usize = 0;
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
const BOOT_READY: usize = 1;

const _: () = assert!(STACK_SIZE.is_power_of_two());
const _: () = assert!(STACK_SIZE >= 16);

/// Arguments established by the machine runtime for one hart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BootParams {
    hart_id: usize,
    opaque: usize,
}

impl BootParams {
    /// Create a `BootParams` from hart id and opaque register value.
    pub const fn new(hart_id: usize, opaque: usize) -> Self {
        Self { hart_id, opaque }
    }

    /// Architectural hart ID read from `mhartid`.
    pub const fn hart_id(&self) -> usize {
        self.hart_id
    }

    /// Opaque value passed in `a1` by the previous boot stage.
    pub const fn opaque(&self) -> usize {
        self.opaque
    }
}

/// Storage backing the early per-hart boot stacks.
///
/// The storage contains [`MAX_HARTS`] adjacent slots of [`STACK_SIZE`] bytes
/// and is aligned to the 16-byte RISC-V stack ABI. Slot `i` occupies
/// `i * STACK_SIZE..(i + 1) * STACK_SIZE`; startup initializes `sp` to the
/// exclusive upper bound because stacks grow toward lower addresses.
///
/// This type only defines the memory layout. It neither assigns slots nor
/// tracks hart ownership. The reset path performs that allocation atomically
/// before any hart uses a slot. Its field remains private so safe code cannot
/// create overlapping mutable access to stack storage.
///
/// On RISC-V, the runtime-owned instance is placed in a dedicated `NOLOAD`
/// linker section. The type itself is target-independent so its size and
/// alignment contract can be checked by host-side tests.
#[repr(C, align(16))]
pub struct BootStacks(UnsafeCell<[u8; MAX_HARTS * STACK_SIZE]>);

// Each hart exclusively accesses the stack slot assigned by NEXT_STACK.
unsafe impl Sync for BootStacks {}

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[used]
#[unsafe(link_section = ".uninit.rustsbi_machine.stacks")]
static BOOT_STACKS: BootStacks = BootStacks(UnsafeCell::new([0; MAX_HARTS * STACK_SIZE]));

// These values are deliberately in loadable .data rather than .bss: secondary
// harts use them while the boot hart is clearing BSS.
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[used]
#[unsafe(link_section = ".data.rustsbi_machine.boot")]
static NEXT_STACK: AtomicUsize = AtomicUsize::new(0);

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[used]
#[unsafe(link_section = ".data.rustsbi_machine.boot")]
static BOOT_STATE: AtomicUsize = AtomicUsize::new(BOOT_INITIALIZING);

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
core::arch::global_asm!(
    include_str!("entry.S"),
    next_stack = sym NEXT_STACK,
    boot_state = sym BOOT_STATE,
    boot_stacks = sym BOOT_STACKS,
    rust_entry = sym rust_entry,
    boot_ready = const BOOT_READY,
    max_harts = const MAX_HARTS,
    stack_shift = const STACK_SHIFT,
    xlen = const usize::BITS,
);

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[unsafe(no_mangle)]
extern "C" fn rust_entry(hart_id: usize, opaque: usize) -> ! {
    main(BootParams::new(hart_id, opaque))
}

/// Built-in entry used until `#[entry]` is implemented.
#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
fn main(params: BootParams) -> ! {
    crate::println!("rustsbi-machine: hart {} entered", params.hart_id());
    halt()
}

/// Permanently halts the current hart with interrupts disabled.
///
/// This is not equivalent to [`std::thread::park`]: it has no wake token or
/// matching `unpark` operation, establishes no memory-ordering edge, and never
/// returns. Because RISC-V defines `wfi` as a hint, an implementation may treat
/// it as a no-op; the surrounding loop still guarantees that execution never
/// proceeds past this function.
pub fn halt() -> ! {
    match () {
        #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
        () => {
            // SAFETY: rustsbi-machine runs in M-mode. Disabling machine
            // interrupts makes the permanent-stop contract independent of the
            // caller's current interrupt state.
            unsafe {
                core::arch::asm!(
                    "csrci mstatus, 8",
                    "csrw mie, zero",
                    options(nomem, nostack)
                )
            };
            loop {
                // SAFETY: WFI is an architectural hint and does not access
                // memory or the stack. The loop handles spurious resumption.
                unsafe { core::arch::asm!("wfi", options(nomem, nostack)) };
            }
        }
        #[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
        () => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use core::mem::{align_of, size_of};

    use super::{BootParams, BootStacks, MAX_HARTS, STACK_SIZE};

    #[test]
    fn boot_params_distinguish_boot_hart_from_secondaries() {
        let boot = BootParams::new(7, 0x1234);
        assert_eq!(boot.hart_id(), 7);
        assert_eq!(boot.opaque(), 0x1234);
    }

    #[test]
    fn boot_stacks_layout_matches_startup_contract() {
        assert_eq!(size_of::<BootStacks>(), MAX_HARTS * STACK_SIZE);
        assert_eq!(align_of::<BootStacks>(), 16);
    }
}
