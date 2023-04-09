use std::{error::Error, ffi::CString, iter::once, os::fd::RawFd, ptr::null};

use cstr::cstr;

use libc::{
    c_char, c_int, dup2, fchdir, fexecve, fork, kill, open, pid_t, waitpid, O_CLOEXEC, O_DIRECTORY, O_RDONLY, O_RDWR, SIGKILL,
    STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO,
};

use crate::{common::file::FileDescriptor, linux::errno::syscall, utils::scoped::Scoped};

pub fn create_process(
    executable: &str,
    arguments: &[String],
    working_dir: &str,
    environments: &[String],
    extra_fds: &[FileDescriptor],
    stdin: Option<FileDescriptor>,
    stdout: Option<FileDescriptor>,
    stderr: Option<FileDescriptor>,
) -> Result<RawFd, Box<dyn Error>> {
    unsafe {
        let nul_fd = Scoped::new_fd(syscall(|| open(cstr!("/dev/null").as_ptr(), O_RDWR, O_CLOEXEC))?);

        let executable = CString::new(executable)?;
        let executable_fd = Scoped::new_fd(syscall(|| open(executable.as_ptr(), O_RDONLY | O_CLOEXEC))?);

        let working_dir = CString::new(working_dir)?;
        let working_dir_fd = Scoped::new_fd(syscall(|| open(working_dir.as_ptr(), O_RDONLY | O_DIRECTORY | O_CLOEXEC))?);

        syscall(|| match fork() {
            0 => {
                let do_exec = || -> ! {
                    syscall(|| fchdir(*working_dir_fd)).unwrap();

                    for dup_pair in [(stdin, STDIN_FILENO), (stdout, STDOUT_FILENO), (stderr, STDERR_FILENO)] {
                        syscall(|| {
                            if let Some(fd) = dup_pair.0 {
                                dup2(fd as i32, dup_pair.1)
                            } else {
                                dup2(*nul_fd, dup_pair.1)
                            }
                        })
                        .unwrap();
                    }

                    if let Ok(dir) = std::fs::read_dir("/proc/self/fd") {
                        for entry in dir.into_iter() {
                            if let Ok(entry) = entry {
                                let fd = entry.file_name().to_str().unwrap().parse::<i32>();
                                if let Ok(fd) = fd {
                                    if fd == *executable_fd || fd == STDIN_FILENO || fd == STDOUT_FILENO || fd == STDERR_FILENO {
                                        continue;
                                    }

                                    let fd = fd as FileDescriptor;
                                    if extra_fds.contains(&fd) {
                                        continue;
                                    }
                                }
                            }
                        }
                    }

                    let arguments = arguments
                        .iter()
                        .map(|s| CString::new(s.as_bytes()).unwrap())
                        .collect::<Vec<CString>>();
                    let arguments = arguments
                        .iter()
                        .map(|s| s.as_ptr())
                        .chain(once(null()))
                        .collect::<Vec<*const c_char>>();

                    let environments = environments
                        .iter()
                        .map(|s| CString::new(s.as_bytes()).unwrap())
                        .collect::<Vec<CString>>();
                    let environments = environments
                        .iter()
                        .map(|s| s.as_ptr())
                        .chain(once(null()))
                        .collect::<Vec<*const c_char>>();

                    syscall(|| fexecve(*executable_fd, arguments.as_ptr(), environments.as_ptr())).unwrap();

                    panic!("unreachable");
                };

                do_exec()
            }
            pid => Ok(pid as RawFd),
        })?
    }
}

pub fn wait_process(handle: FileDescriptor) -> i32 {
    unsafe {
        let mut ret: c_int = -1;

        waitpid(handle as pid_t, &mut ret, 0);

        ret
    }
}

pub fn kill_process(handle: FileDescriptor) {
    unsafe {
        kill(handle as pid_t, SIGKILL);
    }
}
