use serde::{self, Deserialize, Serialize};
use serde_json::Value as JsonValue;


#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogOptions {
  pub title: Option<String>,
  pub width: Option<f64>,
  pub height: Option<f64>,
  pub resizable: Option<bool>,
  pub always_on_top: Option<bool>,
  pub is_modal: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag="status")]
pub enum DialogResult {
  Confirm { data: Option<JsonValue> },
  Cancel,
  Closed,
}

#[derive(Clone, Debug)]
pub struct DialogInstanceData {
  pub event_name: String,
}

