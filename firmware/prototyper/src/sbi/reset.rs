//! System reset.
//!
//! # References
//!
//! - Specification: [RISC-V SBI SRST extension](https://docs.riscv.org/reference/sbi/v3.0/ext-sys-reset.html) —
//!   reset types, reasons, and error semantics.

#![forbid(unsafe_code)]

use alloc::boxed::Box;
use rustsbi::SbiRet;
use spin::Mutex;

use crate::driver::{ResetDevice, ResetError, ResetReason, ResetRequest, ResetType};

/// SBI system-reset extension service.
pub struct SbiReset {
    pub reset_dev: Mutex<Box<dyn ResetDevice + Send>>,
}

impl SbiReset {
    pub fn new(reset_dev: Mutex<Box<dyn ResetDevice + Send>>) -> Self {
        Self { reset_dev }
    }

    fn reset(&self, req: ResetRequest) -> SbiRet {
        match self.reset_dev.lock().reset(req) {
            None => SbiRet::invalid_param(),
            Some(ResetError::NotSupported) => SbiRet::not_supported(),
            Some(ResetError::Failed) => SbiRet::failed(),
        }
    }

    #[allow(unused)]
    pub fn fail(&self) -> ! {
        trace!("Test fail, invoke process exit procedure on Reset device");
        let ret = self.reset(ResetRequest {
            reset_type: ResetType::Shutdown,
            reset_reason: ResetReason::SystemFailure,
        });
        error!("System failure reset returned: {ret:?}");
        crate::fail::stop()
    }
}

impl rustsbi::Reset for SbiReset {
    #[inline]
    fn system_reset(&self, reset_type: u32, reset_reason: u32) -> SbiRet {
        let Some(req) = parse_request(reset_type, reset_reason) else {
            return SbiRet::invalid_param();
        };
        self.reset(req)
    }
}

/// Validates both raw parameters before querying or invoking a reset backend.
fn parse_request(reset_type: u32, reset_reason: u32) -> Option<ResetRequest> {
    use rustsbi::spec::srst::{
        RESET_REASON_NO_REASON, RESET_REASON_SYSTEM_FAILURE, RESET_TYPE_COLD_REBOOT,
        RESET_TYPE_SHUTDOWN, RESET_TYPE_WARM_REBOOT,
    };

    let reset_type = match reset_type {
        RESET_TYPE_SHUTDOWN => ResetType::Shutdown,
        RESET_TYPE_COLD_REBOOT => ResetType::ColdReboot,
        RESET_TYPE_WARM_REBOOT => ResetType::WarmReboot,
        0xf000_0000..=0xffff_ffff => ResetType::VendorSpecific(reset_type),
        _ => return None,
    };
    let reset_reason = match reset_reason {
        RESET_REASON_NO_REASON => ResetReason::NoReason,
        RESET_REASON_SYSTEM_FAILURE => ResetReason::SystemFailure,
        0xe000_0000..=0xefff_ffff => ResetReason::SbiSpecific(reset_reason),
        0xf000_0000..=0xffff_ffff => ResetReason::VendorSpecific(reset_reason),
        _ => return None,
    };
    Some(ResetRequest {
        reset_type,
        reset_reason,
    })
}

#[allow(unused)]
pub fn fail() -> ! {
    match crate::sbi::reset() {
        Some(reset) => reset.fail(),
        None => {
            trace!("test fail, begin dead loop");
            loop {
                core::hint::spin_loop()
            }
        }
    }
}
