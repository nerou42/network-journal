%bcond_without check
%global debug_package %{nil}

Name:           network-journal
Version:        0.2.0
Release:        1%{?dist}
Summary:        Webserver and IMAP client to collect standardized browser and mailer reports

License:        GPL-3.0
URL:            https://github.com/nerou42/network-journal
Packager:       nerou GmbH

Source0:        https://github.com/nerou42/%{name}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildArch:      x86_64
BuildRequires:  systemd-rpm-macros
BuildRequires:  gcc, openssl-devel


%description
Collects CSP, NEL, DMARC, SMTP-TLS etc. reports via its own HTTP server and its IMAP client (DMARC only).


%prep
%setup
cargo fetch --locked -q


%build
cargo build -rq
cargo tree --workspace --offline --edges no-build,no-dev,no-proc-macro --no-dedupe --target all --prefix none --format "{l}: {p}" | sed -e "s: ($(pwd)[^)]*)::g" -e "s: / :/:g" -e "s:/: OR :g" | sort -u


%post
%systemd_post %{name}.service


%preun
%systemd_preun %{name}.service


%postun
%systemd_postun_with_restart %{name}.service


%install
install -m 0755 -p -D target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -m 0644 -p -D pkg/%{name}.service %{buildroot}%{_unitdir}/%{name}.service
install -m 0644 -p -D pkg/%{name}.logrotate %{buildroot}%{_sysconfdir}/logrotate.d/%{name}
mkdir -p %{buildroot}%{_sysconfdir}/%{name}
mkdir -p %{buildroot}%{_localstatedir}/log/%{name}


%if %{with check}
%check
cargo test -r
%endif


%files
%license LICENSE.md
%doc README.md CHANGELOG.md CONTRIBUTING.md examples/
%attr(0755, root, root) %{_bindir}/%{name}
%dir %attr(0755, root, root) %{_sysconfdir}/%{name}
#%ghost %config(noreplace) %attr(0600, root, root) %{_sysconfdir}/%{name}/%{name}.yml
%attr(0644, root, root) %{_unitdir}/%{name}.service
%dir %attr(0755, root, root) %{_localstatedir}/log/%{name}
%ghost %attr(0700, root, root) %{_localstatedir}/log/%{name}/%{name}.log
%attr(0644, root, root) %{_sysconfdir}/logrotate.d/%{name}


%changelog
* Wed Aug 06 2025 nerou GmbH <info@nerou.de>
- Add logrotate config

* Mon Aug 04 2025 nerou GmbH <info@nerou.de>
- Initial RPM package
