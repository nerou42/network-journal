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

use log::debug;
use url::Url;

use crate::config::FilterConfig;

#[derive(Clone)]
pub struct Filter {
    config: FilterConfig,
}

impl Filter {
    pub fn new<'a>(config: FilterConfig) -> Self {
        Self { 
            config: config
        }
    }

    /**
     * Disallows invalid URLs and those without a host
     */
    pub fn is_domain_of_url_allowed(&self, url: &str) -> bool {
        if let Ok(parsed_url) = Url::parse(url) {
            if let Some(host) = parsed_url.host_str() {
                return self.is_domain_allowed(host);
            }
        }
        return false;
    }

    pub fn is_domain_allowed(&self, host: &str) -> bool {
        if self.config.domain_whitelist.is_empty() || self.config.domain_whitelist.contains(&host.to_string()) {
            return true;
        } else {
            debug!("got report for domain \"{}\", which is not whitelisted -> drop", host);
            return false;
        }
    }
}
