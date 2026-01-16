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
use log::error;
use openssl::{asn1::Asn1TimeRef, error::ErrorStack, nid::Nid, ssl::{self, HandshakeError, SslConnector, SslMethod, SslVerifyMode}, x509::{CrlStatus, X509Crl, X509NameRef, X509}};
use serde::{Serialize, Serializer};

const CRL_MIME_TYPES: &[&'static str] = &["application/pkix-crl", "application/x-pkcs7-crl"];

#[derive(Serialize, Debug)]
pub struct CertificateIdentifier {
    pub common_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organizational_unit_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_or_province_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality_name: Option<String>
}

impl CertificateIdentifier {

    fn extract_single_entry(props: &X509NameRef, nid: Nid) -> Option<String> {
        let items = props.entries_by_nid(nid).collect::<Vec<_>>();
        if items.is_empty() {
            None
        } else {
            match items[0].data().as_utf8() {
                Ok(utf8) => Some(utf8.to_string()),
                Err(_) => None
            }
        }
    }
}

impl From<&X509NameRef> for CertificateIdentifier {
    
    fn from(props: &X509NameRef) -> Self {
        Self {
            common_name: Self::extract_single_entry(props, Nid::COMMONNAME).unwrap(),
            organization_name: Self::extract_single_entry(props, Nid::ORGANIZATIONNAME),
            organizational_unit_name: Self::extract_single_entry(props, Nid::ORGANIZATIONALUNITNAME),
            country_name: Self::extract_single_entry(props, Nid::COUNTRYNAME),
            state_or_province_name: Self::extract_single_entry(props, Nid::STATEORPROVINCENAME),
            locality_name: Self::extract_single_entry(props, Nid::LOCALITYNAME),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct CertificateInfo {
    pub serial_number: String,
    pub issuer: CertificateIdentifier,
    pub subject: CertificateIdentifier,
    pub subject_alt_names: Vec<String>,
    #[serde(serialize_with = "serialize_datetime")]
    pub not_before: DateTime<FixedOffset>,
    #[serde(serialize_with = "serialize_datetime")]
    pub not_after: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub crl_distribution_urls: Vec<String>
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

    pub fn is_expired(&self) -> bool {
        let now = Utc::now();
        return self.not_before.gt(&now) || self.not_after.lt(&now);
    }

    pub fn get_days_until_expiration(&self) -> i64 {
        let now = Utc::now();
        return (self.not_after.to_utc() - now).num_days();
    }

    fn asn1_date_to_chrono(asn1_time: &Asn1TimeRef) -> Result<DateTime<FixedOffset>, Error> {
        Ok(DateTime::parse_from_str(&asn1_time.to_string().replace("GMT", "+00:00"), "%b %d %T %Y %:z")?)
    }

    fn extract_info_from_cert(cert: &X509) -> Result<Self, Error> {
        let mut info = Self {
            serial_number: cert.serial_number().to_bn()?.to_hex_str()?.to_string(),
            issuer: cert.issuer_name().into(),
            subject: cert.subject_name().into(),
            subject_alt_names: cert.subject_alt_names().unwrap().into_iter().map(|x| x.dnsname().unwrap().to_string()).collect(),
            not_before: Self::asn1_date_to_chrono(&cert.not_before())?,
            not_after: Self::asn1_date_to_chrono(&cert.not_after())?,
            crl_distribution_urls: vec![]
        };

        if let Some(crl_distribution_points) = cert.crl_distribution_points() {
            for cdp in crl_distribution_points.into_iter() {
                if let Some(dp) = cdp.distpoint() {
                    if let Some(fullname) = dp.fullname() {
                        for name in fullname {
                            if let Some(uri) = name.uri() {
                                info.crl_distribution_urls.push(uri.to_string());
                            }
                        }
                    }
                }
            }
        }
        return Ok(info);
    }
}

#[derive(Serialize, Debug)]
pub struct TLSCertificateValidityReport {
    pub certificate: CertificateInfo,
    pub expired: bool,
    pub expires_in_days: i64,
    pub revoked: bool,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "serialize_datetime_opt")]
    pub revoked_since: Option<DateTime<FixedOffset>>
}

pub fn serialize_datetime_opt<S>(
    date: &Option<DateTime<FixedOffset>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = if let Some(d) = date {
        d.to_rfc3339()
    } else {
        String::new()
    };
    serializer.serialize_str(&s)
}

impl TLSCertificateValidityReport {
    
    pub fn create(host: &str, port: u16) -> Result<Option<TLSCertificateValidityReport>, Error> {
        let mut connector = SslConnector::builder(SslMethod::tls())?;
        connector.set_verify(SslVerifyMode::NONE);

        let stream = TcpStream::connect(format!("{}:{}", host, port))?;
        let mut stream = connector.build().connect(host, stream)?;

        let mut res = None;
        if let Some(cert) = stream.ssl().peer_certificate() {
            //println!("{:?}", cert);

            let info = CertificateInfo::extract_info_from_cert(&cert)?;

            let mut revoked = None;
            for dist_point in &info.crl_distribution_urls {
                let response = reqwest::blocking::get(dist_point)?;
                let mime_type = response.headers().get("Content-Type").unwrap().to_str().unwrap();
                if response.status().is_success() && CRL_MIME_TYPES.contains(&mime_type){
                    let crl_res = X509Crl::from_der(&response.bytes()?);
                    if let Ok(crl) = crl_res {
                        let status = crl.get_by_cert(&cert);
                        if let CrlStatus::Revoked(rev) = status {
                            revoked = Some(CertificateInfo::asn1_date_to_chrono(rev.revocation_date())?);
                        }
                    }
                } else {
                    error!("CRL check failed, got status code {} and MIME type \"{}\"", response.status(), mime_type);
                }
            }

            res = Some(Self { 
                expired: info.is_expired(), 
                expires_in_days: info.get_days_until_expiration(),
                certificate: info,
                revoked: revoked.is_some(), 
                revoked_since: revoked
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
    ParseError(chrono::ParseError),
    HttpError(reqwest::Error),
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

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::HttpError(value)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SslErrorStack(e) => write!(f, "{}", e),
            Self::TcpError(e) => write!(f, "{}", e),
            Self::HandshakeError(e) => write!(f, "{}", e),
            Self::ShutdownError(e) => write!(f, "{}", e),
            Self::ParseError(e) => write!(f, "{}", e),
            Self::HttpError(e) => write!(f, "{}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_expired() {
        let rpt_res = TLSCertificateValidityReport::create("expired.badssl.com", 443);
        assert!(rpt_res.is_ok(), "{:?}", rpt_res.err());

        let rpt_opt = rpt_res.unwrap();
        assert!(rpt_opt.is_some());

        let rpt = rpt_opt.unwrap();
        assert!(rpt.expired);
        assert!(rpt.expires_in_days < 0);
        assert!(!rpt.revoked);
    }

    #[test]
    fn check_revoked() {
        let rpt_res = TLSCertificateValidityReport::create("revoked.badssl.com", 443);
        assert!(rpt_res.is_ok(), "{:?}", rpt_res.err());

        let rpt_opt = rpt_res.unwrap();
        assert!(rpt_opt.is_some());

        let rpt = rpt_opt.unwrap();
        assert!(!rpt.expired);
        assert!(rpt.expires_in_days > 0);
        assert!(rpt.revoked);
        assert!(rpt.revoked_since.is_some());
        assert!((Utc::now() - rpt.revoked_since.unwrap().to_utc()).num_days() > 0);
    }

    #[test]
    fn check_self_signed() {
        let rpt_res = TLSCertificateValidityReport::create("self-signed.badssl.com", 443);
        assert!(rpt_res.is_ok(), "{:?}", rpt_res.err());

        let rpt_opt = rpt_res.unwrap();
        assert!(rpt_opt.is_some());

        let rpt = rpt_opt.unwrap();
        assert!(!rpt.expired);
        assert!(rpt.expires_in_days > 0);
        assert!(!rpt.revoked);
    }
}
