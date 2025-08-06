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
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{Report, ReportType};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename = "snake_case")]
pub enum CrashReason {
    #[serde(rename = "oom")]
    OutOfMemory,
    Unresponsive
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PageVisibility {
    Visible,
    Hidden
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct Crash {
    reason: CrashReason,
    #[serde(skip_serializing_if = "Option::is_none")]
    stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_top_level: Option<bool>,
    #[serde(alias = "visibility_state", skip_serializing_if = "Option::is_none")]
    page_visibility: Option<PageVisibility>
}

pub async fn report_crash(report: Json<Report<Crash>>) -> impl Responder {
    if report.r#type == ReportType::Crash {
        info!("CRASH {}", serde_json::to_string_pretty(&report.body).unwrap());
        HttpResponse::Ok()
    } else {
        error!("invalid report type: {:?}", report.r#type);
        HttpResponse::BadRequest()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_report() {
        // source: https://wicg.github.io/crash-reporting/
        let json = r#"{
            "type": "crash",
            "age": 42,
            "url": "https://example.com/",
            "user_agent": "Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0",
            "body": {
                "reason": "oom"
            }
        }"#;
        let res = serde_json::from_str::<Report<Crash>>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Report {
            r#type: ReportType::Crash,
            body: Crash {
                reason: CrashReason::OutOfMemory,
                stack: None,
                is_top_level: None,
                page_visibility: None
            },
            age: Some(42),
            url: "https://example.com/".to_string(),
            user_agent: Some("Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0".to_string()),
        })
    }
}

