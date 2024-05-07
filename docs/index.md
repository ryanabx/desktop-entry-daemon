---
title: Home
layout: home
---

# Desktop Entry Daemon

Desktop Entry Daemon is a userspace DBus API and daemon to manage desktop entries. This could be expanded later to include a system-level component, but the current scope is just for userspace applications.

The current resources a client may register with `desktop-entry-daemon`:

* Desktop Entries with the [Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html)
* Icons with the [Icon Theme Specification](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)


## Lifetimes

There are 3 lifetimes for a resource in `desktop-entry-daemon`:

* **Process** - Resources in this lifetime will be cleared when the calling process exits
* **Session** - Resources in this lifetime will be cleared when the session is restarted OR when the daemon restarts
* **Persistent** - Resources in this lifetime are persistent across reboots and won't be deleted unless explicitly called to do so.

The default directories for these lifetimes are:

* **Process** - `/run/user/$UID/desktop-entry-daemon/process/`
* **Session** - `/run/user/$UID/desktop-entry-daemon/session/`
* **Persistent** - `$HOME/.cache/desktop-entry-daemon/`

## Using the DBus API

The up-to-date XML interface for the API can be found [here](https://github.com/ryanabx/desktop-entry-daemon/blob/master/res/io.ryanabx.DesktopEntry.xml).

The DBus interface can be found at the name `io.ryanabx.DesktopEntry` at the path `/io/ryanabx/DesktopEntry`.

### Example 1 - Entry/Icon for the Process Lifetime

Let's say you are a client and you'd like to register a temporary desktop entry and icon for yourself while you're running. In this case, you'd want to use the `Process` lifetime, so that when your application exits, the entry is removed.

Call the DBus API function `NewProcessEntry` with a string for your application ID `appid` and the plain text of the desktop entry `entry`. The daemon will track the calling process until the process exits, and then delete the entry. You may also call the function `NewProcessIcon` to register an icon with the name `name` and the raw data for the icon `data`.

### Example 2 - Registering an SVG Icon

Lets say you'd like to register a scalable icon for an app or other purpose. Let's say you'd like the icon to be cleared after the session exits, so you'd use the `Session` lifetime. Call `NewSessionIcon` with a `name` of your choice and some SVG text data encoded into [UTF-8](https://en.wikipedia.org/wiki/UTF-8).

> **NOTE:** Session-level and Persistent-level resources have an extra argument `owner` which is a string of your choice that identifies that you own the resource. You may use this string later on if you'd like to force-remove the data you've stored.

## Contributing

This API is open and welcome to community contributions! Please [make an issue](https://github.com/ryanabx/desktop-entry-daemon/issues/new) describing what you'd like to work on, to avoid duplicate work, then make a PR!