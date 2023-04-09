package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.io.FileDescriptor;
import java.io.IOException;
import java.lang.reflect.Field;
import java.nio.ByteBuffer;
import java.nio.channels.SocketChannel;
import java.nio.charset.StandardCharsets;

public class FileTest {
    @Test
    public void createPipe() throws IOException {
        try (final FileCompat.Pipe pipe = FileCompat.createPipe()) {
            final byte[] testData = "114514".getBytes(StandardCharsets.UTF_8);
            final byte[] buffer = new byte[64];

            for (int i = 0; i < 10; i++) {
                pipe.writer().write(testData);
                final int length = pipe.reader().read(buffer);

                Assertions.assertEquals(testData.length, length);
                Assertions.assertEquals(new String(testData, StandardCharsets.UTF_8), new String(buffer, 0, length, StandardCharsets.UTF_8));
            }
        }
    }

    @Test
    public void createSocketChannel() throws IOException {
        try (final FileCompat.UnixSocketPair pair = FileCompat.createUnixSocketPair()) {
            final ByteBuffer testData = StandardCharsets.UTF_8.encode("114514");
            final ByteBuffer buffer = ByteBuffer.allocate(64);

            for (int i = 0; i < 10; i++) {
                pair.first().write(testData.rewind());
                pair.second().read(buffer.clear());

                testData.rewind();
                buffer.flip();

                final byte[] a = new byte[testData.remaining()];
                final byte[] b = new byte[buffer.remaining()];

                testData.get(a);
                buffer.get(b);

                Assertions.assertArrayEquals(a, b);
            }
        }
    }

    @Test
    public void getFileDescriptorFromSocketChannel() throws IOException, ReflectiveOperationException {
        try (final SocketChannel channel = SocketChannel.open()) {
            final Field fFd = channel.getClass().getDeclaredField("fd");
            fFd.setAccessible(true);

            final FileDescriptor fd = (FileDescriptor) fFd.get(channel);
            Assertions.assertEquals(fd, FileCompat.getFileDescriptorFromSocketChannel(channel));
        }
    }

    @Test
    public void getFileDescriptorValue() throws IOException, ReflectiveOperationException {
        try (final SocketChannel channel = SocketChannel.open()) {
            final FileDescriptor fdObj = FileCompat.getFileDescriptorFromSocketChannel(channel);

            final Field fHandle = FileDescriptor.class.getDeclaredField("handle");
            fHandle.setAccessible(true);

            final Field fFd = FileDescriptor.class.getDeclaredField("fd");
            fFd.setAccessible(true);

            final long handle = fHandle.getLong(fdObj);
            final int fd = fFd.getInt(fdObj);

            final long ret;
            if (handle > 0) {
                ret = handle;
            } else {
                ret = fd;
            }

            Assertions.assertEquals(ret, FileCompat.getFileDescriptorHandle(fdObj));
        }
    }

    @Test
    public void setFileDescriptorInheritable() throws IOException {
        try (final SocketChannel channel = SocketChannel.open()) {
            final FileDescriptor fdObj = FileCompat.getFileDescriptorFromSocketChannel(channel);

            FileCompat.setFileDescriptorInheritable(fdObj, true);
            FileCompat.setFileDescriptorInheritable(fdObj, false);
        }
    }
}
