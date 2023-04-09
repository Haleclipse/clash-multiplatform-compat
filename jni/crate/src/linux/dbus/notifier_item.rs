use std::sync::{Arc, Mutex};

use zbus::{dbus_interface, fdo, zvariant::ObjectPath, SignalContext};

use crate::linux::dbus::dbus_menu::DBUS_MENU_PATH;

pub const NOTIFIER_ITEM_PATH: &str = "/StatusNotifierItem";

struct NotifierItemData<L: FnMut() + Send + Sync + 'static> {
    listener: L,
    app_id: String,
    icon: String,
    title: String,
}

pub struct NotifierItem<L: FnMut() + Send + Sync + 'static = fn()> {
    data: Arc<Mutex<NotifierItemData<L>>>,
}

impl<L: FnMut() + Send + Sync + 'static> Clone for NotifierItem<L> {
    fn clone(&self) -> Self {
        Self { data: self.data.clone() }
    }
}

#[dbus_interface(name = "org.kde.StatusNotifierItem")]
impl<L: FnMut() + Send + Sync + 'static> NotifierItem<L> {
    async fn activate(&self, _x: i32, _y: i32) {
        (self.data.lock().unwrap().listener)();
    }

    async fn context_menu(&self, _x: i32, _y: i32) -> () {}

    async fn provide_xdg_activation_token(&self, _token: &str) {}

    async fn scroll(&self, _delta: i32, _orientation: &str) {}

    async fn secondary_activate(&self, _x: i32, _y: i32) {}

    #[dbus_interface(signal)]
    async fn new_attention_icon(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_icon(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_menu(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_overlay_icon(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_status(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_title(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn new_tool_tip(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(property)]
    async fn attention_icon_name(&self) -> String {
        self.data.lock().unwrap().icon.to_owned()
    }

    #[dbus_interface(property)]
    async fn attention_icon_pixmap(&self) -> fdo::Result<Vec<(i32, i32, Vec<u8>)>> {
        Err(fdo::Error::InvalidArgs("unsupported".to_owned()))
    }

    #[dbus_interface(property)]
    async fn attention_movie_name(&self) -> String {
        "".to_string()
    }

    #[dbus_interface(property)]
    async fn category(&self) -> &str {
        "ApplicationStatus"
    }

    #[dbus_interface(property)]
    async fn icon_name(&self) -> String {
        self.data.lock().unwrap().icon.to_owned()
    }

    #[dbus_interface(property)]
    async fn icon_pixmap(&self) -> fdo::Result<Vec<(i32, i32, Vec<u8>)>> {
        Err(fdo::Error::InvalidArgs("unsupported".to_owned()))
    }

    #[dbus_interface(property)]
    async fn icon_theme_path(&self) -> fdo::Result<String> {
        Err(fdo::Error::InvalidArgs("unsupported".to_owned()))
    }

    #[dbus_interface(property)]
    async fn id(&self) -> String {
        self.data.lock().unwrap().app_id.to_owned()
    }

    #[dbus_interface(property)]
    async fn item_is_menu(&self) -> bool {
        false
    }

    #[dbus_interface(property)]
    async fn menu(&self) -> ObjectPath {
        ObjectPath::try_from(DBUS_MENU_PATH).unwrap()
    }

    #[dbus_interface(property)]
    async fn overlay_icon_name(&self) -> String {
        self.data.lock().unwrap().icon.to_owned()
    }

    #[dbus_interface(property)]
    async fn overlay_icon_pixmap(&self) -> fdo::Result<Vec<(i32, i32, Vec<u8>)>> {
        Err(fdo::Error::InvalidArgs("unsupported".to_owned()))
    }

    #[dbus_interface(property)]
    async fn status(&self) -> &str {
        "Active"
    }

    #[dbus_interface(property)]
    async fn title(&self) -> String {
        self.data.lock().unwrap().title.to_owned()
    }

    #[dbus_interface(property)]
    async fn tool_tip(&self) -> (String, Vec<(i32, i32, Vec<u8>)>, String, String) {
        (
            "".to_owned(),
            vec![],
            self.data.lock().unwrap().title.to_owned(),
            "".to_owned(),
        )
    }

    #[dbus_interface(property)]
    async fn window_id(&self) -> i32 {
        0
    }
}

impl<L: FnMut() + Send + Sync + 'static> NotifierItem<L> {
    pub fn new(listener: L, app_id: &str, title: &str, icon: &str) -> Self {
        Self {
            data: Arc::new(Mutex::new(NotifierItemData {
                listener,
                app_id: app_id.to_owned(),
                icon: icon.to_owned(),
                title: title.to_owned(),
            })),
        }
    }
}
