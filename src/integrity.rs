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

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct IntegrityViolation {
    #[serde(rename = "documentURL")]
    document_url: String,
    #[serde(rename = "blockedURL")]
    blocked_url: String,
    destination: String,
    report_only: bool
}

pub async fn report_integrity(report: Json<Report<IntegrityViolation>>) -> impl Responder {
    if report.r#type == ReportType::IntegrityViolation {
        info!("IntegrityViolation {}", serde_json::to_string_pretty(&report.body).unwrap());
        HttpResponse::Ok()
    } else {
        error!("invalid report type: {:?}", report.r#type);
        HttpResponse::BadRequest()
    }
}
