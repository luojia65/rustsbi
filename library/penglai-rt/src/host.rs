use penglai::host::CREATE_ENCLAVE;
use sbi_spec::binary::SbiRet;

pub fn create_enclave() -> SbiRet {
    todo!()
}

pub fn attest_enclave(enclave_id: usize) -> SbiRet {
    todo!()
}

pub fn run_enclave(enclave_id: usize) -> SbiRet {
    todo!()
}

pub fn stop_enclave(enclave_id: usize) -> SbiRet {
    todo!()
}

pub fn resume_enclave(enclave_id: usize, resume_func_id: usize) -> SbiRet {
    todo!()
}

pub fn destroy_enclave(enclave_id: usize) -> SbiRet {
    todo!()
}
