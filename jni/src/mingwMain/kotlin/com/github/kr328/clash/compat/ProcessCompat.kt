package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import windows.*

actual object ProcessCompat {
    actual fun createProcess(
        executablePath: String,
        arguments: List<String>,
        workingDir: String,
        environments: List<String>,
        extraFds: List<FileDescriptor>,
        fdStdin: FileDescriptor?,
        fdStdout: FileDescriptor?,
        fdStderr: FileDescriptor?,
    ): FileDescriptor = memScoped {
        val hNull: HANDLE = syscall("CreateFileA") {
            CreateFileA(
                "nul:",
                (GENERIC_READ or GENERIC_WRITE.toUInt()),
                (FILE_SHARE_READ or FILE_SHARE_WRITE).toUInt(),
                null,
                OPEN_EXISTING,
                0,
                null,
            )
        }!!

        FileCompat.setFileDescriptorInheritable(hNull.toLong(), true)

        try {
            val joinedArguments: String = arguments.joinToString(separator = " ") { "\"$it\"" }
            val joinedEnvironments: ByteArray = environments.joinToString(separator = "", postfix = "\n") { "$it\n" }
                .encodeToByteArray()
                .also {
                    for (idx in it.indices) {
                        if (it[idx] == '\n'.code.toByte()) {
                            it[idx] = 0
                        }
                    }
                }

            val hStdin: HANDLE = fdStdin?.toCPointer() ?: hNull
            val hStdout: HANDLE = fdStdout?.toCPointer() ?: hNull
            val hStderr: HANDLE = fdStderr?.toCPointer() ?: hNull
            val hExtraFds: List<HANDLE> = extraFds.map { it.toCPointer<CPointed>() as HANDLE }

            val attributesListSize: SIZE_TVar = alloc()
            syscall("InitializeProcThreadAttributeList") {
                InitializeProcThreadAttributeList(null, 1, 0, attributesListSize.ptr)
                if (GetLastError() == ERROR_INSUFFICIENT_BUFFER.toUInt()) {
                    SetLastError(ERROR_SUCCESS)
                }
            }

            val attributes: CPointer<ByteVar> = allocArray(attributesListSize.value.toLong())
            syscall("InitializeProcThreadAttributeList") {
                InitializeProcThreadAttributeList(
                    attributes.pointed.ptr.reinterpret(),
                    1,
                    0,
                    attributesListSize.ptr,
                )
            }

            val handles: List<HANDLE> = (hExtraFds + listOf(hStdin, hStdout, hStderr)).toSet().toList()
            syscall("UpdateProcThreadAttribute") {
                UpdateProcThreadAttribute(
                    attributes.pointed.ptr.reinterpret(),
                    0,
                    PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                    allocArrayOf(handles).pointed.ptr,
                    toSize(handles.size * sizeOf<HANDLEVar>()),
                    null,
                    null,
                )
            }

            val startupInfo: STARTUPINFOEXA = alloc<STARTUPINFOEXA>().apply {
                StartupInfo.cb = sizeOf<STARTUPINFOEXA>().toUInt()
                StartupInfo.hStdInput = hStdin
                StartupInfo.hStdOutput = hStdout
                StartupInfo.hStdError = hStderr
                StartupInfo.dwFlags = STARTF_USESTDHANDLES.toUInt()
                lpAttributeList = attributes.pointed.ptr.reinterpret()
            }

            val processInfo: PROCESS_INFORMATION = alloc()
            joinedEnvironments.usePinned { env ->
                syscall("CreateProcessA") {
                    CreateProcessA(
                        executablePath,
                        joinedArguments.cstr.ptr,
                        null,
                        null,
                        TRUE,
                        EXTENDED_STARTUPINFO_PRESENT,
                        env.addressOf(0).reinterpret(),
                        workingDir,
                        startupInfo.StartupInfo.ptr,
                        processInfo.ptr,
                    )
                }
            }

            CloseHandle(processInfo.hThread)

            processInfo.hProcess!!.toLong()
        } finally {
            CloseHandle(hNull)
        }
    }

    actual fun waitProcess(processFd: FileDescriptor): Int = memScoped {
        val ret: UIntVar = alloc<UIntVar>().apply {
            value = 1u
        }

        while (GetExitCodeProcess(processFd.toCPointer(), ret.ptr) == TRUE && ret.value == STATUS_PENDING) {
            WaitForSingleObject(processFd.toCPointer(), INFINITE)
        }

        ret.value.toInt()
    }

    actual fun killProcess(processFd: FileDescriptor) {
        TerminateProcess(processFd.toCPointer(), 255)
    }

    actual fun releaseProcess(processFd: FileDescriptor) {
        CloseHandle(processFd.toCPointer())
    }
}
