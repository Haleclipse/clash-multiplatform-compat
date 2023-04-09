use std::{collections::HashMap, error::Error, sync::Arc};

use futures::executor::block_on;

use zbus::{Connection, ConnectionBuilder};

use crate::{
    common::{
        notifier,
        notifier::{Listener, MenuItem},
    },
    linux::dbus::{
        dbus_menu::{DBusMenu, Item, DBUS_MENU_PATH},
        notifier_item::{NotifierItem, NOTIFIER_ITEM_PATH},
        notifier_watcher::StatusNotifierWatcherProxy,
    },
};

pub fn is_supported() -> bool {
    let version_and_registered = block_on(async {
        let conn = Connection::session().await?;

        let proxy = StatusNotifierWatcherProxy::builder(&conn)
            .destination("org.kde.StatusNotifierWatcher")?
            .path("/StatusNotifierWatcher")?
            .build()
            .await?;

        let version = proxy.protocol_version().await?;
        let registered = proxy.is_status_notifier_host_registered().await?;

        Ok((version, registered)) as Result<(i32, bool), Box<dyn Error>>
    });

    if let Ok((version, registered)) = version_and_registered {
        version >= 0 && registered
    } else {
        false
    }
}

struct Notifier<ML: FnMut(u16) + Sync + Send + 'static> {
    conn: Connection,
    menu: DBusMenu<ML>,
}

struct BuildContext {
    items: HashMap<i32, Item>,
    index: i32,
}

impl BuildContext {
    fn push(&mut self, current: &MenuItem) -> i32 {
        let index = self.index;

        self.index += 1;

        match current {
            MenuItem::Item { title, id } => {
                self.items.insert(index, Item::Item(*id, title.to_owned()));
            }
            MenuItem::SubMenu { title, items } => {
                let children = items.iter().map(|it| self.push(it)).collect::<Vec<_>>();

                self.items.insert(index, Item::Children(title.to_owned(), children));
            }
        };

        index
    }
}

impl<ML: FnMut(u16) + Sync + Send + 'static> notifier::Notifier for Notifier<ML> {
    fn set_menu(&self, layout: Option<&[MenuItem]>) -> Result<(), Box<dyn Error>> {
        block_on(async {
            if let Some(layout) = layout {
                let mut ctx = BuildContext {
                    items: HashMap::new(),
                    index: 1,
                };

                let mut root = Vec::with_capacity(layout.len());

                for it in layout {
                    root.push(ctx.push(it));
                }

                ctx.items.insert(0, Item::Children("".to_string(), root));

                self.menu.set_items(&self.conn, ctx.items).await?;
            } else {
                let mut root = HashMap::new();
                root.insert(0 as i32, Item::Children("".to_owned(), vec![]));

                self.menu.set_items(&self.conn, root).await?;
            }

            Ok(())
        })
    }
}

pub fn add_notifier(
    listener: impl Listener + Send + Sync + 'static,
    app_id: &str,
    title: &str,
    icon: &str,
    is_rtl: bool,
) -> Result<Box<dyn notifier::Notifier>, Box<dyn Error>> {
    block_on(async {
        let item_listener = Arc::new(listener);
        let menu_listener = item_listener.clone();

        let item = NotifierItem::new(move || item_listener.on_active(), app_id, title, icon);
        let menu = DBusMenu::new(move |id| menu_listener.on_menu_active(id), is_rtl);

        let conn = ConnectionBuilder::session()?
            .serve_at(NOTIFIER_ITEM_PATH, item.clone())?
            .serve_at(DBUS_MENU_PATH, menu.clone())?
            .build()
            .await?;

        StatusNotifierWatcherProxy::builder(&conn)
            .destination("org.kde.StatusNotifierWatcher")?
            .path("/StatusNotifierWatcher")?
            .build()
            .await?
            .register_status_notifier_item(NOTIFIER_ITEM_PATH)
            .await?;

        Ok(Box::new(Notifier { conn, menu }) as Box<dyn notifier::Notifier>)
    })
}

#[cfg(test)]
mod tests {
    use std::{error::Error, time::Duration};

    use crate::{
        common::notifier::{Listener, MenuItem},
        linux::{notifier::add_notifier, shell::install_icon, testdata},
    };

    const TEST_APP_ID: &str = "clash-multiplatform-compat-library";
    const TEST_ICON_NAME: &str = "clash-multiplatform-compat-library";
    const TEST_ICON_PATH: &str = "clash-multiplatform.ico";
    const TEST_APP_NAME: &str = "Clash Compat Library (Test)";

    struct ListenerImpl {}

    impl Listener for ListenerImpl {
        fn on_active(&self) {
            println!("active");
        }

        fn on_menu_active(&self, id: u16) {
            println!("menu active: {}", id);
        }
    }

    #[test]
    pub fn test_notifier() -> Result<(), Box<dyn Error>> {
        let data = testdata::TestData::get(TEST_ICON_PATH).unwrap().data;

        install_icon(TEST_ICON_NAME, &data)?;

        let notifier = add_notifier(ListenerImpl {}, TEST_APP_ID, TEST_APP_NAME, TEST_ICON_NAME, false)?;

        notifier.set_menu(Some(&[
            MenuItem::Item {
                title: "Item 0".to_owned(),
                id: 114,
            },
            MenuItem::SubMenu {
                title: "Sub Items".to_string(),
                items: vec![
                    MenuItem::Item {
                        id: 514,
                        title: "Item 2".to_owned(),
                    },
                    MenuItem::Item {
                        id: 1919,
                        title: "Item 3".to_owned(),
                    },
                ],
            },
            MenuItem::Item {
                id: 810,
                title: "Item 3".to_owned(),
            },
        ]))?;

        println!("STARTED");

        std::thread::sleep(Duration::from_secs(10));

        Ok(())
    }
}
