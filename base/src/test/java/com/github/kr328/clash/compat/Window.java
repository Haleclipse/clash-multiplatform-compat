package com.github.kr328.clash.compat;

import javax.swing.*;
import javax.swing.border.EmptyBorder;
import java.awt.*;
import java.awt.event.*;
import java.lang.reflect.Field;
import java.util.concurrent.CompletableFuture;
import java.util.function.Consumer;

public class Window {
    private static long getHandleFromWComponentPeer(final Object peer) {
        try {
            final Class<?> cWComponentPeer = Class.forName("sun.awt.windows.WComponentPeer");
            if (cWComponentPeer.isInstance(peer)) {
                final Field fHWnd = cWComponentPeer.getDeclaredField("hwnd");
                fHWnd.setAccessible(true);

                return fHWnd.getLong(peer);
            }
        } catch (final ReflectiveOperationException ignored) {

        }

        return -1;
    }

    private static long getHandleFromXComponentPeer(final Object peer) {
        try {
            final Class<?> cXBaseWindow = Class.forName("sun.awt.X11.XBaseWindow");
            if (cXBaseWindow.isInstance(peer)) {
                final Field fWindow = cXBaseWindow.getDeclaredField("window");
                fWindow.setAccessible(true);

                return fWindow.getLong(peer);
            }
        } catch (final ReflectiveOperationException ignored) {}

        return -1;
    }

    public static long getNativeHandle(final Frame frame) throws ReflectiveOperationException {
        final Field fPeer = Component.class.getDeclaredField("peer");
        fPeer.setAccessible(true);
        final Object peer = fPeer.get(frame);

        final long wHandle = getHandleFromWComponentPeer(peer);
        if (wHandle > 0) {
            return wHandle;
        }

        final long xHandle = getHandleFromXComponentPeer(peer);
        if (xHandle > 0) {
            return xHandle;
        }

        throw new IllegalArgumentException("Unsupported peer " + peer);
    }

    public static boolean showIsSuccessWindow(final String title, final boolean undecorated, final Consumer<JFrame> consumer) {
        final CompletableFuture<Boolean> isSuccess = new CompletableFuture<>();

        final JButton button = new JButton("Success");

        final JPanel panel = new JPanel();
        panel.setBorder(new EmptyBorder(80, 80, 80, 80));
        panel.setLayout(new FlowLayout());
        panel.add(new JLabel(title));
        panel.add(button);

        final JFrame frame = new JFrame("Java Test: " + title);
        frame.addWindowListener(new WindowAdapter() {
            @Override
            public void windowClosing(final WindowEvent e) {
                isSuccess.complete(false);
            }
        });
        frame.addKeyListener(new KeyAdapter() {
            @Override
            public void keyPressed(final KeyEvent e) {
                if (e.getKeyCode() == KeyEvent.VK_ESCAPE) {
                    frame.setVisible(false);
                    isSuccess.complete(false);
                }
            }
        });
        frame.add(panel);
        frame.setSize(400, 400);
        frame.setUndecorated(undecorated);
        frame.setResizable(true);
        frame.setFocusable(true);
        frame.setVisible(true);

        button.addActionListener(new AbstractAction() {
            @Override
            public void actionPerformed(final ActionEvent e) {
                frame.setVisible(false);
                isSuccess.complete(true);
            }
        });

        consumer.accept(frame);

        return isSuccess.join();
    }

    public static boolean showIsSuccessWindow(final String title, final Consumer<JFrame> consumer) {
        return showIsSuccessWindow(title, false, consumer);
    }

    public static boolean showIsSuccessWindow(final String title) {
        return showIsSuccessWindow(title, (frame) -> {});
    }
}
