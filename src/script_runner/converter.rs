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

use std::path::Path;

use discord_social_rpc::{Activity, ActivityType, Assets, Timestamps};
use log::{debug, warn};
use mlua::{Table, Value};

/// Convert a Lua value returned by `build()` into an `Activity`.
pub fn value_to_activity(value: Value, script_path: &Path) -> Option<Activity> {
    match value {
        Value::Table(tbl) => table_to_activity(&tbl).or_else(|| {
            warn!(
                "Script {} returned a table but could not convert to Activity",
                script_path.display()
            );
            None
        }),
        Value::Nil => {
            debug!(
                "Script {} returned nil, using fallback",
                script_path.display()
            );
            None
        }
        other => {
            warn!(
                "Script {} returned unexpected type {:?}, using fallback",
                script_path.display(),
                other.type_name()
            );
            None
        }
    }
}

/// Extract Activity fields from a Lua table.
fn table_to_activity(tbl: &Table) -> Option<Activity> {
    let name: String = if let Ok(n) = tbl.get("name") { n } else {
        warn!("Lua script returned a table without a 'name' field");
        return None;
    };

    let mut act = Activity::new().set_name(&name);

    if let Ok(atype) = tbl.get::<i32>("activity_type") {
        let t = match atype {
            2 => ActivityType::Listening,
            3 => ActivityType::Watching,
            5 => ActivityType::Competing,
            _ => ActivityType::Playing,
        };
        act = act.set_activity_type(t);
    }

    if let Ok(s) = tbl.get::<String>("state") {
        act = act.set_state(&s);
    }

    if let Ok(d) = tbl.get::<String>("details") {
        act = act.set_details(&d);
    }

    if let Ok(assets_tbl) = tbl.get::<Table>("assets") {
        act = act.set_assets(table_to_assets(&assets_tbl));
    }

    if let Ok(ts_tbl) = tbl.get::<Table>("timestamps") {
        act = act.set_timestamps(table_to_timestamps(&ts_tbl));
    }

    Some(act)
}

/// Extract Assets from a Lua table.
fn table_to_assets(tbl: &Table) -> Assets {
    let mut assets = Assets::new();
    if let Ok(v) = tbl.get::<String>("large_image") {
        assets = assets.set_large_image(&v);
    }
    if let Ok(v) = tbl.get::<String>("large_text") {
        assets = assets.set_large_text(&v);
    }
    if let Ok(v) = tbl.get::<String>("small_image") {
        assets = assets.set_small_image(&v);
    }
    if let Ok(v) = tbl.get::<String>("small_text") {
        assets = assets.set_small_text(&v);
    }
    assets
}

/// Extract Timestamps from a Lua table.
fn table_to_timestamps(tbl: &Table) -> Timestamps {
    let mut ts = Timestamps::new();
    if let Ok(v) = tbl.get::<i64>("start") {
        ts = ts.set_start(v);
    }
    if let Ok(v) = tbl.get::<i64>("end") {
        ts = ts.set_end(v);
    }
    ts
}
