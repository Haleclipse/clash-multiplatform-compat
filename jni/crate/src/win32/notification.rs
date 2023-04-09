use std::{error::Error, str::from_utf8};

use quick_xml::{
    events::{BytesEnd, BytesStart, BytesText, Event},
    Writer,
};
use windows::{
    core::HSTRING,
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

pub fn send_notification(app_id: &str, title: &str, message: &str) -> Result<(), Box<dyn Error>> {
    let notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(app_id))?;

    let mut writer = Writer::new(Vec::<u8>::with_capacity(128));

    writer.write_event(Event::Start(BytesStart::new("toast").with_attributes([("duration", "long")])))?;

    // visual
    {
        writer.write_event(Event::Start(BytesStart::new("visual")))?;

        // binding
        {
            writer.write_event(Event::Start(
                BytesStart::new("binding").with_attributes([("template", "ToastGeneric")]),
            ))?;

            // text
            {
                writer.write_event(Event::Start(BytesStart::new("text").with_attributes([("id", "1")])))?;
                writer.write_event(Event::Text(BytesText::new(title)))?;
                writer.write_event(Event::End(BytesEnd::new("text")))?;
            }

            // text
            {
                writer.write_event(Event::Start(BytesStart::new("text").with_attributes([("id", "2")])))?;
                writer.write_event(Event::Text(BytesText::new(message)))?;
                writer.write_event(Event::End(BytesEnd::new("text")))?;
            }

            writer.write_event(Event::End(BytesEnd::new("binding")))?;
        }

        writer.write_event(Event::End(BytesEnd::new("visual")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("toast")))?;

    let xml = XmlDocument::new()?;

    let str = &HSTRING::from(from_utf8(&writer.into_inner())?);
    xml.LoadXml(str)?;

    let notification = ToastNotification::CreateToastNotification(&xml)?;

    notifier.Show(&notification)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::win32::notification::send_notification;

    const TEST_APP_ID: &str = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

    #[test]
    fn test_send_notification() -> Result<(), Box<dyn Error>> {
        send_notification(TEST_APP_ID, "Clash Compat Library", "This is a test")?;

        Ok(())
    }
}
