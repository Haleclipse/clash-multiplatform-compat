use crate::common::network::SystemProxyConfig;
use std::{error::Error, str::FromStr};

pub fn is_system_proxy_supported() -> bool {
    if let Ok(session_type) = std::env::var("XDG_SESSION_DESKTOP") {
        session_type == "gnome"
    } else {
        false
    }
}

pub fn set_system_proxy(enabled: bool, config: &SystemProxyConfig) -> Result<(), Box<dyn Error>> {
    let session_type = std::env::var("XDG_SESSION_DESKTOP")?;

    match &session_type as &str {
        "gnome" => {
            if enabled {
                let ignore_hosts = config
                    .excludes
                    .iter()
                    .map(|host| format!("\"{host}\""))
                    .collect::<Vec<_>>()
                    .join(",");
                let ignore_hosts = format!("[{ignore_hosts}]");

                let address = std::net::SocketAddr::from_str(&config.address)?;

                let fields = [
                    ("org.gnome.system.proxy", "mode", "manual"),
                    ("org.gnome.system.proxy", "ignore-hosts", &ignore_hosts),
                    ("org.gnome.system.proxy.http", "enabled", "true"),
                    ("org.gnome.system.proxy.http", "host", &address.ip().to_string()),
                    ("org.gnome.system.proxy.http", "port", &address.port().to_string()),
                    ("org.gnome.system.proxy.https", "host", &address.ip().to_string()),
                    ("org.gnome.system.proxy.https", "port", &address.port().to_string()),
                ];

                for (schema, key, value) in fields {
                    if let Err(e) = std::process::Command::new("gsettings")
                        .args(["set", schema, key, value])
                        .status()
                    {
                        let _ = std::process::Command::new("gsettings")
                            .args(["set", "org.gnome.system.proxy", "mode", "none"])
                            .status();

                        return Err(e.into());
                    }
                }
            } else {
                let _ = std::process::Command::new("gsettings")
                    .args(["set", "org.gnome.system.proxy", "mode", "none"])
                    .status();
            }

            Ok(())
        }
        _ => Err("unsupported".into()),
    }
}
