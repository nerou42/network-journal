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

use std::{fmt::Display, io::{Cursor, Read}, str::{from_utf8, Utf8Error}};

use flate2::read::GzDecoder;
use imap::{ImapConnection, Session};
use log::{debug, trace};
use mail_parser::{Message, MessageParser, MimeHeaders};
use quick_xml::DeError;
use serde::{Deserialize, Serialize};
use zip::{result::ZipError, ZipArchive};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DateRange {
    begin: u64,
    end: u64
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ReportMetadata {
    org_name: String,
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_contact_info: Option<String>,
    report_id: String,
    date_range: DateRange,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    error: Vec<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    #[serde(rename = "r")]
    Relaxed,
    #[serde(rename = "s")]
    Strict
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyPublished {
    domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    adkim: Option<Alignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aspf: Option<Alignment>,
    p: Disposition,
    /// required in RFC 7489
    #[serde(skip_serializing_if = "Option::is_none")]
    sp: Option<Disposition>,
    /// required in RFC 7489
    #[serde(skip_serializing_if = "Option::is_none")]
    pct: Option<u8>,
    /// required in RFC 7489
    #[serde(skip_serializing_if = "Option::is_none")]
    fo: Option<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    None,
    Quarantine,
    Reject
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DMARCResult {
    Pass,
    Fail
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyOverrideType {
    Forwarded,
    SampledOut,
    TrustedForwarder,
    MailingList,
    LocalPolicy,
    Other
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyOverrideReason {
    r#type: PolicyOverrideType,
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PolicyEvaluated {
    disposition: Disposition,
    dkim: DMARCResult,
    spf: DMARCResult,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    reason: Vec<PolicyOverrideReason>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Row {
    source_ip: String,
    count: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    policy_evaluated: Vec<PolicyEvaluated>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Identifier {
    #[serde(skip_serializing_if = "Option::is_none")]
    envelope_to: Option<String>,
    /// required in RFC 7489
    #[serde(skip_serializing_if = "Option::is_none")]
    envelope_from: Option<String>,
    header_from: String
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DKIMResult {
    None,
    Pass,
    Fail,
    Policy,
    Neutral,
    #[serde(rename = "temperror")]
    TemporaryError,
    #[serde(rename = "permerror")]
    PermanentError
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DKIMAuthResult {
    domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<String>,
    result: DKIMResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    human_result: Option<String>
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SPFDomainScope {
    Helo,
    #[serde(rename = "mfrom")]
    MailFrom
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SPFResult {
    None,
    Neutral,
    Pass,
    Fail,
    Softfail,
    #[serde(rename = "temperror")]
    TemporaryError,
    #[serde(rename = "permerror")]
    PermanentError
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SPFAuthResult {
    domain: String,
    /// required in RFC 7489
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<SPFDomainScope>,
    result: SPFResult
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AuthResult {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    dkim: Vec<DKIMAuthResult>,
    spf: Vec<SPFAuthResult>
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Record {
    row: Row,
    identifiers: Identifier,
    auth_results: AuthResult
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename = "feedback")]
pub struct DMARCReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<f32>,
    report_metadata: ReportMetadata,
    policy_published: PolicyPublished,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    record: Vec<Record>
}

impl DMARCReport {
    pub fn get_published_policys_domain(&self) -> &String {
        &self.policy_published.domain
    }

    pub fn get_sender_organisation(&self) -> &String {
        &self.report_metadata.org_name
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum DmarcError {
    IMAP(imap::Error),
    Utf8(Utf8Error),
    Gzip(std::io::Error),
    Zip(ZipError),
    ZipRead(std::io::Error),
    Parsing(DeError)
}

impl Display for DmarcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            DmarcError::IMAP(err) => write!(f, "DmarcError while working with IMAP: {}", err),
            DmarcError::Utf8(err) => write!(f, "DmarcError while decoding UTF-8: {}", err),
            DmarcError::Gzip(err) => write!(f, "DmarcError while working with GZIP file: {}", err),
            DmarcError::Zip(err) => write!(f, "DmarcError while working with ZIP file: {}", err),
            DmarcError::ZipRead(err) => write!(f, "DmarcError while reading from ZIP file: {}", err),
            DmarcError::Parsing(err) => write!(f, "DmarcError while parsing: {}", err),
        }
    }
}

pub struct IMAPClient {
    session: Session<Box<dyn ImapConnection>>
}

impl IMAPClient {

    pub fn connect(host: &str, port: u16, username: &str, password: &str) -> Result<Self, imap::Error> {
        let client = imap::ClientBuilder::new(host, port).connect()?;

        // the client we have here is unauthenticated.
        // to do anything useful with the e-mails, we need to log in
        let mut session = client
            .login(username, password)
            .map_err(|e| e.0)?;

        // we want to fetch the first email in the INBOX mailbox
        session.select("INBOX")?;

        Ok(Self {
            session
        })
    }

    pub fn read(&mut self, query: &str) -> Result<Vec<DMARCReport>, DmarcError> {
        // fetch message number 1 in this mailbox, along with its RFC822 field.
        // RFC 822 dictates the format of the body of e-mails
        let search_results = self.session.uid_search(query).map_err(|err| DmarcError::IMAP(err))?;
        if search_results.is_empty() {
            return Ok(vec![]);
        }
        let uid_set = search_results.iter().map(|uid| uid.to_string()).collect::<Vec<String>>().join(",");
        let messages = self.session.uid_fetch(
            uid_set, 
            "RFC822"
        ).map_err(|err| DmarcError::IMAP(err))?;
        trace!("got {} e-mail(s)", messages.len());
        let mut res = vec![];
        let reader = DMARCReader::new();
        for message in messages.iter() {
            if let Some(body) = message.body() {
                trace!("found e-mail: {:?}", message.uid);
                let message = MessageParser::default().parse(&body).unwrap();
                if let Some(report) = reader.parse_message(&message)? {
                    res.push(report);
                }
            }
        }
        trace!("filtered e-mail count: {}", res.len());
        Ok(res)
    }

    pub fn disconnect(&mut self) -> Result<(), imap::Error> {
        // be nice to the server and log out
        self.session.logout()?;

        Ok(())
    }
}

struct DMARCReader {

}

impl DMARCReader {

    fn new() -> DMARCReader {
        DMARCReader {}
    }

    fn parse_message(&self, msg: &Message) -> Result<Option<DMARCReport>, DmarcError> {
        if let Some(attachment) = msg.attachment(0) {
            let mut xml: String = String::new();
            if attachment.is_content_type("text", "xml") {
                xml = from_utf8(attachment.contents()).map_err(|err| DmarcError::Utf8(err))?.to_string();
            } else if attachment.is_content_type("application", "gzip") {
                let mut decoder = GzDecoder::new(attachment.contents());
                decoder.read_to_string(&mut xml).map_err(|err| DmarcError::Gzip(err))?;
            } else if attachment.is_content_type("application", "zip") {
                let reader = Cursor::new(attachment.contents());
                let mut archive = ZipArchive::new(reader).map_err(|err| DmarcError::Zip(err))?;
                archive.by_index(0).unwrap().read_to_string(&mut xml).map_err(|err| DmarcError::ZipRead(err))?;
            } else {
                debug!("unexpected content type: {:?}", attachment.content_type());
                return Ok(None);
            }
            return self.parse_report(&xml).map(|res| Some(res));
        } else {
            debug!("no attachment found");
            return Ok(None);
        }
    }

    fn parse_report(&self, xml: &str) -> Result<DMARCReport, DmarcError> {
        quick_xml::de::from_str(xml).map_err(|err| DmarcError::Parsing(err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_report() {
        let xml = r#"<?xml version="1.0"?>	
            <feedback>	
                <report_metadata>	
                    <org_name>Yahoo</org_name>	
                    <email>dmarchelp@yahooinc.com</email>	
                    <report_id>1665623424.142074</report_id>	
                    <date_range>	
                        <begin>1665532800</begin>	
                        <end>1665619199</end>	
                    </date_range>	
                </report_metadata>	
                <policy_published>	
                    <domain>nerou.de</domain>	
                    <adkim>r</adkim>	
                    <aspf>r</aspf>	
                    <p>reject</p>	
                    <pct>100</pct>
                </policy_published>	
                <record>	
                    <row>	
                        <source_ip>23.88.125.229</source_ip>	
                        <count>1</count>	
                        <policy_evaluated>	
                            <disposition>none</disposition>	
                            <dkim>pass</dkim>	
                            <spf>pass</spf>	
                        </policy_evaluated>	
                    </row>	
                    <identifiers>	
                        <header_from>nerou.de</header_from>	
                    </identifiers>	
                    <auth_results>	
                        <dkim>	
                            <domain>nerou.de</domain>	
                            <selector>default</selector>	
                            <result>pass</result>	
                        </dkim>	
                        <spf>	
                            <domain>nerou.de</domain>
                            <result>pass</result>	
                        </spf>	
                    </auth_results>	
                </record>	
            </feedback>	
            "#;
        let reader = DMARCReader::new();
        let res = reader.parse_report(xml);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), DMARCReport {
            version: None,
            report_metadata: ReportMetadata { 
                org_name: "Yahoo".to_string(), 
                email: "dmarchelp@yahooinc.com".to_string(), 
                extra_contact_info: None, 
                report_id: "1665623424.142074".to_string(), 
                date_range: DateRange { begin: 1665532800, end: 1665619199 }, 
                error: vec![] 
            },
            policy_published: PolicyPublished { 
                domain: "nerou.de".to_string(), 
                adkim: Some(Alignment::Relaxed), 
                aspf: Some(Alignment::Relaxed), 
                p: Disposition::Reject, 
                sp: None, 
                pct: Some(100), 
                fo: None
            },
            record: vec![Record {
                row: Row { 
                    source_ip: "23.88.125.229".to_string(), 
                    count: 1, 
                    policy_evaluated: vec![PolicyEvaluated {
                        disposition: Disposition::None,
                        dkim: DMARCResult::Pass,
                        spf: DMARCResult::Pass,
                        reason: vec![]
                    }]
                },
                identifiers: Identifier {
                    envelope_to: None,
                    envelope_from: None,
                    header_from: "nerou.de".to_string()
                },
                auth_results: AuthResult { 
                    dkim: vec![DKIMAuthResult { 
                        domain: "nerou.de".to_string(), 
                        selector: Some("default".to_string()), 
                        result: DKIMResult::Pass, 
                        human_result: None
                    }], 
                    spf: vec![SPFAuthResult {
                        domain: "nerou.de".to_string(),
                        scope: None,
                        result: SPFResult::Pass
                    }]
                }
            }]
        })
    }
}
