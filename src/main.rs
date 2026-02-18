mod config;
mod device;
mod remap;
mod virtual_device;
mod window;

use std::path::PathBuf;
use tokio::signal;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| default_config_path());

    info!("loading config from {}", config_path.display());
    let loaded = config::load_config(&config_path)?;
    info!("loaded {} remap rules", loaded.rules.len());
    if loaded.copilot_as_meta {
        info!("copilot_as_meta enabled");
    }

    let active_window = window::shared_active_window();

    let watcher = window::start_window_watcher(active_window.clone()).await?;

    let keyboards = device::find_keyboards();
    if keyboards.is_empty() {
        anyhow::bail!("no keyboards found — are you running as root?");
    }

    let mut handles = Vec::new();

    for (path, mut dev) in keyboards {
        let rules = loaded.rules.clone();
        let copilot_as_meta = loaded.copilot_as_meta;
        let aw = active_window.clone();

        device::grab_device(&mut dev)?;

        let mut virt = virtual_device::create_virtual_keyboard(&dev)?;
        virtual_device::release_all_modifiers(&mut virt)?;

        let handle = tokio::spawn(async move {
            let mut remapper = remap::Remapper::new(rules, aw, copilot_as_meta);
            let mut stream = dev.into_event_stream().unwrap();

            loop {
                match stream.next_event().await {
                    Ok(event) => {
                        let remapped = remapper.process_event(event);
                        if let Err(e) = virtual_device::emit_events(&mut virt, &remapped) {
                            error!("failed to emit events: {e}");
                        }
                    }
                    Err(e) => {
                        error!("error reading from {}: {e}", path.display());
                        break;
                    }
                }
            }
        });

        handles.push(handle);
    }

    info!("splash-damage running — press Ctrl+C to stop");

    signal::ctrl_c().await?;
    info!("shutting down");

    watcher.stop_script().await;

    for handle in handles {
        handle.abort();
    }

    Ok(())
}

fn default_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("/etc"))
        .join("splash-damage")
        .join("config.toml")
}
