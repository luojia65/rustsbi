//! Chapter 9. COVE Host Extension (EID #0x434F5648 "COVH").

/// Extension ID for COVE Host Extension.
pub const EID_COVH: usize = crate::eid_from_str("COVH") as _;
pub use fid::*;

/// Declared in §9.
mod fid {
    /// Function ID to get TEE Security Monitor (TSM) information.
    ///
    /// Declared in §9.2.
    pub const GET_TSM_INFO: usize = 0;
    /// Function ID to convert pages.
    ///
    /// Declared in §9.3.
    pub const CONVERT_PAGES: usize = 1;
    /// Function ID to reclaim pages.
    ///
    /// Declared in §9.4.
    pub const RECLAIM_PAGES: usize = 2;
    /// Function ID to initiate global fence.
    ///
    /// Declared in §9.5.
    pub const GLOBAL_FENCE: usize = 3;
    /// Function ID to local fence.
    ///
    /// Declared in §9.6.
    pub const LOCAL_FENCE: usize = 4;
    /// Function ID to create TVM.
    ///
    /// Declared in §9.7.
    pub const CREATE_TVM: usize = 5;
    /// Function ID to finalize TVM.
    ///
    /// Declared in §9.8.
    pub const FINALIZE_TVM: usize = 6;
    /// Function ID to destroy TVM.
    ///
    /// Declared in §9.9.
    pub const DESTROY_TVM: usize = 7;
    /// Function ID to add TVM memory region.
    ///
    /// Declared in §9.10.
    pub const ADD_TVM_MEMORY_REGION: usize = 8;
    /// Function ID to add TVM page table pages.
    ///
    /// Declared in §9.11.
    pub const ADD_TVM_PAGE_TABLE_PAGES: usize = 9;
    /// Function ID to add TVM measured pages.
    ///
    /// Declared in §9.12.
    pub const ADD_TVM_MEASURED_PAGES: usize = 10;
    /// Function ID to add TVM zero pages.
    ///
    /// Declared in §9.13.
    pub const ADD_TVM_ZERO_PAGES: usize = 11;
    /// Function ID to add TVM shared pages.
    ///
    /// Declared in §9.14.
    pub const ADD_TVM_SHARED_PAGES: usize = 12;
    /// Function ID to create TVM vCPU.
    ///
    /// Declared in §9.15.
    pub const CREATE_TVM_VCPU: usize = 13;
    /// Function ID to run TVM vCPU.
    ///
    /// Declared in §9.16.
    pub const RUN_TVM_VCPU: usize = 14;
    /// Function ID to initiate TVM fence.
    ///
    /// Declared in §9.17.
    pub const TVM_FENCE: usize = 15;
    /// Function ID to invalidate TVM pages.
    ///
    /// Declared in §9.18.
    pub const TVM_INVALIDATE_PAGES: usize = 16;
    /// Function ID to validate TVM pages.
    ///
    /// Declared in §9.19.
    pub const TVM_VALIDATE_PAGES: usize = 17;
    /// Function ID to remove TVM pages.
    ///
    /// Declared in §9.20.
    pub const TVM_REMOVE_PAGES: usize = 18;
}
