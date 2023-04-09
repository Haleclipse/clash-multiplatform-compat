use std::{
    collections::HashMap,
    error::Error,
    sync::{Arc, Mutex},
};

use zbus::{
    dbus_interface, fdo,
    zvariant::{Array, Str, Value},
    Connection, SignalContext,
};

pub const DBUS_MENU_PATH: &str = "/MenuBar";

pub enum Item {
    Item(u16, String),
    Children(String, Vec<i32>),
}

struct DBusMenuData<L: FnMut(u16) + Sync + Send + 'static> {
    listener: L,
    is_rtl: bool,
    items: HashMap<i32, Item>,
    version: u32,
}

impl<L: FnMut(u16) + Sync + Send + 'static> DBusMenuData<L> {
    pub fn set_items(&mut self, items: HashMap<i32, Item>) -> Result<(), Box<dyn Error>> {
        for (_, item) in &items {
            if let Item::Children(_, children) = item {
                if !children.iter().all(|id| items.contains_key(id)) {
                    return Err(format!("some id not found in items").into());
                }
            }
        }

        if items.get(&0).filter(|f| matches!(f, Item::Children(_, _))).is_none() {
            return Err("invalid root item".into());
        }

        self.items = items;
        self.version += 1;

        Ok(())
    }

    fn get_item(&self, index: i32, depth: isize) -> Option<(i32, HashMap<&'static str, Value<'static>>, Vec<Value<'static>>)> {
        if let Some(item) = self.items.get(&index) {
            match item {
                Item::Item(_, title) => {
                    let mut properties: HashMap<&'static str, Value<'static>> = HashMap::new();
                    properties.insert("enabled", Value::Bool(true));
                    properties.insert("label", Value::Str(Str::from(title.to_owned())));
                    properties.insert("visible", Value::Bool(true));

                    Some((index, properties, vec![]))
                }
                Item::Children(title, children) => {
                    let mut properties: HashMap<&'static str, Value<'static>> = HashMap::new();
                    properties.insert("children-display", Value::Str("submenu".into()));

                    if index != 0 {
                        properties.insert("label", Value::Str(Str::from(title.to_owned())));
                    }

                    let children = if depth != 0 {
                        children
                            .iter()
                            .filter_map(|c| self.get_item(*c, depth - 1))
                            .collect::<Vec<_>>()
                    } else {
                        vec![]
                    };

                    Some((index, properties, Vec::try_from(Value::Array(Array::from(children))).unwrap()))
                }
            }
        } else {
            None
        }
    }

    fn get_layout_version(&self) -> u32 {
        self.version
    }

    fn emit_click_event(&mut self, index: i32) {
        if let Some(item) = self.items.get(&index) {
            if let Item::Item(id, _) = item {
                (self.listener)(*id);
            }
        }
    }

    fn get_text_direction(&self) -> &'static str {
        if self.is_rtl {
            "rtl"
        } else {
            "ltr"
        }
    }
}

pub struct DBusMenu<L: FnMut(u16) + Sync + Send + 'static = fn(u16)> {
    data: Arc<Mutex<DBusMenuData<L>>>,
}

impl<L: FnMut(u16) + Sync + Send + 'static> Clone for DBusMenu<L> {
    fn clone(&self) -> Self {
        DBusMenu { data: self.data.clone() }
    }
}

#[dbus_interface(name = "com.canonical.dbusmenu")]
impl<L: FnMut(u16) + Sync + Send + 'static> DBusMenu<L> {
    async fn about_to_show(&self, _id: i32) -> bool {
        false
    }

    async fn about_to_show_group(&self, _ids: Vec<i32>) -> (Vec<i32>, Vec<i32>) {
        (vec![], vec![])
    }

    async fn event(&self, id: i32, event_id: &str, _data: Value<'_>, _timestamp: u32) {
        if event_id == "clicked" {
            self.data.lock().unwrap().emit_click_event(id);
        }
    }

    async fn event_group(&self, events: Vec<(i32, &str, Value<'_>, u32)>) -> Vec<i32> {
        for ev in events {
            self.event(ev.0, ev.1, ev.2, ev.3).await
        }

        vec![]
    }

    async fn get_group_properties(
        &self,
        ids: Vec<i32>,
        _property_names: Vec<&str>,
    ) -> fdo::Result<Vec<(i32, HashMap<&str, Value>)>> {
        let data = self.data.lock().unwrap();

        let mut ret = Vec::with_capacity(ids.len());
        for id in ids {
            if let Some((id, properties, _)) = data.get_item(id, 0) {
                ret.push((id, properties));
            } else {
                return Err(fdo::Error::FileNotFound("item not found".to_owned()));
            }
        }

        Ok(ret)
    }

    async fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        _property_names: Vec<&str>,
    ) -> fdo::Result<(u32, (i32, HashMap<&str, Value>, Vec<Value>))> {
        let data = self.data.lock().unwrap();

        if let Some(item) = data.get_item(parent_id, recursion_depth as isize) {
            Ok((data.get_layout_version(), item))
        } else {
            Err(fdo::Error::FileNotFound(
                format!("parent id {} not found", parent_id).to_owned(),
            ))
        }
    }

    async fn get_property(&self, id: i32, name: &str) -> fdo::Result<Value> {
        let data = self.data.lock().unwrap();

        if let Some((_, mut properties, _)) = data.get_item(id, 0) {
            if let Some(value) = properties.remove(&name) {
                return Ok(value);
            }
        }

        Err(fdo::Error::FileNotFound("property not found".to_owned()))
    }

    #[dbus_interface(signal)]
    async fn items_properties_updated(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(signal)]
    async fn layout_updated(ctx: &SignalContext<'_>) -> zbus::Result<()> {}

    #[dbus_interface(property)]
    async fn icon_theme_path(&self) -> fdo::Result<Vec<String>> {
        Err(fdo::Error::InvalidArgs("unsupported".to_owned()))
    }

    #[dbus_interface(property)]
    async fn status(&self) -> &str {
        "normal"
    }

    #[dbus_interface(property)]
    async fn text_direction(&self) -> &str {
        self.data.lock().unwrap().get_text_direction()
    }

    #[dbus_interface(property)]
    async fn version(&self) -> u32 {
        4
    }
}

impl<L: FnMut(u16) + Sync + Send + 'static> DBusMenu<L> {
    pub fn new(listener: L, is_rtl: bool) -> Self {
        let mut default_items = HashMap::new();
        default_items.insert(0, Item::Children("".to_owned(), vec![]));

        DBusMenu {
            data: Arc::new(Mutex::new(DBusMenuData {
                listener,
                is_rtl,
                items: default_items,
                version: 0,
            })),
        }
    }

    pub async fn set_items(&self, conn: &Connection, items: HashMap<i32, Item>) -> Result<(), Box<dyn Error>> {
        self.data.lock().unwrap().set_items(items)?;

        DBusMenu::<L>::layout_updated(&SignalContext::new(conn, DBUS_MENU_PATH)?).await?;

        Ok(())
    }
}
