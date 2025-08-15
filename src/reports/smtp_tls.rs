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

use actix_web::{http::header, web::{Data, Json}, HttpRequest, HttpResponse, Responder};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{reports::{handle_report, ReportType}, WebState};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
struct DateRange {
    start_datetime: String,
    end_datetime: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
enum PolicyType {
    #[serde(rename = "tlsa")]
    TLSA,
    #[serde(rename = "sts")]
    STS,
    NoPolicyFound
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
struct Policy {
    policy_type: PolicyType,
    policy_string: Vec<String>,
    policy_domain: String,
    mx_host: Vec<String>
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
struct Summary {
    total_successful_session_count: u64,
    total_failure_session_count: u64
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
struct FailureDetails {
    result_type: String,
    sending_mta_ip: String,
    receiving_mx_hostname: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiving_mx_helo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    receiving_ip: Option<String>,
    failed_session_count: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_information: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    failure_reason_code: Option<String>
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
struct PoliciesItem {
    policy: Policy,
    summary: Summary,
    failure_details: Vec<FailureDetails>
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct SMTPTLSReport {
    organization_name: String,
    date_range: DateRange,
    contact_info: String,
    report_id: String,
    policies: Vec<PoliciesItem>
}

impl SMTPTLSReport {
    pub fn get_policy_domains(&self) -> Vec<&str> {
        let mut domains: Vec<&str> = vec![];
        for policy in &self.policies {
            domains.push(&policy.policy.policy_domain);
        }
        domains
    }
}

pub async fn report_smtp_tls(state: Data<WebState>, req: HttpRequest, report: Json<SMTPTLSReport>) -> impl Responder {
    let res = handle_report(
        &ReportType::SMTPTLSRPT(&report), 
        req.headers().get(header::USER_AGENT).map(|h| h.to_str().unwrap()),
        &state.filter
    );
    match res {
        Ok(_) => HttpResponse::Ok(),
        Err(err) => {
            error!("{} in {:?}", err, report);
            HttpResponse::BadRequest()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_report() {
        // source: https://www.rfc-editor.org/rfc/rfc8460
        let json = r#"{
            "organization-name": "Company-X",
            "date-range": {
                "start-datetime": "2016-04-01T00:00:00Z",
                "end-datetime": "2016-04-01T23:59:59Z"
            },
            "contact-info": "sts-reporting@company-x.example",
            "report-id": "5065427c-23d3-47ca-b6e0-946ea0e8c4be",
            "policies": [
                {
                    "policy": {
                        "policy-type": "sts",
                        "policy-string": [
                            "version: STSv1",
                            "mode: testing",
                            "mx: *.mail.company-y.example",
                            "max_age: 86400"
                        ],
                        "policy-domain": "company-y.example",
                        "mx-host": ["*.mail.company-y.example"]
                    },
                    "summary": {
                        "total-successful-session-count": 5326,
                        "total-failure-session-count": 303
                    },
                    "failure-details": [
                        {
                            "result-type": "certificate-expired",
                            "sending-mta-ip": "2001:db8:abcd:0012::1",
                            "receiving-mx-hostname": "mx1.mail.company-y.example",
                            "failed-session-count": 100
                        }, 
                        {
                            "result-type": "starttls-not-supported",
                            "sending-mta-ip": "2001:db8:abcd:0013::1",
                            "receiving-mx-hostname": "mx2.mail.company-y.example",
                            "receiving-ip": "203.0.113.56",
                            "failed-session-count": 200,
                            "additional-information": "https://reports.company-x.example/report_info ? id = 5065427 c - 23 d3# StarttlsNotSupported"
                        }, 
                        {
                            "result-type": "validation-failure",
                            "sending-mta-ip": "198.51.100.62",
                            "receiving-ip": "203.0.113.58",
                            "receiving-mx-hostname": "mx-backup.mail.company-y.example",
                            "failed-session-count": 3,
                            "failure-reason-code": "X509_V_ERR_PROXY_PATH_LENGTH_EXCEEDED"
                        }
                    ]
                }
            ]
        }"#;
        let res = serde_json::from_str::<SMTPTLSReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), SMTPTLSReport { 
            organization_name: "Company-X".to_string(), 
            date_range: DateRange { 
                start_datetime: "2016-04-01T00:00:00Z".to_string(),
                end_datetime: "2016-04-01T23:59:59Z".to_string()
            }, 
            contact_info: "sts-reporting@company-x.example".to_string(), 
            report_id: "5065427c-23d3-47ca-b6e0-946ea0e8c4be".to_string(), 
            policies: vec![PoliciesItem { 
                policy: Policy { 
                    policy_type: PolicyType::STS, 
                    policy_string: vec![
                        "version: STSv1".to_string(),
                        "mode: testing".to_string(),
                        "mx: *.mail.company-y.example".to_string(),
                        "max_age: 86400".to_string()
                    ], 
                    policy_domain: "company-y.example".to_string(), 
                    mx_host: vec!["*.mail.company-y.example".to_string()]
                }, 
                summary: Summary { 
                    total_successful_session_count: 5326,
                    total_failure_session_count: 303
                }, 
                failure_details: vec![
                    FailureDetails { 
                        result_type: "certificate-expired".to_string(), 
                        sending_mta_ip: "2001:db8:abcd:0012::1".to_string(), 
                        receiving_mx_hostname: "mx1.mail.company-y.example".to_string(), 
                        receiving_mx_helo: None,
                        receiving_ip: None,
                        failed_session_count: 100,
                        additional_information: None,
                        failure_reason_code: None
                    },
                    FailureDetails { 
                        result_type: "starttls-not-supported".to_string(),
                        sending_mta_ip: "2001:db8:abcd:0013::1".to_string(),
                        receiving_mx_hostname: "mx2.mail.company-y.example".to_string(),
                        receiving_mx_helo: None,
                        receiving_ip: Some("203.0.113.56".to_string()),
                        failed_session_count: 200,
                        additional_information: Some("https://reports.company-x.example/report_info ? id = 5065427 c - 23 d3# StarttlsNotSupported".to_string()),
                        failure_reason_code: None
                    },
                    FailureDetails { 
                        result_type: "validation-failure".to_string(), 
                        sending_mta_ip: "198.51.100.62".to_string(), 
                        receiving_mx_hostname: "mx-backup.mail.company-y.example".to_string(), 
                        receiving_mx_helo: None,
                        receiving_ip: Some("203.0.113.58".to_string()),
                        failed_session_count: 3, 
                        additional_information: None,
                        failure_reason_code: Some("X509_V_ERR_PROXY_PATH_LENGTH_EXCEEDED".to_string())
                    }
                ]
            }]
        })
    }
}
