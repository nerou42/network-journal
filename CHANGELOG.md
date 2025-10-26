# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/nerou42/network-journal/compare/v0.5.2...HEAD)


## [0.5.2](https://github.com/nerou42/network-journal/compare/v0.5.1...v0.5.2) - 2025-10-26

### Changed

- Replace crate confy with config for better backwards compatibility when extending the config file

### Fixed

- Minor issues with SPEC file and Fedora Packaging Guidelines
- Build compatibility with distro's native Cargo (older version, thus Rust edition 2021 compatibility necessary)


## [0.5.1](https://github.com/nerou42/network-journal/compare/v0.5.0...v0.5.1) - 2025-09-24

### Added

- Configuration file reference
- Grafana dashboard example

### Changed

- Update imap dependency to pre-release version since latest stable has outdated and vulnerable dependencies

### Fixed

- Some SMTP TLS Reports could not be parsed:
    - Content-Encoding/Content-Type mismatch: `Content-Encoding != 'gzip' && Content-type == 'application/tlsrpt+gzip'`
    - Some report senders (e.g. Google) skip `failure_details` property if there are no failures
- Log raw SMTP TLS Reports payload on parse error
- Compatibility issues with RPM SPEC file


## [0.5.0](https://github.com/nerou42/network-journal/compare/v0.4.0...v0.5.0) - 2025-08-16

### Added

- Domain filter for incoming reports to block spam
- Derivation of additional metrics:
    - host, path and query of the origin URL
    - browser name and version as well as OS name and version based on user agent

### Changed

- Default log level in service file from info to debug (actix logs validation errors as debug)

### Fixed

- Handling of CSP report with level 2 content type and level 3 payload


## [0.4.0](https://github.com/nerou42/network-journal/compare/v0.3.0...v0.4.0) - 2025-08-12

### Added

- Support for the following types of reports:
    - DMARC

### Changed

- Improve config file change handling by implementing default


## [0.3.0](https://github.com/nerou42/network-journal/compare/v0.2.0...v0.3.0) - 2025-08-09

### Added

- Support for the following types of reports:
    - COEP
    - COOP
- CHANGELOG.md

### Changed

- Major restructuring to parse Reporting API reports more flexible
    - Support for single as well as multiple reports (JSON object vs. array of objects)
    - Single endpoint to handle all Reporting API based reports

### Deprecated

- The following endpoints should be replaced by `/reporting-api` and will be removed in one of the following releases
    - `/crash`
    - `/deprecation`
    - `/integrity`
    - `/intervention`
    - `/nel`
    - `/permissions`

### Fixed

- Binary path in service file
- Alloy config not dropping all the log statements any more


## [0.2.0](https://github.com/nerou42/network-journal/compare/v0.1.0...v0.2.0) - 2025-08-06

### Added

- Support for the following types of reports:
    - integrity-violation
    - intervention 
    - permissions-policy-violation
- CONTRIBUTING.md
- RPM package build resources
- Grafana Alloy example config
- logrotate config (included in RPM package)
- Info about logging format to README.md
- Tests for parsers based on examples from RFCs, W3Cs and MDN

### Changed

- Output format of reports in log files is closer to original report formats now

### Fixed

- NEL report parsing:
    - `sampling_fraction` is actually a float and not unsigned int
    - `url` is actually optional


## [0.1.0](https://github.com/nerou42/network-journal/releases/tag/v0.1.0) - 2025-07-23

### Added

- Support for the following types of reports:
    - Crash
    - CSP (all levels, excluding CSP hash report)
    - Deprecation
    - NEL
    - SMTP TLS
