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

use actix_web::{web::Json, HttpMessage, HttpRequest, HttpResponse, Responder};
use log::{error, info};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct DateRange {
    #[serde(rename = "start-datetime")]
    start_datetime: String,
    #[serde(rename = "end-datetime")]
    end_datetime: String,
}

#[derive(Serialize, Deserialize, Debug)]
enum PolicyType {
    #[serde(rename(deserialize = "tlsa"))]
    TLSA,
    #[serde(rename(deserialize = "sts"))]
    STS,
    #[serde(rename(deserialize = "no-policy-found"))]
    NONE
}

#[derive(Serialize, Deserialize, Debug)]
struct Policy {
    #[serde(rename = "policy-type")]
    policy_type: PolicyType,
    #[serde(rename = "policy-string")]
    policy_string: Vec<String>,
    #[serde(rename = "policy-domain")]
    policy_domain: String,
    #[serde(rename = "mx-host")]
    mx_host: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct Summary {
    #[serde(rename(deserialize = "total-successful-session-count"))]
    total_successful_session_count: u64,
    #[serde(rename(deserialize = "total-failure-session-count"))]
    total_failure_session_count: u64
}

#[derive(Serialize, Deserialize, Debug)]
struct FailureDetails {
    #[serde(rename(deserialize = "result-type"))]
    result_type: String,
    #[serde(rename(deserialize = "sending-mta-ip"))]
    sending_mta_ip: String,
    #[serde(rename(deserialize = "receiving-mx-hostname"))]
    receiving_mx_hostname: String,
    #[serde(rename(deserialize = "receiving-mx-helo"))]
    receiving_mx_helo: Option<String>,
    #[serde(rename(deserialize = "receiving-ip"))]
    receiving_ip: String,
    #[serde(rename(deserialize = "failed-session-count"))]
    failed_session_count: u64,
    #[serde(rename(deserialize = "additional-information"))]
    additional_information: Option<String>,
    #[serde(rename(deserialize = "failure-reason-code"))]
    failure_reason_code: String
}

#[derive(Serialize, Deserialize, Debug)]
struct PoliciesItem {
    policy: Policy,
    summary: Summary,
    #[serde(rename = "failure-details", default)]
    failure_details: Vec<FailureDetails>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SMTPTLSReport {
    #[serde(rename(deserialize = "organization-name"))]
    organization_name: String,
    #[serde(rename(deserialize = "date-range"))]
    date_range: DateRange,
    #[serde(rename(deserialize = "contact-info"))]
    contact_info: String,
    #[serde(rename(deserialize = "report-id"))]
    report_id: String,
    policies: Vec<PoliciesItem>
}

pub async fn report_smtp_tls(req: HttpRequest, report: Json<SMTPTLSReport>) -> impl Responder {
    if req.content_type() == "application/tlsrpt+json" || req.content_type() == "application/tlsrpt+gzip" {
        info!("SMTP-TLS-RPT {}", serde_json::to_string_pretty(&report).unwrap());
        HttpResponse::Ok()
    } else {
        error!("invalid content type: {}", req.content_type());
        HttpResponse::BadRequest()
    }
}
