//! Installable early console and formatted output support.
//!
//! The runtime owns only publication and serialization. A board, bootloader,
//! or RTOS layer owns the concrete UART or other transport and installs it once
//! with [`install`].

use core::cell::UnsafeCell;
use core::fmt::{self, Write};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const EMPTY: u8 = 0;
const INSTALLING: u8 = 1;
const READY: u8 = 2;

/// A synchronous byte sink suitable for the early machine console.
pub trait ConsoleDevice: Sync {
    /// Writes as many leading bytes as possible and returns the count written.
    ///
    /// Returning zero stops the current formatted write.
    fn write(&self, bytes: &[u8]) -> usize;
}

/// Error returned when a console has already been installed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlreadyInstalled;

struct ConsoleSlot {
    state: AtomicU8,
    device: UnsafeCell<Option<&'static dyn ConsoleDevice>>,
}

// `state` publishes the UnsafeCell contents once; installed devices are Sync.
unsafe impl Sync for ConsoleSlot {}

static CONSOLE: ConsoleSlot = ConsoleSlot {
    state: AtomicU8::new(EMPTY),
    device: UnsafeCell::new(None),
};
static PRINT_LOCK: AtomicBool = AtomicBool::new(false);

/// Installs the process-wide early console.
///
/// Installation is one-shot so that a writer can never retain a stale device
/// while another hart replaces it. The device may itself be a forwarding
/// adapter if a later RTOS wants to change the actual output route.
pub fn install(device: &'static dyn ConsoleDevice) -> Result<(), AlreadyInstalled> {
    CONSOLE
        .state
        .compare_exchange(EMPTY, INSTALLING, Ordering::Acquire, Ordering::Relaxed)
        .map_err(|_| AlreadyInstalled)?;

    // SAFETY: the successful state transition gives this caller exclusive
    // write access, and `device` remains valid for the entire program.
    unsafe { *CONSOLE.device.get() = Some(device) };
    CONSOLE.state.store(READY, Ordering::Release);
    Ok(())
}

/// Returns whether a console backend has been published.
pub fn is_installed() -> bool {
    CONSOLE.state.load(Ordering::Acquire) == READY
}

fn installed() -> Option<&'static dyn ConsoleDevice> {
    if !is_installed() {
        return None;
    }

    // SAFETY: READY is only stored with Release after the slot is initialized;
    // the value is immutable after publication.
    unsafe { *CONSOLE.device.get() }
}

struct PrintGuard;

impl PrintGuard {
    fn acquire() -> Self {
        while PRINT_LOCK
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        Self
    }
}

impl Drop for PrintGuard {
    fn drop(&mut self) {
        PRINT_LOCK.store(false, Ordering::Release);
    }
}

struct ConsoleWriter {
    device: &'static dyn ConsoleDevice,
}

impl Write for ConsoleWriter {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let mut bytes = text.as_bytes();
        while !bytes.is_empty() {
            let written = self.device.write(bytes).min(bytes.len());
            if written == 0 {
                return Err(fmt::Error);
            }
            bytes = &bytes[written..];
        }
        Ok(())
    }
}

/// Formatting target used by [`crate::print!`] and [`crate::println!`].
#[doc(hidden)]
pub fn _print(args: fmt::Arguments<'_>) {
    let Some(device) = installed() else {
        return;
    };

    let _guard = PrintGuard::acquire();
    let _ = ConsoleWriter { device }.write_fmt(args);
}

#[cfg(test)]
mod tests {
    extern crate std;

    use self::std::{sync::Mutex, vec::Vec};
    use super::{ConsoleDevice, install};

    struct OneByteAtATime(Mutex<Vec<u8>>);

    impl ConsoleDevice for OneByteAtATime {
        fn write(&self, bytes: &[u8]) -> usize {
            self.0.lock().unwrap().push(bytes[0]);
            1
        }
    }

    static DEVICE: OneByteAtATime = OneByteAtATime(Mutex::new(Vec::new()));

    #[test]
    fn print_handles_partial_writes_inside_utf8_characters() {
        install(&DEVICE).unwrap();
        crate::println!("hart α: {}", 7);
        assert_eq!(&*DEVICE.0.lock().unwrap(), "hart α: 7\n".as_bytes());
    }
}
