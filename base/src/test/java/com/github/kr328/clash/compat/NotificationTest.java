package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.io.InputStream;
import java.nio.file.Path;
import java.util.Objects;

public class NotificationTest {
    private static final String TEST_APP_ID = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe";

    @Test
    void showNotification() throws Exception {
        Assumptions.assumeTrue(NotificationCompat.isSupported());
        Assumptions.assumeTrue(System.getProperty("os.name").toLowerCase().contains("win"));

        NotificationCompat.sendNotification(TEST_APP_ID, "Clash Compat Library (JVM)", "This is a test from jvm.");

        Assertions.assertTrue(Window.showIsSuccessWindow("Notification"));
    }

    @Test
    void notificationWithSelfAppId() throws Exception {
        Assumptions.assumeTrue(NotificationCompat.isSupported());

        final byte[] iconData;
        try (final InputStream stream = Objects.requireNonNull(NotificationTest.class.getResource("/clash-multiplatform.ico")).openStream()) {
            iconData = stream.readAllBytes();
        }

        ShellCompat.installIcon("clash-multiplatform", iconData);

        ShellCompat.installShortcut(
                "clash-multiplatform-compat-library",
                "Clash Multiplatform Compat (JVM)",
                "clash-multiplatform",
                Path.of(System.getProperty("java.home") + "/bin/jshell")
        );

        NotificationCompat.sendNotification(
                "clash-multiplatform-compat-library",
                "Notification from Compat",
                "This is a test"
        );

        Assertions.assertTrue(Window.showIsSuccessWindow("Notification with App Icon"));

        ShellCompat.uninstallShortcut("clash-multiplatform-compat-library", "Clash Multiplatform Compat (JVM)");
    }
}
