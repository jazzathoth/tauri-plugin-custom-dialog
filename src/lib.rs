use tauri::plugin::{Builder, TauriPlugin};
use tauri::{ Manager, Runtime };
mod desktop;mod commands;
mod error;
mod models;
// use serde::de::DeserializeOwned;
pub use error::{Error, Result};
pub use models::{DialogOptions, DialogResult};
use desktop::CustomDialog;
mod event_wrappers;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the custom-dialog APIs.
pub trait CustomDialogExt<R: Runtime> {
  fn custom_dialog(&self) -> &CustomDialog<R>;
}

impl<R: Runtime, T: Manager<R>> crate::CustomDialogExt<R> for T {
  fn custom_dialog(&self) -> &CustomDialog<R> {
    self.state::<CustomDialog<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("custom-dialog")
    .invoke_handler(tauri::generate_handler![commands::start_custom_dialog, commands::finish_dialog,])
    .setup(|app, _| {
      let custom_dialog = desktop::init::<R>(app)?;
      app.manage(custom_dialog);
      Ok(())
    })
    .build()
}
