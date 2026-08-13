//! Bare-metal runtime foundation for RustSBI firmware.
//!
//! `rustsbi-machine` owns the machine-mode reset entry, early per-hart stacks,
//! cold/warm hart synchronization, BSS initialization, and an installable early
//! console. It deliberately does not own device discovery, concrete UART drivers,
//! SBI dispatch, allocation, scheduling, or an RTOS policy.
//!
//! The current runtime expects an in-place RISC-V image and the conventional
//! machine-mode entry ABI: `a0` contains the hart ID and `a1` contains an opaque
//! boot argument. The runtime reads `mhartid` itself and preserves `a1` in
//! [`BootParams`]. A final binary must link `rustsbi-machine.x`, for example with
//! `-C link-arg=-Trustsbi-machine.x`.
//!
//! Until the public `#[entry]` macro is introduced, all initialized harts enter a
//! built-in `fn main(BootParams) -> !` that reports the hart and halts it.
#![no_std]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(all(
    any(target_arch = "riscv32", target_arch = "riscv64"),
    not(target_feature = "a")
))]
compile_error!("rustsbi-machine multi-hart startup requires the RISC-V A extension");

pub mod boot;
pub mod console;
mod macros;

pub use boot::BootParams;
