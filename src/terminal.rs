//! The `terminal` module provides a low-level abstraction over the terminal device.
//!
//! It handles the interaction with the operating system to:
//! * Enter and exit **Raw Mode** (disabling canonical input and local echo).
//! * Read and write to the underlying TTY file descriptor.
//! * Query terminal capabilities like window size.
//!
//! # Architecture
//! This module uses the **Dependency Injection** pattern to facilitate testing.
//! * [`System`]: A trait defining the low-level OS operations.
//! * [`LibcSystem`]: The production implementation using `libc` FFI.
//! * [`Terminal`]: The high-level wrapper used by the application.

use std::ffi::c_void;
use std::io;
use std::os::fd::RawFd;

/// Abstraction over system calls relative to the terminal.
///
/// This trait acts as a "seam" for testing, allowing the `Terminal` struct to
/// interact with a mock OS during unit tests instead of making real syscalls.
pub trait System {
    /// Opens a file descriptor to the current TTY (usually `/dev/tty`).
    fn open_tty(&self) -> io::Result<RawFd>;

    /// Enables "Raw Mode" on the specified file descriptor.
    ///
    /// This disables:
    /// * **Canonical Mode** (line buffering).
    /// * **Echo** (displaying typed characters).
    /// * **Signal Processing** (Ctrl+C, Ctrl+Z handling by the OS).
    ///
    /// Returns the original `termios` configuration so it can be restored later.
    fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios>;

    /// Restores the terminal to its original configuration (Canonical Mode).
    fn disable_raw(&self, fd: RawFd, original: &libc::termios) -> io::Result<()>;

    /// queries the kernel for the current terminal window size (cols, rows).
    fn get_window_size(&self, fd: RawFd) -> io::Result<(u16, u16)>;

    /// Reads raw bytes from the file descriptor into the buffer.
    fn read(&self, fd: RawFd, buf: &mut [u8]) -> io::Result<usize>;

    /// Writes raw bytes from the buffer to the file descriptor.
    fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize>;
}

/// The production implementation of [`System`] using `libc` calls.
///
/// This struct performs `unsafe` FFI calls to the underlying OS.
pub struct LibcSystem;

impl System for LibcSystem {
    fn open_tty(&self) -> io::Result<RawFd> {
        unsafe {
            // map_err is needed because CString::new fails if the string contains null bytes,
            // returning a NulError which doesn't auto-convert to io::Error.
            let path = std::ffi::CString::new("/dev/tty")
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?;

            let fd = libc::open(path.as_ptr(), libc::O_RDWR);
            if fd < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(fd)
        }
    }

    fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios> {
        unsafe {
            let mut termios = std::mem::zeroed();

            // Load current attributes
            if libc::tcgetattr(fd, &mut termios) < 0 {
                return Err(io::Error::last_os_error());
            }

            let original = termios;

            // Input flags: Disable software flow control & special handling
            termios.c_iflag &=
                !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);

            // Output flags: Disable post-processing (e.g. \n -> \r\n translation)
            termios.c_oflag &= !(libc::OPOST);

            // Control flags: Set 8 bits per character
            termios.c_cflag |= libc::CS8;

            // Local flags: Disable Echo, Canonical mode, Extended input, and Signals
            termios.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);

            // Apply changes immediately (TCSAFLUSH drains output before changing)
            if libc::tcsetattr(fd, libc::TCSAFLUSH, &termios) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok(original)
        }
    }

    fn disable_raw(&self, fd: RawFd, original: &libc::termios) -> io::Result<()> {
        unsafe {
            if libc::tcsetattr(fd, libc::TCSAFLUSH, original) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
    }

    fn get_window_size(&self, fd: RawFd) -> io::Result<(u16, u16)> {
        unsafe {
            let mut winsize = libc::winsize {
                ws_col: 0,
                ws_row: 0,
                ws_xpixel: 0,
                ws_ypixel: 0,
            };

            // TIOCGWINSZ = Terminal IO Control Get Window SiZe
            if libc::ioctl(fd, libc::TIOCGWINSZ, &mut winsize) < 0 {
                return Err(io::Error::last_os_error());
            }

            Ok((winsize.ws_col, winsize.ws_row))
        }
    }

    fn read(&self, fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
        unsafe {
            let bytes = libc::read(fd, buf.as_mut_ptr() as *mut c_void, buf.len());
            if bytes < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(bytes as usize)
        }
    }

    fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize> {
        unsafe {
            let bytes = libc::write(fd, buf.as_ptr() as *const c_void, buf.len());
            if bytes < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(bytes as usize)
        }
    }
}

use std::fmt;

/// A high-level wrapper around the terminal state and I/O.
///
/// This struct manages the lifecycle of **Raw Mode**.
/// * On creation: It opens the TTY and enables Raw Mode.
/// * On drop: It automatically restores the original terminal configuration.
pub struct Terminal {
    /// The abstract system backend (Real or Mock).
    system: Box<dyn System>,
    /// The file descriptor of the active TTY.
    fd: RawFd,
    /// The original terminal attributes, preserved for restoration on exit.
    original_termios: Option<libc::termios>,
}

impl fmt::Debug for Terminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Terminal")
            .field("fd", &self.fd)
            .finish_non_exhaustive()
    }
}

impl Terminal {
    /// Creates a new `Terminal` instance using the default `LibcSystem`.
    ///
    /// This will attempt to open `/dev/tty` and enter Raw Mode immediately.
    pub fn new() -> io::Result<Self> {
        Self::new_with_system(Box::new(LibcSystem))
    }

    /// Creates a new `Terminal` with a specific system backend.
    ///
    /// This is primarily used for dependency injection in tests.
    pub fn new_with_system(system: Box<dyn System>) -> io::Result<Self> {
        let fd = system.open_tty()?;
        let termios = system.enable_raw(fd)?;
        Ok(Self {
            system,
            fd,
            original_termios: Some(termios),
        })
    }

    /// Returns the current size of the terminal as `(cols, rows)`.
    pub fn size(&self) -> io::Result<(u16, u16)> {
        self.system.get_window_size(self.fd)
    }

    /// Reads raw bytes from the terminal into the provided buffer.
    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        self.system.read(self.fd, buf)
    }

    /// Writes raw bytes to the terminal.
    pub fn write(&self, buf: &[u8]) -> io::Result<usize> {
        self.system.write(self.fd, buf)
    }
}

/// Automatically restores the terminal configuration when the struct goes out of scope.
impl Drop for Terminal {
    fn drop(&mut self) {
        if let Some(termios) = self.original_termios
            && let Err(e) = self.system.disable_raw(self.fd, &termios)
        {
            log!("Error restoring terminal: {}", e);
        }
    }
}

// ... existing test modules (integration_tests and tests) ...
// (I have omitted the test code here for brevity as it remains unchanged,
//  but you should keep it in your file)

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_libc_system_open_tty() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open /dev/tty");
        assert!(fd > 0);
        unsafe { libc::close(fd) };
    }

    #[test]
    #[ignore]
    fn test_libc_system_raw_mode() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open TTY");

        let original = sys.enable_raw(fd).expect("Failed to enable raw");

        let mut current: libc::termios = unsafe { std::mem::zeroed() };
        unsafe { libc::tcgetattr(fd, &mut current) };
        assert_eq!(current.c_lflag & libc::ECHO, 0, "ECHO should be disabled");

        sys.disable_raw(fd, &original)
            .expect("Failed to disable raw");

        unsafe { libc::tcgetattr(fd, &mut current) };
        assert_eq!(
            current.c_lflag & libc::ECHO,
            original.c_lflag & libc::ECHO,
            "ECHO state should be restored"
        );

        unsafe { libc::close(fd) };
    }

    #[test]
    #[ignore]
    fn test_libc_system_io() {
        let sys = LibcSystem;
        let fd = sys.open_tty().expect("Failed to open TTY");

        let msg = b"Integration Test: Hello World\r\n";
        let written = sys.write(fd, msg).expect("Failed to write");
        assert_eq!(written, msg.len());

        let size = sys.get_window_size(fd).expect("Failed to get window size");
        assert!(size.0 > 0);
        assert!(size.1 > 0);

        unsafe { libc::close(fd) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mocks::MockSystem;

    #[test]
    fn test_terminal_initialization() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();
        let _term = Terminal::new_with_system(Box::new(mock)).unwrap();

        let log = log_ref.lock().unwrap();
        // Check that log contains specific strings
        assert!(log.contains(&"open_tty".to_string()));
        // Check that at least one entry starts with "enable_raw"
        assert!(log.iter().any(|entry| entry.starts_with("enable_raw")));
    }

    #[test]
    fn test_lifecycle_and_delegation() {
        let mock = MockSystem::new();
        let log_ref = mock.log.clone();

        {
            let term = Terminal::new_with_system(Box::new(mock)).expect("Failed to init terminal");

            term.size().unwrap();
            term.write(b"foo").unwrap();

            let mut buf = [0u8; 10];
            term.read(&mut buf).unwrap();
        } // Drop happens here

        let log = log_ref.lock().unwrap();
        // Note: Indices depend on exact call order.
        assert_eq!(log[0], "open_tty");
        // enable_raw(100) -> 100 is the hardcoded FD in the Mock
        assert_eq!(log[1], "enable_raw(100)");
        assert_eq!(log[2], "get_window_size(100)");
        assert_eq!(log[3], "write(100, 3 bytes)");
        assert_eq!(log[4], "read(100)");
        assert_eq!(log[5], "disable_raw(100)");
    }

    #[test]
    fn test_initialization_failure_open() {
        let mut mock = MockSystem::new();
        mock.fail_open = true;

        let res = Terminal::new_with_system(Box::new(mock));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::Other);
    }

    #[test]
    fn test_initialization_failure_enable_raw() {
        let mut mock = MockSystem::new();
        mock.fail_enable_raw = true;

        let res = Terminal::new_with_system(Box::new(mock));
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), io::ErrorKind::Other);
    }
}

#[cfg(test)]
pub(crate) mod mocks {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    pub struct MockSystem {
        pub log: Arc<Mutex<Vec<String>>>,
        pub input_buffer: Arc<Mutex<Vec<u8>>>,
        pub fail_open: bool,
        pub fail_enable_raw: bool,
    }

    impl MockSystem {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn push_input(&self, data: &[u8]) {
            self.input_buffer.lock().unwrap().extend_from_slice(data);
        }

        fn push_log(&self, msg: &str) {
            self.log.lock().unwrap().push(msg.to_string());
        }
    }

    impl System for MockSystem {
        fn open_tty(&self) -> io::Result<RawFd> {
            self.push_log("open_tty");
            if self.fail_open {
                return Err(io::Error::new(io::ErrorKind::Other, "Mock Open Failed"));
            }
            Ok(100)
        }

        fn enable_raw(&self, fd: RawFd) -> io::Result<libc::termios> {
            self.push_log(&format!("enable_raw({})", fd));
            if self.fail_enable_raw {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "Mock Enable Raw Failed",
                ));
            }
            // Return empty termios
            Ok(unsafe { std::mem::zeroed() })
        }

        fn disable_raw(&self, fd: RawFd, _original: &libc::termios) -> io::Result<()> {
            self.push_log(&format!("disable_raw({})", fd));
            Ok(())
        }

        fn get_window_size(&self, fd: RawFd) -> io::Result<(u16, u16)> {
            self.push_log(&format!("get_window_size({})", fd));
            Ok((80, 24))
        }

        fn read(&self, fd: RawFd, buf: &mut [u8]) -> io::Result<usize> {
            self.push_log(&format!("read({})", fd));
            let mut input = self.input_buffer.lock().unwrap();
            if input.is_empty() {
                return Ok(0);
            }
            let len = std::cmp::min(buf.len(), input.len());
            // copy_from_slice handles copying data
            buf[..len].copy_from_slice(&input[..len]);
            // Remove read bytes from the "mock input stream"
            input.drain(0..len);
            Ok(len)
        }

        fn write(&self, fd: RawFd, buf: &[u8]) -> io::Result<usize> {
            self.push_log(&format!("write({}, {} bytes)", fd, buf.len()));
            Ok(buf.len())
        }
    }
}
