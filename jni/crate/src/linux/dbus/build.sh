#!/usr/bin/env sh

cd "$(dirname "$0")" || exit 1

zbus-xmlgen xml/org.freedesktop.portal.Request.xml > request.rs
zbus-xmlgen xml/org.freedesktop.portal.Settings.xml > settings.rs
zbus-xmlgen xml/org.freedesktop.portal.FileChooser.xml > file_chooser.rs
zbus-xmlgen xml/org.freedesktop.portal.OpenURI.xml > open_uri.rs
zbus-xmlgen xml/org.freedesktop.portal.Notification.xml > notifications.rs
zbus-xmlgen xml/org.kde.StatusNotifierWatcher.xml > notifier_watcher.rs
