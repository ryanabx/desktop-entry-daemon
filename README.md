![Desktop Entry Daemon Banner](res/desktopentry.png)

**A D-Bus API and daemon to manage desktop entries!**

## Project Showcase

#### Projects that use this D-Bus Service:

* [container-desktop-entries](https://github.com/ryanabx/container-desktop-entries) (Supports Toolbox, Docker, and Podman containers)
* appimage-desktop-entries (maybe coming soon?)
* your client here?

## Building/Installing/Uninstalling

**BUILD**

```bash
just build
```

**INSTALL**

```bash
just install
systemctl --user enable --now desktop-entry-daemon
```

**UNINSTALL**

```bash
systemctl --user stop desktop-entry-daemon
systemctl --user disable desktop-entry-daemon
just uninstall
```

## Contributing

Make a PR! It'd be helpful to make an issue as well to let people know what you intend to work on!

