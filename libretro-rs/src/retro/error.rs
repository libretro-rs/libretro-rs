use core::fmt::{Display, Formatter};
use std::error::Error;

macro_rules! retro_error {
  ($name:ident, $description:expr) => {
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct $name(());

    impl $name {
      pub fn new() -> Self {
        Self(())
      }
    }

    impl Display for $name {
      fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, $description)
      }
    }

    impl Error for $name {}

    impl<T> From<::core::result::Result<T, Box<dyn Error>>> for $name {
      fn from(_value: ::core::result::Result<T, Box<dyn Error>>) -> Self {
        Self::new()
      }
    }
  };
}

retro_error!(CoreError, "a libretro API function call did not succeed");
retro_error!(CommandError, "a libretro environment command did not succeed");

impl From<CommandError> for CoreError {
  fn from(_value: CommandError) -> Self {
    Self::new()
  }
}

pub type Result<T> = ::core::result::Result<T, CoreError>;
