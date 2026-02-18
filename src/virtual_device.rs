use evdev::uinput::{VirtualDevice, VirtualDeviceBuilder};
use evdev::{AttributeSet, Device, InputEvent, Key};
use tracing::info;

pub fn create_virtual_keyboard(source: &Device) -> std::io::Result<VirtualDevice> {
    let mut builder = VirtualDeviceBuilder::new()?
        .name("splash-damage virtual keyboard");

    if let Some(keys) = source.supported_keys() {
        let mut key_set = AttributeSet::<Key>::new();
        for key in keys.iter() {
            key_set.insert(key);
        }
        builder = builder.with_keys(&key_set)?;
    }

    let virt = builder.build()?;
    info!("created virtual keyboard device");
    Ok(virt)
}

pub fn release_all_modifiers(virt: &mut VirtualDevice) -> std::io::Result<()> {
    let modifiers = [
        Key::KEY_LEFTCTRL,
        Key::KEY_RIGHTCTRL,
        Key::KEY_LEFTSHIFT,
        Key::KEY_RIGHTSHIFT,
        Key::KEY_LEFTALT,
        Key::KEY_RIGHTALT,
        Key::KEY_LEFTMETA,
        Key::KEY_RIGHTMETA,
    ];
    let mut events: Vec<InputEvent> = modifiers
        .iter()
        .map(|k| InputEvent::new(evdev::EventType::KEY, k.code(), 0))
        .collect();
    events.push(InputEvent::new(evdev::EventType::SYNCHRONIZATION, 0, 0));
    virt.emit(&events)
}

pub fn emit_events(virt: &mut VirtualDevice, events: &[InputEvent]) -> std::io::Result<()> {
    virt.emit(events)
}
