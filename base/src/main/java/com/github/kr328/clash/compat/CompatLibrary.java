package com.github.kr328.clash.compat;

import org.jetbrains.annotations.Nullable;

import java.io.IOException;
import java.io.InputStream;
import java.net.MalformedURLException;
import java.net.URL;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.attribute.FileTime;
import java.util.Comparator;
import java.util.Locale;
import java.util.Objects;
import java.util.stream.Stream;

public final class CompatLibrary {
    @Nullable
    private static Path overrideExtractPath = null;

    public static void setOverrideExtractPath(@Nullable final Path overrideExtractPath) {
        CompatLibrary.overrideExtractPath = overrideExtractPath;
    }

    public static void load() {
        Objects.requireNonNull(Loader.instance);
    }

    private static final class Loader {
        static Object instance = new Object();

        static {
            final String fileName = switch (Os.current) {
                case Windows -> switch (Arch.current) {
                    case Amd64 -> "compat-amd64.dll";
                };
                case Linux -> switch (Arch.current) {
                    case Amd64 -> "libcompat-amd64.so";
                };
            };

            try {
                final URL libraryURL;
                if (System.getenv("COMPAT_LIBRARY_PATH") != null) {
                    try {
                        libraryURL = Path.of(System.getenv("COMPAT_LIBRARY_PATH")).toUri().toURL();
                    } catch (final MalformedURLException e) {
                        throw new RuntimeException(e);
                    }
                } else {
                    libraryURL = Objects.requireNonNull(Loader.class.getResource("/com/github/kr328/clash/compat/" + fileName));
                }

                if ("file".equals(libraryURL.getProtocol())) {
                    System.load(Path.of(libraryURL.toURI()).toAbsolutePath().toString());
                } else {
                    final Path extractPath;
                    if (overrideExtractPath != null) {
                        extractPath = overrideExtractPath;

                        Files.createDirectories(extractPath);
                    } else {
                        extractPath = Files.createTempDirectory("clash-multiplatform-compat-");

                        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                            try (final Stream<Path> files = Files.walk(extractPath)) {
                                for (final Path file : files.sorted(Comparator.reverseOrder()).toList()) {
                                    Files.delete(file);
                                }
                            } catch (final IOException e) {
                                // ignored
                            }
                        }));
                    }

                    final Path libraryPath = extractPath.resolve(fileName);

                    boolean skipExtract = false;
                    if (libraryURL.getProtocol().equals("jar")) {
                        final String jarPath = libraryURL.getFile().split("\\|", 2)[0];

                        try {
                            final FileTime libraryModified = Files.getLastModifiedTime(libraryPath);
                            final FileTime jarModified = Files.getLastModifiedTime(Path.of(jarPath));

                            skipExtract = jarModified.compareTo(libraryModified) <= 0;
                        } catch (final Exception ignored) {
                            // ignored
                        }
                    }

                    if (!skipExtract) {
                        try (final InputStream stream = libraryURL.openStream()) {
                            Files.copy(stream, libraryPath, StandardCopyOption.REPLACE_EXISTING);
                        }
                    }

                    System.load(libraryPath.toAbsolutePath().toString());
                }
            } catch (final Exception e) {
                throw new LinkageError("Load " + fileName, e);
            }
        }

        enum Os {
            Windows, Linux;

            public static final Os current;

            static {
                final String osName = System.getProperty("os.name").toLowerCase(Locale.ROOT);

                if (osName.contains("windows")) {
                    current = Os.Windows;
                } else if (osName.contains("linux")) {
                    current = Os.Linux;
                } else {
                    throw new IllegalArgumentException("Unsupported os " + osName);
                }
            }
        }

        enum Arch {
            Amd64;

            public static final Arch current;

            static {
                final String archName = System.getProperty("os.arch").toLowerCase(Locale.ROOT);

                if (archName.contains("amd64") || archName.contains("x86_64")) {
                    current = Amd64;
                } else {
                    throw new IllegalArgumentException("Unsupported arch " + archName);
                }
            }
        }
    }
}
