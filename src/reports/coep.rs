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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Disposition {
    Enforce,
    Reporting
}

 #[derive(Serialize, Deserialize, PartialEq, Debug)]
 #[serde(rename_all = "camelCase")]
pub struct CrossOriginEmbedderPolicyViolation {
    r#type: String,
    #[serde(rename = "blockedURL")]
    blocked_url: String,
    disposition: Disposition
}

#[cfg(test)]
mod tests {
    use crate::reports::reporting_api::{Report, ReportType, ReportingApiReport};

    use super::*;

    #[test]
    fn parse_report() {
        let json = r#"{
            "age": 7,
            "body": {
                "disposition": "reporting",
                "blockedURL": "https://example.com/",
                "type": "access-to-opener"
            },
            "type": "coep",
            "url": "bar.example/foo",
            "user_agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36"
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::COEP(CrossOriginEmbedderPolicyViolation {
                disposition: Disposition::Reporting,
                blocked_url: "https://example.com/".to_string(),
                r#type: "access-to-opener".to_string()
            }),
            age: Some(7),
            url: "bar.example/foo".to_string(),
            user_agent: Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36".to_string()),
        }));
    }
}
