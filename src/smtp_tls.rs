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

use actix_web::{web::Json, HttpResponse, Responder};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct DateRange {
    start_datetime: String,
    end_datetime: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
enum PolicyType {
    #[serde(rename = "tlsa")]
    TLSA,
    #[serde(rename = "sts")]
    STS,
    NoPolicyFound
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Policy {
    policy_type: PolicyType,
    policy_string: Vec<String>,
    policy_domain: String,
    mx_host: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct Summary {
    total_successful_session_count: u64,
    total_failure_session_count: u64
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct FailureDetails {
    result_type: String,
    sending_mta_ip: String,
    receiving_mx_hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiving_mx_helo: Option<String>,
    receiving_ip: String,
    failed_session_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_information: Option<String>,
    failure_reason_code: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct PoliciesItem {
    policy: Policy,
    summary: Summary,
    failure_details: Vec<FailureDetails>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SMTPTLSReport {
    organization_name: String,
    date_range: DateRange,
    contact_info: String,
    report_id: String,
    policies: Vec<PoliciesItem>
}

pub async fn report_smtp_tls(report: Json<SMTPTLSReport>) -> impl Responder {
    info!("SMTP-TLS-RPT {}", serde_json::to_string_pretty(&report).unwrap());
    HttpResponse::Ok()
}
