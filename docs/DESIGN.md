# Desktop Entry Daemon Design

On Startup:

* Create a temporary directory for desktop entries with desktop entry daemon
* Set the XDG_DATA_DIRS environment variable for desktop entry daemon

Daemon Startup:

* Erase old files (if they exist)

Daemon interface:

* Take in a share path (contains icons/applications/etc)
* Copy path to desktop-entry-daemon path

Daemon exit:

* Erase old files (if they exist)