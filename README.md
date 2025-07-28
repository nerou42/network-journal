# network-journal

This project is about handling all the reports browsers (Content Security Policy, Network Error Logging etc.) and e-mail servers (DMARC, SMTP TLS etc.) can send nowadays.

To do that, this project contains a webserver, that will listen to incoming reports, validate them, filter them, structure them and log them to a file. 
This log file can be read by your log monitoring tools like an ELK-stack or Grafana Loki. 
With that, you can generate diagrams, configure alerts, you name it.

## Current state

### Supported reports

- [x] [Crash Reports](https://wicg.github.io/crash-reporting/) (in a context of websites)
- [x] Content Security Policy (Level 1, [2](https://www.w3.org/TR/CSP2/) and [3](https://www.w3.org/TR/CSP3/)) reports ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP))
- [x] [Deprecations](https://wicg.github.io/deprecation-reporting/) (in a context of websites)
- [x] [Network Error Logging](https://www.w3.org/TR/network-error-logging/) ([MDN](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Network_Error_Logging))
- [x] [SMTP TLS Reports](https://www.rfc-editor.org/rfc/rfc8460)
- [ ] [DMARC aggregate Reports](https://www.rfc-editor.org/rfc/rfc7489.html)
- [ ] [Permissions Policy](https://w3c.github.io/webappsec-permissions-policy/)
- [ ] [Integrity Policy](https://w3c.github.io/webappsec-subresource-integrity/)
- [ ] [Intervention Reports](https://wicg.github.io/intervention-reporting/)

### Supported report handling

- [x] Webserver listening to incoming reports
- [x] Report validation
- [ ] Filtering (e.g. for your own domains to prevent spam)
- [x] Log reports to file

### Supported installation methods

- [x] Build from source
- [x] Provide systemd service file
- [ ] RPM package
- [ ] DEB package

## Install

Run the executable once (with the `--config` parameter set to a path of your liking) to generate the default configuration file.

**Note**: Some reporters require TLS to be enabled. If you are using some reverse proxy on the other hand, you do not need to enable TLS in this context but on your proxy.

## Configure your reports

In the following, `example.com` needs to be replaced with your frontend-, e-mail- or network-journal domain respectively.

**Note**: All `Reporting-Endpoints` headers discussed below should be combined into one like so `Reporting-Endpoints: crash-reporting="...", "csp-endpoint="..."`.

### Crash

Add the following HTTP header to your HTTP responses:

`Reporting-Endpoints: crash-reporting="https://example.com/crash"`

### CSP (Content Security Policy)

Add the following HTTP headers to your HTTP responses:

1. `Reporting-Endpoints: csp-endpoint="https://example.com/csp"`
1. `Content-Security-Policy: [...]; report-to csp-endpoint`
    Since `report-to` is not yet supported by all browsers, you probably should do the following instead:
    `Content-Security-Policy: [...]; report-to csp-endpoint; report-uri https://example.com/csp`

### Deprecation

Add the following HTTP header to your HTTP responses:

`Reporting-Endpoints: default="https://example.com/deprecation"`

### NEL (Network Error Logging)

Add the following HTTP headers to your HTTP responses:

1. `Report-To: { "group": "nel", "max_age": 31556952, "endpoints": [{ "url": "https://example.com/csp-reports" }]}` (deprecated) or
    `Reporting-Endpoints: nel="https://example.com/nel"` (not yet supported by all browsers)
1. `NEL: { "report_to": "nel", "max_age": 31536000, "include_subdomains": true }`

### SMTP TLS

Add the following DNS entry for your domain:

`_smtp._tls.example.com IN TXT "v=TLSRPTv1; rua=https://example.com/tlsrpt"`

## License

This project is licensed under the [GPLv3.0 license](LICENSE.md).
