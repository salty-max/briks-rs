use std::fs::File;
use std::io::{self, Write};
use std::sync::Mutex;

// We will use a global mutex to protect the file access
static LOGGER: Mutex<Option<File>> = Mutex::new(None);

pub fn init() -> io::Result<()> {
    let file = File::create("debug.log")?;
    let mut guard = LOGGER.lock().unwrap();
    *guard = Some(file);
    Ok(())
}

pub fn log(msg: &str) {
    if let Ok(mut guard) = LOGGER.lock()
        && let Some(file) = guard.as_mut()
    {
        let _ = writeln!(file, "{}", msg);
    }
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logger::log(&format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_logging() {
        // Initialize
        init().unwrap();

        // Log something
        log!("Hello {}", "World");
        log!("Another line");

        // Verify content
        let content = fs::read_to_string("debug.log").unwrap();
        assert!(content.contains("Hello World"));
        assert!(content.contains("Another line"));

        // Cleanup
        fs::remove_file("debug.log").unwrap();
    }
}
