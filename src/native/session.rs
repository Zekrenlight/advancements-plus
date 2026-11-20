// SPDX-License-Identifier: MIT
// Advancements+ game side: tree cache, plaques, and commands.
// This support module sits in the game (beside native_hooks/client_tick_post.rs).
// Rules live in `advplus_logic`; this file only talks to the game.

use mod_advplus::logic as rules;
use sunrise_client::advancements::{Frame, Progress};
use rules::persist::{dump_prefs, load_prefs, FILE_NAME};
use rules::{close, Entry, FrameKind, Prefs};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::time::Instant;

pub const MOD_ID: &str = "advplus";
pub const TITLE: &str = "Advancements+";
pub const VERSION: &str = "1.0.0";
pub const BLURB: &str = "A clearer advancements screen for Sunrise, with a full tree, search, and on-screen plaques.";
pub const AUTHOR: &str = "Zekrenlight";

#[derive(Clone, Debug)]
pub struct Plaque {
    pub id: String,
    pub title: String,
    pub icon: String,
    pub frame: FrameKind,
    pub started: Instant,
    pub sound_in: bool,
    pub sound_out: bool,
}

#[derive(Clone, Debug)]
pub struct Session {
    pub prefs: Prefs,
    pub ready: bool,
    pub claim: bool,
    pub return_to_pause: bool,
    pub entries: Vec<Entry>,
    pub tab: usize,
    pub scroll: f32,
    pub pan: [f32; 2],
    pub zoom: f32,
    pub bar_grab: Option<f32>,
    pub drag: Option<(f32, f32, [f32; 2])>,
    pub query: String,
    pub query_focus: bool,
    pub want_close: bool,
    pub hover: Option<String>,
    pub plaques: Vec<Plaque>,
    pub settings_tab: bool,
    pub pan_init: bool,
    pub orient: rules::listing::Orient,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            prefs: Prefs::default(),
            ready: false,
            claim: false,
            return_to_pause: false,
            entries: Vec::new(),
            tab: 0,
            scroll: 0.0,
            pan: [0.0, 0.0],
            zoom: 1.0,
            bar_grab: None,
            drag: None,
            query: String::new(),
            query_focus: false,
            want_close: false,
            hover: None,
            plaques: Vec::new(),
            settings_tab: false,
            pan_init: false,
            orient: rules::listing::Orient::Ltr,
        }
    }
}

thread_local! {
    static SESSION: RefCell<Session> = RefCell::new(Session::default());
}

pub fn revise<R>(f: impl FnOnce(&mut Session) -> R) -> R {
    SESSION.with(|c| f(&mut c.borrow_mut()))
}

pub fn claimed() -> bool {
    revise(|s| s.claim)
}

pub fn ensure_ready(game: &crate::game::Game<'_>) {
    revise(|s| {
        if s.ready {
            return;
        }
        let path = game.native_data_dir(MOD_ID).join(FILE_NAME);
        if let Ok(text) = fs::read_to_string(&path) {
            s.prefs = load_prefs(&text);
        }
        s.ready = true;
    });
}

pub fn write_prefs(game: &crate::game::Game<'_>) {
    let dir = game.native_data_dir(MOD_ID);
    let _ = fs::create_dir_all(&dir);
    let text = revise(|s| dump_prefs(&s.prefs));
    let _ = fs::write(dir.join(FILE_NAME), text);
}

pub fn sync(game: &crate::game::Game<'_>) {
    let tree = &game.adv.tree;
    let kids = tree.children();
    let visible = if game.adv.positions.is_some() {
        vec![true; tree.list.len()]
    } else {
        tree.visible(&kids, |i| game.adv.progress.done(&tree.list[i]))
    };
    let mut entries = Vec::new();
    for (i, a) in tree.list.iter().enumerate() {
        let Some(d) = &a.display else {
            continue;
        };
        let title = resolve_component(&d.title);
        let description = resolve_component(&d.description);
        let obtained = obtained_of(&game.adv.progress, &a.id);
        let frame = match d.frame {
            Frame::Goal => FrameKind::Goal,
            Frame::Challenge => FrameKind::Challenge,
            Frame::Task => FrameKind::Task,
        };
        entries.push(Entry {
            id: a.id.clone(),
            parent: a.parent.clone(),
            title: if title.is_empty() {
                rules::text::label_of(&a.id)
            } else {
                title
            },
            description,
            frame,
            icon: d.icon.item.to_string(),
            hidden: d.hidden,
            x: 0.0,
            y: 0.0,
            criteria: a.criteria.iter().map(|c| c.name.clone()).collect(),
            requirements: a.requirements.clone(),
            obtained,
            visible: visible.get(i).copied().unwrap_or(false),
            recipe: rules::is_recipe_id(&a.id),
        });
    }
    // Attach to the nearest painted ancestor (skip display-less nodes) and
    // lay each root out adaptively (LTR or top-down) from parent links.
    // Packet positions are not used: in practice they stacked some trees
    // vertically.
    let shown: BTreeSet<String> = entries
        .iter()
        .filter(|e| !e.recipe)
        .map(|e| e.id.clone())
        .collect();
    let parents: BTreeMap<String, String> = tree
        .list
        .iter()
        .filter_map(|a| a.parent.clone().map(|p| (a.id.clone(), p)))
        .collect();
    for e in &mut entries {
        e.parent = nearest_shown(&e.id, &parents, &shown);
    }
    let root_ids: Vec<String> = rules::filter::roots(&entries)
        .into_iter()
        .map(|e| e.id.clone())
        .collect();
    for id in root_ids {
        rules::listing::layout_tab(&mut entries, &id, rules::listing::DEFAULT_VIEW);
    }
    revise(|s| {
        s.zoom = rules::listing::snap_zoom(s.zoom);
        s.entries = entries;
        let n = rules::filter::roots(&s.entries).len();
        if n == 0 {
            s.tab = 0;
        } else if !s.settings_tab && s.tab >= n {
            s.tab = n - 1;
        }
    });
}

fn nearest_shown(
    id: &str,
    parents: &BTreeMap<String, String>,
    shown: &BTreeSet<String>,
) -> Option<String> {
    let mut cur = parents.get(id).cloned();
    let mut guard = 0;
    while let Some(p) = cur {
        if shown.contains(&p) {
            return Some(p);
        }
        cur = parents.get(&p).cloned();
        guard += 1;
        if guard > 64 {
            return None;
        }
    }
    None
}

fn obtained_of(progress: &Progress, id: &str) -> BTreeSet<String> {

    progress
        .criteria
        .get(id)
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default()
}

fn resolve_component(v: &serde_json::Value) -> String {
    // Same path the vanilla advancements screen uses: Component → Text → plain,
    // which looks the key up in the current language.
    let looked = sunrise_client::scoreboard::comp_text(v).plain();
    let polished = rules::text::polish_resolved(&looked);
    if !polished.is_empty() {
        return polished;
    }
    let raw = v.to_string();
    rules::text::resolve_display(&raw, &|key| {
        let t = sunrise_client::lang::get(key);
        if t != key {
            Some(t)
        } else {
            None
        }
    })
}

pub fn roots() -> Vec<Entry> {
    revise(|s| rules::filter::roots(&s.entries).into_iter().cloned().collect())
}

pub fn current_root() -> Option<String> {
    revise(|s| {
        if s.settings_tab {
            return None;
        }
        rules::filter::roots(&s.entries)
            .get(s.tab)
            .map(|e| e.id.clone())
    })
}

pub fn take_over(game: &mut crate::game::Game<'_>) {
    let is_adv = matches!(game.screen, crate::game::Screen::Advancements);
    if is_adv {
        revise(|s| {
            let (claim, want) = close::apply_take_over(s.claim, s.want_close);
            s.claim = claim;
            s.want_close = want;
        });
        game.load_advancements();
        sync(game);
    }
}

pub fn close(game: &mut crate::game::Game<'_>) {
    let dest = revise(|s| {
        let d = close::decide_close(s.claim, s.want_close, s.return_to_pause);
        s.want_close = false;
        s.claim = false;
        s.return_to_pause = false;
        s.query_focus = false;
        s.drag = None;
        s.bar_grab = None;
            s.pan_init = false;
        d
    });
    let Some(dest) = dest else {
        return;
    };
    match dest {
        close::Destination::Pause => {
            game.screen = crate::game::Screen::Paused;
        }
        close::Destination::Playing => {
            game.screen = crate::game::Screen::Playing;
            game.capture(true);
        }
    }
}

pub fn close_if_asked(game: &mut crate::game::Game<'_>) {
    if revise(|s| s.claim && s.want_close) {
        close(game);
    }
}

pub fn release_stale_claim(game: &crate::game::Game<'_>) {
    let is_adv = matches!(game.screen, crate::game::Screen::Advancements);
    revise(|s| {
        if close::should_drop_claim(s.claim, is_adv) {
            s.claim = false;
            s.want_close = false;
            s.return_to_pause = false;
            s.query_focus = false;
            s.drag = None;
            s.bar_grab = None;
            s.pan_init = false;
        }
    });
}

pub fn ours_open(game: &crate::game::Game<'_>) -> bool {
    claimed()
        && matches!(game.screen, crate::game::Screen::Advancements)
}

pub fn intercept_pause(
    game: &crate::game::Game<'_>,
    screen: &mod_api::ScreenView<'_>,
) -> bool {
    if screen.id != "paused" {
        return false;
    }
    let extra = extra_pause_rows(game);
    let rect = rules::listing::pause_advancements_rect(screen.rect[2], screen.rect[3], extra);
    let p = (screen.mouse[0], screen.mouse[1]);
    if p.0 >= rect[0] && p.0 < rect[0] + rect[2] && p.1 >= rect[1] && p.1 < rect[1] + rect[3] {
        revise(|s| {
            s.claim = true;
            s.return_to_pause = true;
            s.want_close = false;
        });
        // Let vanilla open the Advancements screen; we replace its draw.
        return false;
    }
    false
}

fn has_pause_additions(game: &crate::game::Game<'_>) -> bool {
    game.remote.as_ref().is_some_and(|remote| {
        remote
            .registries
            .tag("minecraft:dialog", "minecraft:pause_screen_additions")
            .is_some_and(|tag| !tag.is_empty())
    })
}

fn extra_pause_rows(game: &crate::game::Game<'_>) -> u8 {
    let row3 = !game.server_links().is_empty() || has_pause_additions(game);
    // Full-width rows other code registers above Options (Sunrise 9425651 dropped the
    // split-screen row; registered pause pages are the only extra rows left).
    let added = sunrise_hooks::pages::engine::all()
        .iter()
        .filter(|page| page.place() == sunrise_hooks::pages::Place::Pause)
        .count();
    u8::from(row3).saturating_add(u8::try_from(added).unwrap_or(u8::MAX))
}

pub fn on_advancements_key(game: &crate::game::Game<'_>, key: &str, pressed: bool) {
    if !pressed {
        return;
    }
    if key != "key.keyboard.l" && key != "key.advancements" {
        return;
    }
    if matches!(game.screen, crate::game::Screen::Playing) {
        revise(|s| {
            s.claim = true;
            s.return_to_pause = false;
            s.want_close = false;
        });
    }
}

pub fn toggle_panel(game: &mut crate::game::Game<'_>) {
    ensure_ready(game);
    let action = revise(|s| close::toggle_from_claimed(s.claim));
    match action {
        close::Toggle::Close => {
            revise(|s| s.want_close = true);
            close_if_asked(game);
        }
        close::Toggle::Open => {
            let from_pause = matches!(game.screen, crate::game::Screen::Paused);
            revise(|s| {
                s.claim = true;
                s.return_to_pause = from_pause;
                s.want_close = false;
            });
            game.open_advancements(from_pause);
        }
    }
}

pub fn on_frame(game: &mut crate::game::Game<'_>, _elapsed: std::time::Duration) {
    ensure_ready(game);
    take_over(game);
    close_if_asked(game);
    release_stale_claim(game);
    tick_plaques(game);
}

pub fn on_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) {
    let rest = command
        .strip_prefix("/advplus")
        .or_else(|| command.strip_prefix("advplus"))
        .map(str::trim);
    let Some(rest) = rest else {
        return;
    };
    *ran = true;
    ensure_ready(game);
    let mut parts = rest.split_whitespace();
    let verb = parts.next().unwrap_or("help");
    match verb {
        "panel" | "" => toggle_panel(game),
        "help" => game.native_chat("Advancements+ commands: /advplus panel | help"),
        other => game.native_chat(&format!("Unknown /advplus subcommand '{other}'. Try /advplus help.")),
    }
}

pub fn enqueue_plaque(game: &crate::game::Game<'_>, kind: &str, id: &str) -> bool {
    ensure_ready(game);
    let prefs = revise(|s| s.prefs.clone());
    if !prefs.plaques {
        return false;
    }
    if kind == "recipe" {
        return !prefs.show_recipe;
    }
    if kind != "advancement" {
        return false;
    }
    if rules::is_recipe_id(id) && !prefs.show_recipe {
        return true;
    }
    let tree = &game.adv.tree;
    let Some(a) = tree.get(id) else {
        return false;
    };
    let Some(d) = &a.display else {
        return rules::is_recipe_id(id) && !prefs.show_recipe;
    };
    let frame = match d.frame {
        Frame::Goal => FrameKind::Goal,
        Frame::Challenge => FrameKind::Challenge,
        Frame::Task => FrameKind::Task,
    };
    if !prefs.shows_frame(frame) {
        return true;
    }
    let title = {
        let t = resolve_component(&d.title);
        if t.is_empty() {
            rules::text::label_of(id)
        } else {
            t
        }
    };
    revise(|s| {
        s.plaques.push(Plaque {
            id: id.to_string(),
            title,
            icon: d.icon.item.to_string(),
            frame,
            started: Instant::now(),
            sound_in: false,
            sound_out: false,
        });
    });
    true
}

fn play_toast(game: &crate::game::Game<'_>, event: &str) {
    // Sunrise's official client-mod path: queue a UI sound for the next audio
    // update. The earlier port kept `game.audio.play`, which is not the mod
    // sound API (`play_local_sound` / `ModSounds::queue`). A windowless Game
    // (no output) would also swallow the one-shot flags if we marked the sound
    // played there.
    if !game.native_window() {
        return;
    }
    let _ = game.play_local_sound(&mod_api::SoundPlay::ui(event), 0);
}

fn tick_plaques(game: &mut crate::game::Game<'_>) {
    let duration = revise(|s| s.prefs.duration_ms());
    let fade = rules::plaques::FADE_MS;
    let mut play_in = false;
    let mut play_out = false;
    let mut play_challenge = false;
    // Only consume the one-shot flags on the windowed client, so a silent
    // Game that shares this thread-local session cannot prevent the play.
    let can_play = game.native_window();
    revise(|s| {
        s.plaques.retain_mut(|p| {
            let elapsed = Instant::now().saturating_duration_since(p.started).as_secs_f32() * 1000.0;
            let phase = rules::plaques::phase(elapsed, duration, fade);
            if can_play && matches!(phase, rules::plaques::Phase::In(_)) && !p.sound_in {
                p.sound_in = true;
                play_in = true;
                if p.frame == FrameKind::Challenge {
                    play_challenge = true;
                }
            }
            if can_play && matches!(phase, rules::plaques::Phase::Out(_)) && !p.sound_out {
                p.sound_out = true;
                play_out = true;
            }
            !matches!(phase, rules::plaques::Phase::Done)
        });
        if s.plaques.len() > 4 {
            s.plaques.drain(0..s.plaques.len() - 4);
        }
    });
    if play_in {
        play_toast(game, "minecraft:ui.toast.in");
    }
    if play_challenge {
        play_toast(game, "minecraft:ui.toast.challenge_complete");
    }
    if play_out {
        play_toast(game, "minecraft:ui.toast.out");
    }
}

pub fn settings_buttons() -> Vec<String> {
    vec!["Open Advancements+".into()]
}

pub fn settings_click(game: &mut crate::game::Game<'_>, button: usize) {
    if button == 0 {
        toggle_panel(game);
    }
}
