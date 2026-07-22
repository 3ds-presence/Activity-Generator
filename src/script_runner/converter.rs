use std::path::PathBuf;

use discord_social_rpc::{Activity, ActivityType, Assets, Timestamps};
use log::{debug, warn};
use mlua::{Table, Value};

/// Convert a Lua value returned by `build()` into an `Activity`.
pub fn value_to_activity(value: Value, script_path: &PathBuf) -> Option<Activity> {
    match value {
        Value::Table(tbl) => table_to_activity(&tbl).or_else(|| {
            warn!(
                "Script {} returned a table but could not convert to Activity",
                script_path.display()
            );
            None
        }),
        Value::Nil => {
            debug!("Script {} returned nil, using fallback", script_path.display());
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
    let name: String = match tbl.get("name") {
        Ok(n) => n,
        Err(_) => {
            warn!("Lua script returned a table without a 'name' field");
            return None;
        }
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