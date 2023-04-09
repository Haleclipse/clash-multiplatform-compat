use std::{
    error::Error,
    ops::Deref,
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use futures::{
    executor::{block_on, ThreadPool},
    future::RemoteHandle,
    task::SpawnExt,
    StreamExt,
};
use once_cell::sync::Lazy;

use zbus::{zvariant::Value, Connection};

use crate::{
    common::theme::{Holder, Listener},
    linux::dbus::settings::{SettingChangedStream, SettingsProxy},
};

async fn is_night_mode_async() -> Result<bool, Box<dyn Error>> {
    let conn = Connection::session().await?;
    let proxy = SettingsProxy::builder(&conn)
        .destination("org.freedesktop.portal.Desktop")?
        .path("/org/freedesktop/portal/desktop")?
        .build()
        .await?;

    let ret: zbus::zvariant::OwnedValue = proxy.read("org.freedesktop.appearance", "color-scheme").await?;
    if let Value::Value(value) = ret.deref() {
        if let Value::U32(value) = value.deref() {
            // 0: No preference
            // 1: Prefer dark appearance
            // 2: Prefer light appearance
            return Ok(*value == 1);
        }
    }

    Err(format!("unable to parse result: {}", ret.value_signature()).into())
}

pub fn is_night_mode() -> Result<bool, Box<dyn Error>> {
    return block_on(is_night_mode_async());
}

pub fn is_supported() -> bool {
    return is_night_mode().is_ok();
}

static POOL: Lazy<ThreadPool> = Lazy::new(|| ThreadPool::new().unwrap());

pub fn add_night_mode_listener(listener: impl Listener + Send + Sync + 'static) -> Result<Box<dyn Holder>, Box<dyn Error>> {
    let mut stream: SettingChangedStream = block_on(async {
        let conn = Connection::session().await?;
        let proxy: SettingsProxy = SettingsProxy::builder(&conn)
            .destination("org.freedesktop.portal.Desktop")?
            .path("/org/freedesktop/portal/desktop")?
            .build()
            .await?;

        Ok(proxy.receive_setting_changed().await?) as Result<SettingChangedStream, Box<dyn Error>>
    })?;

    let token = POOL.spawn_with_handle(async move {
        let value = AtomicBool::new(is_night_mode_async().await.unwrap_or(false));

        loop {
            if let Some(signal) = stream.next().await {
                if let Ok(args) = signal.args() {
                    if args.namespace == "org.freedesktop.appearance" && args.key == "color-scheme" {
                        if let Value::U32(v) = args.value {
                            // 0: No preference
                            // 1: Prefer dark appearance
                            // 2: Prefer light appearance
                            let new_value = v == 1;

                            if value.compare_exchange(!new_value, new_value, Relaxed, Relaxed).is_ok() {
                                listener.on_changed(new_value);
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        }
    })?;

    struct HolderImpl(RemoteHandle<()>);

    impl Holder for HolderImpl {}

    Ok(Box::new(HolderImpl(token)))
}

#[cfg(test)]
mod tests {
    use std::{error::Error, time::Duration};

    use crate::{
        common::theme::Listener,
        linux::theme::{add_night_mode_listener, is_night_mode, is_supported},
    };

    #[test]
    pub fn test_is_supported() {
        assert!(is_supported())
    }

    #[test]
    pub fn test_is_night() -> Result<(), Box<dyn Error>> {
        println!("is_night = {}", is_night_mode()?);

        Ok(())
    }

    #[test]
    pub fn test_listen_theme() -> Result<(), Box<dyn Error>> {
        struct ListenerImpl {}

        impl Listener for ListenerImpl {
            fn on_changed(&self, is_night: bool) {
                println!("[Listener] is_night = {}", is_night);
            }
        }

        let holder = add_night_mode_listener(ListenerImpl {})?;

        std::thread::sleep(Duration::from_secs(10));

        drop(holder);

        println!("DROPPED");

        std::thread::sleep(Duration::from_secs(10));

        Ok(())
    }
}
