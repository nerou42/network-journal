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

use actix_web::{web::Payload, HttpMessage, HttpRequest, HttpResponse, Responder};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{get_body_as_string, Report, ReportType};

#[derive(Deserialize)]
struct CSPReport {
    #[serde(alias = "csp-report")]
    csp_report: CSPViolation,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum CSPReportDisposition {
    Enforce,
    Report
}

#[derive(Serialize, Deserialize, Debug)]
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
    #[serde(rename(deserialize = "violated-directive"))]
    violated_directive: Option<String>,
    #[serde(alias = "original-policy", alias = "originalPolicy")]
    original_policy: String,
    /// new in CSP3
    sample: Option<String>,
    /// new in CSP3
    disposition: Option<CSPReportDisposition>,
    /// new in CSP2
    #[serde(alias = "status-code", alias = "statusCode")]
    status_code: Option<u16>,
    /// new in CSP2
    #[serde(alias = "source-file", alias = "sourceFile")]
    source_file: Option<String>,
    /// new in CSP2
    #[serde(alias = "line-number", alias = "lineNumber")]
    line_number: Option<u64>,
    /// new in CSP2
    #[serde(alias = "column-number", alias = "columnNumber")]
    column_number: Option<u64>
}

pub async fn report_csp(req: HttpRequest, body: Payload) -> impl Responder {
    match req.content_type() {
        "application/reports+json" => {
            let parse_res = match get_body_as_string(body).await {
                Ok(str) => serde_json::from_str::<Report<CSPViolation>>(&str),
                Err(err) => {
                    error!("{}", err);
                    return HttpResponse::BadRequest();
                }
            };
            match parse_res {
                Ok(report) => {
                    if report.r#type == ReportType::CSPViolation {
                        info!("CSP {}", serde_json::to_string_pretty(&report.body).unwrap());
                        HttpResponse::Ok()
                    } else {
                        error!("invalid report type: {:?}", report.r#type);
                        HttpResponse::BadRequest()
                    }
                },
                Err(err) => {
                    error!("failed to parse report: {}", err);
                    HttpResponse::BadRequest()
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
