use std::sync::atomic::{AtomicPtr, Ordering};
use std::fmt::{self, Display};
use std::error::Error;
use std::io::Write;
use termcolor::{WriteColor, ColorSpec, Color, StandardStream, ColorChoice};

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

pub enum UseColor {
    Never,
    Always,
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


#[derive(Debug)]
pub enum CloutError {
    AlreadyInit,
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
    write: Box<dyn WriteColor>,
}

static CLOUT: AtomicPtr<Clout> = AtomicPtr::new(std::ptr::null_mut());


pub struct Builder {
    level: Level,
    use_color: UseColor,
}

impl Builder {
    pub fn new() -> Builder {
        Self { level: Level::Status, use_color: UseColor::Auto }
    }

    pub fn with_level(mut self, level: Level) -> Builder {
        self.level = level;
        self
    }

    pub fn with_verbose(mut self, verbose: u8) -> Builder {
        self.level = match verbose {
            0 => Level::Status,
            1 => Level::Info,
            2 => Level::Debug,
            _ => Level::Trace,
        };
        self
    }

    pub fn with_quiet(mut self, quiet: bool) -> Builder {
        if quiet {
            self.level = Level::Error;
        }
        self
    }

    pub fn with_silent(mut self, quiet: bool) -> Builder {
        if quiet {
            self.level = Level::Silent;
        }
        self
    }

    pub fn with_use_color(mut self, use_color: UseColor) -> Builder {
        self.use_color = use_color;
        self
    }

    fn build(self) -> Box<Clout> {
        Box::new(Clout {
            level: self.level,
            write: Box::new(StandardStream::stdout(self.use_color.into())),
        })
    }

    pub fn done(self) -> Result<(), CloutError> {
        let clout = Box::into_raw(self.build());
        let prev = CLOUT.compare_and_swap(std::ptr::null_mut(), clout, Ordering::Relaxed);

        if prev != std::ptr::null_mut() {
            Err(CloutError::AlreadyInit)
        } else {
            Ok(())
        }
    }
}


pub fn init() -> Builder {
    Builder::new()
}

pub fn shutdown() -> Result<(), CloutError> {
    let cur = CLOUT.load(Ordering::Relaxed);
    let prev = CLOUT.compare_and_swap(cur, std::ptr::null_mut(), Ordering::Relaxed);

    if cur.is_null() || prev != cur {
        Err(CloutError::AlreadyShutdown)
    } else {
        unsafe { Box::from_raw(cur) };
        Ok(())
    }
}

fn get_clout() -> &'static mut Clout {
    let clout = CLOUT.load(Ordering::Relaxed);
    unsafe {
        clout.as_mut().expect("attempt to output with clout before initialising")
    }
}


pub fn emit(level: Level, args: fmt::Arguments) {
    let clout = get_clout();

    if clout.level < level {
        return;
    }

    clout.write.set_color(&level.get_color());
    clout.write.write_fmt(args);
    writeln!(clout.write);
    clout.write.reset();
}

#[macro_export]
macro_rules! error {
    ($args:tt) => ($crate::emit($crate::Level::Error, format_args!($args)))
}

#[macro_export]
macro_rules! warn {
    ($args:tt) => ($crate::emit($crate::Level::Warn, format_args!($args)))
}

#[macro_export]
macro_rules! status {
    ($args:tt) => ($crate::emit($crate::Level::Status, format_args!($args)))
}

#[macro_export]
macro_rules! info {
    ($args:tt) => ($crate::emit($crate::Level::Info, format_args!($args)))
}

#[macro_export]
macro_rules! debug {
    ($args:tt) => ($crate::emit($crate::Level::Debug, format_args!($args)))
}

#[macro_export]
macro_rules! trace {
    ($args:tt) => ($crate::emit($crate::Level::Trace, format_args!($args)))
}
