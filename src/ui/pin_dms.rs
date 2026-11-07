//! DMs pinned in categories, as Vencord's PinDMs plugin keeps them: each
//! category a coloured, foldable heading above the DM list, its DMs in the
//! order they were pinned, and the other DMs below under "Direct Messages".
//! A first start with a Discord account takes the categories that account
//! has in Vencord's settings, if there are any.

use std::path::PathBuf;

/// A category of pinned DMs.
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(super) struct PinCategory {
    /// Vencord's id, kept so a category folds by it.
    pub id: String,
    pub name: String,
    /// Its heading's colour, as 0xRRGGBB.
    pub color: u32,
    /// Its DMs by room id, in order.
    pub rooms: Vec<String>,
}

/// What the folding of the DM list's headings is kept under, in the layout's
/// collapsed categories, beside servers' ids.
pub(super) const FOLDS_UNDER: &str = "@me";

/// The heading of the DMs in no category, which folds as a category does.
pub(super) const OTHERS: &str = "dm:others";

/// What a pinned category's heading folds by.
pub(super) fn folds_by(category: &PinCategory) -> String {
    format!("dm:pins:{}", category.id)
}

/// Where Vencord keeps its settings on this system.
fn vencord_settings() -> Option<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(not(windows))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")));
    Some(base?.join("Vencord").join("settings").join("settings.json"))
}

/// The DM categories PinDMs keeps for `user` in Vencord's settings, as
/// Yapfast's; none when Vencord, the plugin or the account has none.
pub(super) fn from_vencord(user: &str) -> Option<Vec<PinCategory>> {
    let text = std::fs::read_to_string(vencord_settings()?).ok()?;
    from_vencord_settings(&text, user)
}

fn from_vencord_settings(text: &str, user: &str) -> Option<Vec<PinCategory>> {
    let settings: serde_json::Value = serde_json::from_str(text).ok()?;
    let lists = &settings["plugins"]["PinDMs"]["userBasedCategoryList"];
    let categories = lists[user].as_array()?;
    let categories: Vec<PinCategory> = categories
        .iter()
        .filter_map(|category| {
            let rooms = category["channels"]
                .as_array()?
                .iter()
                .filter_map(|channel| channel.as_str())
                .filter(|channel| {
                    !channel.is_empty() && channel.bytes().all(|byte| byte.is_ascii_digit())
                })
                .map(|channel| {
                    yapfast_core::discord::room_id(yapfast_core::discord::DM_GUILD, channel)
                })
                .collect();
            Some(PinCategory {
                id: category["id"].as_str()?.to_owned(),
                name: category["name"].as_str().unwrap_or("Pinned").to_owned(),
                color: category["color"]
                    .as_u64()
                    .and_then(|color| u32::try_from(color).ok())
                    .unwrap_or(0x99aab5)
                    & 0xff_ffff,
                rooms,
            })
        })
        .collect();
    (!categories.is_empty()).then_some(categories)
}

impl super::Adapter {
    /// The first time a Discord account shows, takes its PinDMs categories
    /// from Vencord's settings, unless Yapfast has categories already.
    pub(super) fn take_vencord_pins(&mut self) {
        let Some(user) = self.app.state.discord_user.clone() else {
            return;
        };
        if self.layout.vencord_pins_for.as_deref() == Some(user.as_str()) {
            return;
        }
        if self.layout.pin_categories.is_empty()
            && let Some(categories) = from_vencord(&user)
        {
            tracing::info!(
                categories = categories.len(),
                "took the DM categories PinDMs keeps in Vencord"
            );
            self.layout.pin_categories = categories;
        }
        self.layout.vencord_pins_for = Some(user);
        self.layout.write(&self.root);
        self.sidebar = None;
    }
}

/// Where `room` comes among the pinned DMs: its category's place, then its
/// own in the category; none for a DM in no category.
pub(super) fn place(categories: &[PinCategory], room: &str) -> Option<(usize, usize)> {
    categories.iter().enumerate().find_map(|(index, category)| {
        category
            .rooms
            .iter()
            .position(|each| each == room)
            .map(|at| (index, at))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vencords_categories_of_the_account_become_pinned_dms() {
        let settings = r#"{"plugins": {"PinDMs": {"enabled": true, "userBasedCategoryList": {
            "1": [
                {"id": "a", "name": "Work", "color": 15158332, "collapsed": false, "channels": ["11", "12"]},
                {"id": "b", "name": "Studio", "color": 3447003, "channels": ["13", "bad", ""]},
                {"name": "No id", "channels": ["14"]}
            ],
            "2": [{"id": "c", "name": "Theirs", "color": 1, "channels": ["99"]}]
        }}}}"#;
        let categories = from_vencord_settings(settings, "1").unwrap();
        assert_eq!(
            categories,
            [
                PinCategory {
                    id: "a".into(),
                    name: "Work".into(),
                    color: 0xe74c3c,
                    rooms: vec!["discord:@me:11".into(), "discord:@me:12".into()],
                },
                PinCategory {
                    id: "b".into(),
                    name: "Studio".into(),
                    color: 0x3498db,
                    rooms: vec!["discord:@me:13".into()],
                },
            ]
        );
        assert_eq!(place(&categories, "discord:@me:12"), Some((0, 1)));
        assert_eq!(place(&categories, "discord:@me:13"), Some((1, 0)));
        assert_eq!(place(&categories, "discord:@me:99"), None);
        assert!(
            from_vencord_settings(settings, "3").is_none(),
            "another account's"
        );
        assert!(from_vencord_settings("{}", "1").is_none());
    }
}
