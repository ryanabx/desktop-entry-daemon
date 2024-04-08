name := 'desktop-entry-daemon'
tool-name := 'desktop-entry-ctl'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

conf-dir := '/etc'

tool-src := 'target' / 'release' / tool-name
tool-dst := base-dir / 'bin' / tool-name

daemon-src := 'target' / 'release' / name
daemon-dst := base-dir / 'libexec' / name

data-src := 'data' / 'profiles.d' / 'desktop-entry-daemon.sh'
data-dst := conf-dir / 'profiles.d' / 'desktop-entry-daemon.sh'

# Installs files
install:
    install -Dm0755 {{tool-src}} {{tool-dst}}
    install -Dm0755 {{daemon-src}} {{daemon-dst}}
    install -Dm0755 {{data-src}} {{data-dst}}

# Uninstalls installed files
uninstall:
    rm {{tool-dst}}
    rm {{daemon-dst}}
    rm {{data-dst}}