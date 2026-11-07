//! Profile cards and roles: their rows, presence and custom status lines,
//! and what a profile offers about friendship.

use super::*;

/// Only mentions get a number; plain unread activity is a dot, because sync
/// does not deliver every message of a room that isn't open.
/// "Ann is", "Ann and Bob are", or "Ann, Bob and 3 others are".
/// What a profile or DM offers for a Discord friendship, as in
/// ProfileData.friend.
pub(super) fn friend_offer(relation: Relation) -> i32 {
    match relation {
        Relation::Stranger => 1,
        Relation::Outgoing => 2,
        Relation::Incoming => 3,
        Relation::Me | Relation::Friend | Relation::Bot => 0,
    }
}

/// The custom status as the account panel shows it: its words on one line,
/// after its emoji when that's Unicode, and the address of its custom emoji's
/// picture.
pub(super) fn status_line(status: &yapfast_core::chat::CustomStatus) -> (String, Option<String>) {
    let text = one_line(&status.text);
    match (&status.emoji, &status.emoji_id) {
        (_, Some(id)) => (text, yapfast_core::discord::emoji_picture(id)),
        (Some(emoji), None) if text.is_empty() => (emoji.clone(), None),
        (Some(emoji), None) => (format!("{emoji} {text}"), None),
        (None, None) => (text, None),
    }
}

/// A custom status or what someone is doing, as a row shows it: its words on
/// one line, and the address of its custom emoji's picture.
pub(super) fn activity_line(activity: Option<&str>) -> (SharedString, SharedString) {
    let (emoji, words) = activity.map_or((None, ""), yapfast_core::discord::status_parts);
    (one_line(words).into(), emoji.unwrap_or_default().into())
}

/// When ActivityClock's seconds start: 2020 began, in Unix seconds, so
/// Slint's 32-bit integers hold them for decades.
const ACTIVITY_EPOCH: u64 = 1_577_836_800;

/// Now on ActivityClock.
fn activity_now() -> i32 {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    i32::try_from(now.saturating_sub(ACTIVITY_EPOCH)).unwrap_or(i32::MAX)
}

/// Someone's activities as their profile's cards show them.
pub(super) fn activity_rows(activities: &yapfast_core::chat::Activities) -> ModelRc<ActivityRow> {
    if activities.is_empty() {
        return ModelRc::default();
    }
    #[cfg(feature = "discord")]
    let rows: Vec<ActivityRow> = activities.iter().map(activity_row).collect();
    #[cfg(not(feature = "discord"))]
    let rows: Vec<ActivityRow> = Vec::new();
    ModelRc::new(VecModel::from(rows))
}

#[cfg(feature = "discord")]
fn activity_row(activity: &yapfast_core::chat::Activity) -> ActivityRow {
    let seconds = |ms: Option<u64>| {
        ms.map_or(0, |ms| {
            i32::try_from((ms / 1000).saturating_sub(ACTIVITY_EPOCH)).unwrap_or(0)
        })
    };
    let text = |text: &Option<Box<str>>| SharedString::from(text.as_deref().unwrap_or_default());
    // A Hang status's emoji goes before its words, as a status's does.
    let name = match &activity.emoji {
        Some(emoji) => format!("{emoji} {}", activity.name),
        None => activity.name.to_string(),
    };
    let (name_emoji, name) = yapfast_core::discord::status_parts(&name);
    let state = match (&activity.state, activity.party) {
        (Some(state), Some((size, most))) => format!("{state} ({size} of {most})"),
        (None, Some((size, most))) => format!("{size} of {most}"),
        (state, None) => state.as_deref().unwrap_or_default().to_owned(),
    };
    // Spotify's card is titled by the song, then by whom and on which album.
    let (name, details, state) = if activity.spotify() {
        let after = |word: &str, text: &Option<Box<str>>| {
            text.as_deref()
                .map_or(String::new(), |text| format!("{word} {text}"))
        };
        (
            activity.details.as_deref().unwrap_or(name).to_owned(),
            after("by", &activity.state),
            after("on", &activity.large_text),
        )
    } else {
        (
            name.to_owned(),
            activity.details.as_deref().unwrap_or_default().to_owned(),
            state,
        )
    };
    ActivityRow {
        heading: activity.heading().to_uppercase().into(),
        name: name.into(),
        name_emoji: name_emoji.unwrap_or_default().into(),
        details: details.into(),
        state: state.into(),
        large_url: text(&activity.large_image),
        large_text: text(&activity.large_text),
        small_url: text(&activity.small_image),
        small_text: text(&activity.small_text),
        start: seconds(activity.start),
        end: seconds(activity.end),
        url: text(&activity.url),
        link: text(&activity.link),
    }
}

/// Whether a line under a name only says what one of the activities does.
pub(super) fn names_an_activity(line: &str, activities: &yapfast_core::chat::Activities) -> bool {
    #[cfg(feature = "discord")]
    return activities
        .iter()
        .any(|activity| activity.line().as_deref() == Some(line));
    #[cfg(not(feature = "discord"))]
    {
        let _ = (line, activities);
        false
    }
}

impl Adapter {
    /// Keeps ActivityClock current, once a second, while a profile on
    /// screen times an activity, and leaves it be otherwise.
    pub(super) fn tick_activities(&mut self, window: &YapfastWindow) {
        let timed = |data: ProfileData| {
            data.activities
                .iter()
                .any(|row| row.start > 0 || row.end > 0)
        };
        let shown = (self.app.state.profile.is_some() && timed(window.get_profile()))
            || (self.app.state.panel_profile.is_some() && timed(window.get_panel_profile()));
        if !shown {
            self.activity_clock.stop();
            return;
        }
        if self.activity_clock.running() {
            return;
        }
        window.global::<ActivityClock>().set_now(activity_now());
        let window = window.as_weak();
        self.activity_clock.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(1),
            move || {
                if let Some(window) = window.upgrade() {
                    window.global::<ActivityClock>().set_now(activity_now());
                }
            },
        );
    }
}

/// The picker's key for the custom status's emoji: the emoji, or `name:id`.
pub(super) fn status_key(status: &yapfast_core::chat::CustomStatus) -> Option<String> {
    match (&status.emoji, &status.emoji_id) {
        (Some(name), Some(id)) => Some(format!("{name}:{id}")),
        (Some(emoji), None) => Some(emoji.clone()),
        _ => None,
    }
}

/// Role rows, each holding a list of its own, compared as `same_list` does.
pub(super) fn same_role_rows(
    old: &ModelRc<ProfileRoleRow>,
    new: ModelRc<ProfileRoleRow>,
) -> ModelRc<ProfileRoleRow> {
    let rows: Vec<ProfileRoleRow> = new
        .iter()
        .enumerate()
        .map(|(index, row)| match old.row_data(index) {
            Some(shown) => ProfileRoleRow {
                roles: same_list(&shown.roles, row.roles),
            },
            None => row,
        })
        .collect();
    same_list(old, ModelRc::new(VecModel::from(rows)))
}

/// The profile to show in place of `shown`, keeping its lists where they're
/// the same; none when nothing changed.
pub(super) fn changed_profile(shown: &ProfileData, mut data: ProfileData) -> Option<ProfileData> {
    data.servers = same_list(&shown.servers, data.servers);
    data.friends = same_list(&shown.friends, data.friends);
    data.activities = same_list(&shown.activities, data.activities);
    data.badges = same_list(&shown.badges, data.badges);
    data.connections = same_list(&shown.connections, data.connections);
    data.effects = same_list(&shown.effects, data.effects);
    data.role_rows = same_role_rows(&shown.role_rows, data.role_rows);
    data.role_rows_large = same_role_rows(&shown.role_rows_large, data.role_rows_large);
    (*shown != data).then_some(data)
}

/// A profile as a card, the full profile or the DM panel shows it.
pub(super) fn profile_data(
    pictures: &mut HashMap<String, slint::Image>,
    state: &AppState,
    view: &ProfileView,
) -> ProfileData {
    let profile = &view.profile;
    let handle = [profile.handle.as_str(), profile.pronouns.as_str()]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    // The profile keeps the status's own line breaks. A line that only
    // names an activity goes, as its card says more.
    let (activity_emoji, activity) = profile
        .activity
        .as_deref()
        .filter(|line| !names_an_activity(line, &profile.activities))
        .map_or((None, ""), yapfast_core::discord::status_parts);
    ProfileData {
        id: profile.id.clone().into(),
        name: profile.name.clone().into(),
        initial: initial(&profile.name),
        tint: tint(&profile.name),
        picture: cached_picture(pictures, state, profile.avatar.as_deref()),
        decoration_url: profile
            .avatar_decoration
            .as_deref()
            .unwrap_or_default()
            .into(),
        badges: ModelRc::new(VecModel::from(
            profile
                .badges
                .iter()
                .map(|badge| ProfileBadge {
                    id: badge.id.as_str().into(),
                    description: badge.description.as_str().into(),
                    picture: cached_picture(pictures, state, badge.icon.as_deref()),
                    link: badge.link.as_deref().unwrap_or_default().into(),
                })
                .collect::<Vec<_>>(),
        )),
        connections: ModelRc::new(VecModel::from(
            profile
                .connections
                .iter()
                .map(|account| ProfileConnection {
                    kind: account.kind.as_str().into(),
                    service: connection_service(&account.kind).into(),
                    name: account.name.as_str().into(),
                    link: connection_link(&account.kind, &account.id, &account.name)
                        .unwrap_or_default()
                        .into(),
                    verified: account.verified,
                })
                .collect::<Vec<_>>(),
        )),
        theme_first: role_color(profile.theme.map(|theme| theme.0)),
        theme_second: role_color(profile.theme.map(|theme| theme.1)),
        has_theme: profile.theme.is_some(),
        server_tag: profile
            .server_tag
            .as_ref()
            .map(|tag| tag.0.as_str().into())
            .unwrap_or_default(),
        server_tag_badge: cached_picture(
            pictures,
            state,
            profile.server_tag.as_ref().map(|tag| tag.1.as_str()),
        ),
        nameplate: cached_picture(pictures, state, profile.nameplate.as_deref()),
        effects: ModelRc::new(VecModel::from(
            profile
                .effects
                .iter()
                .map(|effect| ProfileEffect {
                    picture: cached_picture(pictures, state, Some(effect.image.as_str())),
                    x: effect.x,
                    y: effect.y,
                    width: effect.width,
                    height: effect.height,
                })
                .collect::<Vec<_>>(),
        )),
        handle: handle.into(),
        banner: cached_picture(pictures, state, profile.banner.as_deref()),
        accent: role_color(profile.accent),
        has_accent: profile.accent.is_some(),
        presence: presence_name(profile.presence).into(),
        activity: activity.into(),
        activity_emoji: activity_emoji.unwrap_or_default().into(),
        activities: activity_rows(&profile.activities),
        bio: markdown(&profile.bio),
        has_bio: !profile.bio.trim().is_empty(),
        role_rows: role_rows(&profile.roles, PROFILE_CARD_WIDTH),
        role_rows_large: role_rows(&profile.roles, PROFILE_LARGE_WIDTH),
        created: profile.created.clone().unwrap_or_default().into(),
        joined: profile.joined.clone().unwrap_or_default().into(),
        servers: ModelRc::new(VecModel::from(
            profile
                .servers
                .iter()
                .map(|server| ProfileServerRow {
                    id: server.id.as_str().into(),
                    name: server.name.as_str().into(),
                    initial: initial(&server.name),
                    picture: cached_picture(pictures, state, server.icon.as_deref()),
                    banner: cached_picture(pictures, state, server.banner.as_deref()),
                    members: server
                        .members
                        .map(|count| count_of(count, "member", "members"))
                        .unwrap_or_default()
                        .into(),
                    channels: count_of(server.channels as u64, "channel", "channels").into(),
                    nick: server.nick.as_deref().unwrap_or_default().into(),
                })
                .collect::<Vec<_>>(),
        )),
        friends: ModelRc::new(VecModel::from(
            profile
                .friends
                .iter()
                .map(|friend| {
                    let known = state.friends.iter().find(|known| known.id == friend.id);
                    ProfileFriendRow {
                        id: friend.id.as_str().into(),
                        name: friend.name.as_str().into(),
                        initial: initial(&friend.name),
                        tint: tint(&friend.id),
                        picture: cached_picture(pictures, state, friend.avatar.as_deref()),
                        presence: presence_name(Some(
                            known
                                .and_then(|known| known.presence)
                                .unwrap_or(Presence::Offline),
                        ))
                        .into(),
                    }
                })
                .collect::<Vec<_>>(),
        )),
        note: if view.loading && !view.kept {
            "Loading the profile...".into()
        } else {
            view.note.clone().unwrap_or_default().into()
        },
        // Matrix IDs start with `@`; Discord user ids are numbers.
        friend: if profile.id.starts_with('@') {
            0
        } else {
            friend_offer(state.relation(&profile.id))
        },
        // Discord's card names them as the profile does.
        message_hint: if state
            .person_choices(&profile.id)
            .contains(&MenuChoice::Message)
        {
            format!("Message @{}", profile.name).into()
        } else {
            Default::default()
        },
    }
}

/// A connected account's service as Discord's profile names it.
pub(super) fn connection_service(kind: &str) -> &str {
    match kind {
        "amazon-music" => "Amazon Music",
        "battlenet" => "Battle.net",
        "bluesky" => "Bluesky",
        "bungie" => "Bungie.net",
        "crunchyroll" => "Crunchyroll",
        "domain" => "Domain",
        "ebay" => "eBay",
        "epicgames" => "Epic Games",
        "facebook" => "Facebook",
        "github" => "GitHub",
        "instagram" => "Instagram",
        "leagueoflegends" => "League of Legends",
        "mastodon" => "Mastodon",
        "paypal" => "PayPal",
        "playstation" | "playstation-stg" => "PlayStation Network",
        "reddit" => "Reddit",
        "riotgames" => "Riot Games",
        "roblox" => "Roblox",
        "samsung" => "Samsung Galaxy",
        "soundcloud" => "SoundCloud",
        "spotify" => "Spotify",
        "skype" => "Skype",
        "steam" => "Steam",
        "tiktok" => "TikTok",
        "twitch" => "Twitch",
        "twitter" => "X",
        "xbox" => "Xbox",
        "youtube" => "YouTube",
        other => other,
    }
}

/// Where a connected account's page is, for the services whose pages its
/// id or name gives; none for the others, which show without a link.
pub(super) fn connection_link(kind: &str, id: &str, name: &str) -> Option<String> {
    let plain = |text: &str| !text.is_empty() && !text.contains(['/', '?', '#', ' ']);
    let (id_ok, name_ok) = (plain(id), plain(name));
    Some(match kind {
        "github" if name_ok => format!("https://github.com/{name}"),
        "twitter" if name_ok => format!("https://x.com/{name}"),
        "twitch" if name_ok => format!("https://www.twitch.tv/{name}"),
        "reddit" if name_ok => format!("https://www.reddit.com/user/{name}"),
        "tiktok" if name_ok => format!("https://www.tiktok.com/@{name}"),
        "instagram" if name_ok => format!("https://www.instagram.com/{name}"),
        "bluesky" if name_ok => format!("https://bsky.app/profile/{name}"),
        "domain" if name_ok => format!("https://{name}"),
        "steam" if id_ok => format!("https://steamcommunity.com/profiles/{id}"),
        "youtube" if id_ok => format!("https://www.youtube.com/channel/{id}"),
        "spotify" if id_ok => format!("https://open.spotify.com/user/{id}"),
        "roblox" if id_ok => format!("https://www.roblox.com/users/{id}/profile"),
        _ => return None,
    })
}

pub(super) fn presence_name(presence: Option<Presence>) -> &'static str {
    match presence {
        Some(Presence::Online) => "online",
        Some(Presence::Mobile) => "mobile",
        Some(Presence::Idle) => "idle",
        Some(Presence::DoNotDisturb) => "dnd",
        Some(Presence::Offline) => "offline",
        None => "",
    }
}

/// Roles as chips, a row at a time. Slint has no wrapping layout, so chips
/// are measured roughly: Atkinson Hyperlegible Next Medium at 13px averages
/// under 7px a letter, as Satoshi Medium does.
pub(super) fn role_rows(roles: &[ProfileRole], width: f32) -> ModelRc<ProfileRoleRow> {
    const GAP: f32 = 4.;
    // Padding, the colour dot and the space after it.
    const CHROME: f32 = 8. + 10. + 6. + 8.;
    let mut rows: Vec<Vec<yapfast_ui::ProfileRole>> = Vec::new();
    let mut used = width;
    for role in roles {
        let chip = CHROME + 7. * role.name.chars().count() as f32;
        if used + GAP + chip > width || rows.is_empty() {
            rows.push(Vec::new());
            used = chip;
        } else {
            used += GAP + chip;
        }
        if let Some(row) = rows.last_mut() {
            row.push(yapfast_ui::ProfileRole {
                name: role.name.clone().into(),
                color: role_color(role.color),
                has_color: role.color.is_some(),
            });
        }
    }
    ModelRc::new(VecModel::from(
        rows.into_iter()
            .map(|roles| ProfileRoleRow {
                roles: ModelRc::new(VecModel::from(roles)),
            })
            .collect::<Vec<_>>(),
    ))
}

/// A Discord role's RGB colour for Slint.
pub(super) fn role_color(rgb: Option<u32>) -> slint::Color {
    rgb.map_or_else(slint::Color::default, |rgb| {
        slint::Color::from_rgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    })
}

#[cfg(test)]
mod connection_tests {
    use super::*;

    #[test]
    fn connected_accounts_link_to_their_pages_where_known() {
        assert_eq!(connection_service("twitter"), "X");
        assert_eq!(connection_service("newservice"), "newservice");
        assert_eq!(
            connection_link("github", "1", "bob-dev").as_deref(),
            Some("https://github.com/bob-dev")
        );
        assert_eq!(
            connection_link("steam", "7656", "Bob").as_deref(),
            Some("https://steamcommunity.com/profiles/7656")
        );
        // Names that would make another address, and services without a
        // page Yapfast can name, have no link.
        assert_eq!(connection_link("github", "1", "a/b"), None);
        assert_eq!(connection_link("xbox", "1", "Bob"), None);
    }
}
