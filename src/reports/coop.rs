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

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum CrossOriginOpenerPolicy {
    UnsafeNone,
    SameOrigin,
    SameOriginAllowPopups,
    #[serde(rename = "same-origin-plus-coep")]
    SameOriginPlusCOEP,
    NoopenerAllowPopups
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case", tag = "type")]
pub enum CrossOriginOpenerPolicyType {
    AccessToOpener {
        property: String,
        #[serde(rename = "openerURL")]
        opener_url: Option<String>,
        #[serde(rename = "openedWindowURL")]
        opened_window_url: Option<String>,
        #[serde(rename = "openedWindowInitialURL")]
        opened_window_initial_url: Option<String>,
        #[serde(rename = "otherURL")]
        other_url: Option<String>
    },
    NavigationToResponse {
        #[serde(rename = "previousResponseURL")]
        previous_response_url: Option<String>
    },
    NavigationFromResponse {
        #[serde(rename = "nextResponseURL")]
        next_response_url: Option<String>
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CrossOriginOpenerPolicyViolation {
    disposition: Disposition,
    effective_policy: CrossOriginOpenerPolicy,
    #[serde(flatten)]
    r#type: CrossOriginOpenerPolicyType,
    referrer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line_number: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    column_number: Option<u64>
}

#[cfg(test)]
mod tests {
    use crate::reports::reporting_api::{Report, ReportType, ReportingApiReport};

    use super::*;

    #[test]
    fn parse_report() {
        // source: https://w3c.github.io/webappsec/mitigation-guidance/COOP/rollouts.html as well as https://html.spec.whatwg.org/multipage/browsers.html
        let json = r#"{
            "age": 6,
            "body": {
                "disposition": "reporting",
                "effectivePolicy": "same-origin",
                "property": "postMessage",
                "referrer": "foo.example",
                "type": "access-to-opener"
            },
            "type": "coop",
            "url": "bar.example/foo",
            "user_agent": "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36"
        }"#;
        let res = serde_json::from_str::<ReportingApiReport>(json);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), ReportingApiReport::Single(Report {
            rpt: ReportType::COOP(CrossOriginOpenerPolicyViolation {
                disposition: Disposition::Reporting,
                effective_policy: CrossOriginOpenerPolicy::SameOrigin,
                r#type: CrossOriginOpenerPolicyType::AccessToOpener { 
                    property: "postMessage".to_string(),
                    opener_url: None,
                    opened_window_url: None,
                    opened_window_initial_url: None,
                    other_url: None
                },
                referrer: Some("foo.example".to_string()),
                source_file: None,
                line_number: None,
                column_number: None
            }),
            age: Some(6),
            url: "bar.example/foo".to_string(),
            user_agent: Some("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.141 Safari/537.36".to_string()),
        }));
    }
}
