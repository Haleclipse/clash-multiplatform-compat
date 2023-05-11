use std::iter::once;

pub fn string_to_os_utf16(string: &str) -> Vec<u16> {
    return string.encode_utf16().chain(once(0)).collect::<Vec<u16>>();
}

pub fn join_arguments(arguments: &[String]) -> String {
    if arguments.is_empty() {
        return "".to_owned();
    }

    let mut ret = String::new();

    for s in arguments {
        ret.push('"');

        for c in s.chars() {
            if c == '"' {
                ret.push('\\');
            }

            ret.push(c);
        }

        ret.push('"');

        ret.push(' ')
    }

    ret
}
