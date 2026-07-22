use discord_social_rpc::{Activity};


/// Merge a script-provided activity into the default one.
///
/// For each string field: if the script set it (non-empty), use the script value;
/// otherwise keep the default value.
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
