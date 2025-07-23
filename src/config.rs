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

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NetworkJournalConfig {
    /// listen address, defaults to 127.0.0.1
    pub listen: String,
    /// defaults to 8080
    pub port: u16,
    pub tls: TlsConfig
}

impl Default for NetworkJournalConfig {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1".to_string(),
            port: 8080,
            tls: TlsConfig { 
                enable: false, 
                cert: None, 
                key: None
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TlsConfig {
    /// default false
    pub enable: bool,
    /// PEM encoded certificate file
    pub cert: Option<PathBuf>,
    /// PEM encoded private key file
    pub key: Option<PathBuf>
}
