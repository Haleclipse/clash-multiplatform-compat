package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Assumptions;
import org.junit.jupiter.api.Test;

import java.io.IOException;

public class WindowTest {
    @Test
    void isSupported() {
        WindowCompat.isSupported();
    }

    @Test
    void setBorderless() {
        Assumptions.assumeTrue(WindowCompat.isSupported());

        Assertions.assertTrue(Window.showIsSuccessWindow("Window", true, frame -> {
            final WindowCompat.Context ctx;
            try {
                ctx = WindowCompat.setBorderless(Window.getNativeHandle(frame));
            } catch (final IOException | ReflectiveOperationException e) {
                throw new RuntimeException(e);
            }

            ctx.setFrameSize(WindowCompat.WindowFrame.EDGE_INSETS, 5);
            ctx.setFrameSize(WindowCompat.WindowFrame.TITLE_BAR, 40);
            ctx.setControlPosition(WindowCompat.WindowControl.CLOSE_BUTTON, 0, 0, 40, 40);
            ctx.setControlPosition(WindowCompat.WindowControl.BACK_BUTTON, 80, 0, 40, 40);
        }));
    }
}
