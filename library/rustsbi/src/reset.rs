use sbi_spec::binary::SbiRet;

/// System Reset extension (SRST, EID `0x53525354`).
///
/// Provides a function that allows the supervisor software to request system-level reboot or shutdown.
///
/// The term "system" refers to the world-view of supervisor software and the underlying SBI implementation
/// could be machine mode firmware or hypervisor.
///
/// Ref: [SBI v3.0, Section 10](https://raw.githubusercontent.com/riscv-non-isa/riscv-sbi-doc/v3.0/src/ext-sys-reset.adoc).
pub trait Reset {
    /// Reset the system based on provided `reset_type` and `reset_reason`.
    ///
    /// This is a synchronous call and does not return if it succeeds.
    ///
    /// # Warm reboot and cold reboot
    ///
    /// When supervisor software is running natively, the SBI implementation is machine mode firmware.
    /// In this case, shutdown is equivalent to physical power down of the entire system, and
    /// cold reboot is equivalent to a physical power cycle of the entire system.
    /// Further, warm reboot is equivalent to a power cycle of the main processor and parts of the system
    /// but not the entire system.
    ///
    /// For example, on a server class system with a BMC (board management controller),
    /// a warm reboot will not power cycle the BMC
    /// whereas a cold reboot will definitely power cycle the BMC.
    ///
    /// When supervisor software is running inside a virtual machine,
    /// the SBI implementation is a hypervisor.
    /// The shutdown,
    /// cold reboot and warm reboot will behave functionally the same as the native case but might
    /// not result in any physical power changes.
    ///
    /// # Parameters
    ///
    /// `reset_type` is a 32-bit reset selector with the encodings in the RISC-V
    /// SBI Specification v3.0, Section 10.1 (Table 26):
    ///
    /// | Value | Meaning |
    /// |:------|:--------|
    /// | `0x00000000` | Shutdown |
    /// | `0x00000001` | Cold reboot |
    /// | `0x00000002` | Warm reboot |
    /// | `0x00000003..=0xEFFFFFFF` | Reserved for future use |
    /// | `0xF0000000..=0xFFFFFFFF` | Vendor-specific or platform-specific reset type |
    ///
    /// `reset_reason` encodes an optional reset reason in 32 bits, using the
    /// values in Table 27:
    ///
    /// | Value | Meaning |
    /// |:------|:--------|
    /// | `0x00000000` | No reason |
    /// | `0x00000001` | System failure |
    /// | `0x00000002..=0xDFFFFFFF` | Reserved for future use |
    /// | `0xE0000000..=0xEFFFFFFF` | Reason specific to the SBI implementation |
    /// | `0xF0000000..=0xFFFFFFFF` | Vendor-specific or platform-specific reset reason |
    ///
    /// # Return value
    ///
    /// A return indicates failure. [`SbiRet::error`] follows the System Reset
    /// error table in SBI v3.0, Section 10.1 (Table 28). The error-return
    /// convention in Section 3 does not define [`SbiRet::value`] for this function.
    ///
    /// | Error Code | Description |
    /// |:-----------|:------------|
    /// | `SbiRet::invalid_param()` | Either parameter has a reserved value, or a platform-specific value that has no implementation. |
    /// | `SbiRet::not_supported()` | The reset type has an implementation and is not reserved, but the platform lacks at least one required dependency. |
    /// | `SbiRet::failed()` | The request could not be completed for another unknown or unspecified reason. |
    fn system_reset(&self, reset_type: u32, reset_reason: u32) -> SbiRet;
    /// Function internal to macros. Do not use.
    #[doc(hidden)]
    #[inline]
    fn _rustsbi_probe(&self) -> usize {
        sbi_spec::base::UNAVAILABLE_EXTENSION.wrapping_add(1)
    }
}

impl<T: Reset> Reset for &T {
    #[inline]
    fn system_reset(&self, reset_type: u32, reset_reason: u32) -> SbiRet {
        T::system_reset(self, reset_type, reset_reason)
    }
}

impl<T: Reset> Reset for Option<T> {
    #[inline]
    fn system_reset(&self, reset_type: u32, reset_reason: u32) -> SbiRet {
        self.as_ref().map_or(SbiRet::not_supported(), |inner| {
            T::system_reset(inner, reset_type, reset_reason)
        })
    }
    #[inline]
    fn _rustsbi_probe(&self) -> usize {
        match self {
            Some(_) => sbi_spec::base::UNAVAILABLE_EXTENSION.wrapping_add(1),
            None => sbi_spec::base::UNAVAILABLE_EXTENSION,
        }
    }
}
