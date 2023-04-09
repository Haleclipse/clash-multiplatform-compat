use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use futures::executor::block_on;

use zbus::{
    zvariant::{Array, Value},
    Connection,
};

use crate::linux::dbus::notifications::NotificationProxy;

fn find_icon_for_app_id(app_id: &str) -> Result<String, Box<dyn Error>> {
    let desktop_path = home::home_dir().map(|p| p.join(".local/share/applications").join(app_id).with_extension("desktop"));

    let desktop_path = if let Some(path) = desktop_path.filter(|p| p.is_file()) {
        path
    } else {
        Path::new("/usr/share/applications").join(app_id).with_extension("desktop")
    };

    for line in BufReader::new(File::open(&desktop_path)?).lines() {
        let line = line?;

        let kv = line.splitn(2, "=").collect::<Vec<_>>();
        if kv.len() != 2 {
            continue;
        }

        if kv[0] == "Icon" {
            return Ok(kv[1].to_owned());
        }
    }

    Err(format!("desktop file {} invalid", desktop_path.to_str().unwrap())
        .to_owned()
        .into())
}

pub fn is_supported() -> bool {
    let version = block_on(async {
        let conn = Connection::session().await?;

        let version = NotificationProxy::builder(&conn)
            .destination("org.freedesktop.portal.Desktop")?
            .path("/org/freedesktop/portal/desktop")?
            .build()
            .await?
            .version()
            .await?;

        Ok(version) as Result<u32, Box<dyn Error>>
    });

    if let Ok(version) = version {
        version >= 1
    } else {
        false
    }
}

pub fn send_notification(app_id: &str, title: &str, message: &str) -> Result<(), Box<dyn Error>> {
    block_on(async {
        let icon = find_icon_for_app_id(app_id)?;

        let conn = Connection::session().await?;

        let mut options: HashMap<&str, Value> = HashMap::new();
        options.insert("title", Value::from(title));
        options.insert("body", Value::from(message));
        options.insert("icon", Value::from(("themed", Value::from(Array::from(&[icon][..])))));

        NotificationProxy::builder(&conn)
            .destination("org.freedesktop.portal.Desktop")?
            .path("/org/freedesktop/portal/desktop")?
            .build()
            .await?
            .add_notification(&rand::random::<u64>().to_string(), options)
            .await?;

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::linux::notification::send_notification;

    const TEST_APP_ID: &str = "clash-compat-library";

    #[test]
    pub fn test_show_notification() -> Result<(), Box<dyn Error>> {
        send_notification(TEST_APP_ID, "Clash Compat Test", "This is a test")?;

        Ok(())
    }
}
