# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased](https://github.com/nerou42/network-journal/compare/v0.2.0...HEAD)

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


## [0.2.0](https://github.com/nerou42/network-journal/compare/v0.1.0...v0.2.0)

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


## [0.1.0](https://github.com/nerou42/network-journal/releases/tag/v0.1.0)

### Added

- Support for the following types of reports:
    - Crash
    - CSP (all levels, excluding CSP hash report)
    - Deprecation
    - NEL
    - SMTP TLS
