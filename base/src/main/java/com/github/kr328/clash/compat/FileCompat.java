package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;

import java.io.FileDescriptor;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.IOException;
import java.lang.ref.Cleaner;
import java.net.ProtocolFamily;
import java.net.SocketAddress;
import java.net.StandardProtocolFamily;
import java.net.UnixDomainSocketAddress;
import java.nio.channels.SocketChannel;
import java.nio.channels.spi.SelectorProvider;

public final class FileCompat {
    private static final Cleaner fdCleaner = Cleaner.create();

    static {
        CompatLibrary.load();
    }

    private static native FileDescriptor nativeGetFileDescriptorFromSocketChannel(
            @NotNull final SocketChannel channel
    );

    public static FileDescriptor getFileDescriptorFromSocketChannel(
            @NotNull final SocketChannel channel
    ) {
        return nativeGetFileDescriptorFromSocketChannel(channel);
    }

    private static native long nativeGetFileDescriptorHandle(
            @NotNull final FileDescriptor fileDescriptor
    );

    public static long getFileDescriptorHandle(
            @NotNull final FileDescriptor fileDescriptor
    ) {
        return nativeGetFileDescriptorHandle(fileDescriptor);
    }

    private static native void nativeSetFileDescriptorInheritable(
            @NotNull final FileDescriptor fd,
            final boolean inheritable
    );

    public static void setFileDescriptorInheritable(
            @NotNull final FileDescriptor fd,
            final boolean inheritable
    ) {
        nativeSetFileDescriptorInheritable(fd, inheritable);
    }

    private static native void nativeCloseFileDescriptor(
            @NotNull final FileDescriptor fd
    );

    private static native void nativeCreatePipe(
            @NotNull final FileDescriptor reader,
            @NotNull final FileDescriptor writer
    ) throws IOException;

    public static @NotNull Pipe createPipe() throws IOException {
        final FileDescriptor reader = new FileDescriptor();
        final FileDescriptor writer = new FileDescriptor();

        nativeCreatePipe(reader, writer);

        final FileInputStream readerStream = new FileInputStream(reader);
        final FileOutputStream writerStream = new FileOutputStream(writer);

        fdCleaner.register(readerStream, () -> nativeCloseFileDescriptor(reader));
        fdCleaner.register(writerStream, () -> nativeCloseFileDescriptor(writer));

        return new Pipe(readerStream, writerStream);
    }

    private static native void nativeCreateUnixSocketPair(
            @NotNull final FileDescriptor first,
            @NotNull final FileDescriptor second
    ) throws IOException;

    private static native SocketChannel nativeNewSocketChannel(
            final SelectorProvider sp,
            final ProtocolFamily family,
            final FileDescriptor fd,
            final SocketAddress address
    );

    public static UnixSocketPair createUnixSocketPair() throws IOException {
        final FileDescriptor first = new FileDescriptor();
        final FileDescriptor second = new FileDescriptor();

        nativeCreateUnixSocketPair(first, second);

        final SocketChannel firstChannel = nativeNewSocketChannel(
                SelectorProvider.provider(),
                StandardProtocolFamily.UNIX,
                first,
                UnixDomainSocketAddress.of("socketpair-0")
        );
        final SocketChannel secondChannel = nativeNewSocketChannel(
                SelectorProvider.provider(),
                StandardProtocolFamily.UNIX,
                second,
                UnixDomainSocketAddress.of("socketpair-1")
        );

        fdCleaner.register(firstChannel, () -> nativeCloseFileDescriptor(first));
        fdCleaner.register(secondChannel, () -> nativeCloseFileDescriptor(second));

        return new UnixSocketPair(firstChannel, secondChannel);
    }

    public record Pipe(@NotNull FileInputStream reader, @NotNull FileOutputStream writer) implements AutoCloseable {
        @Override
        public void close() {
            Utils.closeSilent(reader);
            Utils.closeSilent(writer);
        }
    }

    public record UnixSocketPair(@NotNull SocketChannel first, @NotNull SocketChannel second) implements AutoCloseable {
        @Override
        public void close() {
            Utils.closeSilent(first);
            Utils.closeSilent(second);
        }
    }
}
