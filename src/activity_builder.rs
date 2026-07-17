use discord_social_rpc::{Activity, ActivityType, Assets};

/// Builds a Discord `Activity` for a known game.
pub fn build_known_activity(
    title_id: &str,
    short: &str,
    publisher: &str,
    assets_base_url: &str,
) -> Activity {
    let image_url = format!("{}/{}/icon.png", assets_base_url, title_id);

    Activity::new()
        .set_name(short)
        .set_activity_type(ActivityType::Playing)
        .set_details(publisher)
        .set_state("3ds-presence.top")
        .set_assets(Assets::new().set_large_image(&image_url))
}

/// Builds a fallback Discord `Activity` for an unknown title ID.
pub fn build_unknown_activity(
    title_id: &str,
) -> Activity {
    Activity::new()
        .set_name("3DS game")
        .set_activity_type(ActivityType::Playing)
        .set_details(&format!("TitleID : {}", title_id))
        .set_state("3ds-presence.top")
}