use tauri::{AppHandle, Manager, Runtime, State, Window, Emitter};

use crate::models::{ DialogResult, DialogOptions };
use crate::Result;
use crate::CustomDialogExt;
use crate::desktop::CustomDialog;

#[tauri::command]
pub(crate) async fn start_custom_dialog<R: Runtime>(
  app_handle: AppHandle<R>,
  window: Window<R>,
  name: String,
  url: String,
  options: Option<DialogOptions>
) -> Result<DialogResult> {
  let parent_window_label = Some(window.label().to_string());
  let dialog_manager = app_handle.state::<CustomDialog<R>>();
  dialog_manager.custom_dialog(
    parent_window_label,
    name,
    url,
    options,
  ).await
}

#[tauri::command]
pub(crate) async fn finish_dialog<R: Runtime>(
  app_handle: AppHandle<R>,
  window: Window<R>,
  state: State<'_, CustomDialog<R>>,
  result: DialogResult
) -> Result<()> {
  let window_label = window.label().to_string();
  let event_name = {
    let guard = state.active_dialogs.lock().unwrap();
    if let Some(ev) = guard.get(&window_label) {
      ev.event_name.clone()
    } else {
      return Err(crate::Error::Tauri("Error: failed to get event name.".to_string()));
    }
  };
  if let Err(e) = app_handle.emit(&event_name, result) {
    eprintln!("Error emitting dialog result event '{}': {}", event_name, e);
  }
  Ok(()) 
}
