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

use std::path::PathBuf;

use actix_cors::Cors;
use actix_web::{dev::Service, guard::{self, Header}, http::header::{self, HeaderValue}, main, web::{resource, Payload}, App, HttpServer};
use clap::{crate_name, crate_version, Parser};
use futures_util::future::FutureExt;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use serde::Deserialize;
use simple_logger::SimpleLogger;

use crate::{config::NetworkJournalConfig, crash::report_crash, csp::report_csp, deprecation::report_deprecation, nel::report_nel, smtp_tls::report_smtp_tls};

mod config;
mod crash;
mod csp;
mod deprecation;
mod nel;
mod smtp_tls;

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = "Copyright (C) 2025 nerou GmbH This program comes with ABSOLUTELY NO WARRANTY. This is free software, and you are welcome to redistribute it under certain conditions.")]
struct Args {
    #[arg(short, long, value_name="FILE.yml", default_value = "network-journal.yml")]
    config: PathBuf
}

#[derive(Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
enum ReportType {
    #[serde(rename(deserialize = "crash"))]
    Crash,
    #[serde(rename(deserialize = "csp-violation"))]
    CSPViolation,
    #[serde(rename(deserialize = "deprecation"))]
    Deprecation,
    #[serde(rename(deserialize = "integrity-violation"))]
    IntegrityViolation,
    #[serde(rename(deserialize = "intervention"))]
    Intervention,
    #[serde(rename(deserialize = "network-error"))]
    NetworkError,
}

#[allow(unused)]
#[derive(Deserialize)]
struct Report<T> {
    //context: object,
    r#type: ReportType,
    //destination: String,
    body: T,
    age: Option<u32>,
    url: String,
    user_agent: Option<String>
}

async fn get_body_as_string(body: Payload) -> Result<String, String> {
    match body.to_bytes().await {
        Ok(bytes) => {
            match String::from_utf8(bytes.to_vec()) {
                Ok(str) => Ok(str),
                Err(err) => Err(format!("failed to convert raw payload to string: {}", err))
            }
        },
        Err(err) => Err(format!("failed to convert retrieve raw payload from payload: {}", err))
    }
}

#[main]
async fn main() -> std::io::Result<()> {
    SimpleLogger::new().env().init().unwrap();

    let args = Args::parse();

    let cfg = match confy::load_path::<NetworkJournalConfig>(args.config) {
        Ok(cfg) => cfg,
        Err(err) => panic!("config file could not be opened: {}", err)
    };

    let server_string: &'static str = format!("{}/{}", crate_name!(), crate_version!()).leak();
    let server = HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["POST", "OPTIONS"])
            .allowed_header(header::CONTENT_TYPE);
        
        App::new()
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
            .service(resource("/crash").post(report_crash))
            .service(resource("/csp")
                .guard(guard::Any(Header("content-type", "application/reports+json")).or(Header("content-type", "application/csp-report")))
                .post(report_csp))
            .service(resource("/deprecation")
                .guard(Header("content-type", "application/reports+json"))
                .post(report_deprecation))
            .service(resource("/nel")
                .guard(Header("content-type", "application/reports+json"))
                .post(report_nel))
            .service(resource("/tlsrpt").post(report_smtp_tls))
    });
    let bound_server = if cfg.tls.enable && cfg.tls.key.is_some() && cfg.tls.cert.is_some() {
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(cfg.tls.key.unwrap(), SslFiletype::PEM)
            .unwrap();
        builder.set_certificate_chain_file(cfg.tls.cert.unwrap()).unwrap();

        server.bind_openssl(format!("{}:{}", cfg.listen, cfg.port), builder)?
    } else {
        server.bind((cfg.listen, cfg.port))?
    };
    bound_server.run().await
}
