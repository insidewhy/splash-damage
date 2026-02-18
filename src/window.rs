use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use zbus::proxy;

const KWIN_SCRIPT: &str = r#"
function notifyActiveWindow(window) {
    if (window) {
        callDBus(
            "com.splashdamage.ActiveWindow",
            "/active_window",
            "com.splashdamage.ActiveWindow",
            "NotifyActiveWindow",
            window.resourceClass || "",
            window.caption || ""
        );
    }
}

workspace.windowActivated.connect(notifyActiveWindow);

var current = workspace.activeWindow;
if (current) {
    notifyActiveWindow(current);
}
"#;

#[proxy(
    interface = "org.kde.kwin.Scripting",
    default_service = "org.kde.KWin",
    default_path = "/Scripting"
)]
trait KWinScripting {
    #[zbus(name = "loadScript")]
    fn load_script(&self, path: &str) -> zbus::Result<i32>;
}

#[proxy(
    interface = "org.kde.kwin.Script",
    default_service = "org.kde.KWin"
)]
trait KWinScript {
    #[zbus(name = "run")]
    fn run(&self) -> zbus::Result<()>;
    #[zbus(name = "stop")]
    fn stop(&self) -> zbus::Result<()>;
}

#[derive(Debug, Clone)]
pub struct ActiveWindow {
    pub resource_class: String,
}

pub type SharedActiveWindow = Arc<RwLock<Option<ActiveWindow>>>;

pub fn shared_active_window() -> SharedActiveWindow {
    Arc::new(RwLock::new(None))
}

struct ActiveWindowService {
    state: SharedActiveWindow,
}

#[zbus::interface(name = "com.splashdamage.ActiveWindow")]
impl ActiveWindowService {
    async fn notify_active_window(&self, resource_class: &str, caption: &str) {
        let mut state = self.state.write().await;
        *state = Some(ActiveWindow {
            resource_class: resource_class.to_string(),
        });
        info!(
            resource_class,
            caption, "active window changed"
        );
    }
}

fn find_session_bus_address() -> anyhow::Result<String> {
    if let Ok(addr) = std::env::var("DBUS_SESSION_BUS_ADDRESS") {
        return Ok(addr);
    }

    let uid = nix::unistd::Uid::current();
    let runtime_path = format!("/run/user/{uid}/bus");
    if std::path::Path::new(&runtime_path).exists() {
        return Ok(format!("unix:path={runtime_path}"));
    }

    anyhow::bail!("could not find session D-Bus socket (DBUS_SESSION_BUS_ADDRESS not set)");
}

pub struct WindowWatcher {
    pub connection: zbus::Connection,
    pub script_id: i32,
}

impl WindowWatcher {
    pub async fn stop_script(&self) {
        let path = format!("/Scripting/Script{}", self.script_id);
        if let Ok(proxy) = KWinScriptProxy::builder(&self.connection)
            .path(path.as_str())
            .unwrap()
            .build()
            .await
        {
            let _ = proxy.stop().await;
            info!("stopped kwin script {}", self.script_id);
        }
    }
}

pub async fn start_window_watcher(
    state: SharedActiveWindow,
) -> anyhow::Result<WindowWatcher> {
    let bus_addr = find_session_bus_address()?;
    info!("connecting to session bus at {bus_addr}");

    let session = zbus::connection::Builder::address(bus_addr.as_str())?
        .build()
        .await?;

    session
        .object_server()
        .at("/active_window", ActiveWindowService { state })
        .await?;

    session.request_name("com.splashdamage.ActiveWindow").await?;

    info!("registered D-Bus service com.splashdamage.ActiveWindow");

    let script_path = write_kwin_script()?;

    // Retry loading the KWin script — on login, KWin may not be ready yet.
    let mut script_id = None;
    for attempt in 1..=30 {
        match load_kwin_script(&session, &script_path).await {
            Ok(id) => {
                script_id = Some(id);
                break;
            }
            Err(e) => {
                warn!(attempt, "waiting for KWin: {e}");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        }
    }

    let script_id = script_id.ok_or_else(|| anyhow::anyhow!("KWin not available after 30 attempts"))?;
    info!(script_id, "loaded kwin script");

    Ok(WindowWatcher {
        connection: session,
        script_id,
    })
}

async fn load_kwin_script(
    session: &zbus::Connection,
    script_path: &std::path::Path,
) -> anyhow::Result<i32> {
    let scripting_proxy = KWinScriptingProxy::new(session).await?;

    // Stop any stale scripts left from previous runs.
    for id in 1..=32 {
        let path = format!("/Scripting/Script{id}");
        if let Ok(proxy) = KWinScriptProxy::builder(session)
            .path(path.as_str())
            .unwrap()
            .build()
            .await
        {
            let _ = proxy.stop().await;
        }
    }

    let script_id = scripting_proxy
        .load_script(&script_path.to_string_lossy())
        .await?;

    let script_obj_path = format!("/Scripting/Script{script_id}");
    let script_proxy = KWinScriptProxy::builder(session)
        .path(script_obj_path.as_str())?
        .build()
        .await?;

    script_proxy.run().await?;
    info!("started kwin script");

    Ok(script_id)
}

fn write_kwin_script() -> anyhow::Result<std::path::PathBuf> {
    let dir = std::env::temp_dir().join("splash-damage");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("active_window.js");
    std::fs::write(&path, KWIN_SCRIPT)?;
    Ok(path)
}
