// SPDX-License-Identifier: MIT
// Advancements+ tabbed card: tree or list, search, filters, and plaques.

use super::advplus as hook;
use crate::game::Game;
use mod_advplus::logic as rules;
use sunrise_client::hud::HudBatch;
use sunrise_client::items::ItemStack;
use mod_api::ScreenView;
use rules::listing::{self, BAR_GAP, BAR_W, ROW};
use rules::{Entry, Filter, FrameKind, PlaqueSide, ViewMode};

const CARD: u32 = 0xFF0F1118;
const TAB: u32 = 0xFF161A22;
const CHIP: u32 = 0xFF1E2330;
const TEXT: u32 = 0xFFE8DCC8;
const DIM: u32 = 0xFF8B93A7;
const SEARCH: u32 = 0xFF0A0C11;
const TASK: u32 = 0xFFC4A574;
const GOAL: u32 = 0xFF6BA3C9;
const CHALLENGE: u32 = 0xFF9A6BC8;
const NODE_BG: u32 = 0xFF161A22;
const NODE_DONE: u32 = 0xFF242A36;
const LINE: u32 = 0xFF3A4252;
const TIP: u32 = 0xFF0A0C11;

/// The live accent (`accent` setting, `0xRRGGBB`; Sunrise orange by default), set each paint.
static ACCENT_RGB: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0xF9801D);
/// Takes the accent colour the card draws with (`0xRRGGBB`).
pub fn set_accent(rgb: u32) {
    ACCENT_RGB.store(rgb & 0xFF_FFFF, std::sync::atomic::Ordering::Relaxed);
}
/// The accent, opaque: titles, values, the 2 px top edge.
fn accent() -> u32 {
    0xFF00_0000 | ACCENT_RGB.load(std::sync::atomic::Ordering::Relaxed)
}
/// The card's top edge: the accent.
fn edge() -> u32 {
    accent()
}
/// The selected tab / On chip: the accent darkened into the card.
fn tab_on() -> u32 {
    let a = accent();
    let ch = |s: u32| {
        let x = ((a >> s) & 0xFF) as f32;
        let b = ((CARD >> s) & 0xFF) as f32;
        ((x * 0.42 + b * 0.58).round() as u32).min(255) << s
    };
    0xFF00_0000 | ch(16) | ch(8) | ch(0)
}

pub fn ours(game: &Game<'_>, screen: &ScreenView<'_>) -> bool {
    hook::claimed() && screen.id == "advancements"
}

fn hit(p: (f32, f32), r: [f32; 4]) -> bool {
    p.0 >= r[0] && p.0 < r[0] + r[2] && p.1 >= r[1] && p.1 < r[1] + r[3]
}

fn clip_to(batch: &mut HudBatch, start: usize, r: [f32; 4]) {
    if start >= batch.vertices.len() {
        return;
    }
    batch.clip_since(start, r[0], r[1], r[0] + r[2].max(0.0), r[1] + r[3].max(0.0));
}

fn frame(screen: &ScreenView<'_>) -> rules::layout::Layout {
    rules::layout::compute(screen.rect[2], screen.rect[3])
}

fn fill(game: &Game<'_>, batch: &mut HudBatch, r: [f32; 4], c: u32) {
    game.native_fill(batch, r, c);
}

fn ink(game: &Game<'_>, batch: &mut HudBatch, text: &str, pos: [f32; 2], c: u32) {
    game.native_text(batch, text, pos, c, true);
}

fn ink_center(game: &Game<'_>, batch: &mut HudBatch, text: &str, cx: f32, y: f32, c: u32) {
    let w = HudBatch::text_width(&game.gui.atlas, text);
    ink(game, batch, text, [cx - w * 0.5, y], c);
}

fn chip(game: &Game<'_>, batch: &mut HudBatch, r: [f32; 4], label: &str, on: bool, hover: bool, color: u32) {
    let bg = if on {
        color
    } else if hover {
        0xFF2A3142
    } else {
        CHIP
    };
    fill(game, batch, r, bg);
    let at = batch.vertices.len();
    ink_center(game, batch, label, r[0] + r[2] * 0.5, r[1] + 3.0, TEXT);
    clip_to(batch, at, r);
}

fn frame_color(kind: FrameKind) -> u32 {
    match kind {
        FrameKind::Task => TASK,
        FrameKind::Goal => GOAL,
        FrameKind::Challenge => CHALLENGE,
    }
}

fn bar_track(list: [f32; 4]) -> [f32; 4] {
    [list[0] + list[2] + BAR_GAP, list[1], BAR_W, list[3]]
}

pub fn paint(game: &Game<'_>, batch: &mut HudBatch, screen: &ScreenView<'_>) {
    set_accent(crate::game::advplus_card::accent(game));
    if !ours(game, screen) {
        return;
    }
    hook::ensure_ready(game);
    hook::sync(game);
    let ui = frame(screen);
    fill(game, batch, [ui.x, ui.y, ui.w, ui.h], CARD);
    fill(game, batch, [ui.x, ui.y, ui.w, 2.0], edge());
    let title_clip = [ui.x + 12.0, ui.y, (ui.done[0] - 4.0 - (ui.x + 12.0)).max(0.0), 26.0];
    let title_at = batch.vertices.len();
    ink(game, batch, "Advancements+", [ui.x + 12.0, ui.y + 8.0], accent());
    clip_to(batch, title_at, title_clip);
    let p = (screen.mouse[0], screen.mouse[1]);
    chip(game, batch, ui.done, "Done", hit(p, ui.done), hit(p, ui.done), edge());
    paint_tabs(game, batch, &ui, p);
    paint_body(game, batch, &ui, p);
}

fn tab_list() -> (Vec<Entry>, bool, usize) {
    hook::revise(|s| (rules::filter::roots(&s.entries).into_iter().cloned().collect(), s.settings_tab, s.tab))
}

fn paint_tabs(game: &Game<'_>, batch: &mut HudBatch, ui: &rules::layout::Layout, p: (f32, f32)) {
    let (roots, settings, tab) = tab_list();
    let n = roots.len() + 1;
    for (i, root) in roots.iter().enumerate() {
        let r = ui.tab_rect(i, n);
        let on = !settings && tab == i;
        fill(game, batch, r, if on { tab_on() } else { TAB });
        let (done, total) = hook::revise(|s| rules::progress::tab_counts(&s.entries, &root.id));
        let cap = fit_tab_text(game, &root.title, done, total, (r[2] - 6.0).max(8.0));
        let at = batch.vertices.len();
        ink_center(
            game,
            batch,
            &cap,
            r[0] + r[2] * 0.5,
            r[1] + 4.0,
            if on { TEXT } else { DIM },
        );
        clip_to(batch, at, r);
        let _ = p;
    }
    let r = ui.tab_rect(roots.len(), n);
    fill(game, batch, r, if settings { tab_on() } else { TAB });
    let cap = fit_plain_text(game, "Settings", "Set", "S", (r[2] - 6.0).max(8.0));
    let at = batch.vertices.len();
    ink_center(game, batch, &cap, r[0] + r[2] * 0.5, r[1] + 4.0, if settings { TEXT } else { DIM });
    clip_to(batch, at, r);
}

fn fit_tab_text(game: &Game<'_>, title: &str, done: usize, total: usize, max_w: f32) -> String {
    let suffix = format!(" {done}/{total}");
    let counts = format!("{done}/{total}");
    if HudBatch::text_width(&game.gui.atlas, &suffix) > max_w {
        return counts;
    }
    let full = format!("{title}{suffix}");
    if HudBatch::text_width(&game.gui.atlas, &full) <= max_w {
        return full;
    }
    let n = title.chars().count();
    let mut best = counts;
    for take in 1..=n {
        let cut: String = title.chars().take(take).collect();
        let s = format!("{cut}…{suffix}");
        if HudBatch::text_width(&game.gui.atlas, &s) <= max_w {
            best = s;
        } else {
            break;
        }
    }
    best
}

fn fit_plain_text(game: &Game<'_>, full: &str, short: &str, letter: &str, max_w: f32) -> String {
    for s in [full, short, letter] {
        if HudBatch::text_width(&game.gui.atlas, s) <= max_w {
            return s.to_string();
        }
    }
    letter.to_string()
}

fn paint_body(game: &Game<'_>, batch: &mut HudBatch, ui: &rules::layout::Layout, p: (f32, f32)) {
    let (query, focus, filter, mode, settings) = hook::revise(|s| {
        (s.query.clone(), s.query_focus, s.prefs.filter, s.prefs.view, s.settings_tab)
    });
    fill(game, batch, ui.search, SEARCH);
    let shown = if query.is_empty() {
        if focus {
            "_".to_string()
        } else if ui.search[2] < 120.0 {
            "Search...".to_string()
        } else {
            "Search advancements...".to_string()
        }
    } else if focus {
        format!("{query}_")
    } else {
        query.clone()
    };
    let search_at = batch.vertices.len();
    ink(
        game,
        batch,
        &shown,
        [ui.search[0] + 4.0, ui.search[1] + 3.0],
        if query.is_empty() && !focus { DIM } else { TEXT },
    );
    clip_to(batch, search_at, ui.search);
    let fl = if ui.compact_tools { filter.label() } else { filter.label() };
    chip(game, batch, ui.filter, fl, filter != Filter::All, hit(p, ui.filter), tab_on());
    chip(game, batch, ui.mode, mode.label(), mode == ViewMode::List, hit(p, ui.mode), tab_on());

    if settings {
        paint_settings(game, batch, ui, p);
        return;
    }
    match mode {
        ViewMode::List => paint_list(game, batch, ui, p),
        ViewMode::Tree => paint_tree(game, batch, ui, p),
    }
}

fn paint_settings(game: &Game<'_>, batch: &mut HudBatch, ui: &rules::layout::Layout, p: (f32, f32)) {
    let prefs = hook::revise(|s| s.prefs.clone());
    let rows = [
        ("Plaques", if prefs.plaques { "On" } else { "Off" }),
        ("Duration", ""),
        ("Tasks", if prefs.show_task { "Show" } else { "Hide" }),
        ("Goals", if prefs.show_goal { "Show" } else { "Hide" }),
        ("Challenges", if prefs.show_challenge { "Show" } else { "Hide" }),
        ("Recipes", if prefs.show_recipe { "Show" } else { "Hide" }),
        ("Position", prefs.position.label()),
        ("Default view", prefs.view.label()),
    ];
    let at = batch.vertices.len();
    for (i, (name, value)) in rows.iter().enumerate() {
        let y = ui.view[1] + 6.0 + i as f32 * ROW;
        let r = [ui.view[0], y, ui.view[2], ROW - 2.0];
        fill(game, batch, r, if hit(p, r) { 0xFF2A3142 } else { 0xFF1E2330 });
        ink(game, batch, name, [r[0] + 8.0, r[1] + 5.0], TEXT);
        let val = if *name == "Duration" {
            format!("{} s", prefs.duration_s)
        } else {
            (*value).to_string()
        };
        let vw = HudBatch::text_width(&game.gui.atlas, &val);
        ink(game, batch, &val, [r[0] + r[2] - vw - 8.0, r[1] + 5.0], accent());
    }
    clip_to(batch, at, ui.view);
}

fn current_shown() -> Vec<Entry> {
    hook::revise(|s| {
        let roots = rules::filter::roots(&s.entries);
        let Some(root) = roots.get(s.tab) else {
            return Vec::new();
        };
        rules::filter::shown(&s.entries, &root.id, s.prefs.filter, &s.query)
            .into_iter()
            .cloned()
            .collect()
    })
}

fn paint_list(game: &Game<'_>, batch: &mut HudBatch, ui: &rules::layout::Layout, p: (f32, f32)) {
    let rows = current_shown();
    let total = rows.len();
    let list = ui.view;
    let scroll = hook::revise(|s| listing::clamp_scroll(s.scroll, total, list[3], ROW));
    hook::revise(|s| s.scroll = scroll);
    let (start, end) = listing::shown_rows(scroll, total, list[3], ROW);
    let list_at = batch.vertices.len();
    for i in start..end {
        let Some(vis) = listing::visible_row_rect(list, i, scroll, ROW) else {
            continue;
        };
        let y = list[1] + listing::row_top(i, scroll, ROW);
        let r = [list[0], y, list[2], ROW - 2.0];
        let e = &rows[i];
        fill(game, batch, r, if hit(p, vis) { 0xFF2A3142 } else { 0xFF1E2330 });
        fill(game, batch, [r[0], r[1], 3.0, r[3]], frame_color(e.frame));
        if !e.icon.is_empty() && game.res.items().get(&e.icon).is_some() {
            let stack = ItemStack::new(&e.icon, 1);
            game.native_gui_draw_item_with(batch, &stack, [r[0] + 6.0, r[1] + 3.0], None);
        }
        let title = if e.visible || e.done() {
            e.title.as_str()
        } else {
            "???"
        };
        ink(
            game,
            batch,
            title,
            [r[0] + 28.0, r[1] + 5.0],
            if e.done() { accent() } else if e.visible { TEXT } else { DIM },
        );
        let (d, t) = e.counts();
        let val = format!("{d}/{t}");
        let vw = HudBatch::text_width(&game.gui.atlas, &val);
        ink(game, batch, &val, [r[0] + r[2] - vw - 8.0, r[1] + 5.0], DIM);
    }
    if total == 0 {
        ink(game, batch, "No advancements to show.", [list[0] + 8.0, list[1] + 12.0], DIM);
    }
    clip_to(batch, list_at, list);
    paint_bar(game, batch, list, scroll, total, ROW, bar_hover(p, list, scroll, total, ROW));
    if let Some(i) = (start..end).find(|&i| {
        listing::visible_row_rect(list, i, scroll, ROW).is_some_and(|r| hit(p, r))
    }) {
        paint_tooltip(game, batch, ui, p, &rows[i]);
    }
}


fn tab_rows(entries: &[Entry], root: &str) -> Vec<Entry> {
    entries
        .iter()
        .filter(|e| !e.recipe && rules::filter::in_tab(entries, e, root))
        .cloned()
        .collect()
}

fn ensure_tree_layout(ui: &rules::layout::Layout) -> listing::Orient {
    let root_id = hook::current_root();
    hook::revise(|s| {
        let Some(id) = root_id.as_deref() else {
            return listing::Orient::Ltr;
        };
        let orient = listing::layout_tab(&mut s.entries, id, ui.view);
        s.orient = orient;
        if !s.pan_init {
            let rows = tab_rows(&s.entries, id);
            let root = rows.iter().find(|e| e.id == id).cloned();
            let (pan, zoom) = listing::opening_camera(&rows, root.as_ref(), ui.view, orient);
            s.pan = pan;
            s.zoom = zoom;
            s.pan_init = true;
        }
        orient
    })
}

fn paint_tree(game: &Game<'_>, batch: &mut HudBatch, ui: &rules::layout::Layout, p: (f32, f32)) {
    let orient = ensure_tree_layout(ui);
    let rows = current_shown();
    let (pan, zoom) = hook::revise(|s| {
        let pan = listing::clamp_pan(s.pan, &rows, ui.view, s.zoom);
        s.pan = pan;
        (pan, s.zoom)
    });
    let view = ui.view;
    fill(game, batch, view, 0xFF0A0C11);
    let at = batch.vertices.len();
    // Connectors first, under the icons, and only for a visible parent→child pair.
    paint_connectors(game, batch, &rows, view, pan, zoom, orient);
    let mut hover = None;
    for e in &rows {
        let r = listing::node_rect(view, pan, zoom, e.x, e.y);
        let vis = listing::visible_node_rect(view, pan, zoom, e.x, e.y);
        let over = vis.is_some_and(|v| hit(p, v));
        if over {
            hover = Some(e.clone());
        }
        paint_node_frame(game, batch, r, e.frame, e.done(), e.visible);
        if e.visible || e.done() {
            if !e.icon.is_empty() && game.res.items().get(&e.icon).is_some() {
                let stack = ItemStack::new(&e.icon, 1);
                let z = listing::snap_zoom(zoom);
                let cx = r[0] + r[2] * 0.5;
                let cy = r[1] + r[3] * 0.5;
                let start = batch.vertices.len();
                game.native_gui_draw_item_with(
                    batch,
                    &stack,
                    [cx - listing::ICON * 0.5, cy - listing::ICON * 0.5], None);
                if (z - 1.0).abs() > 0.001 {
                    batch.scale_since(start, cx, cy, z, z);
                }
            }
        } else {
            ink_center(game, batch, "?", r[0] + r[2] * 0.5, r[1] + r[3] * 0.5 - 5.0, DIM);
        }
    }
    clip_to(batch, at, view);
    hook::revise(|s| s.hover = hover.as_ref().map(|e| e.id.clone()));
    if let Some(e) = hover {
        paint_tooltip(game, batch, ui, p, &e);
    }
}

fn paint_connectors(
    game: &Game<'_>,
    batch: &mut HudBatch,
    rows: &[Entry],
    view: [f32; 4],
    pan: [f32; 2],
    zoom: f32,
    orient: listing::Orient,
) {
    if orient == listing::Orient::Ltr {
        for (pid, cid) in listing::edges(rows) {
            let Some(parent) = rows.iter().find(|o| o.id == pid) else {
                continue;
            };
            let Some(child) = rows.iter().find(|o| o.id == cid) else {
                continue;
            };
            let a = listing::node_rect(view, pan, zoom, parent.x, parent.y);
            let b = listing::node_rect(view, pan, zoom, child.x, child.y);
            let color = if child.done() && parent.done() { accent() } else { LINE };
            draw_elbow(game, batch, a, b, color);
        }
        return;
    }
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<String, Vec<&Entry>> = BTreeMap::new();
    for (pid, cid) in listing::edges(rows) {
        let Some(child) = rows.iter().find(|o| o.id == cid) else {
            continue;
        };
        groups.entry(pid).or_default().push(child);
    }
    for (pid, kids) in groups {
        let Some(parent) = rows.iter().find(|o| o.id == pid) else {
            continue;
        };
        let a = listing::node_rect(view, pan, zoom, parent.x, parent.y);
        let rects: Vec<[f32; 4]> = kids
            .iter()
            .map(|c| listing::node_rect(view, pan, zoom, c.x, c.y))
            .collect();
        let all_done = parent.done() && kids.iter().all(|c| c.done());
        let color = if all_done { accent() } else { LINE };
        for seg in listing::ttb_segments(a, &rects) {
            draw_seg(game, batch, seg[0], seg[1], color);
        }
    }
}

/// A 2 px connector from `a` to `b` on whole pixels; its ends are squared off by half the
/// thickness so the corners of an elbow meet without notches.
fn draw_seg(game: &Game<'_>, batch: &mut HudBatch, a: [f32; 2], b: [f32; 2], color: u32) {
    let r = listing::seg_rect(a, b);
    fill(game, batch, r, color);
}

fn draw_elbow(game: &Game<'_>, batch: &mut HudBatch, parent: [f32; 4], child: [f32; 4], color: u32) {
    let pts = listing::elbow(parent, child);
    let end = listing::elbow_child_end(child);
    draw_seg(game, batch, pts[0], pts[1], color);
    draw_seg(game, batch, pts[1], pts[2], color);
    draw_seg(game, batch, pts[2], end, color);
}

fn mute(c: u32, done: bool) -> u32 {
    if done {
        return c;
    }
    let a = c >> 24;
    let r = ((c >> 16) & 0xff) * 5 / 8;
    let g = ((c >> 8) & 0xff) * 5 / 8;
    let b = (c & 0xff) * 5 / 8;
    (a << 24) | (r << 16) | (g << 8) | b
}

fn stroke_rect(game: &Game<'_>, batch: &mut HudBatch, r: [f32; 4], t: f32, c: u32) {
    fill(game, batch, [r[0], r[1], r[2], t], c);
    fill(game, batch, [r[0], r[1] + r[3] - t, r[2], t], c);
    fill(game, batch, [r[0], r[1], t, r[3]], c);
    fill(game, batch, [r[0] + r[2] - t, r[1], t, r[3]], c);
}

fn paint_node_frame(
    game: &Game<'_>,
    batch: &mut HudBatch,
    r: [f32; 4],
    kind: FrameKind,
    done: bool,
    _visible: bool,
) {
    let fill_c = if done { NODE_DONE } else { NODE_BG };
    let accent = mute(frame_color(kind), done);
    let t = if done { 2.5 } else { 2.0 };
    match kind {
        FrameKind::Task => {
            fill(game, batch, r, fill_c);
            stroke_rect(game, batch, r, t, accent);
        }
        FrameKind::Goal => {
            // Octagon: square with clipped corners.
            fill(game, batch, r, fill_c);
            let cut = (r[2].min(r[3]) * 0.22).clamp(3.0, 6.0);
            let bg = 0xFF0A0C11;
            fill(game, batch, [r[0], r[1], cut, cut], bg);
            fill(game, batch, [r[0] + r[2] - cut, r[1], cut, cut], bg);
            fill(game, batch, [r[0], r[1] + r[3] - cut, cut, cut], bg);
            fill(game, batch, [r[0] + r[2] - cut, r[1] + r[3] - cut, cut, cut], bg);
            // Octagon outline (12 segments approximated with bars).
            let x0 = r[0] + cut;
            let x1 = r[0] + r[2] - cut;
            let y0 = r[1] + cut;
            let y1 = r[1] + r[3] - cut;
            fill(game, batch, [x0, r[1], (x1 - x0).max(1.0), t], accent);
            fill(game, batch, [x0, r[1] + r[3] - t, (x1 - x0).max(1.0), t], accent);
            fill(game, batch, [r[0], y0, t, (y1 - y0).max(1.0)], accent);
            fill(game, batch, [r[0] + r[2] - t, y0, t, (y1 - y0).max(1.0)], accent);
            // Corner diagonals as thick short bars.
            fill(game, batch, [r[0], y0 - t, cut, t], accent);
            fill(game, batch, [x0 - t, r[1], t, cut], accent);
            fill(game, batch, [x1, r[1], t, cut], accent);
            fill(game, batch, [x1, y0 - t, cut, t], accent);
            fill(game, batch, [r[0], y1, cut, t], accent);
            fill(game, batch, [x0 - t, y1, t, cut], accent);
            fill(game, batch, [x1, y1, t, cut], accent);
            fill(game, batch, [x1, y1, cut, t], accent);
        }
        FrameKind::Challenge => {
            // Double square frame with mid-edge pips — a heavier, more ornate look.
            let grow = 2.0;
            let outer = [r[0] - grow, r[1] - grow, r[2] + grow * 2.0, r[3] + grow * 2.0];
            fill(game, batch, r, fill_c);
            stroke_rect(game, batch, r, t, accent);
            stroke_rect(game, batch, outer, 1.5, accent);
            let pip = 3.0;
            let mx = r[0] + r[2] * 0.5;
            let my = r[1] + r[3] * 0.5;
            fill(game, batch, [mx - pip * 0.5, outer[1] - 1.0, pip, pip], accent);
            fill(game, batch, [mx - pip * 0.5, outer[1] + outer[3] - pip + 1.0, pip, pip], accent);
            fill(game, batch, [outer[0] - 1.0, my - pip * 0.5, pip, pip], accent);
            fill(game, batch, [outer[0] + outer[2] - pip + 1.0, my - pip * 0.5, pip, pip], accent);
        }
    }
}

fn paint_tooltip(game: &Game<'_>, batch: &mut HudBatch, ui: &rules::layout::Layout, p: (f32, f32), e: &Entry) {
    let (done, rest) = rules::progress::criterion_lists(&e.criteria, &e.obtained);
    let (d, t) = e.counts();
    let mut lines: Vec<(String, u32)> = Vec::new();
    let title = if e.visible || e.done() {
        e.title.clone()
    } else {
        "Locked advancement".into()
    };
    lines.push((title, accent()));
    lines.push((e.frame.label().to_string(), frame_color(e.frame)));
    if e.visible || e.done() {
        if !e.description.is_empty() {
            lines.push((e.description.clone(), TEXT));
        }
        lines.push((format!("Progress {d}/{t}"), DIM));
        if !done.is_empty() {
            let shown: Vec<_> = done.iter().take(6).map(|c| format!("+ {}", pretty_crit(c))).collect();
            for s in shown {
                lines.push((s, 0xFF8FCB8F));
            }
            if done.len() > 6 {
                lines.push((format!("+ {} more", done.len() - 6), DIM));
            }
        }
        if !rest.is_empty() {
            let shown: Vec<_> = rest.iter().take(6).map(|c| format!("· {}", pretty_crit(c))).collect();
            for s in shown {
                lines.push((s, DIM));
            }
            if rest.len() > 6 {
                lines.push((format!("· {} more", rest.len() - 6), DIM));
            }
        }
    }
    let mut w = 80.0f32;
    for (s, _) in &lines {
        w = w.max(HudBatch::text_width(&game.gui.atlas, s) + 16.0);
    }
    w = w.min((ui.w - 20.0).max(80.0));
    let h = 8.0 + lines.len() as f32 * 12.0;
    let mut x = p.0 + 14.0;
    let mut y = p.1 + 14.0;
    if x + w > ui.x + ui.w - 6.0 {
        x = (ui.x + ui.w - 6.0 - w).max(ui.x + 6.0);
    }
    if y + h > ui.y + ui.h - 6.0 {
        y = (p.1 - h - 8.0).max(ui.y + 6.0);
    }
    let r = [x, y, w, h];
    fill(game, batch, r, TIP);
    fill(game, batch, [r[0], r[1], r[2], 2.0], frame_color(e.frame));
    let at = batch.vertices.len();
    for (i, (s, c)) in lines.iter().enumerate() {
        ink(game, batch, s, [r[0] + 8.0, r[1] + 5.0 + i as f32 * 12.0], *c);
    }
    clip_to(batch, at, r);
}

fn pretty_crit(name: &str) -> String {
    rules::text::humanize(name)
}

fn paint_bar(game: &Game<'_>, batch: &mut HudBatch, list: [f32; 4], scroll: f32, total: usize, row: f32, hover: bool) {
    let track = bar_track(list);
    if track[3] <= 0.0 {
        return;
    }
    fill(game, batch, track, 0xFF0A0C11);
    let Some(thumb) = listing::thumb(scroll, total, list[3], track[3], row) else {
        return;
    };
    fill(
        game,
        batch,
        [track[0], track[1] + thumb.y, track[2], thumb.h],
        if hover { accent() } else { tab_on() },
    );
}

fn bar_hover(p: (f32, f32), list: [f32; 4], scroll: f32, total: usize, row: f32) -> bool {
    let track = bar_track(list);
    let Some(thumb) = listing::thumb(scroll, total, list[3], track[3], row) else {
        return hit(p, track);
    };
    hit(p, [track[0], track[1] + thumb.y, track[2], thumb.h])
}

fn click_bar(p: (f32, f32), list: [f32; 4], total: usize, scroll: f32, row: f32) -> bool {
    let track = bar_track(list);
    if !hit(p, track) {
        return false;
    }
    let local = p.1 - track[1];
    hook::revise(|s| {
        let view = list[3];
        if let Some(thumb) = listing::thumb(scroll, total, view, track[3], row) {
            if local >= thumb.y && local < thumb.y + thumb.h {
                s.bar_grab = Some(local - thumb.y);
                return;
            }
        }
        s.scroll = listing::scroll_at_track(local, total, view, track[3], row);
        if let Some(thumb) = listing::thumb(s.scroll, total, view, track[3], row) {
            s.bar_grab = Some((local - thumb.y).clamp(0.0, thumb.h));
        }
    });
    true
}

pub fn click(game: &Game<'_>, screen: &ScreenView<'_>, button: i32) -> bool {
    if hook::intercept_pause(game, screen) {
        return true;
    }
    if !ours(game, screen) || button != 0 {
        return false;
    }
    let ui = frame(screen);
    let p = (screen.mouse[0], screen.mouse[1]);
    if hit(p, ui.done) {
        hook::revise(|s| s.want_close = true);
        return true;
    }
    let (roots, _, _) = tab_list();
    let n = roots.len() + 1;
    for (i, _) in roots.iter().enumerate() {
        if hit(p, ui.tab_rect(i, n)) {
            hook::revise(|s| {
                s.tab = i;
                s.settings_tab = false;
                s.scroll = 0.0;
                s.pan = [0.0, 0.0];
                s.pan_init = false;
                s.query_focus = false;
            });
            return true;
        }
    }
    if hit(p, ui.tab_rect(roots.len(), n)) {
        hook::revise(|s| {
            s.settings_tab = true;
            s.query_focus = false;
        });
        return true;
    }
    if hit(p, ui.search) {
        hook::revise(|s| s.query_focus = true);
        return true;
    }
    hook::revise(|s| s.query_focus = false);
    if hit(p, ui.filter) {
        hook::revise(|s| s.prefs.filter = s.prefs.filter.next());
        hook::write_prefs(game);
        return true;
    }
    if hit(p, ui.mode) {
        hook::revise(|s| s.prefs.view = s.prefs.view.next());
        hook::write_prefs(game);
        return true;
    }
    if hook::revise(|s| s.settings_tab) {
        click_settings(game, &ui, p);
        return true;
    }
    let mode = hook::revise(|s| s.prefs.view);
    if mode == ViewMode::List {
        let rows = current_shown();
        let list = ui.view;
        let scroll = hook::revise(|s| listing::clamp_scroll(s.scroll, rows.len(), list[3], ROW));
        if click_bar(p, list, rows.len(), scroll, ROW) {
            return true;
        }
    } else if hit(p, ui.view) {
        hook::revise(|s| s.drag = Some((p.0, p.1, s.pan)));
    }
    true
}

fn click_settings(game: &Game<'_>, ui: &rules::layout::Layout, p: (f32, f32)) {
    for i in 0..8 {
        let y = ui.view[1] + 6.0 + i as f32 * ROW;
        let r = [ui.view[0], y, ui.view[2], ROW - 2.0];
        if !hit(p, r) {
            continue;
        }
        hook::revise(|s| match i {
            0 => s.prefs.plaques = !s.prefs.plaques,
            1 => s.prefs.cycle_duration(),
            2 => s.prefs.show_task = !s.prefs.show_task,
            3 => s.prefs.show_goal = !s.prefs.show_goal,
            4 => s.prefs.show_challenge = !s.prefs.show_challenge,
            5 => s.prefs.show_recipe = !s.prefs.show_recipe,
            6 => s.prefs.position = s.prefs.position.next(),
            7 => s.prefs.view = s.prefs.view.next(),
            _ => {}
        });
        hook::write_prefs(game);
        break;
    }
}

pub fn wheel(game: &Game<'_>, screen: &ScreenView<'_>, scroll: [f64; 2]) -> bool {
    if !ours(game, screen) {
        return false;
    }
    let dy = if scroll[1].abs() > scroll[0].abs() {
        scroll[1]
    } else {
        scroll[0]
    };
    if dy == 0.0 {
        return true;
    }
    let ui = frame(screen);
    let mode = hook::revise(|s| s.prefs.view);
    let settings = hook::revise(|s| s.settings_tab);
    if settings {
        return true;
    }
    let p = (screen.mouse[0], screen.mouse[1]);
    if mode == ViewMode::List {
        let rows = current_shown();
        hook::revise(|s| {
            s.scroll = listing::apply_wheel(s.scroll, dy as f32, rows.len(), ui.view[3], ROW);
        });
    } else if hit(p, ui.view) {
        let rows = current_shown();
        hook::revise(|s| {
            let old = listing::snap_zoom(s.zoom);
            let new = listing::zoom_step(old, dy as f32);
            if (new - old).abs() > 0.001 {
                s.pan = listing::zoom_pan(s.pan, old, new, ui.view, [p.0, p.1]);
                s.zoom = new;
                s.pan = listing::clamp_pan(s.pan, &rows, ui.view, s.zoom);
            }
        });
    }
    true
}

pub fn drag(game: &Game<'_>, screen: &ScreenView<'_>, button: i32) -> bool {
    if !ours(game, screen) || button != 0 {
        return false;
    }
    let ui = frame(screen);
    let p = (screen.mouse[0], screen.mouse[1]);
    let grab = hook::revise(|s| s.bar_grab);
    if let Some(off) = grab {
        let rows = current_shown();
        hook::revise(|s| {
            let track = bar_track(ui.view);
            s.scroll = listing::scroll_from_drag(p.1 - track[1], off, rows.len(), ui.view[3], track[3], ROW);
        });
        return true;
    }
    let drag = hook::revise(|s| s.drag);
    if let Some((x0, y0, pan0)) = drag {
        hook::revise(|s| {
            let z = listing::snap_zoom(s.zoom);
            s.pan = [pan0[0] - (p.0 - x0) / z, pan0[1] - (p.1 - y0) / z];
        });
        return true;
    }
    false
}

pub fn release(game: &Game<'_>, screen: &ScreenView<'_>, button: i32) -> bool {
    if !ours(game, screen) {
        return false;
    }
    hook::revise(|s| {
        s.bar_grab = None;
        s.drag = None;
    });
    let _ = (game, button);
    true
}

pub fn key(game: &Game<'_>, screen: &ScreenView<'_>, key: &str) -> bool {
    if !ours(game, screen) {
        return false;
    }
    if key == "key.keyboard.escape" || key == "key.keyboard.l" || key == "key.advancements" {
        hook::revise(|s| s.want_close = true);
        return true;
    }
    let focus = hook::revise(|s| s.query_focus);
    if !focus {
        return true;
    }
    if key == "key.keyboard.backspace" {
        hook::revise(|s| {
            s.query.pop();
            s.scroll = 0.0;
        });
        return true;
    }
    let _ = game;
    true
}

pub fn typed(game: &Game<'_>, screen: &ScreenView<'_>, c: char) -> bool {
    if !ours(game, screen) {
        return false;
    }
    let focus = hook::revise(|s| s.query_focus);
    if !focus {
        return true;
    }
    if !c.is_control() && c != '§' {
        hook::revise(|s| {
            if s.query.len() < 40 {
                s.query.push(c);
                s.scroll = 0.0;
            }
        });
    }
    let _ = game;
    true
}

pub fn paint_plaques(game: &Game<'_>, batch: &mut HudBatch) {
    hook::ensure_ready(game);
    set_accent(crate::game::advplus_card::accent(game));
    let (plaques, prefs) = hook::revise(|s| (s.plaques.clone(), s.prefs.clone()));
    if plaques.is_empty() {
        return;
    }
    let sw = batch.width;
    let sh = batch.height;
    let duration = prefs.duration_ms();
    // Show the newest plaque only (queue waits behind it).
    let Some(p) = plaques.first() else {
        return;
    };
    let elapsed = std::time::Instant::now()
        .saturating_duration_since(p.started)
        .as_secs_f32()
        * 1000.0;
    let phase = rules::plaques::phase(elapsed, duration, rules::plaques::FADE_MS);
    if matches!(phase, rules::plaques::Phase::Done) {
        return;
    }
    let r = rules::plaques::plaque_rect(sw, sh, prefs.position, phase);
    fill(game, batch, r, CARD);
    fill(game, batch, [r[0], r[1], r[2], 3.0], frame_color(p.frame));
    fill(game, batch, [r[0], r[1] + r[3] - 2.0, r[2], 2.0], edge());
    if !p.icon.is_empty() && game.res.items().get(&p.icon).is_some() {
        let stack = ItemStack::new(&p.icon, 1);
        game.native_gui_draw_item_with(batch, &stack, [r[0] + 8.0, r[1] + 12.0], None);
    }
    ink(game, batch, p.frame.label(), [r[0] + 32.0, r[1] + 6.0], frame_color(p.frame));
    let at = batch.vertices.len();
    ink(game, batch, &p.title, [r[0] + 32.0, r[1] + 20.0], TEXT);
    clip_to(batch, at, [r[0] + 30.0, r[1], r[2] - 36.0, r[3]]);
    let _ = PlaqueSide::Top;
}
