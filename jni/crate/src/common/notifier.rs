use std::error::Error;

pub enum MenuItem {
    Item { title: String, id: u16 },
    SubMenu { title: String, items: Vec<MenuItem> },
}

pub trait Listener {
    fn on_active(&self);
    fn on_menu_active(&self, id: u16);
}

pub trait Notifier {
    fn set_menu(&self, layout: Option<&[MenuItem]>) -> Result<(), Box<dyn Error>>;
}
