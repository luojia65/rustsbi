#![no_std]

// §9
pub mod host;
// §10
pub mod interrupt;
// §11
pub mod guest;

/// Converts SBI EID from str.
const fn eid_from_str(name: &str) -> i32 {
    match *name.as_bytes() {
        [a] => i32::from_be_bytes([0, 0, 0, a]),
        [a, b] => i32::from_be_bytes([0, 0, a, b]),
        [a, b, c] => i32::from_be_bytes([0, a, b, c]),
        [a, b, c, d] => i32::from_be_bytes([a, b, c, d]),
        _ => unreachable!(),
    }
}
#[cfg(test)]
mod tests {
    use static_assertions::const_assert_eq;
    // §9
    #[test]
    fn test_cove_host() {
        use crate::host::*;
        const_assert_eq!(0x434F5648, EID_COVH);
        const_assert_eq!(0, GET_TSM_INFO);
        const_assert_eq!(1, CONVERT_PAGES);
        const_assert_eq!(2, RECLAIM_PAGES);
        const_assert_eq!(3, GLOBAL_FENCE);
        const_assert_eq!(4, LOCAL_FENCE);
        const_assert_eq!(5, CREATE_TVM);
        const_assert_eq!(6, FINALIZE_TVM);
        const_assert_eq!(7, DESTROY_TVM);
        const_assert_eq!(8, ADD_TVM_MEMORY_REGION);
        const_assert_eq!(9, ADD_TVM_PAGE_TABLE_PAGES);
        const_assert_eq!(10, ADD_TVM_MEASURED_PAGES);
        const_assert_eq!(11, ADD_TVM_ZERO_PAGES);
        const_assert_eq!(12, ADD_TVM_SHARED_PAGES);
        const_assert_eq!(13, CREATE_TVM_VCPU);
        const_assert_eq!(14, RUN_TVM_VCPU);
        const_assert_eq!(15, TVM_FENCE);
        const_assert_eq!(16, TVM_INVALIDATE_PAGES);
        const_assert_eq!(17, TVM_VALIDATE_PAGES);
        const_assert_eq!(18, TVM_REMOVE_PAGES);
    }
    // TODO test_cove_interrupt
    // TODO test_cove_guest
}
