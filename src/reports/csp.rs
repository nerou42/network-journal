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

use actix_web::{web::{Data, Payload}, HttpMessage, HttpRequest, HttpResponse, Responder};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{get_body_as_string, reports::reporting_api::{handle_reporting_api_report, ReportingApiReport}, WebState};

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
struct CSPReport {
    csp_report: CSPViolation,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CSPReportDisposition {
    Enforce,
    Report
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct CSPViolation {
    #[serde(alias = "document-uri", alias = "documentURL")]
    document_url: String,
    referrer: Option<String>,
    #[serde(alias = "blocked-uri", alias = "blockedURL")]
    blocked_url: Option<String>,
    /// new in CSP2
    #[serde(alias = "effective-directive", alias = "effectiveDirective")]
    effective_directive: String,
    /// removed in CSP3, required in CSP2
    #[serde(rename(deserialize = "violated-directive"), skip_serializing_if = "Option::is_none")]
    violated_directive: Option<String>,
    #[serde(alias = "original-policy", alias = "originalPolicy")]
    original_policy: String,
    /// new in CSP3
    #[serde(skip_serializing_if = "Option::is_none")]
    sample: Option<String>,
    /// new in CSP3
    #[serde(skip_serializing_if = "Option::is_none")]
    disposition: Option<CSPReportDisposition>,
    /// new in CSP2
    #[serde(alias = "status-code", alias = "statusCode", skip_serializing_if = "Option::is_none")]
    status_code: Option<u16>,
    /// new in CSP2
    #[serde(alias = "source-file", alias = "sourceFile", skip_serializing_if = "Option::is_none")]
    source_file: Option<String>,
    /// new in CSP2
    #[serde(alias = "line-number", alias = "lineNumber", skip_serializing_if = "Option::is_none")]
    line_number: Option<u64>,
    /// new in CSP2
    #[serde(alias = "column-number", alias = "columnNumber", skip_serializing_if = "Option::is_none")]
    column_number: Option<u64>
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct CSPHash {
    document_url: String,
    subresource_url: String,
    hash: String,
    r#type: String,
    destination: String
}

pub async fn report_csp(state: Data<WebState>, req: HttpRequest, body: Payload) -> impl Responder {
    match req.content_type() {
        "application/reports+json" => {
            match get_body_as_string(body).await {
                Ok(str) => {
                    let report_parse_res = serde_json::from_str::<ReportingApiReport>(&str);
                    let handle_res = match report_parse_res {
                        Ok(reports) => {
                            handle_reporting_api_report(&reports, &state.filter).await
                        },
                        Err(err) => {
                            error!("failed to parse report: {} in {}", err, str);
                            return HttpResponse::BadRequest();
                        }
                    };
                    match handle_res {
                        Ok(_) => HttpResponse::Ok(),
                        Err(err) => {
                            error!("failed to handle report(s): {} in {:?}", err, str);
                            HttpResponse::BadRequest()
                        }
                    }
                },
                Err(err) => {
                    error!("{}", err);
                    return HttpResponse::BadRequest();
                }
            }
        },
        "application/csp-report" => {
            let parse_res = match get_body_as_string(body).await {
                Ok(str) => serde_json::from_str::<CSPReport>(&str),
                Err(err) => {
                    error!("{}", err);
                    return HttpResponse::BadRequest();
                }
            };
            match parse_res {
                Ok(report) => {
                    info!("CSP {}", serde_json::to_string_pretty(&report.csp_report).unwrap());
                    HttpResponse::Ok()
                },
                Err(err) => {
                    error!("failed to parse report: {}", err);
                    HttpResponse::BadRequest()
                }
            }
        },
        ct => {
            error!("unexpected content type: {} (UA: {:?})", ct, req.headers().get("User-Agent"));
            HttpResponse::BadRequest()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reports::reporting_api::{Report, ReportType};

    use super::*;

    #[test]
    fn parse_report_lvl2() {
        // source: https://www.w3.org/TR/CSP2/
        let json = r#"{
            "csp-report": {
                "document-uri": "http://example.org/page.html",
                "referrer": "http://evil.example.com/haxor.html",
                "blocked-uri": "http://evil.example.com/image.png",
                "violated-directive": "default-src 'self'",
                "effective-directive": "img-src",
                "original-policy": "default-src 'self'; report-uri http://example.org/csp-report.cgi"
            }
        }"#;
        let res = serde_json::from_str::<CSPReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), CSPReport {
            csp_report: CSPViolation { 
                document_url: "http://example.org/page.html".to_string(),
                referrer: Some("http://evil.example.com/haxor.html".to_string()),
                blocked_url: Some("http://evil.example.com/image.png".to_string()),
                effective_directive: "img-src".to_string(),
                violated_directive: Some("default-src 'self'".to_string()),
                original_policy: "default-src 'self'; report-uri http://example.org/csp-report.cgi".to_string(),
                sample: None,
                disposition: None,
                status_code: None,
                source_file: None,
                line_number: None,
                column_number: None
            }
        });
    }

    #[test]
    fn parse_report_lvl3_violation() {
        // source: https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/CSP
        let json = r#"{
            "age": 53531,
            "body": {
                "blockedURL": "inline",
                "columnNumber": 39,
                "disposition": "enforce",
                "documentURL": "https://example.com/csp-report",
                "effectiveDirective": "script-src-elem",
                "lineNumber": 121,
                "originalPolicy": "default-src 'self'; report-to csp-endpoint-name",
                "referrer": "https://www.google.com/",
                "sample": "console.log(\"lo\")",
                "sourceFile": "https://example.com/csp-report",
                "statusCode": 200
            },
            "type": "csp-violation",
            "url": "https://example.com/csp-report",
            "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36"
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::CSPViolation(CSPViolation {
                document_url: "https://example.com/csp-report".to_string(),
                referrer: Some("https://www.google.com/".to_string()),
                blocked_url: Some("inline".to_string()),
                effective_directive: "script-src-elem".to_string(),
                violated_directive: None,
                original_policy: "default-src 'self'; report-to csp-endpoint-name".to_string(),
                sample: Some("console.log(\"lo\")".to_string()),
                disposition: Some(CSPReportDisposition::Enforce),
                status_code: Some(200),
                source_file: Some("https://example.com/csp-report".to_string()),
                line_number: Some(121),
                column_number: Some(39)
            }),
            age: Some(53531),
            url: "https://example.com/csp-report".to_string(),
            user_agent: Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36".to_string())
        }));
    }

    #[test]
    fn parse_report_lvl3_hash() {
        // source: https://www.w3.org/TR/CSP3/
        let json = r#"{
            "type": "csp-hash",
            "age": 12,
            "url": "https://example.com/",
            "user_agent": "Mozilla/5.0 (X11; Linux i686; rv:132.0) Gecko/20100101 Firefox/132.0",
            "body": {
                "document_url": "https://example.com/",
                "subresource_url": "https://example.com/main.js",
                "hash": "sha256-85738f8f9a7f1b04b5329c590ebcb9e425925c6d0984089c43a022de4f19c281",
                "type": "subresource",
                "destination": "script"
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::CSPHash(CSPHash {
                document_url: "https://example.com/".to_string(),
                subresource_url: "https://example.com/main.js".to_string(),
                hash: "sha256-85738f8f9a7f1b04b5329c590ebcb9e425925c6d0984089c43a022de4f19c281".to_string(),
                r#type: "subresource".to_string(),
                destination: "script".to_string()
            }),
            age: Some(12),
            url: "https://example.com/".to_string(),
            user_agent: Some("Mozilla/5.0 (X11; Linux i686; rv:132.0) Gecko/20100101 Firefox/132.0".to_string())
        }));
    }
}
