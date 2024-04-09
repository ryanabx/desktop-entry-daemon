# Generated by rust2rpm 26
%bcond_without check

# prevent library files from being installed
%global cargo_install_lib 0

%global crate desktop-entry-daemon

%global ver ###
%global commit ###
%global date ###

Name:           desktop-entry-daemon
Version:        %{ver}~%{date}
Release:        %autorelease
Summary:        A daemon for managing transient desktop entries

SourceLicense:  Apache-2.0
# FIXME: paste output of %%cargo_license_summary here
License:        # FIXME
# LICENSE.dependencies contains a full license breakdown

URL:            https://github.com/ryanabx/desktop-entry-daemon
Source:         desktop-entry-daemon-0.1.0.tar.xz
Source:         desktop-entry-daemon-0.1.0-vendor.tar.xz

BuildRequires:  cargo-rpm-macros >= 26
BuildRequires:  rustc
BuildRequires:  cargo

BuildRequires:  systemd-rpm-macros

Requires:       dbus

%global _description %{expand:
%{summary}.}

%description %{_description}

%prep
%autosetup -n %{crate}-%{ver} -p1 -a1
%cargo_prep -N
cat .vendor/config.toml >> .cargo/config

%build
%cargo_build
%{cargo_license_summary}
%{cargo_license} > LICENSE.dependencies
%{cargo_vendor_manifest}

%install
just rootdir=%{buildroot} prefix=%{_prefix} install
# %%cargo_install

%if %{with check}
%check
%cargo_test
%endif

%post
%systemd_post %{name}.service

%preun
%systemd_preun %{name}.service

%postun
%systemd_postun_with_restart %{name}.service

%files
%license LICENSE
%license LICENSE.dependencies
# %%license cargo-vendor.txt
%doc README.md
%{_libexecdir}/%{name}
%{_unitdir}/%{name}.service
%{_sysconfdir}/profile.d/%{name}.sh

%changelog
%autochangelog