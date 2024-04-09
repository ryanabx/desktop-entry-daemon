name := 'desktop-entry-daemon'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

conf-dir := '/etc'
lib-dir := '/lib'

daemon-src := 'target' / 'release' / name
daemon-dst := base-dir / 'libexec' / name

data-src := 'data' / 'desktop-entry-daemon.profile.d.in'
data-dst := conf-dir / 'profile.d' / 'desktop-entry-daemon.sh'

service-src := 'data' / 'desktop-entry-daemon.service.in'
service-dst := lib-dir / 'systemd' / 'user' / 'desktop-entry-daemon.service'

build *args:
    cargo build --release {{args}}

install:
    install -Dm0755 {{daemon-src}} {{daemon-dst}}
    install -Dm0644 {{data-src}} {{data-dst}}
    install -Dm0644 {{service-src}} {{service-dst}}

uninstall:
    rm -f {{daemon-dst}}
    rm -f {{data-dst}}
    rm -f {{service-dst}}