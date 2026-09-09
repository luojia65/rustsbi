//! SRST validation and unsupported-request regression tests for Prototyper.

use sbi_spec::{binary::SbiRet, srst::*};
use sbi_testing::sbi;

struct RawParam(u32);

impl sbi::ResetType for RawParam {
    fn raw(&self) -> u32 {
        self.0
    }
}

impl sbi::ResetReason for RawParam {
    fn raw(&self) -> u32 {
        self.0
    }
}

fn check(reset_type: u32, reset_reason: u32, expected: SbiRet) {
    assert_eq!(
        sbi::system_reset(RawParam(reset_type), RawParam(reset_reason)),
        expected,
        "SRST type {reset_type:#010x}, reason {reset_reason:#010x}"
    );
}

pub(super) fn test() {
    let standard_types = [
        RESET_TYPE_SHUTDOWN,
        RESET_TYPE_COLD_REBOOT,
        RESET_TYPE_WARM_REBOOT,
    ];
    let vendor_types = [0xf000_0000, u32::MAX];
    let reserved_types = [3, 0x7fff_ffff, 0xe000_0000, 0xefff_ffff];
    let reserved_reasons = [2, 0x7fff_ffff, 0xdfff_ffff];
    let specific_reasons = [0xe000_0000, 0xefff_ffff, 0xf000_0000, u32::MAX];
    let valid_reasons = [
        RESET_REASON_NO_REASON,
        RESET_REASON_SYSTEM_FAILURE,
        0xe000_0000,
        0xefff_ffff,
        0xf000_0000,
        u32::MAX,
    ];

    for reset_type in reserved_types {
        for reset_reason in valid_reasons.into_iter().chain(reserved_reasons) {
            check(reset_type, reset_reason, SbiRet::invalid_param());
        }
    }
    // Invalid reasons are rejected even when the type is unimplemented.
    for reset_type in standard_types.into_iter().chain(vendor_types) {
        for reset_reason in reserved_reasons {
            check(reset_type, reset_reason, SbiRet::invalid_param());
        }
    }
    for reset_type in vendor_types {
        for reset_reason in valid_reasons {
            check(reset_type, reset_reason, SbiRet::invalid_param());
        }
    }
    for reset_type in standard_types {
        for reset_reason in specific_reasons {
            check(reset_type, reset_reason, SbiRet::invalid_param());
        }
    }

    println!("Sbi `SRST` validation test pass");
    // The caller exercises Shutdown + NoReason after all tests complete.
}
