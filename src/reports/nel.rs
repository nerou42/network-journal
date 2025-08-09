/**
 * network-journal - collect network reports and print them to file
 * Copyright (C) 2025 nerou GmbH
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[serde(rename = "dns")]
    DNS,
    Connection,
    Application
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NetworkError {
    elapsed_time: u64,
    method: String,
    phase: Phase,
    protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_headers: Option<HashMap<String, Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_headers: Option<HashMap<String, Vec<String>>>,
    sampling_fraction: f32,
    server_ip: String,
    status_code: u16,
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::reports::reporting_api::{Report, ReportingApiReport, ReportType};

    use super::*;

    #[test]
    fn parse_report_ex3() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://www.example.com/",
            "body": {
                "sampling_fraction": 0.5,
                "referrer": "http://example.com/",
                "server_ip": "2001:DB8:0:0:0:0:0:42",
                "protocol": "h2",
                "method": "GET",
                "request_headers": {},
                "response_headers": {},
                "status_code": 200,
                "elapsed_time": 823,
                "phase": "application",
                "type": "http.protocol.error"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 823,
                method: "GET".to_string(),
                phase: Phase::Application,
                protocol: "h2".to_string(),
                referrer: Some("http://example.com/".to_string()),
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::new()),
                sampling_fraction: 0.5,
                server_ip: "2001:DB8:0:0:0:0:0:42".to_string(),
                status_code: 200,
                r#type: "http.protocol.error".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://www.example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex4() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://widget.com/thing.js",
            "body": {
                "sampling_fraction": 1.0,
                "referrer": "https://www.example.com/",
                "server_ip": "",
                "protocol": "",
                "method": "GET",
                "request_headers": {},
                "response_headers": {},
                "status_code": 0,
                "elapsed_time": 143,
                "phase": "dns",
                "type": "dns.name_not_resolved"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 143,
                method: "GET".to_string(),
                phase: Phase::DNS,
                protocol: "".to_string(),
                referrer: Some("https://www.example.com/".to_string()),
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::new()),
                sampling_fraction: 1.0,
                server_ip: "".to_string(),
                status_code: 0,
                r#type: "dns.name_not_resolved".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://widget.com/thing.js".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex6() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://new-subdomain.example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {},
                "response_headers": {},
                "status_code": 0,
                "elapsed_time": 48,
                "phase": "dns",
                "type": "dns.name_not_resolved"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 48,
                method: "GET".to_string(),
                phase: Phase::DNS,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::new()),
                sampling_fraction: 1.0,
                server_ip: "".to_string(),
                status_code: 0,
                r#type: "dns.name_not_resolved".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://new-subdomain.example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex8() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "192.0.2.1",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {},
                "response_headers": {
                    "ETag": ["01234abcd"]
                },
                "status_code": 200,
                "elapsed_time": 1392,
                "phase": "application",
                "type": "ok"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 1392,
                method: "GET".to_string(),
                phase: Phase::Application,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::from([("ETag".to_string(), vec!["01234abcd".to_string()])])),
                sampling_fraction: 1.0,
                server_ip: "192.0.2.1".to_string(),
                status_code: 200,
                r#type: "ok".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex9() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "192.0.2.1",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {
                    "If-None-Match": ["01234abcd"]
                },
                "response_headers": {
                    "ETag": ["01234abcd"]
                },
                "status_code": 304,
                "elapsed_time": 45,
                "phase": "application",
                "type": "ok"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 45,
                method: "GET".to_string(),
                phase: Phase::Application,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::from([("If-None-Match".to_string(), vec!["01234abcd".to_string()])])),
                response_headers: Some(HashMap::from([("ETag".to_string(), vec!["01234abcd".to_string()])])),
                sampling_fraction: 1.0,
                server_ip: "192.0.2.1".to_string(),
                status_code: 304,
                r#type: "ok".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex10() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "192.0.2.1",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {
                    "If-None-Match": ["01234abcd"]
                },
                "response_headers": {
                    "ETag": ["56789ef01"]
                },
                "status_code": 200,
                "elapsed_time": 935,
                "phase": "application",
                "type": "ok"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 935,
                method: "GET".to_string(),
                phase: Phase::Application,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::from([("If-None-Match".to_string(), vec!["01234abcd".to_string()])])),
                response_headers: Some(HashMap::from([("ETag".to_string(), vec!["56789ef01".to_string()])])),
                sampling_fraction: 1.0,
                server_ip: "192.0.2.1".to_string(),
                status_code: 200,
                r#type: "ok".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex12() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "192.0.2.1",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {},
                "response_headers": {},
                "status_code": 200,
                "elapsed_time": 57,
                "phase": "application",
                "type": "ok"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 57,
                method: "GET".to_string(),
                phase: Phase::Application,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::new()),
                sampling_fraction: 1.0,
                server_ip: "192.0.2.1".to_string(),
                status_code: 200,
                r#type: "ok".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex13() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "192.0.2.2",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {},
                "response_headers": {},
                "status_code": 200,
                "elapsed_time": 34,
                "phase": "application",
                "type": "ok"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 34,
                method: "GET".to_string(),
                phase: Phase::Application,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::new()),
                sampling_fraction: 1.0,
                server_ip: "192.0.2.2".to_string(),
                status_code: 200,
                r#type: "ok".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex14() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "192.0.2.3",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {},
                "response_headers": {},
                "status_code": 0,
                "elapsed_time": 0,
                "phase": "dns",
                "type": "dns.address_changed"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 0,
                method: "GET".to_string(),
                phase: Phase::DNS,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::new()),
                sampling_fraction: 1.0,
                server_ip: "192.0.2.3".to_string(),
                status_code: 0,
                r#type: "dns.address_changed".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://example.com/".to_string(),
            user_agent: None,
        }));
    }

    #[test]
    fn parse_report_ex15() {
        // source: https://www.w3.org/TR/network-error-logging/
        let json = r#"{
            "age": 0,
            "type": "network-error",
            "url": "https://example.com/",
            "body": {
                "sampling_fraction": 1.0,
                "server_ip": "192.0.2.1",
                "protocol": "http/1.1",
                "method": "GET",
                "request_headers": {},
                "response_headers": {},
                "status_code": 0,
                "elapsed_time": 0,
                "phase": "dns",
                "type": "dns.address_changed"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::NetworkError(NetworkError {
                elapsed_time: 0,
                method: "GET".to_string(),
                phase: Phase::DNS,
                protocol: "http/1.1".to_string(),
                referrer: None,
                request_headers: Some(HashMap::new()),
                response_headers: Some(HashMap::new()),
                sampling_fraction: 1.0,
                server_ip: "192.0.2.1".to_string(),
                status_code: 0,
                r#type: "dns.address_changed".to_string(),
                url: None
            }),
            age: Some(0),
            url: "https://example.com/".to_string(),
            user_agent: None,
        }));
    }
}
