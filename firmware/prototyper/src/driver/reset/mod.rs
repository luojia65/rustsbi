//! Reset drivers.

mod pmic_spacemit_p1;
mod sifive_test;

pub(crate) use pmic_spacemit_p1::I2cAddress;

use alloc::boxed::Box;

use runtime::memory::MemoryRegistry;

use crate::platform::BoardInfo;

/// Parsed reset type accepted by the SRST driver layer.
///
/// Reserved raw values are intentionally not representable here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetType {
    /// SBI standard reset type 0x00000000.
    Shutdown,
    /// SBI standard reset type 0x00000001.
    ColdReboot,
    /// SBI standard reset type 0x00000002.
    WarmReboot,
    /// Vendor / platform specific reset type: 0xF0000000 ..= 0xFFFFFFFF.
    VendorSpecific(u32),
}

/// Parsed reset reason accepted by the SRST driver layer.
///
/// Reserved raw values are intentionally not representable here.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetReason {
    /// SBI standard reset reason 0x00000000.
    NoReason,
    /// SBI standard reset reason 0x00000001.
    SystemFailure,
    /// SBI implementation specific reset reason: 0xE0000000 ..= 0xEFFFFFFF.
    SbiSpecific(u32),
    /// Vendor / platform specific reset reason: 0xF0000000 ..= 0xFFFFFFFF.
    VendorSpecific(u32),
}

/// Fully parsed SRST request.
///
/// This is the unit consumed by the driver layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResetRequest {
    pub reset_type: ResetType,
    pub reset_reason: ResetReason,
}

/// Low-level error category for an SRST backend.
///
/// Important:
/// - `InvalidParam` is intentionally absent.
/// - Successful reset is intentionally absent too, because a successful
///   SRST request does not return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResetError {
    /// The request is implemented, but the platform lacks a required dependency.
    ///
    /// Mapped to SBI RET_ERR_NOT_SUPPORTED.
    #[allow(
        dead_code,
        reason = "current backends acquire their dependencies at bind time"
    )]
    NotSupported,
    /// The reset failed for unspecified or unknown other reasons.
    ///
    /// Mapped to SBI RET_ERR_FAILED.
    Failed,
}

pub trait ResetBackend {
    /// Backend-specific command produced by validating a reset request.
    type Request;

    /// Validate the reset request and prepare its backend-specific command.
    ///
    /// If this returns `None`, the upper layer returns SBI_ERR_INVALID_PARAM
    /// without calling `system_reset`.
    ///
    /// This checks whether the parameter values have an implementation, not
    /// whether all runtime dependencies are available. Missing dependencies
    /// are reported by `system_reset` as `ResetError::NotSupported`.
    fn prepare_reset(&self, req: ResetRequest) -> Option<Self::Request>;

    /// Attempt to reset the system.
    ///
    /// Semantics:
    /// - `req` is the command returned by this backend's `prepare_reset`;
    /// - parameter validation and command selection are already complete;
    /// - if this function returns, it must be an error path.
    fn system_reset(&mut self, req: Self::Request) -> ResetError;
}

/// Object-safe adapter for reset backends with different command types.
pub(crate) trait ResetDevice {
    /// Validate and execute a request without exposing the command type.
    ///
    /// `None` means the request is invalid for this backend. `Some(error)`
    /// means execution failed. A successful reset does not return.
    fn reset(&mut self, req: ResetRequest) -> Option<ResetError>;
}

impl<B: ResetBackend> ResetDevice for B {
    fn reset(&mut self, req: ResetRequest) -> Option<ResetError> {
        let command = self.prepare_reset(req)?;
        Some(self.system_reset(command))
    }
}

pub(crate) const SIFIVE_TEST_COMPATIBLES: [&str; 1] = ["sifive,test0"];
pub(crate) const P1_PMIC_COMPATIBLES: [&str; 2] = ["spacemit,p1", "ky,spm8821"];
pub(crate) const PMIC_I2C_COMPATIBLES: [&str; 2] = ["spacemit,k1-i2c", "ky,i2c"];

/// Binds the reset device selected during platform discovery.
pub(super) fn bind(
    board: &BoardInfo,
    memory: &mut MemoryRegistry,
) -> runtime::Result<Option<Box<dyn ResetDevice + Send>>> {
    if let Some(registers) = board.reset {
        return Ok(Some(sifive_test::bind(registers, memory)?));
    }
    if let Some((registers, pmic_address)) = board.pmic_reset {
        return Ok(Some(pmic_spacemit_p1::bind(
            registers,
            pmic_address,
            board.timebase_frequency_hz,
            memory,
        )?));
    }
    Ok(None)
}
