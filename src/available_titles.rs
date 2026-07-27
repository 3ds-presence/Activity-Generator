// 3DS Presence — Discord Rich Presence for Nintendo 3DS
// Copyright (C) 2026 3DS Presence - LeonLeBreton
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::collections::HashSet;

use log::{debug, warn};

/// Path to the JSON file listing known title IDs, relative to the assets base URL.
const AVAILABLE_TITLES_PATH: &str = "available_titles.json";

/// A set of known title IDs that have icons on the assets server.
///
/// Loaded once at startup from `{assets_base_url}/available_titles.json`.
/// Unknown titles will use a fallback icon (`specials/3ds_logo.png`).
pub struct AvailableTitles {
    titles: HashSet<String>,
}

impl AvailableTitles {
    /// Fetch the list of available title IDs from the assets server.
    ///
    /// If the fetch fails (network issue, first start, etc.), logs a warning
    /// and returns an empty set — all unknown titles will get the fallback icon.
    pub async fn load(assets_base_url: &str) -> Self {
        let url = format!("{}/{}", assets_base_url.trim_end_matches('/'), AVAILABLE_TITLES_PATH);

        match Self::fetch(&url).await {
            Ok(titles) => {
                debug!("Loaded {} available title IDs from {url}", titles.len());
                Self { titles }
            }
            Err(e) => {
                warn!(
                    "Failed to fetch available titles from {url}: {e}. \
                     All titles will use the fallback 3ds_logo.png icon."
                );
                Self {
                    titles: HashSet::new(),
                }
            }
        }
    }

    /// Check whether a title ID has an icon on the assets server.
    #[must_use]
    pub fn contains(&self, title_id: &str) -> bool {
        self.titles.contains(title_id)
    }

    /// Fetch and parse the JSON from the given URL.
    async fn fetch(url: &str) -> Result<HashSet<String>, reqwest::Error> {
        let response = reqwest::get(url).await?;
        let titles: Vec<String> = response.json().await?;
        Ok(titles.into_iter().collect())
    }
}