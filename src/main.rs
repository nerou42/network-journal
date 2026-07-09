/**
 * network-journal - collect network reports and print them to file
 * Copyright (C) 2026 nerou GmbH
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

use std::{path::PathBuf, sync::LazyLock, thread::{sleep, Builder}, time::Duration};

use actix_cors::Cors;
use actix_web::{App, HttpServer, dev::Service, guard::{self, Header}, http::header::{self, HeaderValue}, main, web::{Bytes, Data, JsonConfig, PayloadConfig, resource}};
use clap::{crate_name, crate_version, Parser};
use futures_util::future::FutureExt;
use log::{error, trace, warn, LevelFilter};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use simple_logger::SimpleLogger;

use crate::{
    config::NetworkJournalConfig, processing::filter::Filter, reports::{
        csp::report_csp, dmarc::IMAPClient, handle_report, reporting_api::reporting_api, smtp_tls::report_smtp_tls, tls_cert_validity::TLSCertificateValidityReport, ReportType
    }
};

mod config;
mod reports;
mod processing;

static CONFIG: LazyLock<NetworkJournalConfig> = LazyLock::new(|| {
    let args = Args::parse();

    NetworkJournalConfig::read(args.config.to_str().unwrap())
});

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = "Copyright (C) 2026 nerou GmbH This program comes with ABSOLUTELY NO WARRANTY. This is free software, and you are welcome to redistribute it under certain conditions.")]
struct Args {
    /// path to configuration file
    #[arg(short, long, value_name="FILE.yml", default_value = "network-journal.yml")]
    config: PathBuf
}

struct WebState {
    filter: Filter
}

fn get_body_as_string(bytes: Bytes) -> Result<String, String> {
    match String::from_utf8(bytes.to_vec()) {
        Ok(str) => Ok(str),
        Err(err) => Err(format!("failed to convert raw payload to string: {}", err))
    }
}

#[main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new()
        .with_module_level("reqwest", LevelFilter::Info)
        .with_module_level("hyper_util", LevelFilter::Info)
        .with_module_level("mio::poll", LevelFilter::Info)
        .env().init().unwrap();

    let _tls_cert_check_thread_handle = if !CONFIG.certificate_check.domains.is_empty() {
        Some(Builder::new().name("tls_cert_check".to_string()).spawn(move || {
            trace!("TLS certificate check thread started");

            loop {
                for domain in &CONFIG.certificate_check.domains {
                    let cert_res = TLSCertificateValidityReport::create(domain.domain.as_str(), domain.port);
                    match cert_res {
                        Ok(cert_opt) => {
                            match cert_opt {
                                Some(rpt) => {
                                    //println!("{:?}", rpt.certificate);
                                    if let Err(err) = handle_report(&ReportType::TLSCertificateValidity(&rpt), None, None) {
                                        error!("{}", err);
                                    }
                                },
                                None => warn!("no certiticate found for domain {}:{}", domain.domain, domain.port)
                            }
                        },
                        Err(err) => error!("failed to get certificate for domain {}:{}: {}", domain.domain, domain.port, err)
                    }
                }

                sleep(Duration::from_secs(86400));
            }
        }))
    } else {
        None
    };

    let filter = Filter::new(&CONFIG.filter);
    let _imap_thread_handle = if CONFIG.imap.enable {
        let filter_imap = filter.clone();
        Some(Builder::new().name("imap".to_string()).spawn(move || {
            trace!("IMAP thread started");

            loop {
                let imap_connect_res = IMAPClient::connect(
                    &CONFIG.imap.host,
                    CONFIG.imap.port,
                    &CONFIG.imap.username,
                    &CONFIG.imap.password()
                );

                match imap_connect_res {
                    Ok(mut imap_client) => {
                        trace!("IMAP connection established");
                        match imap_client.read("UNANSWERED UNSEEN UNDELETED UNDRAFT SUBJECT \"Report Domain:\"") {
                            Ok(reports) => {
                                for report in reports {
                                    if let Err(err) = handle_report(&ReportType::DMARC(&report), None, Some(&filter_imap)) {
                                        error!("{}", err);
                                    }
                                }
                            },
                            Err(err) => error!("unable to read message: {}", err)
                        }
                        if let Err(err) = imap_client.disconnect() {
                            error!("failed to disconnect from IMAP server: {}", err);
                        }
                    },
                    Err(err) => {
                        error!("failed to connect to IMAP server: {}", err);
                        continue;
                    }
                }

                sleep(Duration::from_secs(300));
            }
        }))
    } else {
        None
    };

    let server_string: &'static str = format!("{}/{}", crate_name!(), crate_version!()).leak();
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["POST", "OPTIONS"])
            .allowed_header(header::CONTENT_TYPE);

        App::new()
            .app_data(PayloadConfig::new(CONFIG.max_payload_size as usize * 1024 * 1024))
            .app_data(JsonConfig::default().limit(CONFIG.max_payload_size as usize * 1024 * 1024))
            .app_data(Data::new(WebState { 
                filter: filter.clone()
            }))
            .wrap(cors)
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    if let Ok(mut resp) = res {
                        
                        resp.headers_mut().append(header::SERVER, HeaderValue::from_str(server_string).unwrap());
                        Ok(resp)
                    } else {
                        res
                    }
                })
            })
            .service(resource("/reporting-api")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))
            .service(resource("/crash")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))
            .service(resource("/csp")
                .guard(guard::Any(Header("content-type", "application/reports+json")).or(Header("content-type", "application/csp-report")))
                .post(report_csp))
            .service(resource("/deprecation")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))
            .service(resource("/integrity")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))
            .service(resource("/intervention")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))
            .service(resource("/nel")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))
            .service(resource("/permissions")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))
            .service(resource("/tlsrpt")
                .guard(guard::Any(Header("content-type", "application/tlsrpt+gzip")).or(Header("content-type", "application/tlsrpt+json")))
                .post(report_smtp_tls))
    });
    let bound_server = if CONFIG.tls.enable && CONFIG.tls.key.is_some() && CONFIG.tls.cert.is_some() {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(CONFIG.tls.key.as_ref().unwrap(), SslFiletype::PEM)
            .unwrap();
        builder.set_certificate_chain_file(CONFIG.tls.cert.as_ref().unwrap()).unwrap();

        server.bind_openssl(format!("{}:{}", CONFIG.listen, CONFIG.port), builder)?
    } else {
        server.bind((CONFIG.listen.as_ref(), CONFIG.port))?
    };
    bound_server.run().await
}

#[cfg(test)]
mod tests {
    use actix_web::{App, http, test};

    use crate::{config::FilterConfig, reports::{crash::{Crash, CrashReason}, reporting_api::{Report, ReportType, ReportingApiReport}}};

    use super::*;

    static FILTER_CONFIG: LazyLock<FilterConfig> = LazyLock::new(|| FilterConfig::default());

    #[actix_web::test]
    async fn test_max_payload_json() {
        let app = test::init_service(App::new()
            .app_data(PayloadConfig::new(3))
            .app_data(JsonConfig::default().limit(10))
            .app_data(Data::new(WebState {
                filter: Filter::new(&FILTER_CONFIG)
            }))
            .service(resource("/reporting-api")
                .guard(Header("content-type", "application/reports+json"))
                .post(reporting_api))).await;
        let req = test::TestRequest::post()
            .uri("/reporting-api")
            .set_json(ReportingApiReport::Single(Report {
                rpt: ReportType::Crash(Crash {
                    reason: CrashReason::OutOfMemory,
                    stack: None,
                    is_top_level: None,
                    page_visibility: None
                }),
                age: None,
                url: String::new(),
                user_agent: None
            }))
            .insert_header((http::header::CONTENT_TYPE, "application/reports+json"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(413, resp.status().as_u16(), "{:?}", resp.status());
    }

    #[actix_web::test]
    async fn test_max_payload_bytes() {
        let app = test::init_service(App::new()
            .app_data(PayloadConfig::new(10))
            .app_data(JsonConfig::default().limit(3))
            .app_data(Data::new(WebState {
                filter: Filter::new(&FILTER_CONFIG)
            }))
            .service(resource("/tlsrpt")
                .guard(Header("content-type", "application/tlsrpt+json"))
                .post(report_smtp_tls))).await;
        let req = test::TestRequest::post()
            .uri("/tlsrpt")
            .set_payload("hello world!")
            .insert_header((http::header::CONTENT_TYPE, "application/tlsrpt+json"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(413, resp.status().as_u16(), "{:?}", resp.status());
    }
}
