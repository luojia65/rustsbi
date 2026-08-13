#![cfg_attr(any(target_arch = "riscv32", target_arch = "riscv64"), no_std)]
#![cfg_attr(any(target_arch = "riscv32", target_arch = "riscv64"), no_main)]

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
use rustsbi_machine as _;

#[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo<'_>) -> ! {
    rustsbi_machine::println!("panic: {info}");
    rustsbi_machine::boot::halt()
}

#[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
fn main() {}
