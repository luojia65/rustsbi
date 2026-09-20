//! K3 X100 AMP-window PMA initialization.
//!
//! The register map and 64-byte cache line come from the pinned [K3 header].
//! The [AMP patch] locates a stacked-bound PMA entry instead of assuming entry 6.
//! Cache cleaning follows the vendor [cache helper], including its fences.
//! The full 512 KiB SRAM range follows the vendor [U-Boot K3 configuration].
//! These are AP/X100 facilities, not RT24 CSRs.
//!
//! [U-Boot K3 configuration]: https://github.com/spacemit-com/uboot-2022.10/blob/218ad1711d67c51bf32b88c3dcdd86271e1ae4f1/include/configs/k3.h#L43-L44
//! [K3 header]: https://github.com/Oveln/opensbi-spacemit/blob/7a2df083ed06373c506e2e6f4e09bbd168202f2d/platform/generic/include/spacemit/k3/core_common.h
//! [AMP patch]: https://github.com/Oveln/opensbi-spacemit/blob/7a2df083ed06373c506e2e6f4e09bbd168202f2d/platform/generic/spacemit/spacemit_k3.c
//! [cache helper]: https://github.com/Oveln/opensbi-spacemit/blob/7a2df083ed06373c506e2e6f4e09bbd168202f2d/include/sbi_utils/cache/cache.h

use super::SpacemitK3Soc;

impl SpacemitK3Soc {
    /// Sets this X100 hart's shared SRAM PMA attribute to IO.
    ///
    /// Call in M-mode before supervisor entry on every AP hart. The loader must
    /// establish the stacked-bound PMA table and make the SRAM accessible.
    /// Existing bounds and unrelated attributes are preserved.
    ///
    /// # Errors
    ///
    /// Returns `AccessDenied` if a CSR access faults, `Overflow` or
    /// `InvalidArgs` for invalid bounds, and `NotEnoughResources` if no entry
    /// covers the complete window.
    ///
    /// # Panics
    ///
    /// Panics when called on a non-RV64 target.
    pub fn initialize_amp_pma(self) -> crate::Result<()> {
        #[cfg(not(target_arch = "riscv64"))]
        unimplemented!("K3 X100 PMA initialization requires RV64 M-mode");

        #[cfg(target_arch = "riscv64")]
        {
            riscv::interrupt::free(configure_rv64)
        }
    }
}

#[cfg(target_arch = "riscv64")]
fn configure_rv64() -> crate::Result<()> {
    use crate::trap::{read_csr_guarded as read, write_csr_guarded as write};
    use core::arch::asm;

    // Vendor SRAM_BASE_ADDR / SRAM_TOTAL_SIZE, not the AMP protocol footprint.
    const SRAM_BASE: usize = 0xc080_0000;
    const SRAM_SIZE: usize = 512 * 1024;
    const IO: usize = 0x22;
    const CACHE_LINE: usize = 64;

    let addresses = (|| -> Result<_, crate::trap::Error> {
        Ok([
            read::<0x7e0>()?,
            read::<0x7e1>()?,
            read::<0x7e2>()?,
            read::<0x7e3>()?,
            read::<0x7e4>()?,
            read::<0x7e5>()?,
            read::<0x7e6>()?,
            read::<0x7e7>()?,
            read::<0x7e8>()?,
            read::<0x7e9>()?,
            read::<0x7ea>()?,
            read::<0x7eb>()?,
            read::<0x7ec>()?,
            read::<0x7ed>()?,
            read::<0x7ee>()?,
            read::<0x7ef>()?,
        ])
    })()
    .map_err(|_| crate::Error::AccessDenied)?;
    let mut bottom = 0;
    for (index, encoded) in addresses.into_iter().enumerate() {
        // PMAADDR contains word-encoded upper bounds. Entry i starts at the
        // previous bound; locate full SRAM coverage without assuming an index.
        let top = encoded.checked_mul(4).ok_or(crate::Error::Overflow)?;
        if top != 0 && top < bottom {
            return Err(crate::Error::InvalidArgs);
        }
        if top != 0 && bottom <= SRAM_BASE && SRAM_BASE + SRAM_SIZE <= top {
            // PMACFG0/2 are 0x7de/0x7df; each contains eight attribute bytes.
            let old = if index < 8 {
                read::<0x7de>()
            } else {
                read::<0x7df>()
            }
            .map_err(|_| crate::Error::AccessDenied)?;
            let shift = (index % 8) * 8;
            let value = (old & !(0xff << shift)) | (IO << shift);

            // SAFETY: the K3 capability authorizes X100 operations in M-mode.
            // Clean the loader-provided SRAM without constructing references;
            // omit nomem so the cache operations and fences also order memory.
            unsafe {
                asm!("fence rw, rw", options(nostack));
                for address in (SRAM_BASE..SRAM_BASE + SRAM_SIZE).step_by(CACHE_LINE) {
                    asm!(
                        ".option push", ".option arch, +zicbom",
                        "cbo.clean 0({address})", ".option pop",
                        address = in(reg) address, options(nostack)
                    );
                }
                asm!("fence rw, rw", "fence.i", options(nostack));
            }
            if value != old {
                if index < 8 {
                    write::<0x7de>(value)
                } else {
                    write::<0x7df>(value)
                }
                .map_err(|_| crate::Error::AccessDenied)?;
            }
            // SAFETY: M-mode synchronization after this hart's PMA update.
            unsafe { asm!("sfence.vma", options(nostack)) };
            return Ok(());
        }
        bottom = top;
    }
    Err(crate::Error::NotEnoughResources)
}
