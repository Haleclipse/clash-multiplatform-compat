use std::iter::once;

pub fn string_to_os_utf16(string: &str) -> Vec<u16> {
    return string.encode_utf16().chain(once(0)).collect::<Vec<u16>>();
}
