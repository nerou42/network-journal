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
    sampling_fraction: f32,
    server_ip: String,
    status_code: u16,
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>
}

pub async fn report_nel(report: Json<Report<NetworkError>>) -> impl Responder {
    if report.r#type == ReportType::NetworkError {
        info!("NEL {}", serde_json::to_string_pretty(&report.body).unwrap());
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
        let res = serde_json::from_str::<Report<NetworkError>>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Report {
            r#type: ReportType::NetworkError,
            body: NetworkError {
                elapsed_time: 823,
                method: "GET".to_string(),
                phase: Phase::Application,
                protocol: "h2".to_string(),
                referrer: Some("http://example.com/".to_string()),
                sampling_fraction: 0.5,
                server_ip: "2001:DB8:0:0:0:0:0:42".to_string(),
                status_code: 200,
                r#type: "http.protocol.error".to_string(),
                url: None
            },
            age: Some(0),
            url: "https://www.example.com/".to_string(),
            user_agent: None,
        })
    }
}
