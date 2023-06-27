use std::{collections::HashSet, mem::size_of, ptr::null_mut};

use windows::{
    core::{PCWSTR, PWSTR},
    imp::{CloseHandle, WaitForSingleObject},
    w,
    Win32::{
        Foundation::{
            GetLastError, ERROR_INSUFFICIENT_BUFFER, FALSE, GENERIC_READ, GENERIC_WRITE, HANDLE, INVALID_HANDLE_VALUE,
            STATUS_PENDING, TRUE,
        },
        Storage::FileSystem::{CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING},
        System::Threading::{
            CreateProcessW, GetExitCodeProcess, InitializeProcThreadAttributeList, TerminateProcess, UpdateProcThreadAttribute,
            CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, INFINITE, LPPROC_THREAD_ATTRIBUTE_LIST,
            PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_HANDLE_LIST, STARTF_USESTDHANDLES, STARTUPINFOEXW, STARTUPINFOW,
        },
        UI::WindowsAndMessaging::SW_HIDE,
    },
};

use crate::{
    common::file::FileDescriptor,
    utils::scoped::Scoped,
    win32::{
        error::Error,
        file::set_file_descriptor_inheritable,
        strings::{join_arguments, Win32StringIntoExt},
    },
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
        let nul_file = CreateFileW(
            w!("nul:"),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            Default::default(),
            INVALID_HANDLE_VALUE,
        )?;
        let nul_file = Scoped::new(nul_file, |h| close_handle(*h));

        set_file_descriptor_inheritable(nul_file.0 as FileDescriptor, true)?;

        let executable = executable.to_win32_utf16();
        let mut joined_arguments = join_arguments(arguments).to_win32_utf16();
        let working_dir = working_dir.to_win32_utf16();
        let mut joined_environments = Vec::<u16>::new();
        for env in environments {
            joined_environments.extend(env.encode_utf16());
            joined_environments.push(0);
        }
        joined_environments.push(0);

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

        let mut startup_info = STARTUPINFOEXW::default();
        startup_info.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup_info.StartupInfo.hStdInput = stdin;
        startup_info.StartupInfo.hStdOutput = stdout;
        startup_info.StartupInfo.hStdError = stderr;
        startup_info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup_info.StartupInfo.wShowWindow = SW_HIDE.0 as u16;
        startup_info.lpAttributeList = LPPROC_THREAD_ATTRIBUTE_LIST(attributes.as_mut_ptr().cast());

        let mut process_info = PROCESS_INFORMATION::default();
        if CreateProcessW(
            PCWSTR::from_raw(executable.as_ptr()),
            PWSTR::from_raw(joined_arguments.as_mut_ptr()),
            None,
            None,
            TRUE,
            EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
            Some(joined_environments.as_ptr().cast()),
            PCWSTR::from_raw(working_dir.as_ptr()),
            &startup_info.StartupInfo as *const STARTUPINFOW,
            &mut process_info as *mut PROCESS_INFORMATION,
        ) == FALSE
        {
            return Err(Error::with_current("CreateProcessW").into());
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
