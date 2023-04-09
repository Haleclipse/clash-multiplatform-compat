package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.awt.*;
import java.io.InputStream;
import java.util.List;
import java.util.Locale;
import java.util.Objects;

public class NotifierTest {
    @Test
    void create() throws Exception {
        Assumptions.assumeTrue(NotifierCompat.isSupported());

        final byte[] iconData;
        try (final InputStream stream = Objects.requireNonNull(NotificationTest.class.getResource("/clash-multiplatform.ico")).openStream()) {
            iconData = stream.readAllBytes();
        }

        ShellCompat.installIcon("clash-multiplatform", iconData);

        final NotifierCompat.Notifier notifier = NotifierCompat.add(
                "clash-multiplatform-compat-library",
                "Clash for Desktop",
                "clash-multiplatform",
                ComponentOrientation.getOrientation(Locale.getDefault()).isLeftToRight(),
                new NotifierCompat.Listener() {
                    @Override
                    public void onActive() {
                        System.out.println("onActive");
                    }

                    @Override
                    public void onMenuActive(final short id) {
                        System.out.println("onMenuClick " + id);
                    }
                });

        notifier.setMenuItems(List.of(
                new NotifierCompat.MenuItem.Item("Item 114", (short) 114),
                new NotifierCompat.MenuItem.SubMenu("Sub Item", List.of(
                        new NotifierCompat.MenuItem.Item("Item 514", (short) 514),
                        new NotifierCompat.MenuItem.Item("Item 1919", (short) 1919),
                        new NotifierCompat.MenuItem.Item("Item 810", (short) 810)
                ))
        ));

        Assertions.assertTrue(Window.showIsSuccessWindow("Notifier"));

        notifier.remove();

        Assertions.assertTrue(Window.showIsSuccessWindow("Notifier (Removed)"));
    }
}
