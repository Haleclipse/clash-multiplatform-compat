package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.io.FileDescriptor;
import java.io.IOException;
import java.lang.ref.Cleaner;
import java.nio.file.Path;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.CompletionStage;

public final class ProcessCompat {
    static {
        CompatLibrary.load();
    }

    private static native long nativeCreateProcess(
            @NotNull final String path,
            @NotNull final String[] args,
            @NotNull final String workingDir,
            @NotNull final String[] environments,
            @NotNull final FileDescriptor[] extraFds,
            @Nullable final FileDescriptor fdStdin,
            @Nullable final FileDescriptor fdStdout,
            @Nullable final FileDescriptor fdStderr
    ) throws IOException;

    private static native int nativeWaitProcess(long handle);

    @NotNull
    public static Process createProcess(
            @NotNull final Path executablePath,
            @NotNull final List<String> arguments,
            @Nullable final Path workingDir,
            @Nullable final Map<String, String> environments,
            @Nullable final FileDescriptor fdStdin,
            @Nullable final FileDescriptor fdStdout,
            @Nullable final FileDescriptor fdStderr,
            @Nullable final List<FileDescriptor> fds
    ) throws IOException {
        final String nativeExecutablePath = Objects.requireNonNull(executablePath)
                .toAbsolutePath().toString();
        final String[] nativeArguments = Objects.requireNonNull(arguments)
                .toArray(String[]::new);
        final String nativeWorkingDir = Objects.requireNonNullElse(workingDir, Path.of("."))
                .toAbsolutePath().toString();
        final String[] nativeEnvironments = Objects.requireNonNullElse(environments, System.getenv())
                .entrySet().stream()
                .map(e -> e.getKey() + "=" + e.getValue())
                .toArray(String[]::new);
        final FileDescriptor[] nativeFds = Objects.requireNonNullElse(fds, Collections.<FileDescriptor>emptyList())
                .toArray(FileDescriptor[]::new);

        final long handle = nativeCreateProcess(
                nativeExecutablePath,
                nativeArguments,
                nativeWorkingDir,
                nativeEnvironments,
                nativeFds,
                fdStdin,
                fdStdout,
                fdStderr
        );

        final CompletableFuture<Integer> result = new CompletableFuture<>();
        final Thread monitor = new Thread(() -> result.complete(nativeWaitProcess(handle)));

        monitor.setDaemon(true);
        monitor.start();

        return new Process(handle, result);
    }

    private static native void nativeKillProcess(long handle);

    private static native void nativeReleaseProcess(long handle);

    public static class Process implements AutoCloseable {
        private static final Cleaner cleaner = Cleaner.create();

        @NotNull
        private final Cleaner.Cleanable cleanable;
        @NotNull
        private final CompletableFuture<Integer> result;

        private Process(final long handle, @NotNull final CompletableFuture<Integer> result) {
            this.cleanable = cleaner.register(this, () -> {
                nativeKillProcess(handle);
                nativeReleaseProcess(handle);
            });

            this.result = result;
        }

        @Override
        public void close() {
            cleanable.clean();
        }

        @NotNull
        public CompletionStage<Integer> getResult() {
            return result.minimalCompletionStage();
        }
    }
}
