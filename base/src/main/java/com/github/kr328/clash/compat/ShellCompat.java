package com.github.kr328.clash.compat;

import org.jetbrains.annotations.Blocking;
import org.jetbrains.annotations.NonBlocking;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.io.IOException;
import java.nio.file.Path;
import java.util.List;

public final class ShellCompat {
    static {
        CompatLibrary.load();
    }

    private static native boolean nativeIsSupported();

    public static boolean isSupported() {
        return nativeIsSupported();
    }

    private static native @Nullable String nativeRunPickFile(
            long windowHandle,
            String windowTitle,
            @NotNull NativePickerFilter[] filters
    ) throws IOException;

    @Nullable
    @Blocking
    public static Path runPickFile(
            final long windowHandle,
            @Nullable String windowTitle,
            @Nullable List<PickerFilter> filters
    ) throws IOException {
        if (windowTitle == null) {
            windowTitle = "Open...";
        }

        if (filters == null) {
            filters = List.of(new PickerFilter("All Files", List.of("*")));
        }

        final NativePickerFilter[] nativeFilers = filters.stream()
                .map(f -> new NativePickerFilter(f.name, f.extensions.toArray(new String[0])))
                .toArray(NativePickerFilter[]::new);

        final String path = nativeRunPickFile(windowHandle, windowTitle, nativeFilers);
        if (path != null) {
            return Path.of(path);
        }

        return null;
    }

    private static native void nativeRunLaunchFile(
            long windowHandle,
            final @Nullable String path
    ) throws IOException;

    @NonBlocking
    public static void runLaunchFile(
            final long windowHandle,
            @NotNull final Path path
    ) throws IOException {
        nativeRunLaunchFile(windowHandle, path.toAbsolutePath().toString());
    }

    public record PickerFilter(String name, List<String> extensions) {
    }

    private record NativePickerFilter(String name, String[] extensions) {
    }
}
