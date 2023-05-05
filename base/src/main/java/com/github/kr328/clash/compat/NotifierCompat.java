package com.github.kr328.clash.compat;

import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

import java.io.IOException;
import java.lang.ref.Cleaner;
import java.util.List;

public final class NotifierCompat {
    private static final Cleaner notifierCleaner = Cleaner.create();

    static {
        CompatLibrary.load();
    }

    private static native boolean nativeIsSupported();

    public static boolean isSupported() {
        return nativeIsSupported();
    }

    private static native long nativeAdd(
            @NotNull final String applicationId,
            @NotNull final String name,
            @NotNull final String iconName,
            final boolean isRtl,
            @NotNull final NotifierCompat.Listener listener
    ) throws IOException;

    @NotNull
    public static Notifier add(
            @NotNull final String applicationId,
            @NotNull final String title,
            @NotNull final String iconName,
            final boolean isRtl,
            @NotNull final NotifierCompat.Listener listener
    ) throws IOException {
        return new Notifier(nativeAdd(applicationId, title, iconName, isRtl, listener));
    }

    private static native void nativeSetMenu(final long ptr, @NotNull final NativeMenuItem @Nullable [] items) throws IOException;

    private static native long nativeRemove(final long ptr);

    @SuppressWarnings("unused")
    public interface Listener {
        void onActive();

        void onMenuActive(short id);
    }

    public interface MenuItem {
        record Item(@NotNull String title, short id) implements MenuItem {
        }

        record SubMenu(@NotNull String title, @NotNull List<@NotNull MenuItem> items) implements MenuItem {
        }
    }

    private record NativeMenuItem(@NotNull String title, short id, @NotNull NativeMenuItem @Nullable [] subItems) {
    }

    public static class Notifier {
        private final long ptr;
        private final Cleaner.Cleanable cleanable;

        Notifier(final long ptr) {
            this.ptr = ptr;
            this.cleanable = notifierCleaner.register(this, () -> nativeRemove(ptr));
        }

        private static NativeMenuItem[] toNativeMenuItems(@NotNull final List<@NotNull MenuItem> items) {
            return items.stream().map(item -> {
                if (item instanceof final MenuItem.Item menuItem) {
                    if (menuItem.id < 0) {
                        throw new IllegalArgumentException("Invalid item id " + menuItem.id);
                    }

                    return new NativeMenuItem(menuItem.title, menuItem.id, null);
                } else if (item instanceof final MenuItem.SubMenu subMenu) {
                    return new NativeMenuItem(subMenu.title, (short) -1, toNativeMenuItems(subMenu.items));
                } else {
                    throw new IllegalArgumentException("Unsupported menu item " + item);
                }
            }).toArray(NativeMenuItem[]::new);
        }

        public void setMenuItems(@Nullable final List<@NotNull MenuItem> items) throws IOException {
            if (items != null) {
                nativeSetMenu(ptr, toNativeMenuItems(items));
            } else {
                nativeSetMenu(ptr, null);
            }
        }

        public void remove() {
            cleanable.clean();
        }
    }
}
