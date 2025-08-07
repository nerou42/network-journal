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
#[serde(rename_all = "camelCase")]
pub struct Deprecation {
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    anticipated_removal: Option<String>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column_number: Option<u64>
}

#[cfg(test)]
mod tests {
    use crate::reporting_api::{Report, ReportType, ReportingApiReport};

    use super::*;

    #[test]
    fn parse_report() {
        // source: https://wicg.github.io/deprecation-reporting/
        let json = r#"{
            "type": "deprecation",
            "age": 32,
            "url": "https://example.com/",
            "user_agent": "Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0",
            "body": {
                "id": "websql",
                "anticipatedRemoval": "2020-01-01",
                "message": "WebSQL is deprecated and will be removed in Chrome 97 around January 2020",
                "sourceFile": "https://example.com/index.js",
                "lineNumber": 1234,
                "columnNumber": 42
            }
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::Deprecation(Deprecation {
                id: "websql".to_string(),
                anticipated_removal: Some("2020-01-01".to_string()),
                message: "WebSQL is deprecated and will be removed in Chrome 97 around January 2020".to_string(),
                source_file: Some("https://example.com/index.js".to_string()),
                line_number: Some(1234),
                column_number: Some(42)
            }),
            age: Some(32),
            url: "https://example.com/".to_string(),
            user_agent: Some("Mozilla/5.0 (X11; Linux x86_64; rv:60.0) Gecko/20100101 Firefox/60.0".to_string()),
        }));
    }
}
