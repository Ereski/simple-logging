|Crate|Documentation|Linux/OS X|Windows|
|:---:|:-----------:|:--------:|:-----:|
|[![Crate](https://img.shields.io/crates/v/simple-logging.svg)](https://crates.io/crates/simple-logging)|[![Documentation](https://docs.rs/simple-logging/badge.svg)](https://docs.rs/simple-logging/)|[![Build Status](https://travis-ci.org/Ereski/simple-logging.svg?branch=master)](https://travis-ci.org/Ereski/simple-logging)|[![Build Status](https://ci.appveyor.com/api/projects/status/github/Ereski/simple-logging?svg=true&branch=master)](https://ci.appveyor.com/project/Ereski/simple-logging)|

A simple logger for the [`log`](https://crates.io/crates/log) facade. One
log message is written per line. Each line also includes the time it was
logged, the logging level and the ID of the thread.

# Examples

Most users will simply need to call [`log_to_file()`](https://docs.rs/simple-logging/2/simple_logging/fn.log_to_file.html)
with the path to the log file and minimum log level:

```rust
use log::LevelFilter;

simple_logging::log_to_file("test.log", LevelFilter::Info);
```

Or use [`log_to_stderr()`](https://docs.rs/simple-logging/2/simple_logging/fn.log_to_stderr.html) if simply logging to
`stderr`:

```rust
use log::LevelFilter;

simple_logging::log_to_stderr(LevelFilter::Info);
```

For more control, [`log_to()`](https://docs.rs/simple-logging/2/simple_logging/fn.log_to.html) can be used with an
arbitrary sink implementing
[`Write`](https://doc.rust-lang.org/std/io/trait.Write.html) +
[`Send`](https://doc.rust-lang.org/std/marker/trait.Send.html) + `'static`:

```rust
use log::LevelFilter;
use std::io;

simple_logging::log_to(io::sink(), LevelFilter::Info);
```
