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

use actix_web::{web::{Data, Json}, HttpResponse, Responder};
use log::error;
use serde::{Deserialize, Serialize};

use crate::{processing::filter::Filter, reports::{
    self, 
    coep::CrossOriginEmbedderPolicyViolation, 
    coop::CrossOriginOpenerPolicyViolation, 
    crash::Crash, 
    csp::{CSPHash, CSPViolation}, 
    deprecation::Deprecation, 
    handle_report, 
    integrity::IntegrityViolation, 
    intervention::Intervention, 
    nel::NetworkError, 
    permissions::PermissionsPolicyViolation
}, WebState};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "kebab-case", tag = "type", content = "body")]
pub enum ReportType {
    #[serde(rename = "coep")]
    COEP(CrossOriginEmbedderPolicyViolation),
    #[serde(rename = "coop")]
    COOP(CrossOriginOpenerPolicyViolation),
    Crash(Crash),
    #[serde(rename = "csp-hash")]
    CSPHash(CSPHash),
    #[serde(rename = "csp-violation")]
    CSPViolation(CSPViolation),
    Deprecation(Deprecation),
    IntegrityViolation(IntegrityViolation),
    Intervention(Intervention),
    NetworkError(NetworkError),
    PermissionsPolicyViolation(PermissionsPolicyViolation),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct Report {
    #[serde(flatten)]
    pub rpt: ReportType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<u32>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(untagged)]
pub enum ReportingApiReport {
    Single(Report),
    Multi(Vec<Report>)
}

pub async fn handle_reporting_api_report(reports: &ReportingApiReport, filter: &Filter) -> Result<(), serde_json::Error> {
    match reports {
        ReportingApiReport::Single(report) => handle_report(&reports::ReportType::ReportingAPI(report), filter).await,
        ReportingApiReport::Multi(reports) => {
            let mut res = Ok(());
            for report in reports {
                let handle_res = handle_report(&reports::ReportType::ReportingAPI(report), filter).await;
                if handle_res.is_err() {
                    res = handle_res;
                    break;
                }
            }
            res
        }
    }
}

pub async fn reporting_api(state: Data<WebState>, reports: Json<ReportingApiReport>) -> impl Responder {
    let rpts = reports.into_inner();
    let res = handle_reporting_api_report(&rpts, &state.filter).await;
    match res {
        Ok(_) => HttpResponse::Ok(),
        Err(err) => {
            error!("failed to handle report(s): {} in {:?}", err, rpts);
            HttpResponse::BadRequest()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::reports::crash::CrashReason;

    use super::*;

    #[test]
    fn parse_single() {
        // source: https://wicg.github.io/crash-reporting/
        let json = r#"{
  "type": "crash",
  "body": {
    "reason": "oom"
  },
  "age": 42,
  "url": "https://example.com/",
  "user_agent": "Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0"
}"#;
        let deser_res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(deser_res.is_ok());
        if let Ok(report) = deser_res {
            assert_eq!(report, ReportingApiReport::Single(Report {
                rpt: ReportType::Crash(Crash {
                    reason: CrashReason::OutOfMemory,
                    stack: None,
                    is_top_level: None,
                    page_visibility: None
                }),
                age: Some(42),
                url: "https://example.com/".to_string(),
                user_agent: Some("Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0".to_string())
            }));
            let ser_res = serde_json::to_string_pretty(&report);
            assert!(ser_res.is_ok());
            assert_eq!(json, ser_res.unwrap());
        }
    }

    #[test]
    fn parse_multi() {
        // source: https://wicg.github.io/crash-reporting/
        let json = r#"[
  {
    "type": "crash",
    "body": {
      "reason": "oom"
    },
    "age": 42,
    "url": "https://example.com/",
    "user_agent": "Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0"
  }
]"#;
        let deser_res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(deser_res.is_ok());
        if let Ok(report) = deser_res {
            assert_eq!(report, ReportingApiReport::Multi(vec![Report {
                rpt: ReportType::Crash(Crash {
                    reason: CrashReason::OutOfMemory,
                    stack: None,
                    is_top_level: None,
                    page_visibility: None
                }),
                age: Some(42),
                url: "https://example.com/".to_string(),
                user_agent: Some("Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0".to_string())
            }]));
            let ser_res = serde_json::to_string_pretty(&report);
            assert!(ser_res.is_ok());
            assert_eq!(json, ser_res.unwrap());
        }
    }
}
