name := 'xdg-temp-daemon'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

conf-dir := '/etc'
lib-dir := '/lib'

daemon-src := 'target' / 'release' / name
daemon-dst := base-dir / 'libexec' / name

data-src := 'data' / 'xdg-temp-daemon.profile.d.in'
data-dst := conf-dir / 'profile.d' / 'xdg-temp-daemon.sh'

service-src := 'data' / 'xdg-temp-daemon.service.in'
service-dst := lib-dir / 'systemd' / 'user' / 'xdg-temp-daemon.service'

clean-service-src := 'data' / 'xdg-temp-daemon-clean.service.in'
clean-service-dst := lib-dir / 'systemd' / 'user' / 'xdg-temp-daemon-clean.service'

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