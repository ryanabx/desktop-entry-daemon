![](res/desktopentry.png)
A dbus api and daemon to manage temporary desktop entries!

The desktop entries are cleaned after rebooting.

## DBus API

Introspection XML can be found in the introspection/out folder: `net.ryanabx.DesktopEntry.xml`

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

