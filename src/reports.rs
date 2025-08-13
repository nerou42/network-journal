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

use log::info;
use serde::Serialize;

use crate::{
    reports::{csp::CSPReport, smtp_tls::SMTPTLSReport}, 
    processing::user_agent::{analyze_user_agent, Client, Device}
};

pub mod coep;
pub mod coop;
pub mod crash;
pub mod csp;
pub mod deprecation;
pub mod dmarc;
pub mod integrity;
pub mod intervention;
pub mod nel;
pub mod permissions;
pub mod reporting_api;
pub mod smtp_tls;

#[derive(Serialize, Debug)]
#[serde(untagged)]
enum ReportType<'a> {
    ReportingAPI(&'a reporting_api::Report),
    CSPLvl2(&'a CSPReport),
    SMTPTLSRPT(&'a SMTPTLSReport)
}

#[derive(Serialize, Default, Debug)]
struct Derived {
    pub client: Client,
    pub os: Client,
    pub device: Device
}

#[derive(Serialize, Debug)]
struct DecoratedReport<'a> {
    report: &'a ReportType<'a>,
    derived: Derived
}

async fn handle_report(report: &ReportType<'_>, user_agent: Option<&str>) -> Result<(), serde_json::Error> {
    let mut decorated = DecoratedReport {
        report: report,
        derived: Derived::default()
    };
    if let Some(ua) = user_agent {
        (decorated.derived.client, decorated.derived.os, decorated.derived.device) = analyze_user_agent(ua);
    }
    
    match report {
        ReportType::ReportingAPI(rpt) => {
            if let Some(user_agent) = &rpt.user_agent {
                (decorated.derived.client, decorated.derived.os, decorated.derived.device) = analyze_user_agent(user_agent);
            }
            serde_json::to_string_pretty(&decorated).map(|serialized_report| {
                match rpt.rpt {
                    reporting_api::ReportType::COEP(_) => info!("COEP {}", serialized_report),
                    reporting_api::ReportType::COOP(_) => info!("COOP {}", serialized_report),
                    reporting_api::ReportType::Crash(_) => info!("Crash {}", serialized_report),
                    reporting_api::ReportType::CSPHash(_) => info!("CSP-Hash {}", serialized_report),
                    reporting_api::ReportType::CSPViolation(_) => info!("CSP {}", serialized_report),
                    reporting_api::ReportType::Deprecation(_) => info!("Decprecation {}", serialized_report),
                    reporting_api::ReportType::IntegrityViolation(_) => info!("IntegrityViolation {}", serialized_report),
                    reporting_api::ReportType::Intervention(_) => info!("Intervention {}", serialized_report),
                    reporting_api::ReportType::NetworkError(_) => info!("NEL {}", serialized_report),
                    reporting_api::ReportType::PermissionsPolicyViolation(_) => info!("PermissionsPolicyViolation {}", serialized_report),
                }
            })
        },
        ReportType::CSPLvl2(_rpt) => {
            serde_json::to_string_pretty(&decorated).map(|serialized_report| {
                info!("CSP {}", serialized_report)
            })
        },
        ReportType::SMTPTLSRPT(_rpt) => {
            serde_json::to_string_pretty(&decorated).map(|serialized_report| {
                info!("SMTP-TLS-RPT {}", serialized_report)
            })
        }
    }
}
