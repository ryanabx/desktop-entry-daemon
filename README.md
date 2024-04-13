![](res/desktopentry.png)
A dbus api and daemon to manage temporary desktop entries!

The desktop entries are cleaned after rebooting.

## DBus API

Introspection XML is in the root of this repo: `net.ryanabx.DesktopEntry`

### Basic rundown

`register_entry` takes a list of strings that identify paths to .desktop entries to register.

`register_icons` takes a two lists of strings - one to identify the full path to the icon, the other identifies what supbath of the icons directory to save to (for example, `hicolor/48x48/apps/`)

### Projects that use this dbus service

* [container-desktop-entries](https://github.com/ryanabx/container-desktop-entries) (Supports Toolbox, Docker, and Podman containers)
* appimage-desktop-entries (Coming soon!)

## Build/Install/Uninstall

**BUILD**

    just build

**INSTALL**

    just install
    systemctl --user enable desktop-entry-daemon
    systemctl --user start desktop-entry-daemon

**UNINSTALL**

    systemctl --user stop desktop-entry-daemon
    systemctl --user disable desktop-entry-daemon
    just uninstall

## Contributing

Make a PR! It'd be helpful to make an issue as well to let people know what you intend to work on!

