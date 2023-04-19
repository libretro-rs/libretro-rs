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
  };
}

retro_error!(LoadGameError, "failed to load game");
retro_error!(SerializeError, "failed to serialize state");
retro_error!(UnserializeError, "failed to unserialize state");
retro_error!(CommandError, "failed to execute environment command");

impl From<CommandError> for LoadGameError {
  fn from(_value: CommandError) -> Self {
    Self::default()
  }
}
