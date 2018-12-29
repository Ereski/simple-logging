//! A simple logger for the [`log`](https://crates.io/crates/log) facade. One
//! log message is written per line. Each line also includes the time it was
//! logged, the logging level and the ID of the thread.
//!
//! # Examples
//!
//! Most users will simply need to call [`log_to_file()`](fn.log_to_file.html)
//! with the path to the log file and minimum log level:
//!
//! ```rust
//! # extern crate log;
//! # extern crate simple_logging;
//! use log::LevelFilter;
//!
//! # fn main() {
//! simple_logging::log_to_file("test.log", LevelFilter::Info);
//! # }
//! ```
//!
//! Or use [`log_to_stderr()`](fn.log_to_stderr.html) if simply logging to
//! `stderr`:
//!
//! ```rust
//! # extern crate log;
//! # extern crate simple_logging;
//! use log::LevelFilter;
//!
//! # fn main() {
//! simple_logging::log_to_stderr(LevelFilter::Info);
//! # }
//! ```
//!
//! For more control, [`log_to()`](fn.log_to.html) can be used with an
//! arbitrary sink implementing
//! [`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) +
//! [`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) + `'static`:
//!
//! ```rust
//! # extern crate log;
//! # extern crate simple_logging;
//! use log::LevelFilter;
//! use std::io;
//!
//! # fn main() {
//! simple_logging::log_to(io::sink(), LevelFilter::Info);
//! # }
//! ```
//!
//! # Log format
//!
//! Each and every log message obeys the following fixed and easily-parsable
//! format:
//!
//! ```text
//! [<hh>:<mm>:<ss>.<SSS>] (<thread-id>) <level> <message>\n
//! ```
//!
//! Where `<hh>` denotes hours zero-padded to at least two digits, `<mm>`
//! denotes minutes zero-padded to two digits, `<ss>` denotes seconds
//! zero-padded to two digits and `<SSS>` denotes miliseconds zero-padded to
//! three digits. `<thread-id>` is an implementation-specific alphanumeric ID.
//! `<level>` is the log level as defined by `log::LogLevel` and padded right
//! with spaces. `<message>` is the log message. Note that `<message>` is
//! written to the log as-is, including any embedded newlines.
//!
//! # Errors
//!
//! Any errors returned by the sink when writing are ignored.
//!
//! # Performance
//!
//! The logger relies on a global `Mutex` to serialize access to the user
//! supplied sink.

#[macro_use]
extern crate lazy_static;
#[cfg(not(test))]
extern crate log;
extern crate thread_id;

#[cfg(test)]
#[macro_use]
extern crate log;
#[cfg(test)]
extern crate regex;

// TODO: include the changelog as a module when
// https://github.com/rust-lang/rust/issues/44732 stabilises

use log::{LevelFilter, Log, Metadata, Record};
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;
use std::time::Instant;

lazy_static! {
    static ref LOGGER: SimpleLogger = SimpleLogger {
        inner: Mutex::new(None),
    };
}

struct SimpleLogger {
    inner: Mutex<Option<SimpleLoggerInner>>,
}

impl SimpleLogger {
    // Set this `SimpleLogger`'s sink and reset the start time.
    fn renew<T: Write + Send + 'static>(&self, sink: T) {
        *self.inner.lock().unwrap() = Some(SimpleLoggerInner {
            start: Instant::now(),
            sink: Box::new(sink),
        });
    }
}

impl Log for SimpleLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        if let Some(ref mut inner) = *self.inner.lock().unwrap() {
            inner.log(record);
        }
    }

    fn flush(&self) {}
}

struct SimpleLoggerInner {
    start: Instant,
    sink: Box<Write + Send>,
}

impl SimpleLoggerInner {
    fn log(&mut self, record: &Record) {
        let now = self.start.elapsed();
        let seconds = now.as_secs();
        let hours = seconds / 3600;
        let minutes = (seconds / 60) % 60;
        let seconds = seconds % 60;
        let miliseconds = now.subsec_nanos() / 1_000_000;

        let _ = write!(
            self.sink,
            "[{:02}:{:02}:{:02}.{:03}] ({:x}) {:6} {}\n",
            hours,
            minutes,
            seconds,
            miliseconds,
            thread_id::get(),
            record.level(),
            record.args()
        );
    }
}

/// Configure the [`log`](https://crates.io/crates/log) facade to log to a file.
///
/// # Examples
///
/// ```rust
/// # extern crate log;
/// # extern crate simple_logging;
/// use log::LevelFilter;
///
/// # fn main() {
/// simple_logging::log_to_file("test.log", LevelFilter::Info);
/// # }
/// ```
pub fn log_to_file<T: AsRef<Path>>(
    path: T,
    max_log_level: LevelFilter,
) -> io::Result<()> {
    let file = File::create(path)?;
    log_to(file, max_log_level);

    Ok(())
}

/// Configure the [`log`](https://crates.io/crates/log) facade to log to
/// `stderr`.
///
/// # Examples
///
/// ```rust
/// # extern crate log;
/// # extern crate simple_logging;
/// use log::LevelFilter;
///
/// # fn main() {
/// simple_logging::log_to_stderr(LevelFilter::Info);
/// # }
/// ```
pub fn log_to_stderr(max_log_level: LevelFilter) {
    log_to(io::stderr(), max_log_level);
}

/// Configure the [`log`](https://crates.io/crates/log) facade to log to a
/// custom sink.
///
/// # Examples
///
/// ```rust
/// # extern crate log;
/// # extern crate simple_logging;
/// use log::LevelFilter;
/// use std::io;
///
/// # fn main() {
/// simple_logging::log_to(io::sink(), LevelFilter::Info);
/// # }
/// ```
pub fn log_to<T: Write + Send + 'static>(sink: T, max_log_level: LevelFilter) {
    LOGGER.renew(sink);
    log::set_max_level(max_log_level);
    // The only possible error is if this has been called before
    let _ = log::set_logger(&*LOGGER);
    // TODO: too much?
    assert_eq!(log::logger() as *const Log, &*LOGGER as *const Log);
}

#[cfg(test)]
mod tests {
    use log_to;

    use log::LevelFilter::Info;
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
        let buf = Arc::new(Mutex::new(Vec::new()));
        let proxy = VecProxy(buf.clone());
        log_to(proxy, Info);

        // Test filtering
        debug!("filtered");
        assert!(buf.lock().unwrap().is_empty());

        // Test message format
        let pat = Regex::new(
            r"^\[\d\d:\d\d:\d\d.\d\d\d] \([0-9a-zA-Z]+\) INFO   test\n$",
        )
        .unwrap();
        info!("test");
        let line = str::from_utf8(&buf.lock().unwrap()).unwrap().to_owned();
        assert!(pat.is_match(&line));
    }
}
