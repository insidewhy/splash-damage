use evdev::{Device, EventType};
use std::path::PathBuf;
use tracing::info;

pub fn find_keyboards() -> Vec<(PathBuf, Device)> {
    evdev::enumerate()
        .filter(|(path, device)| {
            let name = device.name().unwrap_or("unknown");

            if name.starts_with("splash-damage") {
                return false;
            }

            let is_keyboard = device.supported_events().contains(EventType::KEY)
                && device.supported_keys().is_some_and(|keys| {
                    keys.contains(evdev::Key::KEY_A) && keys.contains(evdev::Key::KEY_Z)
                });

            if is_keyboard {
                info!("found keyboard: {name} ({})", path.display());
            }
            is_keyboard
        })
        .collect()
}

pub fn grab_device(device: &mut Device) -> std::io::Result<()> {
    device.grab()?;
    info!("grabbed device: {}", device.name().unwrap_or("unknown"));
    Ok(())
}
