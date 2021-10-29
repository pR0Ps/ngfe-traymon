#![windows_subsystem = "windows"]
extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use const_format::formatcp;
use nwg::NativeUi;
use sysinfo::{System, SystemExt};


const PROGRAM_NAME: &str = formatcp!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
const HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

const PROCESS_NAME: &str = "nvstreamer.exe";
const PROCESS_CHECK_INTERVAL: Duration = Duration::from_secs(5);

const ACTIVE_MSG: &str = "Streaming is active";
const INACTIVE_MSG: &str = "Not streaming";

const STARTED_MSG: &str = "Streaming started";
const STOPPED_MSG: &str = "Streaming ended";


#[derive(Default, nwd::NwgUi)]
pub struct SystemTray {
    #[nwg_control]
    window: nwg::MessageWindow,

    #[nwg_resource]
    embed: nwg::EmbedResource,

    #[nwg_resource(source_embed: Some(&data.embed), source_embed_id: 1)]
    active_icon: nwg::Icon,

    #[nwg_resource(source_embed: Some(&data.embed), source_embed_id: 2)]
    inactive_icon: nwg::Icon,

    #[nwg_control(icon: Some(&data.inactive_icon), tip: Some(INACTIVE_MSG))]
    #[nwg_events(MousePressLeftUp: [SystemTray::show_menu], OnContextMenu: [SystemTray::show_menu])]
    tray: nwg::TrayNotification,

    #[nwg_control(parent: window, popup: true)]
    tray_menu: nwg::Menu,

    #[nwg_control(parent: tray_menu, text: PROGRAM_NAME, disabled: true)]
    tray_item_version: nwg::MenuItem,

    #[nwg_control(parent: tray_menu, text: "&About")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::about])]
    tray_item_about: nwg::MenuItem,

    #[nwg_control(parent: tray_menu)]
    tray_sep: nwg::MenuSeparator,

    #[nwg_control(parent: tray_menu, text: "E&xit")]
    #[nwg_events(OnMenuItemSelected: [SystemTray::exit])]
    tray_item_exit: nwg::MenuItem,
}

impl SystemTray {
    fn show_menu(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        self.tray_menu.popup(x, y);
    }

    fn started(&self) {
        self.tray.set_icon(&self.active_icon);
        self.tray.show(
            STARTED_MSG,
            None,
            Some(nwg::TrayNotificationFlags::NO_ICON),
            None,
        );
        self.tray.set_tip(ACTIVE_MSG);
    }

    fn ended(&self) {
        self.tray.set_icon(&self.inactive_icon);
        self.tray.show(
            STOPPED_MSG,
            None,
            Some(nwg::TrayNotificationFlags::NO_ICON),
            None,
        );
        self.tray.set_tip(INACTIVE_MSG);
    }

    fn about(&self) {
        let _ = webbrowser::open(HOMEPAGE);
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    let tray = SystemTray::build_ui(Default::default()).expect("Failed to build UI");

    let (tx, rx) = mpsc::channel::<bool>();

    // Thread that checks if the target process is running
    thread::spawn(move || {
        let mut running = false;
        let mut system = System::new();
        loop {
            system.refresh_processes();
            if running == system.process_by_name(PROCESS_NAME).is_empty() {
                running = !running;
                tx.send(running).unwrap()
            }
            thread::sleep(PROCESS_CHECK_INTERVAL);
        }
    });

    // Dispatch events to the tray icon (blocks until exit)
    nwg::dispatch_thread_events_with_callback(move || {
        if let Ok(m) = rx.try_recv() {
            match m {
                true => tray.started(),
                false => tray.ended(),
            }
        }
        // avoid burning cpu cycles
        thread::sleep(std::time::Duration::from_millis(10));
    });
}
