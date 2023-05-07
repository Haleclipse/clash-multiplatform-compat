package com.github.kr328.clash.compat;

import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

public final class AppTest {
    @Test
    public void setApplicationID() {
        AppCompat.setProcessApplicationID("com.github.kr328.clash.test.ClashCompatTest");

        Assertions.assertTrue(Window.showIsSuccessWindow("Set Application ID"));
    }
}
