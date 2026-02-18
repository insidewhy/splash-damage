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

pub fn emit_events(virt: &mut VirtualDevice, events: &[InputEvent]) -> std::io::Result<()> {
    virt.emit(events)
}
