use std::{io, io::Read};

pub fn get_uid() -> u32 {
    unsafe { libc::geteuid() }
}

pub fn get_gid() -> u32 {
    unsafe { libc::getegid() }
}

pub fn get_selinux_context() -> io::Result<String> {
    let mut file = std::fs::File::open("/proc/self/attr/current")?;
    let mut ret = String::new();

    file.read_to_string(&mut ret)?;

    Ok(ret)
}
