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

use std::{fmt::Display, io, net::TcpStream};

use chrono::{DateTime, FixedOffset, Utc};
use openssl::{error::ErrorStack, nid::Nid, ssl::{self, HandshakeError, SslConnector, SslMethod}};
use serde::{Serialize, Serializer};

#[derive(Serialize, Debug)]
pub struct CertificateIssuer {
    pub common_name: String,
    pub organization_name: String,
    pub country_name: String
}

#[derive(Serialize, Debug)]
pub struct CertificateSubject {
    pub common_name: String,
}

#[derive(Serialize, Debug)]
pub struct CertificateInfo {
    pub serial_number: String,
    pub issuer: CertificateIssuer,
    pub subject: CertificateSubject,
    pub subject_alt_names: Vec<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub not_before: DateTime<FixedOffset>,
    #[serde(serialize_with = "serialize_datetime")]
    pub not_after: DateTime<FixedOffset>
}

pub fn serialize_datetime<S>(
    date: &DateTime<FixedOffset>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.to_rfc3339();
    serializer.serialize_str(&s)
}

impl CertificateInfo {

    pub fn is_valid(&self) -> bool {
        let now = Utc::now();
        return self.not_before.le(&now) && self.not_after.ge(&now);
    }

    pub fn get_days_until_expiration(&self) -> i64 {
        let now = Utc::now();
        return (self.not_after.to_utc() - now).num_days();
    }

    pub fn gather(host: &str, port: u16) -> Result<Option<CertificateInfo>, Error> {
        let connector = SslConnector::builder(SslMethod::tls())?.build();

        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        let mut stream = connector.connect(host, stream)?;

        let mut res = None;
        if let Some(cert) = stream.ssl().peer_certificate() {
            println!("{:?}", cert);
            res = Some(CertificateInfo {
                serial_number: cert.serial_number().to_bn()?.to_hex_str()?.to_string(),
                issuer: CertificateIssuer {
                    common_name: cert.issuer_name().entries_by_nid(Nid::COMMONNAME).collect::<Vec<_>>()[0].data().as_utf8()?.to_string(),
                    organization_name: cert.issuer_name().entries_by_nid(Nid::ORGANIZATIONNAME).collect::<Vec<_>>()[0].data().as_utf8()?.to_string(),
                    country_name: cert.issuer_name().entries_by_nid(Nid::COUNTRYNAME).collect::<Vec<_>>()[0].data().as_utf8()?.to_string()
                },
                subject: CertificateSubject { 
                    common_name: cert.subject_name().entries_by_nid(Nid::COMMONNAME).collect::<Vec<_>>()[0].data().as_utf8()?.to_string()
                },
                subject_alt_names: cert.subject_alt_names().unwrap().into_iter().map(|x| x.dnsname().unwrap().to_string()).collect(),
                not_before: DateTime::parse_from_str(&cert.not_before().to_string().replace("GMT", "+00:00"), "%b %d %T %Y %:z")?,
                not_after: DateTime::parse_from_str(&cert.not_after().to_string().replace("GMT", "+00:00"), "%b %d %T %Y %:z")?
            });
        }

        stream.shutdown()?;
        Ok(res)
    }
}

#[derive(Debug)]
pub enum Error {
    SslErrorStack(ErrorStack),
    TcpError(io::Error),
    HandshakeError(HandshakeError<TcpStream>),
    ShutdownError(ssl::Error),
    ParseError(chrono::ParseError)
}

impl From<ErrorStack> for Error {
    fn from(value: ErrorStack) -> Self {
        Self::SslErrorStack(value)
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::TcpError(value)
    }
}

impl From<HandshakeError<TcpStream>> for Error {
    fn from(value: HandshakeError<TcpStream>) -> Self {
        Self::HandshakeError(value)
    }
}

impl From<ssl::Error> for Error {
    fn from(value: ssl::Error) -> Self {
        Self::ShutdownError(value)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(value: chrono::ParseError) -> Self {
        Self::ParseError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SslErrorStack(e) => write!(f, "{}", e),
            Self::TcpError(e) => write!(f, "{}", e),
            Self::HandshakeError(e) => write!(f, "{}", e),
            Self::ShutdownError(e) => write!(f, "{}", e),
            Self::ParseError(e) => write!(f, "{}", e)
        }
    }
}
