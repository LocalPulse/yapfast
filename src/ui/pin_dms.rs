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

/// The category editor's target: an existing category, or a new one for a DM.
#[derive(Clone, Debug, Default)]
pub(super) struct CategoryEdit {
    pub id: Option<String>,
    pub room: Option<String>,
}

pub(super) const PIN_TO: &str = "dm-pin:";
pub(super) const PIN_NEW: &str = "dm-pin:new";

impl super::Adapter {
    /// The categories a DM can join, then the category editor.
    pub(super) fn pin_category_rows(&self) -> Vec<super::MenuRow> {
        self.layout
            .pin_categories
            .iter()
            .map(|category| super::MenuRow {
                id: format!("{PIN_TO}{}", category.id).into(),
                text: category.name.clone().into(),
                ..Default::default()
            })
            .chain([super::MenuRow {
                id: PIN_NEW.into(),
                text: "New category...".into(),
                separated: true,
                ..Default::default()
            }])
            .collect()
    }

    pub(super) fn choose_pin_category(
        &mut self,
        window: &super::YapfastWindow,
        room: String,
        id: &str,
    ) {
        if id == PIN_NEW {
            self.open_category_editor(window, None, Some(room));
        } else if let Some(category) = id.strip_prefix(PIN_TO) {
            self.put_in_category(window, room, category);
        }
    }

    pub(super) fn open_category_editor(
        &mut self,
        window: &super::YapfastWindow,
        id: Option<String>,
        room: Option<String>,
    ) {
        let current = id.as_ref().and_then(|id| {
            self.layout
                .pin_categories
                .iter()
                .find(|category| &category.id == id)
        });
        window.set_pin_category_name(
            current
                .map_or("Pinned", |category| category.name.as_str())
                .into(),
        );
        window.set_pin_category_color(
            current
                .map(|category| format!("#{:06x}", category.color))
                .unwrap_or_else(|| "#99aab5".into())
                .into(),
        );
        window.set_pin_category_error("".into());
        window.set_pin_category_editing(id.is_some());
        self.pin_category_edit = Some(CategoryEdit { id, room });
        window.set_pin_category_open(true);
    }

    pub(super) fn close_category_editor(&mut self, window: &super::YapfastWindow) {
        self.pin_category_edit = None;
        window.set_pin_category_open(false);
    }

    pub(super) fn save_category(&mut self, window: &super::YapfastWindow) {
        let Some(edit) = self.pin_category_edit.clone() else {
            return;
        };
        let name = window.get_pin_category_name().trim().to_owned();
        let color_text = window.get_pin_category_color();
        let color_text = color_text.trim().trim_start_matches('#');
        let color = u32::from_str_radix(color_text, 16)
            .ok()
            .filter(|_| color_text.len() == 6);
        if name.is_empty() || color.is_none() {
            window.set_pin_category_error(
                "Enter a name and a six-digit color, such as #99aab5.".into(),
            );
            return;
        }
        let id = edit.id.unwrap_or_else(|| {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos();
            format!("local-{nonce}")
        });
        if let Some(current) = self
            .layout
            .pin_categories
            .iter_mut()
            .find(|category| category.id == id)
        {
            current.name = name;
            current.color = color.unwrap();
        } else {
            self.layout.pin_categories.push(PinCategory {
                id: id.clone(),
                name,
                color: color.unwrap(),
                rooms: Vec::new(),
            });
        }
        if let Some(room) = edit.room {
            self.put_in_category(window, room, &id);
        }
        self.layout.write(&self.root);
        self.sidebar = None;
        self.show_sidebar(window);
        self.close_category_editor(window);
    }

    pub(super) fn delete_category(&mut self, window: &super::YapfastWindow) {
        let Some(edit) = self.pin_category_edit.take() else {
            return;
        };
        if let Some(id) = edit.id {
            self.layout
                .pin_categories
                .retain(|category| category.id != id);
            self.layout.write(&self.root);
            self.sidebar = None;
            self.show_sidebar(window);
        }
        window.set_pin_category_open(false);
    }

    fn put_in_category(&mut self, window: &super::YapfastWindow, room: String, id: &str) {
        if self.app.state.pinned.contains(&room) {
            self.action(
                window,
                yapfast_core::application::Action::Pin {
                    room: room.clone(),
                    pinned: false,
                },
            );
        }
        for category in &mut self.layout.pin_categories {
            category.rooms.retain(|each| each != &room);
        }
        if let Some(category) = self
            .layout
            .pin_categories
            .iter_mut()
            .find(|category| category.id == id)
        {
            category.rooms.push(room);
        }
        self.layout.write(&self.root);
        self.sidebar = None;
        self.show_sidebar(window);
    }
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
