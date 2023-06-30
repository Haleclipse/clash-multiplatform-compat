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
            @NotNull String windowTitle,
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

        final NativePickerFilter[] nativeFilters = filters.stream()
                .map(f -> new NativePickerFilter(f.name, f.extensions.toArray(new String[0])))
                .toArray(NativePickerFilter[]::new);

        final String path = nativeRunPickFile(windowHandle, windowTitle, nativeFilters);
        if (path != null) {
            return Path.of(path);
        }

        return null;
    }

    private static native @Nullable String nativeRunSaveFile(
            final long windowHandle,
            @NotNull final String fileName,
            @NotNull String windowTitle,
            @NotNull NativePickerFilter[] filters
    ) throws IOException;

    @Nullable
    @Blocking
    public static Path runSaveFile(
            final long windowHandle,
            @NotNull final String fileName,
            @Nullable String windowTitle,
            @Nullable List<PickerFilter> filters
    ) throws IOException {
        if (windowTitle == null) {
            windowTitle = "Save...";
        }

        if (filters == null) {
            filters = List.of(new PickerFilter("All Files", List.of("*")));
        }

        final NativePickerFilter[] nativeFilters = filters.stream()
                .map(f -> new NativePickerFilter(f.name, f.extensions.toArray(new String[0])))
                .toArray(NativePickerFilter[]::new);

        final String path = nativeRunSaveFile(windowHandle, fileName, windowTitle, nativeFilters);
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

    private static native void nativeInstallIcon(@NotNull final String name, final byte @NotNull [] data) throws IOException;

    public static void installIcon(@NotNull final String name, final byte @NotNull [] data) throws IOException {
        nativeInstallIcon(name, data);
    }

    private static native void nativeInstallShortcut(
            @NotNull final String applicationId,
            @NotNull final String applicationName,
            @NotNull final String iconName,
            @NotNull final String executablePath,
            @NotNull final String[] arguments
    ) throws IOException;

    public static void installShortcut(@NotNull final String applicationId,
                                       @NotNull final String applicationName,
                                       @NotNull final String iconName,
                                       @NotNull final Path executablePath,
                                       @NotNull final String... arguments
    ) throws IOException {
        nativeInstallShortcut(applicationId, applicationName, iconName, executablePath.toAbsolutePath().toString(), arguments);
    }

    private static native void nativeUninstallShortcut(
            @NotNull final String applicationId,
            @NotNull final String applicationName
    ) throws IOException;

    public static void uninstallShortcut(@NotNull final String applicationId, @NotNull final String applicationName) throws IOException {
        nativeUninstallShortcut(applicationId, applicationName);
    }

    private static native boolean nativeIsRunOnBootExisted(@NotNull final String applicationId);

    public static boolean isRunOnBootExisted(@NotNull final String applicationId) {
        return nativeIsRunOnBootExisted(applicationId);
    }

    private static native void nativeSetRunOnBoot(
            @NotNull final String applicationId,
            @NotNull final String executablePath,
            @NotNull final String @NotNull [] arguments
    ) throws IOException;

    public static void setRunOnBoot(
            @NotNull final String applicationId,
            @NotNull final Path executablePath,
            @NotNull final String... arguments) throws IOException {
        nativeSetRunOnBoot(applicationId, executablePath.toAbsolutePath().toString(), arguments);
    }

    private static native void nativeRemoveRunOnBoot(@NotNull final String applicationId) throws IOException;

    public static void removeRunOnBoot(@NotNull final String applicationId) throws IOException {
        nativeRemoveRunOnBoot(applicationId);
    }

    public record PickerFilter(@NotNull String name, @NotNull List<@NotNull String> extensions) {
    }

    private record NativePickerFilter(@NotNull String name, @NotNull String @NotNull [] extensions) {
    }
}
