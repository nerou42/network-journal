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
#[serde(rename = "snake_case")]
pub enum CrashReason {
    #[serde(rename = "oom")]
    OutOfMemory,
    Unresponsive
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum PageVisibility {
    Visible,
    Hidden
}

#[derive(Serialize, Deserialize, Debug)]
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
