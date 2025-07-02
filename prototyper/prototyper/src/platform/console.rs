use bouffalo_hal::uart::RegisterBlock as BflbUartRegisterBlock;
use embedded_io::{Read, Write};
use uart_sifive::MmioUartSifive;
use uart_xilinx::MmioUartAxiLite;
use uart16550::{Register, Uart16550};
use crate::{platform::PreviousStage, sbi::console::ConsoleDevice};

pub(crate) const UART16650U8_COMPATIBLE: [&str; 1] = ["ns16550a"];
pub(crate) const UART16650U32_COMPATIBLE: [&str; 1] = ["snps,dw-apb-uart"];
pub(crate) const UARTAXILITE_COMPATIBLE: [&str; 1] = ["xlnx,xps-uartlite-1.00.a"];
pub(crate) const UARTBFLB_COMPATIBLE: [&str; 1] = ["bflb,bl808-uart"];
pub(crate) const UARTSIFIVE_COMPATIBLE: [&str; 1] = ["sifive,uart0"];

#[doc(hidden)]
#[allow(unused)]
#[derive(Clone, Copy, Debug)]
pub enum MachineConsoleType {
    Uart16550U8,
    Uart16550U32,
    UartAxiLite,
    UartBflb,
    UartSifive,
}

/// For Uart 16550
pub struct Uart16550Wrap<R: Register> {
    inner: *const Uart16550<R>,
}

impl<R: Register> Uart16550Wrap<R> {
    pub fn new(base: PreviousStage) -> Self {
        Self {
            inner: base.base as *const Uart16550<R>,
        }
    }
}

impl<R: Register> ConsoleDevice for Uart16550Wrap<R> {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        unsafe { (*self.inner).read(buf) }
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        unsafe { (*self.inner).write(buf) }
    }
}

/// For Uart AxiLite
impl ConsoleDevice for MmioUartAxiLite {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        Self::read(self, buf)
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        Self::write(self, buf)
    }
}

/// Wrapper of UartSifive, warp for initialization.
pub struct UartSifiveWrap {
    inner: MmioUartSifive,
}

impl UartSifiveWrap {
    pub fn new(addr: PreviousStage) -> Self {
        let inner = MmioUartSifive::new(addr.base);
        inner.disable_interrupt();
        inner.enable_read();
        inner.enable_write();
        // TODO: calcuate & set div register
        Self { inner }
    }
}

/// For Uart Sifive
impl ConsoleDevice for UartSifiveWrap {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        self.inner.read(buf)
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        self.inner.write(buf)
    }
}

/// For Uart BFLB
pub struct UartBflbWrap {
    inner: bouffalo_hal::uart::BlockingSerial<'static>,
}

impl UartBflbWrap {
    pub fn new(base: PreviousStage) -> Self {
        use bouffalo_hal::uart::BlockingSerial;
        Self {
            inner: unsafe { BlockingSerial::steal_freerun(base) },
        }
    }
}

impl bouffalo_hal::uart::Instance<'static> for PreviousStage {
    #[inline]
    fn register_block(self) -> &'static BflbUartRegisterBlock {
        unsafe { &*(self.base as *const BflbUartRegisterBlock) }
    }
}

impl ConsoleDevice for UartBflbWrap {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        self.inner.read(buf).unwrap()
    }

    fn write(&mut self, buf: &[u8]) -> usize {
        let ans = self.inner.write(buf).unwrap();
        self.inner.flush().unwrap();
        ans
    }
}
