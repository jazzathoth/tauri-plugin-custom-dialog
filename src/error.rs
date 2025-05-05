use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  // #[error(transparent)]
  // Io(#[from] std::io::Error),
  Tauri(String),
  WindowNotFound(String),
  UrlParse(String),
  DialogSetup(String),
}

impl Serialize for Error {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string().as_ref())
  }
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Error::Tauri(e) => write!(f, "Tauri error: {}", e),
      Error::WindowNotFound(e) => write!(f, "Window not found: {}", e),
      Error::UrlParse(e) => write!(f, "Url parse error: {}", e),
      Error::DialogSetup(e) => write!(f, "Dialog setup error: {}", e),
    }
  }
}

impl From<tauri::Error> for Error {
  fn from(error: tauri::Error) -> Self {
    Error::Tauri(error.to_string())
  }
}


