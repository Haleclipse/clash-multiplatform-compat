use std::{collections::HashSet, ffi::CString, mem::size_of, ptr::null_mut};

use cstr::cstr;
use windows::{
    core::{PCSTR, PSTR},
    imp::{CloseHandle, WaitForSingleObject},
    Win32::{
        Foundation::{
            GetLastError, ERROR_INSUFFICIENT_BUFFER, FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
            STATUS_PENDING, TRUE,
        },
        Storage::FileSystem::{CreateFileA, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::Threading::{
            CreateProcessA, GetExitCodeProcess, InitializeProcThreadAttributeList, TerminateProcess, UpdateProcThreadAttribute,
            EXTENDED_STARTUPINFO_PRESENT, INFINITE, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST, STARTF_USESTDHANDLES, STARTUPINFOA, STARTUPINFOEXA,
        },
    },
};

use crate::{
    common::file::FileDescriptor,
    utils::scoped::Scoped,
    win32::{error::Error, file::set_file_descriptor_inheritable},
};

fn close_handle(h: HANDLE) {
    unsafe {
        CloseHandle(h.0);
    }
}

pub fn create_process(
    executable: &str,
    arguments: &[String],
    working_dir: &str,
    environments: &[String],
    extra_fds: &[FileDescriptor],
    stdin: Option<FileDescriptor>,
    stdout: Option<FileDescriptor>,
    stderr: Option<FileDescriptor>,
) -> Result<FileDescriptor, Box<dyn std::error::Error>> {
    unsafe {
        let nul_file = CreateFileA(
            PCSTR(cstr!("nul:").as_ptr().cast()),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            Default::default(),
            INVALID_HANDLE_VALUE,
        )?;
        let nul_file = Scoped::new(nul_file, |h| close_handle(*h));

        set_file_descriptor_inheritable(nul_file.0 as FileDescriptor, true)?;

        let executable = CString::new(executable)?;
        let mut joined_arguments = String::new() + "\"" + &arguments.join("\" \"") + "\"\0";
        let working_dir = CString::new(working_dir)?;
        let mut joined_environments = Vec::<u8>::new();
        for env in environments {
            joined_environments.extend_from_slice(env.as_bytes());
            joined_environments.push(b'\0');
        }
        joined_environments.push(b'\0');

        let stdin = stdin.map(|v| HANDLE(v as isize)).unwrap_or(*nul_file);
        let stdout = stdout.map(|v| HANDLE(v as isize)).unwrap_or(*nul_file);
        let stderr = stderr.map(|v| HANDLE(v as isize)).unwrap_or(*nul_file);

        let mut attributes_size: usize = 0;
        if InitializeProcThreadAttributeList(
            LPPROC_THREAD_ATTRIBUTE_LIST(null_mut()),
            1,
            0,
            &mut attributes_size as *mut usize,
        ) == FALSE
            && GetLastError() != ERROR_INSUFFICIENT_BUFFER
        {
            return Err(Error::with_current("InitializeProcThreadAttributeList").into());
        }

        let mut attributes = vec![0 as u8; attributes_size];
        if InitializeProcThreadAttributeList(
            LPPROC_THREAD_ATTRIBUTE_LIST(attributes.as_mut_ptr().cast()),
            1,
            0,
            &mut attributes_size as *mut usize,
        ) == FALSE
        {
            return Err(Error::with_current("InitializeProcThreadAttributeList").into());
        }

        let mut inheritable_handles = HashSet::new();
        for fd in extra_fds {
            inheritable_handles.insert(*fd as isize);
        }
        inheritable_handles.insert(stdin.0);
        inheritable_handles.insert(stdout.0);
        inheritable_handles.insert(stderr.0);

        let inheritable_handles = inheritable_handles.into_iter().map(|v| HANDLE(v)).collect::<Vec<_>>();

        if UpdateProcThreadAttribute(
            LPPROC_THREAD_ATTRIBUTE_LIST(attributes.as_mut_ptr().cast()),
            0,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
            Some(inheritable_handles.as_ptr().cast()),
            inheritable_handles.len() * size_of::<HANDLE>(),
            None,
            None,
        ) == FALSE
        {
            return Err(Error::with_current("UpdateProcThreadAttribute").into());
        }

        let mut startup_info = STARTUPINFOEXA::default();
        startup_info.StartupInfo.cb = size_of::<STARTUPINFOEXA>() as u32;
        startup_info.StartupInfo.hStdInput = stdin;
        startup_info.StartupInfo.hStdOutput = stdout;
        startup_info.StartupInfo.hStdError = stderr;
        startup_info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup_info.lpAttributeList = LPPROC_THREAD_ATTRIBUTE_LIST(attributes.as_mut_ptr().cast());

        let mut process_info = PROCESS_INFORMATION::default();
        if CreateProcessA(
            PCSTR(executable.as_ptr().cast()),
            PSTR(joined_arguments.as_mut_ptr()),
            None,
            None,
            TRUE,
            EXTENDED_STARTUPINFO_PRESENT,
            Some(joined_environments.as_ptr().cast()),
            PCSTR(working_dir.as_ptr().cast()),
            &startup_info.StartupInfo as *const STARTUPINFOA,
            &mut process_info as *mut PROCESS_INFORMATION,
        ) == FALSE
        {
            return Err(Error::with_current("CreateProcessA").into());
        }

        close_handle(process_info.hThread);

        Ok(process_info.hProcess.0 as FileDescriptor)
    }
}

pub fn wait_process(handle: FileDescriptor) -> i32 {
    let mut ret: i32 = -1;

    unsafe {
        while GetExitCodeProcess(HANDLE(handle as isize), (&mut ret as *mut i32).cast()) == TRUE && ret == STATUS_PENDING.0 {
            WaitForSingleObject(handle as isize, INFINITE);
        }
    }

    ret
}

pub fn kill_process(handle: FileDescriptor) {
    unsafe {
        TerminateProcess(HANDLE(handle as isize), 255);
    }
}

pub fn release_process(handle: FileDescriptor) {
    close_handle(HANDLE(handle as isize))
}
