// SPDX-License-Identifier: MIT
// Advancements+ rules: progress, filters, layout, plaques, prefs, and close.

use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FrameKind {
    Task,
    Goal,
    Challenge,
}

impl FrameKind {
    pub fn as_name(self) -> &'static str {
        match self {
            FrameKind::Task => "task",
            FrameKind::Goal => "goal",
            FrameKind::Challenge => "challenge",
        }
    }

    pub fn from_name(s: &str) -> Self {
        match s {
            "goal" => FrameKind::Goal,
            "challenge" => FrameKind::Challenge,
            _ => FrameKind::Task,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            FrameKind::Task => "Task",
            FrameKind::Goal => "Goal",
            FrameKind::Challenge => "Challenge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    All,
    Done,
    Active,
    Locked,
}

impl Filter {
    pub fn next(self) -> Self {
        match self {
            Filter::All => Filter::Done,
            Filter::Done => Filter::Active,
            Filter::Active => Filter::Locked,
            Filter::Locked => Filter::All,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "All",
            Filter::Done => "Done",
            Filter::Active => "Active",
            Filter::Locked => "Locked",
        }
    }

    pub fn from_name(s: &str) -> Self {
        match s {
            "done" => Filter::Done,
            "active" => Filter::Active,
            "locked" => Filter::Locked,
            _ => Filter::All,
        }
    }

    pub fn as_name(self) -> &'static str {
        match self {
            Filter::All => "all",
            Filter::Done => "done",
            Filter::Active => "active",
            Filter::Locked => "locked",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewMode {
    Tree,
    List,
}

impl ViewMode {
    pub fn next(self) -> Self {
        match self {
            ViewMode::Tree => ViewMode::List,
            ViewMode::List => ViewMode::Tree,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ViewMode::Tree => "Tree",
            ViewMode::List => "List",
        }
    }

    pub fn from_name(s: &str) -> Self {
        if s == "list" {
            ViewMode::List
        } else {
            ViewMode::Tree
        }
    }

    pub fn as_name(self) -> &'static str {
        match self {
            ViewMode::Tree => "tree",
            ViewMode::List => "list",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaqueSide {
    Top,
    Bottom,
}

impl PlaqueSide {
    pub fn next(self) -> Self {
        match self {
            PlaqueSide::Top => PlaqueSide::Bottom,
            PlaqueSide::Bottom => PlaqueSide::Top,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PlaqueSide::Top => "Top",
            PlaqueSide::Bottom => "Bottom",
        }
    }

    pub fn from_name(s: &str) -> Self {
        if s == "bottom" {
            PlaqueSide::Bottom
        } else {
            PlaqueSide::Top
        }
    }

    pub fn as_name(self) -> &'static str {
        match self {
            PlaqueSide::Top => "top",
            PlaqueSide::Bottom => "bottom",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub id: String,
    pub parent: Option<String>,
    pub title: String,
    pub description: String,
    pub frame: FrameKind,
    pub icon: String,
    pub hidden: bool,
    pub x: f32,
    pub y: f32,
    pub criteria: Vec<String>,
    pub requirements: Vec<Vec<String>>,
    pub obtained: BTreeSet<String>,
    pub visible: bool,
    pub recipe: bool,
}

impl Entry {
    pub fn done(&self) -> bool {
        progress::is_done(&self.requirements, &self.obtained)
    }

    pub fn counts(&self) -> (usize, usize) {
        progress::counts(&self.requirements, &self.obtained)
    }

    pub fn status(&self) -> Filter {
        if self.done() {
            Filter::Done
        } else if self.visible {
            Filter::Active
        } else {
            Filter::Locked
        }
    }
}

pub mod progress {
    use super::*;

    pub fn is_done(requirements: &[Vec<String>], obtained: &BTreeSet<String>) -> bool {
        !requirements.is_empty()
            && requirements
                .iter()
                .all(|group| group.iter().any(|c| obtained.contains(c)))
    }

    pub fn counts(requirements: &[Vec<String>], obtained: &BTreeSet<String>) -> (usize, usize) {
        let done = requirements
            .iter()
            .filter(|g| g.iter().any(|c| obtained.contains(c)))
            .count();
        (done, requirements.len())
    }

    pub fn percent(done: usize, total: usize) -> u8 {
        if total == 0 {
            0
        } else {
            ((done * 100) / total).min(100) as u8
        }
    }

    pub fn criterion_lists(
        criteria: &[String],
        obtained: &BTreeSet<String>,
    ) -> (Vec<String>, Vec<String>) {
        let mut done = Vec::new();
        let mut rest = Vec::new();
        for c in criteria {
            if obtained.contains(c) {
                done.push(c.clone());
            } else {
                rest.push(c.clone());
            }
        }
        (done, rest)
    }

    pub fn tab_counts(entries: &[Entry], root: &str) -> (usize, usize) {
        let mut done = 0usize;
        let mut total = 0usize;
        for e in entries {
            if e.recipe {
                continue;
            }
            if e.id == root || belongs_to(entries, &e.id, root) {
                total += 1;
                if e.done() {
                    done += 1;
                }
            }
        }
        (done, total)
    }

    fn belongs_to(entries: &[Entry], id: &str, root: &str) -> bool {
        let mut cur = Some(id.to_string());
        let mut guard = 0;
        while let Some(c) = cur {
            if c == root {
                return true;
            }
            cur = entries.iter().find(|e| e.id == c).and_then(|e| e.parent.clone());
            guard += 1;
            if guard > 64 {
                break;
            }
        }
        false
    }
}

pub mod text {
    pub fn plain_from_json(v: &str) -> String {
        let t = v.trim();
        if t.starts_with('{') || t.starts_with('[') {
            if let Ok(val) = parse_loose(t) {
                return val;
            }
        }
        t.trim_matches('"').to_string()
    }

    fn parse_loose(s: &str) -> Result<String, ()> {
        fn walk(s: &str) -> String {
            // Minimal walk: pull "text" and "translate" values and "extra".
            let mut out = String::new();
            if let Some(t) = extract_str(s, "\"text\"") {
                out.push_str(&t);
            } else if let Some(t) = extract_str(s, "\"translate\"") {
                out.push_str(&humanize(&t));
            }
            if let Some(i) = s.find("\"extra\"") {
                out.push_str(&walk(&s[i + "\"extra\"".len()..]));
            }
            if out.is_empty() && s.starts_with('"') {
                out = extract_quoted(s).unwrap_or_default();
            }
            out
        }
        Ok(walk(s))
    }

    fn extract_str(s: &str, key: &str) -> Option<String> {
        let i = s.find(key)?;
        let rest = &s[i + key.len()..];
        let colon = rest.find(':')?;
        extract_quoted(&rest[colon + 1..])
    }

    fn extract_quoted(s: &str) -> Option<String> {
        let s = s.trim();
        let rest = s.strip_prefix('"')?;
        let mut out = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            } else if c == '"' {
                break;
            } else {
                out.push(c);
            }
        }
        Some(out)
    }

    pub fn humanize(id: &str) -> String {
        let path = id.rsplit(':').next().unwrap_or(id);
        let leaf = path.rsplit('/').next().unwrap_or(path);
        let mut out = String::new();
        for (i, part) in leaf.split('_').enumerate() {
            if i > 0 {
                out.push(' ');
            }
            let mut cs = part.chars();
            if let Some(c) = cs.next() {
                out.extend(c.to_uppercase());
                out.extend(cs);
            }
        }
        out
    }

    pub fn label_of(id: &str) -> String {
        humanize(id)
    }

    /// A language key: dotted lowercase path with no spaces (e.g. advancements.adventure.root.title).
    pub fn looks_like_key(s: &str) -> bool {
        let s = s.trim();
        if s.is_empty() || s.contains(' ') || !s.contains('.') {
            return false;
        }
        s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '/'))
    }

    /// The `translate` key inside a component JSON string, if any.
    pub fn translate_key(raw: &str) -> Option<String> {
        let t = raw.trim();
        if looks_like_key(t) {
            return Some(t.to_string());
        }
        if let Some(k) = extract_translate(t) {
            return Some(k);
        }
        None
    }

    fn extract_translate(s: &str) -> Option<String> {
        extract_str(s, "\"translate\"")
    }

    /// When a translation is missing, turn a key into a short title.
    /// `advancements.adventure.root.title` → "Adventure";
    /// `advancements.adventure.salvage_sherd.title` → "Salvage Sherd".
    pub fn fallback_label(key: &str) -> String {
        let mut parts: Vec<&str> = key
            .trim()
            .trim_matches('"')
            .split('.')
            .filter(|p| !p.is_empty())
            .collect();
        if parts.first().is_some_and(|p| *p == "advancements") {
            parts.remove(0);
        }
        if parts
            .last()
            .is_some_and(|p| *p == "title" || *p == "description" || *p == "name")
        {
            parts.pop();
        }
        if parts.last().is_some_and(|p| *p == "root") && parts.len() >= 2 {
            parts.pop();
        }
        let leaf = parts.last().copied().unwrap_or(key);
        let h = humanize(leaf);
        if h.is_empty() {
            humanize(key)
        } else {
            h
        }
    }

    /// Resolve a title/description that may be a translatable component.
    /// `lookup` is the game language (`Some` only when the key is present).
    /// A missed lookup falls back to [`fallback_label`].
    pub fn resolve_display(raw: &str, lookup: &dyn Fn(&str) -> Option<String>) -> String {
        if let Some(key) = translate_key(raw) {
            if let Some(t) = lookup(&key) {
                let t = t.trim();
                if !t.is_empty() && t != key && !looks_like_key(t) {
                    return t.to_string();
                }
            }
            return fallback_label(&key);
        }
        let plain = plain_from_json(raw);
        if plain.is_empty() {
            String::new()
        } else if looks_like_key(&plain) {
            if let Some(t) = lookup(&plain) {
                let t = t.trim();
                if !t.is_empty() && t != plain && !looks_like_key(t) {
                    return t.to_string();
                }
            }
            fallback_label(&plain)
        } else {
            plain
        }
    }

    /// Prefer a looked-up string; if it is still a raw key, humanize it.
    pub fn polish_resolved(text: &str) -> String {
        let t = text.trim();
        if t.is_empty() {
            String::new()
        } else if looks_like_key(t) {
            fallback_label(t)
        } else {
            t.to_string()
        }
    }
}

pub mod filter {
    use super::*;

    pub fn matches_query(id: &str, title: &str, query: &str) -> bool {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return true;
        }
        id.to_ascii_lowercase().contains(&q) || title.to_ascii_lowercase().contains(&q)
    }

    pub fn shown<'a>(entries: &'a [Entry], root: &str, filter: Filter, query: &str) -> Vec<&'a Entry> {
        entries
            .iter()
            .filter(|e| {
                !e.recipe
                    && in_tab(entries, e, root)
                    && matches_query(&e.id, &e.title, query)
                    && match filter {
                        Filter::All => true,
                        Filter::Done => e.done(),
                        Filter::Active => e.visible && !e.done(),
                        Filter::Locked => !e.visible && !e.done(),
                    }
            })
            .collect()
    }

    pub fn in_tab(entries: &[Entry], e: &Entry, root: &str) -> bool {
        if e.id == root {
            return true;
        }
        let mut cur = e.parent.as_deref();
        let mut guard = 0;
        while let Some(p) = cur {
            if p == root {
                return true;
            }
            cur = entries.iter().find(|x| x.id == p).and_then(|x| x.parent.as_deref());
            guard += 1;
            if guard > 64 {
                break;
            }
        }
        false
    }

    pub fn roots(entries: &[Entry]) -> Vec<&Entry> {
        entries
            .iter()
            .filter(|e| !e.recipe && e.parent.as_ref().is_none_or(|p| !entries.iter().any(|o| o.id == *p && o.display_ok())))
            .filter(|e| e.display_ok())
            .collect()
    }

    impl Entry {
        fn display_ok(&self) -> bool {
            !self.title.is_empty() || !self.icon.is_empty()
        }
    }
}

pub mod listing {
    pub const ROW: f32 = 24.0;
    pub const BAR_W: f32 = 6.0;
    pub const BAR_GAP: f32 = 2.0;
    pub const THUMB_MIN: f32 = 16.0;
    pub const NODE: f32 = 26.0;

    pub fn content_h(total: usize, row: f32) -> f32 {
        total as f32 * row
    }

    pub fn max_scroll(total: usize, view_h: f32, row: f32) -> f32 {
        (content_h(total, row) - view_h).max(0.0)
    }

    pub fn clamp_scroll(scroll: f32, total: usize, view_h: f32, row: f32) -> f32 {
        if !scroll.is_finite() {
            return 0.0;
        }
        scroll.clamp(0.0, max_scroll(total, view_h, row))
    }

    pub fn shown_rows(scroll: f32, total: usize, view_h: f32, row: f32) -> (usize, usize) {
        if total == 0 || view_h <= 0.0 || row <= 0.0 {
            return (0, 0);
        }
        let scroll = clamp_scroll(scroll, total, view_h, row);
        let top = scroll;
        let bot = scroll + view_h;
        let mut start = None;
        let mut end = 0;
        for i in 0..total {
            let y0 = i as f32 * row;
            let y1 = y0 + row;
            if y1 > top && y0 < bot {
                if start.is_none() {
                    start = Some(i);
                }
                end = i + 1;
            }
        }
        (start.unwrap_or(0), end)
    }

    pub fn last_row_fully_visible(total: usize, view_h: f32, row: f32) -> bool {
        if total == 0 {
            return true;
        }
        if content_h(total, row) <= view_h + 0.01 {
            let (_, end) = shown_rows(0.0, total, view_h, row);
            return end == total;
        }
        let scroll = max_scroll(total, view_h, row);
        let y0 = (total - 1) as f32 * row - scroll;
        y0 >= -0.01 && y0 + row <= view_h + 0.01
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Thumb {
        pub y: f32,
        pub h: f32,
    }

    pub fn thumb(scroll: f32, total: usize, view_h: f32, track_h: f32, row: f32) -> Option<Thumb> {
        if total == 0 || track_h <= 0.0 || content_h(total, row) <= view_h + 0.01 {
            return None;
        }
        let thumb_h = (view_h / content_h(total, row) * track_h).clamp(THUMB_MIN, track_h);
        let max = max_scroll(total, view_h, row);
        let scroll = clamp_scroll(scroll, total, view_h, row);
        let travel = (track_h - thumb_h).max(0.0);
        let y = if max <= 0.0 { 0.0 } else { (scroll / max) * travel };
        Some(Thumb { y, h: thumb_h })
    }

    pub fn scroll_at_track(local_y: f32, total: usize, view_h: f32, track_h: f32, row: f32) -> f32 {
        let Some(t) = thumb(0.0, total, view_h, track_h, row) else {
            return 0.0;
        };
        let travel = (track_h - t.h).max(0.0);
        if travel <= 0.0 {
            return 0.0;
        }
        let y = (local_y - t.h * 0.5).clamp(0.0, travel);
        (y / travel) * max_scroll(total, view_h, row)
    }

    pub fn scroll_from_drag(
        local_y: f32,
        grab: f32,
        total: usize,
        view_h: f32,
        track_h: f32,
        row: f32,
    ) -> f32 {
        let Some(t) = thumb(0.0, total, view_h, track_h, row) else {
            return 0.0;
        };
        let travel = (track_h - t.h).max(0.0);
        if travel <= 0.0 {
            return 0.0;
        }
        let y = (local_y - grab).clamp(0.0, travel);
        (y / travel) * max_scroll(total, view_h, row)
    }

    pub fn row_top(index: usize, scroll: f32, row: f32) -> f32 {
        index as f32 * row - scroll
    }

    pub fn intersect_rects(a: [f32; 4], b: [f32; 4]) -> Option<[f32; 4]> {
        let x0 = a[0].max(b[0]);
        let y0 = a[1].max(b[1]);
        let x1 = (a[0] + a[2]).min(b[0] + b[2]);
        let y1 = (a[1] + a[3]).min(b[1] + b[3]);
        if x1 > x0 && y1 > y0 {
            Some([x0, y0, x1 - x0, y1 - y0])
        } else {
            None
        }
    }

    pub fn row_box(list: [f32; 4], index: usize, scroll: f32, row: f32) -> [f32; 4] {
        [list[0], list[1] + row_top(index, scroll, row), list[2].max(0.0), row.max(0.0)]
    }

    pub fn visible_row_rect(list: [f32; 4], index: usize, scroll: f32, row: f32) -> Option<[f32; 4]> {
        intersect_rects(row_box(list, index, scroll, row), list)
    }

    pub fn wheel_delta(dy: f32, row: f32) -> f32 {
        if !dy.is_finite() || !row.is_finite() {
            return 0.0;
        }
        dy * row
    }

    pub fn apply_wheel(scroll: f32, dy: f32, total: usize, view_h: f32, row: f32) -> f32 {
        clamp_scroll(scroll - wheel_delta(dy, row), total, view_h, row)
    }

    /// Grid steps between node centres. With 26 px frames this leaves a clear 16 px corridor
    /// between neighbours (12 px at 75 %), so every branch bus sits in open space instead of
    /// under a frame.
    pub const STEP_X: f32 = 42.0;
    pub const STEP_Y: f32 = 42.0;
    /// How far a connector runs into the frames it joins, so it meets them flush (the frames
    /// are drawn over the connectors, hiding the overlap). Challenge frames grow 2 px outward,
    /// so a connector that ends inside the inner frame also crosses the outer ring with no gap.
    pub const LINK_INSET: f32 = 2.0;

    /// The clear space between two neighbouring frames at `zoom`.
    pub fn corridor(zoom: f32) -> f32 {
        let z = snap_zoom(zoom);
        ((STEP_X.min(STEP_Y) - NODE) * z).max(0.0)
    }

    /// A node's frame on screen, on whole pixels.
    pub fn node_rect(view: [f32; 4], pan: [f32; 2], zoom: f32, x: f32, y: f32) -> [f32; 4] {
        let z = snap_zoom(zoom);
        let size = frame_size(z);
        let cx = view[0] + view[2] * 0.5 + (x * STEP_X - pan[0]) * z;
        let cy = view[1] + view[3] * 0.5 + (y * STEP_Y - pan[1]) * z;
        [(cx - size * 0.5).round(), (cy - size * 0.5).round(), size, size]
    }

    pub fn visible_node_rect(
        view: [f32; 4],
        pan: [f32; 2],
        zoom: f32,
        x: f32,
        y: f32,
    ) -> Option<[f32; 4]> {
        intersect_rects(node_rect(view, pan, zoom, x, y), view)
    }

    pub fn content_bounds(entries: &[super::Entry]) -> (f32, f32, f32, f32) {
        if entries.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;
        for e in entries {
            let px = e.x * STEP_X;
            let py = e.y * STEP_Y;
            min_x = min_x.min(px);
            min_y = min_y.min(py);
            max_x = max_x.max(px);
            max_y = max_y.max(py);
        }
        (min_x, min_y, max_x, max_y)
    }

    pub fn clamp_pan(pan: [f32; 2], entries: &[super::Entry], view: [f32; 4], zoom: f32) -> [f32; 2] {
        let z = snap_zoom(zoom);
        let (min_x, min_y, max_x, max_y) = content_bounds(entries);
        if !min_x.is_finite() {
            return [0.0, 0.0];
        }
        let vw = (view[2] / z).max(1.0);
        let vh = (view[3] / z).max(1.0);
        let pad = NODE * 2.0;
        let mid_x = (min_x + max_x) * 0.5;
        let mid_y = (min_y + max_y) * 0.5;
        let half_w = ((max_x - min_x) * 0.5 + pad).max(vw * 0.5);
        let half_h = ((max_y - min_y) * 0.5 + pad).max(vh * 0.5);
        let x = pan[0].clamp(mid_x - half_w, mid_x + half_w);
        let y = pan[1].clamp(mid_y - half_h, mid_y + half_h);
        [
            if x.is_finite() { x } else { 0.0 },
            if y.is_finite() { y } else { 0.0 },
        ]
    }

    /// Wheel zoom stops: 75%, 100%, 125%, 150%.
    pub const ZOOM_STEPS: [f32; 4] = [0.75, 1.0, 1.25, 1.5];
    pub const ICON: f32 = 16.0;

    pub fn snap_zoom(zoom: f32) -> f32 {
        if !zoom.is_finite() {
            return 1.0;
        }
        ZOOM_STEPS
            .iter()
            .copied()
            .min_by(|a, b| (a - zoom).abs().partial_cmp(&(b - zoom).abs()).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(1.0)
    }

    pub fn zoom_step(zoom: f32, dy: f32) -> f32 {
        let z = snap_zoom(zoom);
        let i = ZOOM_STEPS
            .iter()
            .position(|s| (*s - z).abs() < 0.001)
            .unwrap_or(1);
        if !dy.is_finite() || dy == 0.0 {
            return z;
        }
        if dy > 0.0 {
            ZOOM_STEPS[(i + 1).min(ZOOM_STEPS.len() - 1)]
        } else {
            ZOOM_STEPS[i.saturating_sub(1)]
        }
    }

    /// Keep the world point under `cursor` fixed when zooming from `z0` to `z1`.
    pub fn zoom_pan(pan: [f32; 2], z0: f32, z1: f32, view: [f32; 4], cursor: [f32; 2]) -> [f32; 2] {
        let z0 = if z0.is_finite() && z0 > 0.01 { z0 } else { 1.0 };
        let z1 = if z1.is_finite() && z1 > 0.01 { z1 } else { 1.0 };
        let cx = view[0] + view[2] * 0.5;
        let cy = view[1] + view[3] * 0.5;
        let world_x = pan[0] + (cursor[0] - cx) / z0;
        let world_y = pan[1] + (cursor[1] - cy) / z0;
        [
            world_x - (cursor[0] - cx) / z1,
            world_y - (cursor[1] - cy) / z1,
        ]
    }

    /// A frame's side on screen, on whole pixels (19.5 px at 75 % draws as 20).
    pub fn frame_size(zoom: f32) -> f32 {
        (NODE * snap_zoom(zoom)).round()
    }

    pub fn icon_size(zoom: f32) -> f32 {
        ICON * snap_zoom(zoom)
    }

    /// Padding on each side when a 16px icon is scaled with the frame.
    pub fn icon_pad(zoom: f32) -> f32 {
        (frame_size(zoom) - icon_size(zoom)) * 0.5
    }

    pub fn pause_advancements_rect(w: f32, h: f32, extra_rows: u8) -> [f32; 4] {
        let grid_w = 212.0;
        let grid_h = 166.0 + 24.0 * f32::from(extra_rows);
        let x0 = ((w - grid_w) * 0.5).floor();
        let y0 = ((h - grid_h) * 0.25).floor();
        [x0 + 4.0, y0 + 74.0, 98.0, 20.0]
    }

    /// "Adventure 3/45" — the progress stays attached so it can be kept when truncating.
    pub fn tab_label(title: &str, done: usize, total: usize) -> String {
        format!("{title} {done}/{total}")
    }

    /// Fit a tab label into `max_chars`, keeping the " x/y" suffix visible.
    pub fn fit_tab_label(title: &str, done: usize, total: usize, max_chars: usize) -> String {
        let suffix = format!(" {done}/{total}");
        let max_chars = max_chars.max(suffix.chars().count() + 1);
        let budget = max_chars - suffix.chars().count();
        let n = title.chars().count();
        if n <= budget {
            return format!("{title}{suffix}");
        }
        if budget <= 1 {
            return suffix.trim().to_string();
        }
        let cut = title.chars().take(budget.saturating_sub(1)).collect::<String>();
        format!("{cut}…{suffix}")
    }

    /// Parent→child id pairs among `shown` only. A child whose parent is hidden
    /// has no edge (no dangling line, no skip-to-grandparent).
    pub fn edges(shown: &[super::Entry]) -> Vec<(String, String)> {
        let ids: std::collections::BTreeSet<&str> = shown.iter().map(|e| e.id.as_str()).collect();
        let mut out = Vec::new();
        for e in shown {
            let Some(p) = e.parent.as_deref() else {
                continue;
            };
            if ids.contains(p) {
                out.push((p.to_string(), e.id.clone()));
            }
        }
        out
    }

    /// Horizontal elbow: parent right-center → mid x → child left-center.
    /// Returns the three points (parent, mid-at-parent-y, mid-at-child-y is implied
    /// by two segments from mid x). `None` if the child has no room to the right
    /// and we still want a fallback mid.
    pub fn elbow(parent: [f32; 4], child: [f32; 4]) -> [[f32; 2]; 3] {
        let right = parent[0] + parent[2];
        let pr = [right - LINK_INSET, (parent[1] + parent[3] * 0.5).round()];
        let cy = (child[1] + child[3] * 0.5).round();
        // The vertical bus runs down the middle of the corridor between the two frames.
        let mid_x = if child[0] > right {
            ((right + child[0]) * 0.5).round()
        } else {
            // No corridor (only when frames touch): run just past the farther edge.
            (right.max(child[0]) + LINK_INSET).round()
        };
        [pr, [mid_x, pr[1]], [mid_x, cy]]
        // caller draws: pr→(mid,pr.y), (mid,pr.y)→(mid,cl.y), (mid,cl.y)→cl
    }

    /// Where an elbow ends: just inside the child's left edge, at its middle.
    pub fn elbow_child_end(child: [f32; 4]) -> [f32; 2] {
        [child[0] + LINK_INSET, (child[1] + child[3] * 0.5).round()]
    }

    /// Connector thickness in GUI pixels.
    pub const LINK_THICK: f32 = 2.0;

    /// The filled rect of a straight connector from `a` to `b` (horizontal or vertical), on
    /// whole pixels, extended by half its thickness at both ends so elbow corners meet square.
    pub fn seg_rect(a: [f32; 2], b: [f32; 2]) -> [f32; 4] {
        let t = LINK_THICK;
        let h = t * 0.5;
        if (a[0] - b[0]).abs() >= (a[1] - b[1]).abs() {
            let x0 = (a[0].min(b[0]) - h).round();
            let x1 = (a[0].max(b[0]) + h).round();
            [x0, (a[1] - h).round(), (x1 - x0).max(t), t]
        } else {
            let y0 = (a[1].min(b[1]) - h).round();
            let y1 = (a[1].max(b[1]) + h).round();
            [(a[0] - h).round(), y0, t, (y1 - y0).max(t)]
        }
    }

    /// True when two node rects (same zoom/pan) overlap in the view plane.
    pub fn node_rects_overlap(a: [f32; 4], b: [f32; 4]) -> bool {
        let x0 = a[0].max(b[0]);
        let y0 = a[1].max(b[1]);
        let x1 = (a[0] + a[2]).min(b[0] + b[2]);
        let y1 = (a[1] + a[3]).min(b[1] + b[3]);
        x1 > x0 + 0.05 && y1 > y0 + 0.05
    }

    /// Whether any pair of entries would paint overlapping node boxes.
    pub fn any_nodes_overlap(entries: &[super::Entry], view: [f32; 4], pan: [f32; 2], zoom: f32) -> bool {
        let rects: Vec<[f32; 4]> = entries
            .iter()
            .map(|e| node_rect(view, pan, zoom, e.x, e.y))
            .collect();
        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                if node_rects_overlap(rects[i], rects[j]) {
                    return true;
                }
            }
        }
        false
    }

    /// Walk `parent` up until an entry in `entries` is found (skips missing
    /// display-less ancestors).
    fn nearest_in_list<'a>(entries: &'a [super::Entry], mut parent: Option<&'a str>) -> Option<&'a str> {
        let mut guard = 0;
        while let Some(p) = parent {
            if let Some(e) = entries.iter().find(|e| e.id == p && !e.recipe) {
                return Some(e.id.as_str());
            }
            parent = entries.iter().find(|e| e.id == p).and_then(|e| e.parent.as_deref());
            guard += 1;
            if guard > 64 {
                break;
            }
        }
        None
    }

    /// Left-to-right tidy layout from parent links: depth is `x` (columns),
    /// siblings are packed on `y` (rows). Packet positions are ignored.
    pub fn tidy_layout(entries: &mut [super::Entry], root: &str) {
        use std::collections::BTreeMap;
        let n = entries.len();
        if n == 0 {
            return;
        }
        let mut kids: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut root_i = None;
        for (i, e) in entries.iter().enumerate() {
            if e.recipe {
                continue;
            }
            if e.id == root {
                root_i = Some(i);
            }
            if let Some(p) = nearest_in_list(entries, e.parent.as_deref()) {
                kids.entry(p.to_string()).or_default().push(i);
            }
        }
        let Some(root_i) = root_i else {
            return;
        };
        for v in kids.values_mut() {
            v.sort_by_key(|&i| entries[i].id.clone());
        }
        fn walk(
            entries: &[super::Entry],
            kids: &BTreeMap<String, Vec<usize>>,
            i: usize,
            depth: i32,
            next_row: &mut f32,
            pos: &mut Vec<Option<(f32, f32)>>,
        ) -> f32 {
            let children = kids.get(&entries[i].id).cloned().unwrap_or_default();
            if children.is_empty() {
                let y = *next_row;
                *next_row += 1.0;
                pos[i] = Some((depth as f32, y));
                return y;
            }
            let mut ys = Vec::new();
            for c in children {
                ys.push(walk(entries, kids, c, depth + 1, next_row, pos));
            }
            let y = (ys[0] + ys[ys.len() - 1]) * 0.5;
            pos[i] = Some((depth as f32, y));
            y
        }
        let mut pos = vec![None; n];
        let mut next_row = 0.0;
        walk(entries, &kids, root_i, 0, &mut next_row, &mut pos);
        for (e, p) in entries.iter_mut().zip(pos) {
            if let Some((x, y)) = p {
                e.x = x;
                e.y = y;
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Orient {
        /// Depth grows right; siblings pack downward.
        Ltr,
        /// Depth grows down; siblings pack across, wrapping into rows when a
        /// level would be too wide.
        Ttb,
    }

    /// Trees whose widest sibling group is larger than this prefer a top-down
    /// layout so a single fat column does not run off the page.
    pub const FANOUT_TTB: usize = 6;
    pub const DEFAULT_VIEW: [f32; 4] = [0.0, 0.0, 560.0, 280.0];

    fn child_map(entries: &[super::Entry], root: &str) -> (std::collections::BTreeMap<String, Vec<usize>>, Option<usize>) {
        use std::collections::BTreeMap;
        let mut kids: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut root_i = None;
        for (i, e) in entries.iter().enumerate() {
            if e.recipe {
                continue;
            }
            if e.id == root {
                root_i = Some(i);
            }
            if let Some(p) = nearest_in_list(entries, e.parent.as_deref()) {
                kids.entry(p.to_string()).or_default().push(i);
            }
        }
        for v in kids.values_mut() {
            v.sort_by_key(|&i| entries[i].id.clone());
        }
        (kids, root_i)
    }

    pub fn max_fanout(entries: &[super::Entry], root: &str) -> usize {
        let (kids, root_i) = child_map(entries, root);
        let Some(root_i) = root_i else {
            return 0;
        };
        let mut best = 0usize;
        let mut stack = vec![root_i];
        let mut guard = 0;
        while let Some(i) = stack.pop() {
            guard += 1;
            if guard > 4096 {
                break;
            }
            let n = kids.get(&entries[i].id).map(|v| v.len()).unwrap_or(0);
            best = best.max(n);
            if let Some(v) = kids.get(&entries[i].id) {
                stack.extend(v.iter().copied());
            }
        }
        best
    }

    pub fn bbox_aspect(entries: &[super::Entry]) -> f32 {
        let (min_x, min_y, max_x, max_y) = content_bounds(entries);
        if !min_x.is_finite() {
            return 1.0;
        }
        let w = (max_x - min_x + NODE).max(1.0);
        let h = (max_y - min_y + NODE).max(1.0);
        w / h
    }

    fn tree_fits(entries: &[super::Entry], view: [f32; 4], zoom: f32) -> bool {
        let z = snap_zoom(zoom);
        let (min_x, min_y, max_x, max_y) = content_bounds(entries);
        if !min_x.is_finite() {
            return true;
        }
        let pad = NODE * 2.0;
        let cw = (max_x - min_x) + pad * 2.0;
        let ch = (max_y - min_y) + pad * 2.0;
        let vw = (view[2] / z).max(1.0);
        let vh = (view[3] / z).max(1.0);
        cw <= vw + 0.5 && ch <= vh + 0.5
    }

    fn center_pan(entries: &[super::Entry], view: [f32; 4], zoom: f32) -> [f32; 2] {
        let (min_x, min_y, max_x, max_y) = content_bounds(entries);
        clamp_pan([(min_x + max_x) * 0.5, (min_y + max_y) * 0.5], entries, view, zoom)
    }

    fn wrap_row_width(n: usize, view: [f32; 4]) -> f32 {
        if n <= 8 {
            return n as f32 * 64.0; // no wrap: one row
        }
        let va = (view[2].max(1.0) / view[3].max(1.0)).clamp(0.5, 3.0);
        let aimed = (n as f32 * va * (STEP_Y / STEP_X)).sqrt();
        let fit75 = (view[2].max(1.0) / 0.75 / STEP_X).max(4.0);
        aimed.clamp(4.0, fit75.max(6.0).min(n as f32))
    }

    struct TtbBox {
        parts: Vec<(usize, f32, f32)>,
        w: f32,
        h: f32,
    }

    fn place_ttb(
        entries: &[super::Entry],
        kids: &std::collections::BTreeMap<String, Vec<usize>>,
        i: usize,
        view: [f32; 4],
    ) -> TtbBox {
        let children = kids.get(&entries[i].id).cloned().unwrap_or_default();
        if children.is_empty() {
            return TtbBox {
                parts: vec![(i, 0.0, 0.0)],
                w: 1.0,
                h: 1.0,
            };
        }
        let boxes: Vec<TtbBox> = children
            .iter()
            .map(|&c| place_ttb(entries, kids, c, view))
            .collect();
        let max_row_w = wrap_row_width(boxes.len(), view);
        let wrap = boxes.len() > 8;
        let limit = if wrap { max_row_w.max(1.0) } else { f32::INFINITY };
        let mut parts = Vec::new();
        let mut x = 0.0;
        let mut row_h = 0.0;
        let mut row_w = 0.0;
        let mut block_w: f32 = 0.0;
        let mut cy = 1.0;
        let mut row_items = 0usize;
        for b in boxes {
            if row_items > 0 && (row_items >= 1 && x + b.w > limit + 0.001) {
                cy += row_h;
                block_w = block_w.max(row_w);
                x = 0.0;
                row_h = 0.0;
                row_items = 0;
            }
            for (idx, px, py) in b.parts {
                parts.push((idx, px + x, py + cy));
            }
            x += b.w;
            row_w = x;
            row_h = row_h.max(b.h);
            row_items += 1;
        }
        block_w = block_w.max(row_w).max(1.0);
        let block_h = (cy + row_h).max(1.0);
        let px = (block_w - 1.0) * 0.5;
        parts.push((i, px, 0.0));
        TtbBox {
            parts,
            w: block_w,
            h: block_h,
        }
    }

    /// Top-down tidy layout: the root sits at the top, depth is `y`, siblings
    /// spread on `x`. A very wide sibling group wraps into a grid of rows so
    /// the tree stays close to the viewport aspect.
    pub fn tidy_layout_ttb(entries: &mut [super::Entry], root: &str, view: [f32; 4]) {
        let n = entries.len();
        if n == 0 {
            return;
        }
        let (kids, root_i) = child_map(entries, root);
        let Some(root_i) = root_i else {
            return;
        };
        let boxed = place_ttb(entries, &kids, root_i, view);
        let mut pos = vec![None; n];
        for (i, x, y) in boxed.parts {
            pos[i] = Some((x, y));
        }
        for (e, p) in entries.iter_mut().zip(pos) {
            if let Some((x, y)) = p {
                e.x = x;
                e.y = y;
            }
        }
    }

    fn choose_orient(fanout: usize, ltr_aspect: f32, ttb_aspect: f32, view: [f32; 4]) -> Orient {
        if fanout > FANOUT_TTB {
            return Orient::Ttb;
        }
        let va = (view[2].max(1.0) / view[3].max(1.0)).max(0.01);
        let ltr = ltr_aspect.max(0.01);
        let ttb = ttb_aspect.max(0.01);
        let ltr_pen = (ltr.ln() - va.ln()).abs();
        let ttb_pen = (ttb.ln() - va.ln()).abs();
        if ttb_pen + 0.08 < ltr_pen {
            Orient::Ttb
        } else {
            Orient::Ltr
        }
    }

    /// Lay out one root with both orientations and pick LTR or TTB so the
    /// tree fits the viewport's landscape or portrait shape. High-fanout
    /// trees (Adventure, Husbandry, Nether) go top-down; chain-heavy ones
    /// (Minecraft / story) stay left-to-right.
    pub fn layout_tab(entries: &mut [super::Entry], root: &str, view: [f32; 4]) -> Orient {
        let fanout = max_fanout(entries, root);
        tidy_layout(entries, root);
        let ltr: Vec<(f32, f32)> = entries.iter().map(|e| (e.x, e.y)).collect();
        let ltr_aspect = bbox_aspect(entries);
        tidy_layout_ttb(entries, root, view);
        let ttb_aspect = bbox_aspect(entries);
        let orient = choose_orient(fanout, ltr_aspect, ttb_aspect, view);
        if orient == Orient::Ltr {
            for (e, (x, y)) in entries.iter_mut().zip(ltr) {
                e.x = x;
                e.y = y;
            }
        }
        orient
    }

    /// Prefer 100% zoom when the tree fits, otherwise 75%. Pan is centered
    /// when it fits; otherwise the root sits at the left (LTR) or top (TTB).
    pub fn opening_camera(
        entries: &[super::Entry],
        root: Option<&super::Entry>,
        view: [f32; 4],
        orient: Orient,
    ) -> ([f32; 2], f32) {
        for &z in &[1.0, 0.75] {
            if tree_fits(entries, view, z) {
                return (center_pan(entries, view, z), z);
            }
        }
        let z = 0.75;
        let pan = match orient {
            Orient::Ltr => initial_pan(entries, root, view, z),
            Orient::Ttb => {
                let (rx, ry) = root
                    .map(|e| (e.x * STEP_X, e.y * STEP_Y))
                    .unwrap_or_else(|| {
                        let (a, b, c, _d) = content_bounds(entries);
                        ((a + c) * 0.5, b)
                    });
                let want = (NODE * z * 0.5 + 16.0).min(view[3] * 0.22);
                let pan_x = rx;
                let pan_y = ry - (want - view[3] * 0.5) / z;
                clamp_pan([pan_x, pan_y], entries, view, z)
            }
        };
        (pan, z)
    }

    /// Vertical elbows for a parent and its children: a shared bus under the
    /// parent, plus a left spine and per-row buses when children wrap.
    pub fn ttb_segments(parent: [f32; 4], children: &[[f32; 4]]) -> Vec<[[f32; 2]; 2]> {
        if children.is_empty() {
            return Vec::new();
        }
        // From just inside the parent's bottom edge, at its middle.
        let pb = [(parent[0] + parent[2] * 0.5).round(), parent[1] + parent[3] - LINK_INSET];
        let bottom = parent[1] + parent[3];
        let mut items: Vec<[f32; 4]> = children.to_vec();
        items.sort_by(|a, b| a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal));
        let thresh = (items.iter().map(|c| c[3]).fold(0.0, f32::max) * 0.55).max(8.0);
        let mut rows: Vec<Vec<[f32; 4]>> = Vec::new();
        for c in items {
            if let Some(last) = rows.last_mut() {
                let y0 = last[0][1];
                if (c[1] - y0).abs() <= thresh {
                    last.push(c);
                    continue;
                }
            }
            rows.push(vec![c]);
        }
        let mut segs = Vec::new();
        let top_of = |row: &[[f32; 4]]| row.iter().map(|c| c[1]).fold(f32::INFINITY, f32::min);
        let cx_of = |c: [f32; 4]| (c[0] + c[2] * 0.5).round();
        // Each drop ends just inside the child's top edge.
        let top_in = |c: [f32; 4]| c[1] + LINK_INSET;
        // The corridor between frames at this zoom (frames are NODE * zoom wide).
        let zoom = children[0][2] / NODE;
        let gap = corridor(zoom);
        if rows.len() == 1 {
            let row = &rows[0];
            let bus_y = ((bottom + top_of(row)) * 0.5).round();
            let mut lo = pb[0];
            let mut hi = pb[0];
            for c in row {
                let x = cx_of(*c);
                lo = lo.min(x);
                hi = hi.max(x);
                segs.push([[x, bus_y], [x, top_in(*c)]]);
            }
            segs.push([pb, [pb[0], bus_y]]);
            segs.push([[lo, bus_y], [hi, bus_y]]);
            return segs;
        }
        let leftmost = rows
            .iter()
            .flatten()
            .map(|c| c[0])
            .fold(f32::INFINITY, f32::min);
        // The spine runs down the middle of the corridor left of the leftmost child.
        let spine_x = (leftmost - (gap * 0.5).max(LINK_INSET + 2.0)).round();
        let mut prev_bus = ((bottom + top_of(&rows[0])) * 0.5).round();
        segs.push([pb, [pb[0], prev_bus]]);
        segs.push([[pb[0], prev_bus], [spine_x, prev_bus]]);
        for (i, row) in rows.iter().enumerate() {
            let bus_y = if i == 0 {
                prev_bus
            } else {
                // Halfway down the corridor above this row.
                let gap_top = row.iter().map(|c| c[1] - gap).fold(f32::INFINITY, f32::min);
                let y = ((gap_top + top_of(row)) * 0.5).round().max(prev_bus);
                segs.push([[spine_x, prev_bus], [spine_x, y]]);
                y
            };
            let mut hi = spine_x;
            for c in row {
                let x = cx_of(*c);
                hi = hi.max(x);
                segs.push([[x, bus_y], [x, top_in(*c)]]);
            }
            segs.push([[spine_x, bus_y], [hi, bus_y]]);
            prev_bus = bus_y;
        }
        segs
    }

    /// Opening pan: if the tree fits, center its bounds; otherwise sit the root
    /// in the left side of the view so children extend to the right.
    pub fn initial_pan(
        entries: &[super::Entry],
        root: Option<&super::Entry>,
        view: [f32; 4],
        zoom: f32,
    ) -> [f32; 2] {
        let z = snap_zoom(zoom);
        if entries.is_empty() {
            return [0.0, 0.0];
        }
        let (min_x, min_y, max_x, max_y) = content_bounds(entries);
        let pad = NODE * 2.0;
        let cw = (max_x - min_x) + pad * 2.0;
        let ch = (max_y - min_y) + pad * 2.0;
        let vw = (view[2] / z).max(1.0);
        let vh = (view[3] / z).max(1.0);
        if cw <= vw + 0.5 && ch <= vh + 0.5 {
            return [(min_x + max_x) * 0.5, (min_y + max_y) * 0.5];
        }
        let (rx, ry) = root
            .map(|e| (e.x * STEP_X, e.y * STEP_Y))
            .unwrap_or((min_x, (min_y + max_y) * 0.5));
        // Place the root's center near the left of the view (padding).
        let want = (NODE * z + 16.0).min(view[2] * 0.28);
        let pan_x = rx - (want - view[2] * 0.5) / z;
        let pan_y = ry;
        clamp_pan([pan_x, pan_y], entries, view, z)
    }

    /// Distinct frame silhouettes used when painting a node.
    pub fn frame_shape(kind: super::FrameKind) -> &'static str {
        match kind {
            super::FrameKind::Task => "square",
            super::FrameKind::Goal => "rounded",
            super::FrameKind::Challenge => "diamond",
        }
    }
}

pub mod layout {
    pub const MARGIN: f32 = 8.0;
    pub const PAD: f32 = 10.0;
    pub const TITLE_TOP: f32 = 8.0;
    pub const TITLE_TO_TABS: f32 = 28.0;
    pub const TAB_H: f32 = 18.0;
    pub const TAB_GAP: f32 = 2.0;
    pub const TAB_TO_BODY: f32 = 6.0;
    pub const BOTTOM_PAD: f32 = 16.0;
    pub const CHIP_H: f32 = 16.0;
    pub const CHIP_GAP: f32 = 2.0;
    pub const SEARCH_CHIP_GAP: f32 = 4.0;
    pub const TOOL_ROW_GAP: f32 = 4.0;
    pub const TOOL_PAD_AFTER: f32 = 6.0;
    pub const CHIP_W_IDEAL: f32 = 70.0;
    pub const CHIP_W_MIN: f32 = 44.0;
    pub const SEARCH_W_MIN: f32 = 64.0;
    pub const TAB_W_MAX: f32 = 148.0;
    pub const DONE_W: f32 = 52.0;
    pub const DONE_H: f32 = 16.0;
    pub const MIN_LIST: f32 = 24.0;
    pub const PANEL_W_MIN: f32 = 360.0;
    pub const PANEL_W_MAX: f32 = 720.0;
    pub const PANEL_H_MIN: f32 = 260.0;
    pub const PANEL_H_MAX: f32 = 480.0;
    const EPS: f32 = 0.05;

    #[derive(Clone, Debug)]
    pub struct Layout {
        pub x: f32,
        pub y: f32,
        pub w: f32,
        pub h: f32,
        pub done: [f32; 4],
        pub tabs_bar: [f32; 4],
        pub tab_w: f32,
        pub tab_compact: u8,
        pub body: [f32; 4],
        pub search: [f32; 4],
        pub filter: [f32; 4],
        pub mode: [f32; 4],
        pub tools_wrapped: bool,
        pub compact_tools: bool,
        pub view: [f32; 4],
        pub bar: [f32; 4],
        pub header_h: f32,
    }

    impl Layout {
        pub fn panel(&self) -> [f32; 4] {
            [self.x, self.y, self.w, self.h]
        }

        pub fn tab_rect(&self, i: usize, n: usize) -> [f32; 4] {
            let n = n.max(1) as f32;
            let slot = (self.tabs_bar[2] / n).min(TAB_W_MAX + TAB_GAP);
            let w = (slot - TAB_GAP).max(1.0).min(TAB_W_MAX);
            [self.tabs_bar[0] + slot * i as f32, self.tabs_bar[1], w, TAB_H]
        }

        pub fn tab_caption<'a>(&self, full: &'a str, short: &'a str, letter: &'a str) -> &'a str {
            Self::caption_for_width(self.tab_w, full, short, letter)
        }

        pub fn caption_for_width<'a>(w: f32, full: &'a str, short: &'a str, letter: &'a str) -> &'a str {
            if w >= 80.0 {
                full
            } else if w >= 36.0 {
                short
            } else {
                letter
            }
        }

        pub fn hit_rects(&self) -> Vec<[f32; 4]> {
            vec![
                self.panel(),
                self.done,
                self.tabs_bar,
                self.body,
                self.search,
                self.filter,
                self.mode,
                self.view,
                self.bar,
            ]
        }
    }

    pub fn compute(sw: f32, sh: f32) -> Layout {
        let sw = sw.max(1.0);
        let sh = sh.max(1.0);
        let max_w = (sw - MARGIN * 2.0).max(1.0);
        let max_h = (sh - MARGIN * 2.0).max(1.0);
        let want_w = (sw * 0.86).clamp(PANEL_W_MIN.min(max_w), PANEL_W_MAX);
        let want_h = (sh * 0.86).clamp(PANEL_H_MIN.min(max_h), PANEL_H_MAX);
        let w = want_w.min(max_w);
        let h = want_h.min(max_h);
        let x = ((sw - w) * 0.5).clamp(MARGIN, (sw - w - MARGIN).max(MARGIN));
        let y = ((sh - h) * 0.5).clamp(MARGIN, (sh - h - MARGIN).max(MARGIN));
        let x = x.min((sw - MARGIN - w).max(MARGIN));
        let y = y.min((sh - MARGIN - h).max(MARGIN));

        let done_w = DONE_W.min((w - PAD * 2.0).max(24.0));
        let done = [x + w - PAD - done_w, y + TITLE_TOP, done_w, DONE_H];

        let inner_x = x + PAD;
        let inner_w = (w - PAD * 2.0).max(1.0);
        let tab_y = y + TITLE_TO_TABS;
        // Actual tabs use tab_rect, which caps width so a single tab is not stretched.
        let tab_w = ((inner_w / 6.0) - TAB_GAP).max(1.0).min(TAB_W_MAX);
        let tab_compact = if tab_w >= 56.0 {
            0
        } else if tab_w >= 32.0 {
            1
        } else {
            2
        };
        let tabs_bar = [inner_x, tab_y, inner_w, TAB_H];

        let top = TITLE_TO_TABS + TAB_H + TAB_TO_BODY;
        let bottom = BOTTOM_PAD.min((h - top).max(0.0));
        let avail = (h - top - bottom).max(0.0);
        let body = [inner_x, y + top, inner_w, avail];

        let gaps = SEARCH_CHIP_GAP + CHIP_GAP;
        let min_one = SEARCH_W_MIN + CHIP_W_MIN * 2.0 + gaps;
        let need_wrap = inner_w + 0.01 < min_one;
        let header_one = CHIP_H + TOOL_PAD_AFTER;
        let header_two = CHIP_H + TOOL_ROW_GAP + CHIP_H + TOOL_PAD_AFTER;
        let wrap = need_wrap && header_two + MIN_LIST <= avail + 0.01;
        let header_h = if wrap { header_two } else { header_one }.min(avail);

        let (search, filter, mode, compact_tools) = place_toolbar(inner_x, body[1], inner_w, wrap);

        let list_top = body[1] + header_h;
        let list_h = (body[1] + body[3] - list_top).max(0.0);
        let bar_space = super::listing::BAR_W + super::listing::BAR_GAP;
        let list_w = (inner_w - bar_space).max(0.0);
        let view = [inner_x, list_top, list_w, list_h];
        let bar = [view[0] + view[2] + super::listing::BAR_GAP, view[1], super::listing::BAR_W.min(inner_w), list_h];

        let panel = [x, y, w, h];
        Layout {
            x,
            y,
            w,
            h,
            done: clamp_into(done, panel),
            tabs_bar: clamp_into(tabs_bar, panel),
            tab_w,
            tab_compact,
            body: clamp_into(body, panel),
            search: clamp_into(search, body),
            filter: clamp_into(filter, body),
            mode: clamp_into(mode, body),
            tools_wrapped: wrap,
            compact_tools,
            view: clamp_into(view, body),
            bar: clamp_into(bar, body),
            header_h,
        }
    }

    fn place_toolbar(
        x: f32,
        y: f32,
        width: f32,
        wrap: bool,
    ) -> ([f32; 4], [f32; 4], [f32; 4], bool) {
        if wrap {
            let search = [x, y, width.max(1.0), CHIP_H];
            let chip_w = ((width - CHIP_GAP) / 2.0).max(1.0).min(CHIP_W_IDEAL);
            let y2 = y + CHIP_H + TOOL_ROW_GAP;
            let filter = [x, y2, chip_w, CHIP_H];
            let mode = [x + chip_w + CHIP_GAP, y2, chip_w, CHIP_H];
            return (search, filter, mode, true);
        }
        let gaps = SEARCH_CHIP_GAP + CHIP_GAP;
        let chip_budget = (width - gaps - SEARCH_W_MIN).max(0.0);
        let chip_w = (chip_budget / 2.0).clamp(CHIP_W_MIN.min(chip_budget / 2.0), CHIP_W_IDEAL);
        let search_w = (width - (chip_w * 2.0 + CHIP_GAP + SEARCH_CHIP_GAP)).max(1.0);
        let search = [x, y, search_w, CHIP_H];
        let cx = x + search_w + SEARCH_CHIP_GAP;
        let filter = [cx, y, chip_w, CHIP_H];
        let mode = [cx + chip_w + CHIP_GAP, y, chip_w, CHIP_H];
        (search, filter, mode, chip_w < 58.0)
    }

    fn clamp_into(r: [f32; 4], outer: [f32; 4]) -> [f32; 4] {
        let x1 = r[0].max(outer[0]);
        let y1 = r[1].max(outer[1]);
        let x2 = (r[0] + r[2].max(0.0)).min(outer[0] + outer[2]).max(x1);
        let y2 = (r[1] + r[3].max(0.0)).min(outer[1] + outer[3]).max(y1);
        [x1, y1, x2 - x1, y2 - y1]
    }

    pub fn rect_inside(inner: [f32; 4], outer: [f32; 4]) -> bool {
        inner[2] >= -EPS
            && inner[3] >= -EPS
            && inner[0] >= outer[0] - EPS
            && inner[1] >= outer[1] - EPS
            && inner[0] + inner[2] <= outer[0] + outer[2] + EPS
            && inner[1] + inner[3] <= outer[1] + outer[3] + EPS
    }

    pub fn panel_inside_screen(panel: [f32; 4], sw: f32, sh: f32) -> bool {
        panel[2] > 0.0
            && panel[3] > 0.0
            && panel[0] >= MARGIN - EPS
            && panel[1] >= MARGIN - EPS
            && panel[0] + panel[2] <= sw - MARGIN + EPS
            && panel[1] + panel[3] <= sh - MARGIN + EPS
    }
}

pub mod plaques {
    use super::PlaqueSide;

    pub const FADE_MS: f32 = 280.0;
    pub const HEIGHT: f32 = 42.0;
    pub const WIDTH: f32 = 240.0;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum Phase {
        In(f32),
        Hold,
        Out(f32),
        Done,
    }

    pub fn phase(elapsed_ms: f32, duration_ms: f32, fade_ms: f32) -> Phase {
        if !elapsed_ms.is_finite() || elapsed_ms < 0.0 {
            return Phase::In(0.0);
        }
        let fade = fade_ms.max(1.0);
        let hold = duration_ms.max(0.0);
        if elapsed_ms < fade {
            Phase::In((elapsed_ms / fade).clamp(0.0, 1.0))
        } else if elapsed_ms < fade + hold {
            Phase::Hold
        } else if elapsed_ms < fade + hold + fade {
            Phase::Out((1.0 - (elapsed_ms - fade - hold) / fade).clamp(0.0, 1.0))
        } else {
            Phase::Done
        }
    }

    pub fn offset(phase: Phase, side: PlaqueSide) -> f32 {
        let t = match phase {
            Phase::In(t) | Phase::Out(t) => t,
            Phase::Hold => 1.0,
            Phase::Done => 0.0,
        };
        let eased = t * t * (3.0 - 2.0 * t);
        let travel = HEIGHT + 8.0;
        match side {
            PlaqueSide::Top => -travel * (1.0 - eased),
            PlaqueSide::Bottom => travel * (1.0 - eased),
        }
    }

    pub fn plaque_rect(sw: f32, sh: f32, side: PlaqueSide, phase: Phase) -> [f32; 4] {
        let w = WIDTH.min((sw - 16.0).max(80.0));
        let x = ((sw - w) * 0.5).max(8.0);
        let off = offset(phase, side);
        let y = match side {
            PlaqueSide::Top => 10.0 + off,
            PlaqueSide::Bottom => (sh - HEIGHT - 10.0) + off,
        };
        [x, y, w, HEIGHT]
    }
}

pub mod persist {
    use super::{Filter, FrameKind, PlaqueSide, ViewMode};

    pub const FILE_NAME: &str = "advplus.cfg";

    #[derive(Clone, Debug, PartialEq)]
    pub struct Prefs {
        pub plaques: bool,
        pub duration_s: u8,
        pub show_task: bool,
        pub show_goal: bool,
        pub show_challenge: bool,
        pub show_recipe: bool,
        pub position: PlaqueSide,
        pub view: ViewMode,
        pub filter: Filter,
    }

    impl Default for Prefs {
        fn default() -> Self {
            Self {
                plaques: true,
                duration_s: 5,
                show_task: true,
                show_goal: true,
                show_challenge: true,
                show_recipe: false,
                position: PlaqueSide::Top,
                view: ViewMode::Tree,
                filter: Filter::All,
            }
        }
    }

    impl Prefs {
        pub fn shows_frame(&self, frame: FrameKind) -> bool {
            match frame {
                FrameKind::Task => self.show_task,
                FrameKind::Goal => self.show_goal,
                FrameKind::Challenge => self.show_challenge,
            }
        }

        pub fn duration_ms(&self) -> f32 {
            (self.duration_s.clamp(2, 12) as f32) * 1000.0
        }

        pub fn cycle_duration(&mut self) {
            self.duration_s = match self.duration_s {
                2 | 3 => 5,
                5 => 8,
                _ => 3,
            };
        }
    }

    pub fn dump_prefs(p: &Prefs) -> String {
        format!(
            "format=1\nplaques={}\nduration={}\nshow_task={}\nshow_goal={}\nshow_challenge={}\nshow_recipe={}\nposition={}\nview={}\nfilter={}\n",
            yn(p.plaques),
            p.duration_s,
            yn(p.show_task),
            yn(p.show_goal),
            yn(p.show_challenge),
            yn(p.show_recipe),
            p.position.as_name(),
            p.view.as_name(),
            p.filter.as_name(),
        )
    }

    pub fn load_prefs(text: &str) -> Prefs {
        let mut p = Prefs::default();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                continue;
            };
            match k {
                "plaques" => p.plaques = truthy(v),
                "duration" => {
                    if let Ok(n) = v.parse::<u8>() {
                        p.duration_s = n.clamp(2, 12);
                    }
                }
                "show_task" => p.show_task = truthy(v),
                "show_goal" => p.show_goal = truthy(v),
                "show_challenge" => p.show_challenge = truthy(v),
                "show_recipe" => p.show_recipe = truthy(v),
                "position" => p.position = PlaqueSide::from_name(v),
                "view" => p.view = ViewMode::from_name(v),
                "filter" => p.filter = Filter::from_name(v),
                _ => {}
            }
        }
        p
    }

    fn yn(v: bool) -> &'static str {
        if v { "true" } else { "false" }
    }

    fn truthy(v: &str) -> bool {
        matches!(v.trim(), "1" | "true" | "yes" | "on")
    }
}

pub use persist::Prefs;

pub mod close {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Destination {
        Pause,
        Playing,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Toggle {
        Open,
        Close,
    }

    pub fn apply_take_over(claim: bool, want_close: bool) -> (bool, bool) {
        (claim || !want_close, want_close)
    }

    pub fn decide_close(claimed: bool, want_close: bool, return_to_pause: bool) -> Option<Destination> {
        if claimed && want_close {
            Some(if return_to_pause {
                Destination::Pause
            } else {
                Destination::Playing
            })
        } else {
            None
        }
    }

    pub fn toggle_from_claimed(claimed: bool) -> Toggle {
        if claimed {
            Toggle::Close
        } else {
            Toggle::Open
        }
    }

    pub fn should_drop_claim(claimed: bool, screen_is_adv: bool) -> bool {
        claimed && !screen_is_adv
    }

    pub fn auto_return_to_pause(already_claimed: bool, current: bool) -> bool {
        !already_claimed || current
    }
}

pub fn is_recipe_id(id: &str) -> bool {
    let id = id.to_ascii_lowercase();
    id.contains("/recipes/") || id.contains(":recipes/")
}
