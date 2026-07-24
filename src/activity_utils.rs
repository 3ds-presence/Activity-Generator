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

use discord_social_rpc::Activity;

/// Merge script activity into default: non-empty script fields win, else keep default.
pub fn merge_activities(script: &Activity, default: &Activity) -> Activity {
    macro_rules! merge_field {
        ($act:ident, $method:ident, $script:expr, $default:expr) => {
            let val = $script;
            let def = $default;
            if !val.is_empty() {
                $act = $act.$method(&val);
            } else if !def.is_empty() {
                $act = $act.$method(&def);
            }
        };
    }

    let mut act = default.clone();

    merge_field!(act, set_name, script.name(), default.name());
    merge_field!(act, set_state, script.state(), default.state());
    merge_field!(act, set_details, script.details(), default.details());
    act = act.set_activity_type(script.activity_type());

    // Merge assets
    let mut assets = default.assets();
    merge_field!(
        assets,
        set_large_image,
        script.assets().large_image(),
        default.assets().large_image()
    );
    merge_field!(
        assets,
        set_large_text,
        script.assets().large_text(),
        default.assets().large_text()
    );
    merge_field!(
        assets,
        set_small_image,
        script.assets().small_image(),
        default.assets().small_image()
    );
    merge_field!(
        assets,
        set_small_text,
        script.assets().small_text(),
        default.assets().small_text()
    );
    act = act.set_assets(assets);

    act
}
