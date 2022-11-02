use notify_rust::Notification;
use crate::utils::sounds;

// Implementation to create and push new notifications
pub fn PushNotification(title: String, content: String) {
    let summary = format!("Uplink - {}", title);
    let _n = Notification::new()
        .summary(summary.as_ref())
        .body(content.as_ref())
        .show();
    // Play notification sound
    sounds::play(sounds::Sounds::Notification);
}

