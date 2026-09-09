use spec::binary::{Physical, SbiRet};

/// Debug Console extension (DBCN, EID `0x4442434E`).
///
/// The debug console extension defines a generic mechanism for debugging
/// and boot-time early prints from supervisor-mode software.
///
/// DBCN supersedes the legacy console putchar (EID `0x01`) and console getchar
/// (EID `0x02`) extensions, adding multi-byte reads and writes within one SBI call.
///
/// If the underlying physical console has extra bits for error checking
/// (or correction), then these extra bits should be handled by the SBI
/// implementation.
///
/// *NOTE:* It is recommended that bytes sent/received using the debug
/// console extension follow UTF-8 character encoding.
///
/// Ref: [SBI v3.0, Section 12](https://docs.riscv.org/reference/sbi/_attachments/riscv-sbi.pdf#page=51).
pub trait Console {
    /// Write bytes to the debug console from input memory.
    ///
    /// # Non-blocking function
    ///
    /// This is a non-blocking SBI call, and it may do partial or no write operations
    /// if the debug console is not able to accept more bytes.
    ///
    /// # Parameters
    ///
    /// [`Physical::num_bytes`] gives the input byte count (`num_bytes` in the SBI
    /// specification). [`Physical::phys_addr_lo`] and [`Physical::phys_addr_hi`]
    /// encode `base_addr_lo` and `base_addr_hi`, respectively: the lower and upper
    /// XLEN bits of the input buffer's physical base address.
    ///
    /// # Return value
    ///
    /// On success, [`SbiRet::value`] holds the unsigned number of bytes written
    /// (`sbiret.uvalue` in the specification). On error, its value is unspecified
    /// by the general calling convention in Section 3.
    ///
    /// [`SbiRet::error`] follows the Console Write error table in SBI v3.0,
    /// Section 12.1 (Table 50):
    ///
    /// | Error Code | Description |
    /// |:-----------|:------------|
    /// | `SbiRet::success(n)` | Bytes were written successfully. |
    /// | `SbiRet::invalid_param()` | The range encoded by `bytes` fails the shared-memory requirements in [Section 3.2](https://docs.riscv.org/reference/sbi/_attachments/riscv-sbi.pdf#page=17). |
    /// | `SbiRet::denied()` | Console output is not permitted. |
    /// | `SbiRet::failed()` | The write failed because of an I/O error. |
    fn write(&self, bytes: Physical<&[u8]>) -> SbiRet;
    /// Read bytes from the debug console into an output memory.
    ///
    /// # Non-blocking function
    ///
    /// This is a non-blocking SBI call, and it will not write anything
    /// into the output memory if there are no bytes to be read in the
    /// debug console.
    ///
    /// # Parameters
    ///
    /// [`Physical::num_bytes`] gives the output buffer's byte capacity (`num_bytes`
    /// in the SBI specification). [`Physical::phys_addr_lo`] and
    /// [`Physical::phys_addr_hi`] encode `base_addr_lo` and `base_addr_hi`,
    /// respectively: the lower and upper XLEN bits of its physical base address.
    ///
    /// # Return value
    ///
    /// On success, [`SbiRet::value`] holds the unsigned number of bytes read
    /// (`sbiret.uvalue` in the specification). On error, its value is unspecified
    /// by the general calling convention in Section 3.
    ///
    /// [`SbiRet::error`] follows the Console Read error table in SBI v3.0,
    /// Section 12.2 (Table 51):
    ///
    /// | Error Code | Description |
    /// |:-----------|:------------|
    /// | `SbiRet::success(n)` | Bytes were read successfully. |
    /// | `SbiRet::invalid_param()` | The range encoded by `bytes` fails the shared-memory requirements in [Section 3.2](https://docs.riscv.org/reference/sbi/_attachments/riscv-sbi.pdf#page=17). |
    /// | `SbiRet::denied()` | Console input is not permitted. |
    /// | `SbiRet::failed()` | The read failed because of an I/O error. |
    fn read(&self, bytes: Physical<&mut [u8]>) -> SbiRet;
    /// Write a single byte to the debug console.
    ///
    /// # Blocking function
    ///
    /// This SBI call blocks until the byte is written, unless console access is
    /// denied or an I/O error occurs.
    ///
    /// # Return value
    ///
    /// [`SbiRet::value`] (`sbiret.uvalue` in the specification) is zero on every
    /// return, including errors. [`SbiRet::error`] follows the Console Write Byte
    /// error table in SBI v3.0, Section 12.3 (Table 52):
    ///
    /// | Error Code | Description |
    /// |:-----------|:------------|
    /// | `SbiRet::success(0)` | The byte was written. |
    /// | `SbiRet::denied()` | Console output is not permitted. |
    /// | `SbiRet::failed()` | The byte write failed because of an I/O error. |
    fn write_byte(&self, byte: u8) -> SbiRet;
    /// Function internal to macros. Do not use.
    #[doc(hidden)]
    #[inline]
    fn _rustsbi_probe(&self) -> usize {
        sbi_spec::base::UNAVAILABLE_EXTENSION.wrapping_add(1)
    }
}

impl<T: Console> Console for &T {
    #[inline]
    fn write(&self, bytes: Physical<&[u8]>) -> SbiRet {
        T::write(self, bytes)
    }
    #[inline]
    fn read(&self, bytes: Physical<&mut [u8]>) -> SbiRet {
        T::read(self, bytes)
    }
    #[inline]
    fn write_byte(&self, byte: u8) -> SbiRet {
        T::write_byte(self, byte)
    }
}

impl<T: Console> Console for Option<T> {
    #[inline]
    fn write(&self, bytes: Physical<&[u8]>) -> SbiRet {
        self.as_ref()
            .map_or(SbiRet::not_supported(), |inner| T::write(inner, bytes))
    }
    #[inline]
    fn read(&self, bytes: Physical<&mut [u8]>) -> SbiRet {
        self.as_ref()
            .map_or(SbiRet::not_supported(), |inner| T::read(inner, bytes))
    }
    #[inline]
    fn write_byte(&self, byte: u8) -> SbiRet {
        self.as_ref()
            .map_or(SbiRet::not_supported(), |inner| T::write_byte(inner, byte))
    }
    #[inline]
    fn _rustsbi_probe(&self) -> usize {
        match self {
            Some(_) => sbi_spec::base::UNAVAILABLE_EXTENSION.wrapping_add(1),
            None => sbi_spec::base::UNAVAILABLE_EXTENSION,
        }
    }
}
