use sbi_spec::binary::SbiRet;

pub fn exit_enclave(retval: usize) -> SbiRet {
    todo!()
}

pub fn enclave_ocall(ocall_id: usize, arg0: usize, arg1: usize) -> SbiRet {
    todo!()
}

pub fn enclave_acqire(enclave_name: usize) -> SbiRet {
    todo!()
}

pub fn call_enclave(enclave_id: usize) -> SbiRet {
    todo!()
}

pub fn enclave_return() -> SbiRet {
    todo!()
}

pub fn get_report() -> SbiRet {
    todo!()
}
