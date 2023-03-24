package com.github.kr328.clash.compat

import com.github.kr328.clash.compat.os.syscall
import kotlinx.cinterop.*
import windows.*

actual object FileCompat {
    actual val placements: FileDescriptorPlacement = FileDescriptorPlacement(
        file = FileDescriptorPlacement.Placement.Handle,
        socket = FileDescriptorPlacement.Placement.Fd
    )

    actual fun closeFileDescriptor(fd: FileDescriptor) {
        CloseHandle(fd.toCPointer())
    }

    actual fun setFileDescriptorInheritable(fd: FileDescriptor, inheritable: Boolean) {
        syscall("SetHandleInformation") {
            SetHandleInformation(
                fd.toCPointer(),
                HANDLE_FLAG_INHERIT,
                (if (inheritable) TRUE else FALSE).toUInt()
            )
        }
    }

    actual fun createSocketPair(): Pair<FileDescriptor, FileDescriptor> = memScoped {
        val server: SOCKETVar = alloc()
        val first: SOCKETVar = alloc()
        val second: SOCKETVar = alloc()

        server.value = syscall("WSASocketA") {
            WSASocketA(AF_UNIX, SOCK_STREAM, 0, null, 0, WSA_FLAG_OVERLAPPED)
        }

        setFileDescriptorInheritable(server.value.toLong(), false)

        try {
            val tempDirArray: CArrayPointer<ByteVar> = allocArray(MAX_PATH)

            syscall("GetTempPathA") {
                GetTempPathA(MAX_PATH, tempDirArray.pointed.ptr)
            }

            val tempPathArray: CArrayPointer<ByteVar> = allocArray(MAX_PATH)

            syscall("GetTempFileNameA") {
                GetTempFileNameA(tempDirArray.toKString(), "sp", 0, tempPathArray.pointed.ptr)
            }

            DeleteFileA(tempPathArray.toKString())

            val addr: sockaddr_un = alloc()
            addr.sun_family = AF_UNIX.toUShort()
            memcpy(addr.sun_path, tempPathArray.pointed.ptr, 108 - 1)

            syscall("bind") {
                bind(server.value, addr.ptr.reinterpret(), sizeOf<sockaddr_un>().toInt())
            }

            syscall("listen") {
                listen(server.value, 4)
            }

            first.value = syscall("WSASocketA") {
                WSASocketA(AF_UNIX, SOCK_STREAM, 0, null, 0, WSA_FLAG_OVERLAPPED)
            }

            setFileDescriptorInheritable(first.value.toLong(), false)

            syscall("connect") {
                connect(first.value, addr.ptr.reinterpret(), sizeOf<sockaddr_un>().toInt())
            }

            syscall("accept") {
                val addrSize: IntVar = alloc(sizeOf<sockaddr_un>().toInt())

                second.value = accept(server.value, addr.ptr.reinterpret(), addrSize.ptr)
            }

            setFileDescriptorInheritable(second.value.toLong(), false)

            DeleteFileA(tempPathArray.toKString())

            val ret = first.value.toLong() to second.value.toLong()

            first.value = invalidSocket()
            second.value = invalidSocket()

            ret
        } finally {
            closesocket(server.value)
            closesocket(first.value)
            closesocket(second.value)
        }
    }

    actual fun createPipe(): Pair<FileDescriptor, FileDescriptor> = memScoped {
        val reader: HANDLEVar = alloc()
        val writer: HANDLEVar = alloc()

        syscall("CreatePipe") {
            CreatePipe(reader.ptr, writer.ptr, null, 4096)
        }

        try {
            setFileDescriptorInheritable(reader.value!!.toLong(), false)
            setFileDescriptorInheritable(writer.value!!.toLong(), false)

            val ret = reader.value!!.toLong() to writer.value!!.toLong()

            reader.value = INVALID_HANDLE_VALUE
            writer.value = INVALID_HANDLE_VALUE

            ret
        } finally {
            CloseHandle(reader.value)
            CloseHandle(writer.value)
        }
    }
}
