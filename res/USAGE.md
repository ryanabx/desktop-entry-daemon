# Using the D-Bus API

This D-Bus API is for clients who would like to register a [Freedesktop Desktop Entry](https://specifications.freedesktop.org/desktop-entry-spec/latest/) temporarily.

In addition to registering a desktop entry, clients may also register an application icon which will be placed in the `hicolor` theme as specified by the [Freedesktop Icons Specification](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html)

## Usage


### Register a Desktop Entry

The `RegisterEntry` method takes in a string `appid` corresponding to the application ID, and a string `entry` which is the plaintext desktop entry as it would appear in a `.desktop` file.

```xml
<!--
Register a new application entry. The utf-8 encoded `entry` will be validated to be conformant with the
[Desktop Entry Specification](https://specifications.freedesktop.org/desktop-entry-spec/latest/)
Returns an error if the entry failed to register.
requires a valid process identifier to watch, entry goes away after the identified process exits
-->
<method name="RegisterEntry">
    <arg name="appid" type="s" direction="in"/>
    <arg name="entry" type="s" direction="in"/>
</method>
```

### Register an Icon

The `RegisterIcon` method takes in a `name` which will be used when saving the icon, and when identifying the icon according to the [Icon Spec](https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html). The `data` field takes a byte array corresponding to the image data.

It is preferred that the image data be `.png` or `.svg` data, and the daemon will do the job of figuring out the format when it receives the data.

```xml
<!--
Register a new application icon. The icon data should be valid .png or .svg data, and the icon type should be
0 for .png, 1 for .svg. The icon name is the name desktop entries reference when using the icon. The method will
returns true if successful, false otherwise.
-->
<method name="RegisterIcon">
    <arg name="name" type="s" direction="in">
    <arg name="data" type="ay" direction="in"/>
</method>
```

### Watch for desktop entry and icon changes

If you are a client that would like to do something when a desktop entry or icon is added or destroyed, these signals are emitted when those are changed.

```xml
<!--
signal for when an entry is added or destroyed. subscribe to this if you would like to manually
handle refreshing the xdg desktop database, i.e. by using `update-desktop-database`
this is normally handled automatically by desktop-entry-daemon
-->
<signal name="EntryChanged">
    <arg name="appid" type="s"/>
</signal>
<!--
signal for when an icon is added or destroyed. subscribe to this if you would like to manually
handle refreshing the xdg desktop database, i.e. by using `update-desktop-database`
this is normally handled automatically by desktop-entry-daemon
-->
<signal name="IconChanged">
    <arg name="icon_name" type="s"/>
</signal>
```

### Register a client as a change handler

*This use case is still being developed on.*

By default, desktop-entry-daemon will run desktop handling commands commands like `update-desktop-database` every time an icon or entry is added.

If you are a desktop environment and would like to handle changes to desktop entries and icons on your own, register a process as a change handler.

```xml
<!--
register the sender as a change handler for icons and entries. this inhibits the behavior
of desktop-entry-daemon refreshing the database whenever a new icon or entry is added or
removed. along with this, if you'd like to watch changes, subscribe to `icon_changed` and
`entry_changed`
-->
<method name="RegisterChangeHandler"></method>
```