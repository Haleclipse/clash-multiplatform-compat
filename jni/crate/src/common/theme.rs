pub trait Listener {
    fn on_changed(&self, is_night: bool);
}

pub trait Holder {}
