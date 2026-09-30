# cosmic-applet-caffeine

[English](README.md) | [简体中文](README.zh-CN.md)

A simple "caffeine" applet for the [COSMIC](https://system76.com/cosmic) desktop.
It keeps your screen awake by holding an idle inhibitor through the freedesktop
`org.freedesktop.ScreenSaver` D-Bus interface.

## Features

- **Toggle with a click** — left-click the panel icon to turn caffeine on or off.
- **Timed activation** — right-click to open a popup menu and pick 15 minutes,
  30 minutes, 60 minutes, or Infinite.
- **Automatic release** — with a timed option, the inhibitor is released
  automatically when the timer expires.
- **Stateful icon** — the current state is shown by the icon: a full cup
  means active, an empty cup means inactive.
- **Error feedback** — shows a desktop notification and logs to stderr when a
  D-Bus operation fails.
- **Localized** — English and Simplified Chinese included.

## How it works

It connects to the session bus and calls `Inhibit` / `UnInhibit` on
`org.freedesktop.ScreenSaver`, asking the screen saver / idle manager to stop
blanking or locking the screen.

## Requirements

- Rust **1.85+** (the crate uses the 2024 edition) and Cargo.
- A COSMIC session — the applet is built on
  [libcosmic](https://github.com/pop-os/libcosmic) with the `applet` feature.
- A session D-Bus that provides `org.freedesktop.ScreenSaver`.

> The `libcosmic` dependency is pulled from git, so the first build needs network
> access.

## Build & run

```sh
cargo run              # development build and run
cargo build --release  # optimized build
```

## Install

The [`Justfile`](Justfile) wraps the common tasks:

```sh
just build            # cargo build --release
just install          # install to /usr (requires sudo)
just install-local    # install to ~/.local (no sudo; icons still go to /usr)
just uninstall
just uninstall-local
```

## Package as RPM

```sh
cargo install cargo-generate-rpm
just package          # = cargo build --release && cargo generate-rpm
```

The `.rpm` is written to `target/generate-rpm/`. Packaging metadata lives under
`[package.metadata.generate-rpm]` in [`Cargo.toml`](Cargo.toml).

## Localization

Strings are managed with [Fluent](https://projectfluent.org/) via `i18n-embed`:

```
i18n/
├── en/cosmic_applet_caffeine.ftl
└── zh-CN/cosmic_applet_caffeine.ftl
```

Add a new locale by creating `i18n/<lang>/cosmic_applet_caffeine.ftl`. The
language is selected from the desktop environment's locale at startup.

## Project layout

```
src/
├── main.rs      # entry point
├── lib.rs       # applet UI and state machine
├── dbus.rs      # org.freedesktop.ScreenSaver proxy
└── localize.rs  # Fluent loader
assets/          # .desktop entry and icons
i18n/            # translations
Justfile         # build / install / package helpers
```

## License

MIT. See [LICENSE](LICENSE).
