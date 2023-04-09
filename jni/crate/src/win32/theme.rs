use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

use windows::{
    Foundation::TypedEventHandler,
    UI::{
        Color,
        ViewManagement::{UIColorType, UISettings},
    },
};

use crate::common::theme::{Holder, Listener};

pub struct ListenerHolder {
    _ui_settings: UISettings,
}

impl Holder for ListenerHolder {}

fn color_is_night(color: Color) -> bool {
    ((5 * color.G as u32) + (2 * color.R as u32) + color.B as u32) > (8 * 128)
}

pub fn is_night_mode() -> Result<bool, Box<dyn std::error::Error>> {
    let ui_settings = UISettings::new()?;
    let color = ui_settings.GetColorValue(UIColorType::Foreground)?;

    Ok(color_is_night(color))
}

pub fn add_night_mode_listener(
    listener: impl Listener + Sync + Send + 'static,
) -> Result<Box<dyn Holder>, Box<dyn std::error::Error>> {
    let ui_settings = UISettings::new()?;
    let value = AtomicBool::new(is_night_mode()?);

    ui_settings.ColorValuesChanged(&TypedEventHandler::new(move |sender: &Option<UISettings>, _| {
        if let Some(ui_settings) = &sender {
            let color = ui_settings.GetColorValue(UIColorType::Foreground)?;
            let new_value = color_is_night(color);

            if value.compare_exchange(!new_value, new_value, Relaxed, Relaxed).is_ok() {
                listener.on_changed(new_value);
            }
        }

        Ok(())
    }))?;

    Ok(Box::new(ListenerHolder {
        _ui_settings: ui_settings,
    }))
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::{
        common::theme::Listener,
        win32::theme::{add_night_mode_listener, is_night_mode},
    };

    #[test]
    fn test_is_night_mode() -> Result<(), Box<dyn std::error::Error>> {
        println!("is_night = {}", is_night_mode()?);

        Ok(())
    }

    struct ListenerImpl {}

    impl Listener for ListenerImpl {
        fn on_changed(&self, is_night: bool) {
            println!("thread = {:?}", std::thread::current().id());

            println!("is_night = {is_night}");
        }
    }

    #[test]
    fn test_listen_night_mode() -> Result<(), Box<dyn std::error::Error>> {
        let ret = add_night_mode_listener(ListenerImpl {})?;

        println!("RETURNED");

        std::thread::sleep(Duration::from_secs(5));

        drop(ret);

        println!("DROPPED");

        std::thread::sleep(Duration::from_secs(5));

        Ok(())
    }
}
