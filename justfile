name := 'desktop-entry-daemon'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

conf-dir := '/etc'
lib-dir := '/lib'

daemon-src := 'target' / 'release' / name
daemon-dst := base-dir / 'bin' / name

data-src := 'profile' / 'desktop-entry-daemon.sh'
data-dst := conf-dir / 'profile.d' / 'desktop-entry-daemon.sh'

service-src := 'systemd' / 'desktop-entry-daemon.service'
service-dst := lib-dir / 'systemd' / 'user' / 'desktop-entry-daemon.service'

clean-service-src := 'systemd' / 'desktop-entry-daemon-clean.service'
clean-service-dst := lib-dir / 'systemd' / 'user' / 'desktop-entry-daemon-clean.service'

build *args:
    cargo build --release {{args}}

install:
    install -Dm0755 {{daemon-src}} {{daemon-dst}}
    install -Dm0644 {{data-src}} {{data-dst}}
    install -Dm0644 {{service-src}} {{service-dst}}
    install -Dm0644 {{clean-service-src}} {{clean-service-dst}}

uninstall:
    rm -f {{daemon-dst}}
    rm -f {{data-dst}}
    rm -f {{service-dst}}
    rm -f {{clean-service-dst}}