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

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[serde(rename = "dns")]
    DNS,
    Connection,
    Application
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkError {
    elapsed_time: u64,
    method: String,
    phase: Phase,
    protocol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    referrer: Option<String>,
    sampling_fraction: u16,
    server_ip: String,
    status_code: u16,
    r#type: String,
    url: String
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
