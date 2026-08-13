/// Prints formatted text to the installed machine console.
///
/// Output is discarded until [`crate::console::install`] succeeds.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::console::_print(core::format_args!($($arg)*));
    }};
}

/// Prints formatted text followed by a newline to the installed console.
#[macro_export]
macro_rules! println {
    () => {{
        $crate::print!("\n");
    }};
    ($($arg:tt)*) => {{
        $crate::console::_print(core::format_args!("{}\n", core::format_args!($($arg)*)));
    }};
}
