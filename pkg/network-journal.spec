%bcond_without    check
%global           debug_package %{nil}

Name:             network-journal
Version:          0.6.2
Release:          2%{?dist}
Summary:          Webserver and IMAP client to collect standardized browser and mailer reports

License:          GPL-3.0-or-later
URL:              https://github.com/nerou42/network-journal

Source0:          https://github.com/nerou42/%{name}/archive/refs/tags/v%{version}/%{name}-%{version}.tar.gz

ExclusiveArch:    x86_64 aarch64
BuildRequires:    systemd-rpm-macros
BuildRequires:    gcc, openssl-devel, cargo
Requires(pre):    /usr/sbin/useradd
Requires(postun): /usr/sbin/userdel


%description
Collects CSP, NEL, DMARC, SMTP-TLS etc. reports via its own HTTP server and
its IMAP client (DMARC only). All collected data is logged to a file that can
be processed with, e.g. an ELK-stack or Grafana Loki to visualize it and issue
alerts.


%prep
%setup -n %{name}-%{version}
cargo fetch --locked -q


%build
cargo build -rq
cargo tree --workspace --offline --edges no-build,no-dev,no-proc-macro --no-dedupe --target all --prefix none --format "{l}: {p}" | sed -e "s: ($(pwd)[^)]*)::g" -e "s: / :/:g" -e "s:/: OR :g" | sort -u


%pre
if ! id "network-journal" >/dev/null 2>&1
then
    useradd -s /sbin/nologin -r -c "Network Journal Editor" network-journal
fi


%post
%systemd_post %{name}.service


%preun
%systemd_preun %{name}.service


%postun
%systemd_postun_with_restart %{name}.service
if [ $1 -eq 0 ]
then
    userdel network-journal
fi


%install
install -m 0755 -p -D target/release/%{name} %{buildroot}%{_bindir}/%{name}
mkdir -p %{buildroot}%{_sysconfdir}/%{name}
install -m 0600 -p -D %{name}.reference.yml %{buildroot}%{_sysconfdir}/%{name}/%{name}.yml
mkdir -p %{buildroot}%{_datadir}/%{name}
install -m 0644 -p -D regexes.yaml %{buildroot}%{_datadir}/%{name}/regexes.yaml
mkdir -p %{buildroot}%{_sysconfdir}/logrotate.d
install -m 0644 -p -D pkg/%{name}.logrotate %{buildroot}%{_sysconfdir}/logrotate.d/%{name}
mkdir -p %{buildroot}%{_localstatedir}/log/%{name}
install -m 0644 -p -D pkg/%{name}.service %{buildroot}%{_unitdir}/%{name}.service


%if %{with check}
%check
cargo test -r
%endif


%files
%license LICENSE.md
%doc README.md CHANGELOG.md CONTRIBUTING.md examples/
%attr(0755, root, root) %{_bindir}/%{name}
%dir %attr(0755, network-journal, network-journal) %{_sysconfdir}/%{name}
%config(noreplace) %attr(0600, network-journal, network-journal) %{_sysconfdir}/%{name}/%{name}.yml
%attr(0644, root, root) %{_datadir}/%{name}/regexes.yaml
%attr(0644, root, root) %{_unitdir}/%{name}.service
%dir %attr(0755, network-journal, network-journal) %{_localstatedir}/log/%{name}
%ghost %attr(0644, network-journal, network-journal) %{_localstatedir}/log/%{name}/%{name}.log
%config %attr(0644, root, root) %{_sysconfdir}/logrotate.d/%{name}


%changelog
* Sat Feb 14 2026 nerou GmbH <info@nerou.de>
- Add system user to run network-journal

* Sun Oct 26 2025 nerou GmbH <info@nerou.de>
- Add cargo as build dependency
- Fix license notation's SPDX compatibility

* Wed Sep 24 2025 nerou GmbH <info@nerou.de>
- Add configuration reference file
- Fix compatibility issues

* Sat Aug 09 2025 nerou GmbH <info@nerou.de>
- Add regexes.yaml for User-Agent derivation

* Wed Aug 06 2025 nerou GmbH <info@nerou.de>
- Add logrotate config

* Mon Aug 04 2025 nerou GmbH <info@nerou.de>
- Initial RPM package
