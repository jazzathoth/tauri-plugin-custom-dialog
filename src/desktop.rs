use std::collections::HashMap;
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime, WebviewWindow, Manager, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;
use crate::event_wrappers::{AwaitEvent, AwaitWindowCloseEvent, DialogResultFuture};
use crate::error::Result;

use crate::models::*;

pub fn init<R: Runtime>(
  app: &AppHandle<R>,
  // _api: PluginApi<R, C>,
) -> crate::Result<CustomDialog<R>> {
  Ok(CustomDialog {app_handle: app.clone(), active_dialogs: Mutex::new(HashMap::new()),})
}

/// Access to the custom-dialog APIs.
pub struct CustomDialog<R: Runtime> {
  app_handle: AppHandle<R>,
  pub active_dialogs: Mutex<HashMap<String, DialogInstanceData>>,
}

impl<R: Runtime> CustomDialog<R> {
  pub async fn custom_dialog(  
    &self,
    parent_window_label: Option<String>,
    label_prefix: String,
    url: String,
    options: Option<DialogOptions>,)
  -> Result<DialogResult> {

    let dlg_id = Uuid::new_v4().to_string();
    let window_label = format!("{}-{}", label_prefix.replace(|c: char| !c.is_alphanumeric()&& !"/-:_".contains(c), ""), dlg_id);
    println!("Setting window label: {}", window_label);
    let event_name = format!("dialog-result-{}", dlg_id);
    let app_clone = self.app_handle.clone();
     
    let dialog_data = DialogInstanceData{event_name: event_name.clone()};
    {
      let mut active_dialogs = self.active_dialogs.lock().unwrap();
      active_dialogs.insert(window_label.clone(), dialog_data);
    }

    let parent_window: Option<WebviewWindow<R>> = parent_window_label
      .as_ref()
      .and_then(|label| self.app_handle.get_webview_window(&label));

    let is_modal: bool = if let Some(ref opts) = options {
      opts.is_modal.unwrap_or(false)
    } else {
      false
    };


    println!("Opening dialog '{}' with ID: {}", window_label, dlg_id);

    // #### PARENT WINDOW DISABLE ####
    if is_modal {
      if let Some(ref parent) = parent_window {
        let _ = parent.set_enabled(false);
      } else if parent_window_label.is_some() {
        eprintln!("Warning: parent window with label '{}' not found", parent_window_label.unwrap());
      }
    }
    // ###############################

    

    let dialog_url = WebviewUrl::App(url.into());

    let builder = WebviewWindowBuilder::new(&self.app_handle, &window_label, dialog_url);

    let builder = if let Some(opts) = options {
      let mut b = builder;
      if let Some(title) = opts.title {
        b = b.title(title);
      }
      if let Some(width) = opts.width {
        if let Some(height) = opts.height {
          b = b.inner_size(width, height);
        }
      }
      if let Some(resizable) = opts.resizable {
        b = b.resizable(resizable);
      }
      b = b.always_on_top(opts.always_on_top.unwrap_or(true));
      b 
    } else {
      builder.title("Dialog").inner_size(600.0, 400.0).resizable(false).always_on_top(true)
    };

    let build_result = builder
      .center()
      .build();
    

    let dialog_window = match build_result {
      Ok(win) => {
        println!("Dialog window '{}' created successfully.", window_label);
        win
      },
      Err(e) => {
        eprintln!("Failed to create dialog window '{}': {}", window_label, e);
        if is_modal {
          if let Some(ref parent) = parent_window {
            let _ = parent.set_enabled(true);
          }
        }
        return Err(crate::error::Error::WindowNotFound(e.to_string()));
      }
    };


    let result_future = AwaitEvent::new(app_clone, &event_name);
    let window_closed_future = AwaitWindowCloseEvent::new(&dialog_window);

    let combined_result_future = DialogResultFuture { 
      result_event: result_future,
      close_event: window_closed_future,
    };

    println!("Awaiting dialog result for event '{}'...", event_name);
    let result = combined_result_future
      .await;
      
    let final_result = result.unwrap_or_else(|| {
        eprintln!("Combined future await failed.");
        DialogResult::Closed
    });
    println!("Dialog result received for event '{}': {:?}", event_name, final_result);

    let _ = dialog_window.close();

    if let Some(ref parent) = parent_window {
      if is_modal {
        if let Err(e) = parent.set_enabled(true) {
          eprintln!("Failed to re-enable parent window: {}", e);
        }
      }
      if let Err(e) = parent.set_focus() {
        eprintln!("Failed to focus parent window: {}", e);
      }
    } else {
      println!("No parent window provided to re-enable");
    }

    Ok(final_result)
  }
}
