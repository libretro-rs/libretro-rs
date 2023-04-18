use crate::ffi::retro_log_level::*;
use crate::ffi::*;
use c_utf8::*;

/// Trait for types that provide safe access to [`retro_log_printf_t`].
pub trait LogInterface {
  fn log(&mut self, level: retro_log_level, message: &CUtf8);
}

/// Trait for types that offer idiomatic logging methods.
pub trait Logger {
  /// Logs a debugging message.
  fn debug(&mut self, message: &CUtf8);
  /// Logs an informational message.
  fn info(&mut self, message: &CUtf8);
  /// Logs a warning message.
  fn warn(&mut self, message: &CUtf8);
  /// Logs an error message.
  fn error(&mut self, message: &CUtf8);
}

impl<T> Logger for T
where
  T: LogInterface,
{
  fn debug(&mut self, message: &CUtf8) {
    self.log(RETRO_LOG_DEBUG, message);
  }

  fn info(&mut self, message: &CUtf8) {
    self.log(RETRO_LOG_INFO, message);
  }

  fn warn(&mut self, message: &CUtf8) {
    self.log(RETRO_LOG_WARN, message);
  }

  fn error(&mut self, message: &CUtf8) {
    self.log(RETRO_LOG_ERROR, message);
  }
}

type RetroPrintF = unsafe extern "C" fn(level: retro_log_level, fmt: *const c_char, ...);

/// The platform-specific [Logger] provided by [RetroEnvironment::get_log_interface].
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformLogger(RetroPrintF);

impl PlatformLogger {
  pub fn new(callback: RetroPrintF) -> Self {
    Self(callback)
  }
}

impl LogInterface for PlatformLogger {
  fn log(&mut self, level: retro_log_level, message: &CUtf8) {
    unsafe { self.0(level, c_utf8!("%s\n").as_ptr(), message.as_ptr()) }
  }
}

/// A [Logger] that logs to [std::io::Stderr].
/// Primarily used as a fallback when [PlatformLogger] isn't available.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StderrLogger;

impl LogInterface for StderrLogger {
  fn log(&mut self, level: retro_log_level, message: &CUtf8) {
    let label: &'static str = match level {
      RETRO_LOG_DEBUG => "DEBUG",
      RETRO_LOG_INFO => "INFO",
      RETRO_LOG_WARN => "WARN",
      RETRO_LOG_ERROR => "ERROR",
      _ => return,
    };
    eprintln!("[libretro {}] {}", label, message.as_str());
  }
}

/// A [Logger] that uses [StderrLogger] if no [PlatformLogger] is available.
#[derive(Clone, Copy)]
pub struct FallbackLogger<T> {
  callback: fn(Option<&mut T>, retro_log_level, &CUtf8),
  logger: Option<T>,
}

impl<T> FallbackLogger<T>
where
  T: LogInterface,
{
  pub fn new(logger: Option<T>) -> Self {
    match logger {
      Some(_) => Self {
        callback: log_to_logger,
        logger,
      },
      None => Self {
        callback: log_to_stderr,
        logger,
      },
    }
  }
}

impl<T> From<Option<T>> for FallbackLogger<T>
where
  T: LogInterface,
{
  fn from(logger: Option<T>) -> Self {
    FallbackLogger::new(logger)
  }
}

impl<T> LogInterface for FallbackLogger<T>
where
  T: LogInterface,
{
  fn log(&mut self, level: retro_log_level, message: &CUtf8) {
    (self.callback)(self.logger.as_mut(), level, message);
  }
}

fn log_to_logger<T>(cb: Option<&mut T>, level: retro_log_level, msg: &CUtf8)
where
  T: LogInterface,
{
  // Safety: cb was checked in new
  unsafe { cb.unwrap_unchecked() }.log(level, msg);
}

fn log_to_stderr<T>(_cb: Option<&mut T>, level: retro_log_level, msg: &CUtf8)
where
  T: LogInterface,
{
  StderrLogger.log(level, msg);
}

/// A [Logger] that discards all messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NullLogger;

impl LogInterface for NullLogger {
  fn log(&mut self, _level: retro_log_level, _message: &CUtf8) {}
}
