package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

public class ThemeTest {
    @Test
    void isSupported() {
        ThemeCompat.isSupported();
    }

    @Test
    void isNight() throws Exception {
        Assumptions.assumeTrue(ThemeCompat.isSupported());

        final boolean isNight = ThemeCompat.isNight();
        System.out.println("isNight = " + isNight);

        Assertions.assertTrue(Window.showIsSuccessWindow("Theme"));
    }

    @Test
    void addListener() throws Exception {
        Assumptions.assumeTrue(ThemeCompat.isSupported());

        final ThemeCompat.Disposable disposable = ThemeCompat.addListener((v) -> System.out.println("onChanged = " + v));

        Assertions.assertTrue(Window.showIsSuccessWindow("Theme Listener (Added)"));

        disposable.dispose();

        Assertions.assertTrue(Window.showIsSuccessWindow("Theme Listener (Disposed)"));
    }
}
