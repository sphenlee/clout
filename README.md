# Clout

Clout is a *c*ommand *l*ine *out*put library.

It provides a similar interface to the logging crate but with a different focus:
* clout's output is opinionated and not pluggable like logging
* clout provides output with sensible settings for use in command line tools
   * colours are supported for different message levels
   * output is always to stdout (for now)
   * different output levels are selected using settings aligned with typical
     command line argument conventions

Many libraries already output messages to the logging framework, and you generally
don't want all these messages to get displayed to the end user. Clout allows you to
generate output using a logging-style API without having to filter all these messages.
(In fact you can use clout and logging together eg. by sending the logging messages to a file)

clout includes an additional level between `Warn` and `Info`, called `Status` - this is intended
for most output messages.
This is because typically CLI tools provide 3 levels of verbosity (`-v`, `-vv`, and `-vvv` is
a common practice) but logging only provides two levels below info.

## Quickstart Usage

### Setup
To use clout with the default settings:
```rust
    clout::init()
        .done()
        .expect("clout failed to init");
```
This will output messages at `Status` level or higher, and will
use terminal colours when available.

### Messages
Macros similar to logging can be used to output messages.
Clout must be initialised before any messages are output.
Six levels are available - see the documentation for `clout::Level` for
guidance about which level to use.
```rust
    error!("an error");
    warn!("a warining");
    status!("a status message");
    info!("information message");
    debug!("debug message");
    trace!("tracing message");
```
If you're using the `logging` crate at the same time it is suggested to use
these via a `clout::` path.

### Formatting
The macros support the usual formatting based on the `format!` macro.
```rust
    status!("a formatted {}", "message");
```

### More Options
```rust
    clout::init()
        .with_verbose(4)
        .with_quiet(false)
        .with_silent(false)
        .with_use_color(clout::UseColor::Auto)
        .done()
        .expect("clout failed to init");
``` 

* `with_verbose` lets you select a verbosity level from a number. This is useful
  to support `-v`, `-vv` etc.
* `with_quiet` will hide everything except `Error` messages
* `with_silent` will silence everything
* `with_use_color` will control when clout uses colors

For more details see the documentation for `clout::Builder`.

### Shutdown
Clout can be shutdown to prevent memory leaks if you care about that.
```rust
    clout::shutdown().expect("failed to shutdown clout");
```
