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

use std::path::Path;
use serde::{Deserialize, Serialize};
use uaparser_rs::UAParser;
use url::ParseError;

#[derive(Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Client {
    pub family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub patch_minor: Option<String>
}

impl Client {
    fn from_user_agent(ua: uaparser_rs::UserAgent) -> Client {
        Client {
            family: ua.family,
            major: ua.major,
            minor: ua.minor,
            patch: ua.patch,
            patch_minor: ua.patch_minor
        }
    }

    fn from_os(os: uaparser_rs::Os) -> Client {
        Client {
            family: os.family,
            major: os.major,
            minor: os.minor,
            patch: os.patch,
            patch_minor: os.patch_minor
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Device {
    pub family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brand: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>
}

impl Device {
    fn from_device(dev: uaparser_rs::Device) -> Device {
        Device {
            family: dev.family,
            brand: dev.brand,
            model: dev.model
        }
    }
}

pub fn analyze_user_agent(user_agent: &str) -> (Client, Client, Device) {
    #[cfg(debug_assertions)]
    let path = "./regexes.yaml";
    #[cfg(not(debug_assertions))]
    let path = "/usr/share/network-journal/regexes.yaml";
    if Path::new(path).exists() {
        let uap = UAParser::from_yaml(path).unwrap();
        let client_info = uap.parse(user_agent);
        (Client::from_user_agent(client_info.user_agent), Client::from_os(client_info.os), Device::from_device(client_info.device))
    } else {
        (Client::default(), Client::default(), Device::default())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Url {
    pub host: Option<String>,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>
}

pub fn analyze_url(url: &str) -> Result<Url, ParseError> {
    let parsed_url = url::Url::parse(url)?;
    Ok(Url {
        host: parsed_url.host_str().map(|s| s.to_owned()),
        path: parsed_url.path().to_owned(),
        query: parsed_url.query().map(|s| s.to_owned())
    })
}
