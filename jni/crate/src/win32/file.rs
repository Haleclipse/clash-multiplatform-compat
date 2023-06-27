use std::{env::temp_dir, mem::size_of};

use rand::{thread_rng, Rng};
use windows::{
    Win32,
    Win32::{
        Foundation::{CloseHandle, SetHandleInformation, FALSE, HANDLE, HANDLE_FLAGS, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE},
        Networking::WinSock::{
            accept, bind, connect, listen, WSASocketW, ADDRESS_FAMILY, AF_UNIX, INVALID_SOCKET, SOCKADDR_UN, SOCKET, SOCK_STREAM,
            WSA_FLAG_OVERLAPPED,
        },
        System::Pipes::CreatePipe,
    },
};

use error::Error;

use crate::{common::file::FileDescriptor, utils::scoped::Scoped, win32::error};

pub fn set_file_descriptor_inheritable(fd: FileDescriptor, inheritable: bool) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let result = SetHandleInformation(
            HANDLE(fd),
            HANDLE_FLAG_INHERIT.0,
            if inheritable { HANDLE_FLAG_INHERIT } else { HANDLE_FLAGS(0) },
        );

        if result == FALSE {
            Err(Error::with_current("SetHandleInformation").into())
        } else {
            Ok(())
        }
    }
}

pub fn create_socket_pair() -> Result<(FileDescriptor, FileDescriptor), Box<dyn std::error::Error>> {
    let mut first = Scoped::new(INVALID_SOCKET, |s| close_socket(*s));
    let mut second = Scoped::new(INVALID_SOCKET, |s| close_socket(*s));

    unsafe {
        let server = WSASocketW(AF_UNIX as i32, SOCK_STREAM.0, 0, None, 0, WSA_FLAG_OVERLAPPED);
        if server == INVALID_SOCKET {
            return Err(Error::with_current("WSASocketW").into());
        }

        let server = Scoped::new(server, |s| close_socket(*s));

        let mut temp_path = temp_dir().to_str().unwrap().to_string();
        temp_path += &format!("\\socket-pair-{}-{}.socket", std::process::id(), thread_rng().gen::<u16>());

        std::fs::remove_file(&temp_path).ok();
        let _rm = Scoped::new(0, |_| {
            std::fs::remove_file(&temp_path).ok();
        });

        let mut addr = SOCKADDR_UN::default();
        addr.sun_family = ADDRESS_FAMILY(AF_UNIX);
        addr.sun_path[..temp_path.len()].copy_from_slice(temp_path.as_bytes());

        if bind(*server, (&addr as *const SOCKADDR_UN).cast(), size_of::<SOCKADDR_UN>() as i32) < 0 {
            return Err(Error::with_current("bind").into());
        }

        if listen(*server, 4) < 0 {
            return Err(Error::with_current("listen").into());
        }

        *first = WSASocketW(AF_UNIX as i32, SOCK_STREAM.0, 0, None, 0, WSA_FLAG_OVERLAPPED);
        if *first == INVALID_SOCKET {
            return Err(Error::with_current("WSASocketW").into());
        }

        if connect(*first, (&addr as *const SOCKADDR_UN).cast(), size_of::<SOCKADDR_UN>() as i32) < 0 {
            return Err(Error::with_current("connect").into());
        }

        let mut addr_length = size_of::<SOCKADDR_UN>() as i32;
        *second = accept(
            *server,
            Some((&mut addr as *mut SOCKADDR_UN).cast()),
            Some(&mut addr_length as *mut i32),
        );
        if *server == INVALID_SOCKET {
            return Err(Error::with_current("accept").into());
        }

        set_file_descriptor_inheritable(first.0 as FileDescriptor, false)?;
        set_file_descriptor_inheritable(second.0 as FileDescriptor, false)?;
    }

    Ok((
        first.swap(INVALID_SOCKET).0 as FileDescriptor,
        second.swap(INVALID_SOCKET).0 as FileDescriptor,
    ))
}

fn close_handle(handle: HANDLE) {
    unsafe {
        CloseHandle(handle);
    }
}

pub fn create_pipe() -> Result<(FileDescriptor, FileDescriptor), Box<dyn std::error::Error>> {
    let mut reader_fd = Scoped::new(HANDLE::default(), |h| close_handle(*h));
    let mut writer_fd = Scoped::new(HANDLE::default(), |h| close_handle(*h));

    unsafe {
        if CreatePipe(
            &mut reader_fd.value as *mut HANDLE,
            &mut writer_fd.value as *mut HANDLE,
            None,
            4096,
        ) == FALSE
        {
            return Err(Error::with_current("CreatePipe").into());
        }
    }

    set_file_descriptor_inheritable(reader_fd.0, false)?;
    set_file_descriptor_inheritable(reader_fd.0, false)?;

    let r = reader_fd.swap(INVALID_HANDLE_VALUE);
    let w = writer_fd.swap(INVALID_HANDLE_VALUE);

    Ok((r.0, w.0))
}

fn close_socket(socket: SOCKET) {
    unsafe {
        Win32::Networking::WinSock::closesocket(socket);
    }
}
