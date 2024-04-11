# set XDG_DATA_DIRS to include desktop-entry-daemon installations
export XDG_DATA_DIRS
XDG_DATA_DIRS="$XDG_DATA_DIRS:$HOME/.cache/desktop-entry-daemon/share/"