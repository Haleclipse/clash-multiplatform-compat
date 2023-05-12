package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Path;

public class ShellTest {
    @Test
    public void isSupported() {
        ShellCompat.isSupported();
    }

    @Test
    public void runPickFile() {
        Assumptions.assumeTrue(ShellCompat.isSupported());

        Assertions.assertTrue(Window.showIsSuccessWindow("Pick File", frame -> {
            final long nativeHandle;
            try {
                nativeHandle = Window.getNativeHandle(frame);
            } catch (final ReflectiveOperationException e) {
                throw new RuntimeException(e);
            }

            final Path result;
            try {
                result = ShellCompat.runPickFile(nativeHandle, null, null);
            } catch (final IOException e) {
                throw new RuntimeException(e);
            }

            System.out.println(result);

            Assertions.assertNotEquals(0, nativeHandle);
        }));
    }

    @Test
    public void runLaunchFile() {
        Assumptions.assumeTrue(ShellCompat.isSupported());

        Assertions.assertTrue(Window.showIsSuccessWindow("Launch File", frame -> {
            final long nativeHandle;
            try {
                nativeHandle = Window.getNativeHandle(frame);
            } catch (final ReflectiveOperationException e) {
                throw new RuntimeException(e);
            }

            final Path result;
            try {
                result = ShellCompat.runPickFile(nativeHandle, null, null);
            } catch (final IOException e) {
                throw new RuntimeException(e);
            }

            try {
                assert result != null;

                ShellCompat.runLaunchFile(nativeHandle, result);
            } catch (final IOException e) {
                throw new RuntimeException(e);
            }
        }));
    }

    @Test
    public void setRemoveAutoStartEntry() throws Exception {
        ShellCompat.setRunOnBoot(
                "clash-multiplatform-compat-library",
                Path.of(System.getProperty("java.home")).resolve("bin").resolve("java")
        );

        Assertions.assertTrue(ShellCompat.isRunOnBootExisted("clash-multiplatform-compat-library"));

        ShellCompat.removeRunOnBoot("clash-multiplatform-compat-library");

        Assertions.assertFalse(ShellCompat.isRunOnBootExisted("clash-multiplatform-compat-library"));
    }
}
