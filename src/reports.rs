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

use std::fmt::Display;

use log::info;
use serde::Serialize;

use crate::{
    processing::{filter::Filter, derivation::{analyze_url, analyze_user_agent, Client, Device, Url}}, 
    reports::{csp::CSPReport, dmarc::DMARCReport, smtp_tls::SMTPTLSReport}
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
pub mod tls_cert_validity;

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum ReportType<'a> {
    ReportingAPI(&'a reporting_api::Report),
    CSPLvl2(&'a CSPReport),
    SMTPTLSRPT(&'a SMTPTLSReport),
    DMARC(&'a DMARCReport)
}

#[derive(Serialize, Default, Debug)]
struct Derived {
    pub client: Client,
    pub os: Client,
    pub device: Device,
    pub url: Url
}

#[derive(Serialize, Debug)]
struct DecoratedReport<'a> {
    report: &'a ReportType<'a>,
    derived: Derived
}

#[derive(Debug)]
pub enum Error {
    Parse(serde_json::Error),
    Serialize(serde_json::Error)
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(err) => write!(f, "failed to parse report: {}", err),
            Error::Serialize(err) => write!(f, "failed to serialize report: {}", err)
        }
    }
}

pub fn handle_report(report: &ReportType<'_>, user_agent: Option<&str>, filter: &Filter) -> Result<(), Error> {
    let mut decorated = DecoratedReport {
        report: report,
        derived: Derived::default()
    };
    if let Some(ua) = user_agent {
        (decorated.derived.client, decorated.derived.os, decorated.derived.device) = analyze_user_agent(ua);
    }
    
    let rpt_type_str: &str;
    match report {
        ReportType::ReportingAPI(rpt) => {
            if filter.is_domain_of_url_allowed(&rpt.url) {
                if let Ok(parsed_url) = analyze_url(&rpt.url) {
                    decorated.derived.url = parsed_url;
                }
                if let Some(user_agent) = &rpt.user_agent {
                    (decorated.derived.client, decorated.derived.os, decorated.derived.device) = analyze_user_agent(&user_agent);
                }

                rpt_type_str = match rpt.rpt {
                    reporting_api::ReportType::COEP(_) => "COEP",
                    reporting_api::ReportType::COOP(_) => "COOP",
                    reporting_api::ReportType::Crash(_) => "Crash",
                    reporting_api::ReportType::CSPHash(_) => "CSP-Hash",
                    reporting_api::ReportType::CSPViolation(_) => "CSP",
                    reporting_api::ReportType::Deprecation(_) => "Decprecation",
                    reporting_api::ReportType::IntegrityViolation(_) => "IntegrityViolation",
                    reporting_api::ReportType::Intervention(_) => "Intervention",
                    reporting_api::ReportType::NetworkError(_) => "NEL",
                    reporting_api::ReportType::PermissionsPolicyViolation(_) => "PermissionsPolicyViolation",
                };
            } else {
                return Ok(());
            }
        },
        ReportType::CSPLvl2(rpt) => {
            if filter.is_domain_of_url_allowed(&rpt.csp_report.document_url) {
                if let Ok(parsed_url) = analyze_url(&rpt.csp_report.document_url) {
                    decorated.derived.url = parsed_url;
                }
                rpt_type_str = "CSP";
            } else {
                return Ok(());
            }
        },
        ReportType::SMTPTLSRPT(rpt) => {
            decorated.derived.url.host = rpt.get_policy_domains().get(0).map(|s| s.to_string());
            if let Some(host) = &decorated.derived.url.host {
                if !filter.is_domain_allowed(host.as_str()) {
                    return Ok(());
                }
            }
            rpt_type_str = "SMTP-TLS-RPT";
        },
        ReportType::DMARC(rpt) => {
            decorated.derived.url.host = Some(rpt.get_published_policys_domain().to_string());
            if let Some(host) = &decorated.derived.url.host {
                if !filter.is_domain_allowed(host.as_str()) {
                    return Ok(());
                }
            }
            decorated.derived.client.family = rpt.get_sender_organisation().to_string();
            rpt_type_str = "DMARC";
        }
    }
    match serde_json::to_string_pretty(&decorated) {
        Ok(serialized_report) => {
            info!("{} {}", rpt_type_str, serialized_report);
            Ok(())
        },
        Err(err) => Err(Error::Serialize(err))
    }
}
