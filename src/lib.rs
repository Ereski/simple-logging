//! A simple logger for the [`log`](https://crates.io/crates/log) facade. One
//! log message is written per line. Each line also includes the time it was
//! logged, the logging level and the ID of the thread. See
//! [`SimpleLogger`](struct.SimpleLogger.html) for more details.
//!
//! # Examples
//!
//! Most users will simply need to call [`log_to_file()`](fn.log_to_file.html)
//! with the path to the log file and minimum log level:
//!
//! ```rust
//! # extern crate log;
//! # extern crate simple_logging;
//! use log::LogLevelFilter;
//!
//! # fn main() {
//! simple_logging::log_to_file("test.log", LogLevelFilter::Info);
//! # }
//! ```
//!
//! Or use [`log_to_stderr()`](fn.log_to_stderr.html) if simply logging to
//! `stderr`:
//!
//! ```rust
//! # extern crate log;
//! # extern crate simple_logging;
//! use log::LogLevelFilter;
//!
//! # fn main() {
//! simple_logging::log_to_stderr(LogLevelFilter::Info);
//! # }
//! ```
//!
//! For more control, [`log_to()`](fn.log_to.html) can be used with an
//! arbitrary sink implementing
//! [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html):
//!
//! ```rust
//! # extern crate log;
//! # extern crate simple_logging;
//! use log::LogLevelFilter;
//! use std::io;
//!
//! # fn main() {
//! simple_logging::log_to(io::sink(), LogLevelFilter::Info);
//! # }
//! ```
//!
//! # Performance
//!
//! The logger relies on a global lock to serialize access to the user supplied
//! sink.

#[cfg(not(test))]
extern crate log;
extern crate thread_id;

#[cfg(test)]
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate regex;

use log::{Log, LogLevelFilter, LogMetadata, LogRecord, SetLoggerError};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

/// A simple logger for the [`log`](https://crates.io/crates/log) facade.
///
/// Each and every log message obeys the following fixed and easily-parsable
/// format:
///
/// ```text
/// [<hh>:<mm>:<ss>.<SSS>] (<thread-id>) <level> <message>\n
/// ```
///
/// Where `<hh>` denotes hours zero-padded to at least two digits, `<mm>`
/// denotes minutes zero-padded to two digits, `<ss>` denotes seconds
/// zero-padded to two digits and `<SSS>` denotes miliseconds zero-padded to
/// three digits. `<thread-id>` is an implementation-specific alphanumeric ID.
/// `<level>` is the log level as defined by `log::LogLevel` and padded right
/// with spaces. `<message>` is the log message. Note that `<message>` is
/// written to the log as-is, including any embedded newlines.
///
/// # Examples
///
/// `SimpleLogger` implements
/// [`log::Log`](https://docs.rs/log/0.3/log/trait.Log.html), and as such may
/// be used with
/// [`log::set_logger`](https://docs.rs/log/0.3/log/fn.set_logger.html)
/// directly:
///
/// ```rust
/// # extern crate log;
/// # extern crate simple_logging;
/// use log::LogLevelFilter;
/// use simple_logging::SimpleLogger;
/// use std::io;
///
/// # fn main() {
/// log::set_logger(|max_log_level| {
///     max_log_level.set(LogLevelFilter::Info);
///
///     Box::new(SimpleLogger::new(io::sink()))
/// });
/// # }
/// ```
///
/// However, because this is the expected way to use `SimpleLogger`, there is a
/// shorthand function for the operation above: [`log_to()`](fn.log_to.html).
///
/// # Errors
///
/// Any errors returned by the sink when writing are ignored.
///
/// # See also
///
/// [`log_to_file()`](fn.log_to_file.html),
/// [`log_to_stderr()`](fn.log_to_stderr.html), [`log_to()`](fn.log_to.html)
pub struct SimpleLogger<T: Write> {
    start: Instant,
    sink:  Mutex<T>,
}

impl<T: Write> SimpleLogger<T> {
    /// Create a new `SimpleLogger`.
    pub fn new(sink: T) -> Self {
        SimpleLogger {
            start: Instant::now(),
            sink:  Mutex::new(sink),
        }
    }
}

impl<T: Write + Send + Sync> Log for SimpleLogger<T> {
    fn enabled(&self, _: &LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &LogRecord) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let now         = self.start.elapsed();
        let seconds     = now.as_secs();
        let hours       = seconds / 3600;
        let minutes     = (seconds / 60) % 60;
        let seconds     = seconds % 60;
        let miliseconds = now.subsec_nanos() / 1000000;

        let mut sink = self.sink.lock().unwrap();
        let _        = write!(sink,
            "[{:02}:{:02}:{:02}.{:03}] ({}) {:6} {}\n", hours, minutes,
            seconds, miliseconds, thread_id::get(), record.level(),
            record.args());
    }
}

/// Configure the [`log`](https://crates.io/crates/log) facade to log to a file
/// through a [`SimpleLogger`](struct.SimpleLogger.html).
///
/// # Examples
///
/// ```rust
/// # extern crate log;
/// # extern crate simple_logging;
/// use log::LogLevelFilter;
///
/// # fn main() {
/// simple_logging::log_to_file("test.log", LogLevelFilter::Info);
/// # }
/// ```
pub fn log_to_file<T: AsRef<Path>>(path: T, max_log_level: LogLevelFilter)
        -> io::Result<()> {
    let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)?;

    log_to(file, max_log_level)
        // Wrap SetLoggerError into an io::Error just to avoid defining a new
        // error type
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err))
}

/// Configure the [`log`](https://crates.io/crates/log) facade to log to
/// `stderr` through a [`SimpleLogger`](struct.SimpleLogger.html).
///
/// # Examples
///
/// ```rust
/// # extern crate log;
/// # extern crate simple_logging;
/// use log::LogLevelFilter;
///
/// # fn main() {
/// simple_logging::log_to_stderr(LogLevelFilter::Info);
/// # }
/// ```
pub fn log_to_stderr(max_log_level: LogLevelFilter)
        -> Result<(), SetLoggerError> {
    log_to(io::stderr(), max_log_level)
}

/// Configure the [`log`](https://crates.io/crates/log) facade to log to a
/// custom sink through a [`SimpleLogger`](struct.SimpleLogger.html).
///
/// # Examples
///
/// ```rust
/// # extern crate log;
/// # extern crate simple_logging;
/// use log::LogLevelFilter;
/// use std::io;
///
/// # fn main() {
/// simple_logging::log_to(io::sink(), LogLevelFilter::Info);
/// # }
/// ```
pub fn log_to<T: Write + Send + Sync + 'static>(sink: T,
        max_log_level: LogLevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(|log_max_log_level| {
        log_max_log_level.set(max_log_level);

        Box::new(SimpleLogger::new(sink))
    })
}

#[cfg(test)]
mod tests {
    use log_to;

    use log::LogLevelFilter::Info;
    use regex::Regex;
    use std::io;
    use std::io::Write;
    use std::str;
    use std::sync::{Arc, Mutex};

    struct VecProxy(Arc<Mutex<Vec<u8>>>);

    impl Write for VecProxy {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().write(buf)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    // The `log` API forbids calling `set_logger()` more than once in the
    // lifetime of a single program (even after a `shutdown_logger()`), so
    // we stash all tests in a single function.
    // TODO: increase coverage by making `log_to*()` tests integration tests.
    #[test]
    fn test() {
        let buf   = Arc::new(Mutex::new(Vec::new()));
        let proxy = VecProxy(buf.clone());
        log_to(proxy, Info).unwrap();

        // Test filtering
        debug!("filtered");
        assert!(buf.lock().unwrap().is_empty());

        // Test message format
        let pat  = Regex::new(r"^\[\d\d:\d\d:\d\d.\d\d\d] \([0-9a-zA-Z]+\) INFO   test\n$").unwrap();
        info!("test");
        let line = str::from_utf8(&buf.lock().unwrap()).unwrap().to_owned();
        assert!(pat.is_match(&line));
    }
}
