//! Clout is a *c*ommand *l*ine *out*put library.
//!
//! It provides a similar interface to the logging crate but with a different focus:
//! * clout's output is opinionated and not pluggable like logging
//! * clout provides output with sensible settings for use in command line tools
//!    * colours are supported for different message levels
//!    * output is always to stdout (for now)
//!
//! Many libraries already output messages to the logging framework, and you generally
//! don't want all these messages to get displayed to the end user. Clout allows you to
//! generate output using a logging-style API without having to filter all these messages.
//! (In fact you can use clout and logging together eg. by sending the logging messages to a file)
//!
//! clout includes an additional level between `Warn` and `Info`, called `Status` - this is intended
//! for most output messages.
//! This is because typically CLI tools provide 3 levels of verbosity (`-v`, `-vv`, and `-vvv` is
//! a common practice) but logging only provides two levels below info.

use std::fmt::{self, Display};
use std::error::Error;
use std::io::Write;
use termcolor::{WriteColor, ColorSpec, Color, StandardStream, ColorChoice};
use lazy_static::lazy_static;
use std::sync::Mutex;

/// The different levels of importance of a message
/// Also used to determine what level of messages should be displayed
#[derive(PartialOrd, PartialEq)]
pub enum Level {
    /// Display absolutely nothing
    Silent,
    /// Error is for messages indicating that an operation cannot proceed
    Error,
    /// Warn is for messages indicating that an operation will proceed
    /// but may not do what the user wanted
    Warn,
    /// Status is the usual messages that indicate what an operation is doing
    Status,
    /// Info is for messages that the user might find useful but are not essential
    Info,
    /// Debug is for messages that are useful for the developer, or when submitting
    /// bug reports, but are not useful for general use
    Debug,
    /// Trace is for messages that indicate at a low level what the operation is
    /// doing. Usually too noisy for a bug report, but might be used for debugging.
    Trace
}

impl Level {
    fn get_color(&self) -> ColorSpec {
        let mut spec = ColorSpec::new();

        match self {
            Level::Silent => {} // technically unreachable...
            Level::Error => {
                spec.set_fg(Some(Color::Red)).set_bold(true);
            }
            Level::Warn => {
                spec.set_fg(Some(Color::Yellow)).set_bold(true);
            }
            Level::Status => {
            }
            Level::Info => {
                spec.set_fg(Some(Color::White));
            }
            Level::Debug => {
                spec.set_fg(Some(Color::Cyan));
            }
            Level::Trace => {
                spec.set_fg(Some(Color::Magenta));
            }
        };

        spec
    }
}

/// Determine if clout should use colors for output
pub enum UseColor {
    /// Never use colour
    Never,
    /// Always use colour (even if stdout is a pipe)
    Always,
    /// Use colour but only on a terminal (no colour for pipes etc...)
    Auto,
}

impl Into<ColorChoice> for UseColor {
    fn into(self) -> ColorChoice {
        match self {
            UseColor::Never => ColorChoice::Never,
            UseColor::Always => ColorChoice::Auto,
            UseColor::Auto => if atty::is(atty::Stream::Stdout) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            },
        }
    }
}


/// Possible errors returned by clout
#[derive(Debug)]
pub enum CloutError {
    /// Tried to initialise clout when it's already initialised
    AlreadyInit,
    /// Tried to shutdown clout when it's already shutdown
    AlreadyShutdown,
}

impl Display for CloutError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            CloutError::AlreadyInit => write!(f, "clout already initialised"),
            CloutError::AlreadyShutdown => write!(f, "clout already shutdown"),
        }
    }
}

impl Error for CloutError {}


struct Clout {
    level: Level,
    write: StandardStream,
}

lazy_static! {
    static ref CLOUT: Mutex<Option<Clout>> = Mutex::new(None);
}

/// Builder to configuring clout
pub struct Builder {
    level: Level,
    use_color: UseColor,
}

impl Builder {
    /// Construct a new builder with default (Status level, Auto colour)
    pub fn new() -> Builder {
        Self { level: Level::Status, use_color: UseColor::Auto }
    }

    /// Set the message level
    pub fn with_level(mut self, level: Level) -> Builder {
        self.level = level;
        self
    }

    /// Set the message level from a verbosity flag
    /// This is useful for supporting flags like `-v`, `-vv` etc...
    ///
    /// * 0 (the default) => Status
    /// * 1 => Info level
    /// * 2 => Debug
    /// * 3 or greater => Trace
    pub fn with_verbose(mut self, verbose: u8) -> Builder {
        self.level = match verbose {
            0 => Level::Status,
            1 => Level::Info,
            2 => Level::Debug,
            _ => Level::Trace,
        };
        self
    }

    /// If `quiet` is true, set the message level to errors only. Otherwise do nothing.
    /// Useful for supporting a `-q` flag. Call this after calling [Builder::with_verbose].
    pub fn with_quiet(mut self, quiet: bool) -> Builder {
        if quiet {
            self.level = Level::Error;
        }
        self
    }

    /// If `silent` is true, disable all messages, even errors. Otherwise do nothing.
    /// Useful for supporting a `-s` flag. Call this after calling [Builder::with_verbose]
    /// and [Builder::with_quiet].
    pub fn with_silent(mut self, silent: bool) -> Builder {
        if silent {
            self.level = Level::Silent;
        }
        self
    }

    /// Set the colour usage mode.
    pub fn with_use_color(mut self, use_color: UseColor) -> Builder {
        self.use_color = use_color;
        self
    }

    fn build(self) -> Clout {
        Clout {
            level: self.level,
            write: StandardStream::stdout(self.use_color.into()),
        }
    }

    /// Finish configuring clout and install these settings.
    /// No messages may be emitted before this has been called.
    pub fn done(self) -> Result<(), CloutError> {
        let mut clout = CLOUT.lock().unwrap();

        if let Some(_) = *clout {
            Err(CloutError::AlreadyInit)
        } else {
            *clout = Some(self.build());
            Ok(())
        }
    }
}

/// Construct a new [Builder].
/// ```
/// clout::init()
///     .with_level(Level::Info)
///     .done()
///     .expect("failed to initialise clout");
/// ```
pub fn init() -> Builder {
    Builder::new()
}

/// Shutdown clout.
/// Not strictly necessary, but frees memory.
pub fn shutdown() -> Result<(), CloutError> {
    let mut clout = CLOUT.lock().unwrap();
    if clout.is_some() {
        *clout = None;
        Ok(())
    } else {
        Err(CloutError::AlreadyShutdown)
    }
}

fn with_clout<F>(f: F)
where F: FnOnce(&mut Clout) -> ()
{
    let mut clout = CLOUT.lock().unwrap();
    if let Some(ref mut inner) = *clout {
        f(inner)
    } else {
        panic!("attempt to output with clout before initialising")
    }
}

/// Emit a message with a given level and format_args.
/// Prefer the specific macros.
pub fn emit(level: Level, args: fmt::Arguments) {
    with_clout(|clout| {
        if clout.level < level {
            return;
        }

        clout.write.set_color(&level.get_color());
        clout.write.write_fmt(args);
        writeln!(clout.write);
        clout.write.reset();
    });
}

/// Emit an error message
#[macro_export]
macro_rules! error {
    ($($args:tt),+) => ($crate::emit($crate::Level::Error, format_args!($($args),+)))
}

/// Emit a warning message
#[macro_export]
macro_rules! warn {
    ($($args:tt),+) => ($crate::emit($crate::Level::Warn, format_args!($($args),+)))
}

/// Emit a status message
#[macro_export]
macro_rules! status {
    ($($args:tt),+) => ($crate::emit($crate::Level::Status, format_args!($($args),+)))
}

/// Emit an info message
#[macro_export]
macro_rules! info {
    ($($args:tt),+) => ($crate::emit($crate::Level::Info, format_args!($($args),+)))
}

/// Emit a debug message
#[macro_export]
macro_rules! debug {
    ($($args:tt),+) => ($crate::emit($crate::Level::Debug, format_args!($($args),+)))
}

/// Emit a trace message
#[macro_export]
macro_rules! trace {
    ($($args:tt),+) => ($crate::emit($crate::Level::Trace, format_args!($($args),+)))
}
