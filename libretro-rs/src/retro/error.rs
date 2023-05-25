use core::fmt::{Display, Formatter};
use std::error::Error;
use std::fmt::Debug;

#[derive(Clone)]
pub struct LoadGameError<C>(C);

impl<T> LoadGameError<T> {
  pub fn new(init_state: T) -> Self {
    Self(init_state)
  }

  pub fn into_inner(self) -> T {
    self.0
  }
}

impl<T> Debug for LoadGameError<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "LoadGameError")
  }
}

impl<T> Display for LoadGameError<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "core failed to load game")
  }
}

impl<T> Error for LoadGameError<T> {}

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
retro_error!(
  CommandError,
  "a libretro environment command did not succeed"
);

impl From<CommandError> for CoreError {
  fn from(_value: CommandError) -> Self {
    Self::new()
  }
}

impl<T> From<crate::retro::av::pixel::Format<T>> for CoreError {
  fn from(_value: crate::retro::av::pixel::Format<T>) -> Self {
    Self::new()
  }
}
