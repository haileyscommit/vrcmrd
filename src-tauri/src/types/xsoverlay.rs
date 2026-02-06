use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XSOverlayCommand {
    pub sender: String,
    pub target: String,
    pub command: String,
    pub json_data: String,
    pub raw_data: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XSOverlayNotificationObject {
    /// The type of message to send. 1 defines a normal notification.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<i32>,
    /// Used for Media Player, changes the icon on the wrist. (depricated, see note below)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i32>,
    /// How long the notification will stay on screen for in seconds. (default: 0.5 seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<f32>,
    /// Height notification will expand to if it has content other than a title. Default is 175.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,
    /// File path to .ogg audio file. Can be "default", "error", or "warning". Notification will be silent if left empty (default).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Set to true if using Base64 for the icon image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_base64_icon: Option<bool>,
    /// Base64 Encoded image, or file path to image. Can also be "default", "error", or "warning".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Somewhere to put your app name for debugging purposes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_app: Option<String>,
}
impl XSOverlayNotificationObject {
    pub const COMMAND: &'static str = "SendNotification";
}