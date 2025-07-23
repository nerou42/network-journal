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
#[serde(rename_all = "camelCase")]
pub struct Deprecation {
    id: String,
    anticipated_removal: Option<String>,
    message: String,
    source_file: Option<String>,
    line_number: Option<u64>,
    column_number: Option<u64>
}

pub async fn report_deprecation(report: Json<Report<Deprecation>>) -> impl Responder {
    if report.r#type == ReportType::Deprecation {
        info!("DEPRECATION {}", serde_json::to_string_pretty(&report.body).unwrap());
        HttpResponse::Ok()
    } else {
        error!("invalid report type: {:?}", report.r#type);
        HttpResponse::BadRequest()
    }
}
