// SPDX-License-Identifier: MIT
// Advancements+ logic tests: `cargo test --lib`.

use crate::logic as advplus_logic;
use advplus_logic::close::{
    apply_take_over, auto_return_to_pause, decide_close, should_drop_claim, toggle_from_claimed,
    Destination, Toggle,
};
use advplus_logic::filter::{in_tab, matches_query, roots, shown};
use advplus_logic::layout::{compute, panel_inside_screen, rect_inside, MARGIN, TAB_W_MAX};
use advplus_logic::listing::{
    any_nodes_overlap, apply_wheel, bbox_aspect, clamp_pan, clamp_scroll, edges, elbow,
    elbow_child_end, fit_tab_label, frame_shape, frame_size, icon_pad, icon_size, initial_pan,
    last_row_fully_visible, layout_tab, max_fanout, max_scroll, node_rect, opening_camera,
    pause_advancements_rect, row_box, shown_rows, snap_zoom, tab_label, thumb, tidy_layout,
    tidy_layout_ttb, ttb_segments, visible_node_rect, visible_row_rect, wheel_delta, zoom_pan,
    zoom_step, DEFAULT_VIEW, ICON, NODE, Orient, ROW, STEP_X, STEP_Y, ZOOM_STEPS,
};
use advplus_logic::listing::{corridor, seg_rect, LINK_INSET, LINK_THICK};
use advplus_logic::persist::{dump_prefs, load_prefs, Prefs};
use advplus_logic::plaques::{offset, phase, plaque_rect, Phase, FADE_MS, HEIGHT, WIDTH};
use advplus_logic::progress::{criterion_lists, is_done, percent, tab_counts};
use advplus_logic::text::{
    fallback_label, humanize, label_of, looks_like_key, plain_from_json, resolve_display,
    translate_key,
};
use advplus_logic::{is_recipe_id, Entry, Filter, FrameKind, PlaqueSide, ViewMode};
use std::collections::BTreeSet;

fn entry(
    id: &str,
    parent: Option<&str>,
    title: &str,
    obtained: &[&str],
    criteria: &[&str],
    visible: bool,
) -> Entry {
    let obtained: BTreeSet<String> = obtained.iter().map(|s| (*s).to_string()).collect();
    let criteria: Vec<String> = criteria.iter().map(|s| (*s).to_string()).collect();
    let requirements: Vec<Vec<String>> = criteria.iter().map(|c| vec![c.clone()]).collect();
    Entry {
        id: id.into(),
        parent: parent.map(|s| s.to_string()),
        title: title.into(),
        description: format!("{title} description"),
        frame: FrameKind::Task,
        icon: "minecraft:grass_block".into(),
        hidden: false,
        x: 0.0,
        y: 0.0,
        criteria,
        requirements,
        obtained,
        visible,
        recipe: is_recipe_id(id),
    }
}

fn sample_tree() -> Vec<Entry> {
    let mut story = entry(
        "minecraft:story/root",
        None,
        "Minecraft",
        &["crafting_table"],
        &["crafting_table"],
        true,
    );
    story.x = 0.0;
    story.y = 0.0;
    let mut mine = entry(
        "minecraft:story/mine_stone",
        Some("minecraft:story/root"),
        "Getting an Upgrade",
        &[],
        &["stone_pickaxe"],
        true,
    );
    mine.x = 1.0;
    mine.y = 0.0;
    mine.frame = FrameKind::Goal;
    let mut locked = entry(
        "minecraft:story/smelt_iron",
        Some("minecraft:story/mine_stone"),
        "Acquire Hardware",
        &[],
        &["iron"],
        false,
    );
    locked.x = 2.0;
    locked.y = 0.0;
    locked.frame = FrameKind::Challenge;
    let mut nether = entry(
        "minecraft:nether/root",
        None,
        "Nether",
        &[],
        &["entered"],
        true,
    );
    nether.x = 0.0;
    nether.y = 0.0;
    let mut recipe = entry(
        "minecraft:recipes/misc/beacon",
        Some("minecraft:story/root"),
        "Beacon",
        &["beacon"],
        &["beacon"],
        true,
    );
    recipe.recipe = true;
    recipe.x = 1.0;
    recipe.y = 1.0;
    vec![story, mine, locked, nether, recipe]
}

#[test]
fn labels_and_json_text_read_cleanly() {
    assert_eq!(label_of("minecraft:story/mine_stone"), "Mine Stone");
    assert_eq!(humanize("stone_pickaxe"), "Stone Pickaxe");
    assert_eq!(plain_from_json(r#"{"text":"Hello"}"#), "Hello");
    assert_eq!(plain_from_json(r#"{"text":"A","extra":[{"text":"B"}]}"#), "AB");
    assert!(is_recipe_id("minecraft:recipes/misc/beacon"));
    assert!(!is_recipe_id("minecraft:story/root"));
}

#[test]
fn translation_fallback_humanizes_keys_when_lookup_fails() {
    assert!(looks_like_key("advancements.adventure.root.title"));
    assert!(looks_like_key("advancements.adventure.salvage_sherd.description"));
    assert!(!looks_like_key("Adventure"));
    assert!(!looks_like_key("Respecting the Remnants"));
    assert_eq!(
        translate_key(r#"{"translate":"advancements.adventure.root.title"}"#).as_deref(),
        Some("advancements.adventure.root.title")
    );
    assert_eq!(
        fallback_label("advancements.adventure.root.title"),
        "Adventure"
    );
    assert_eq!(
        fallback_label("advancements.adventure.salvage_sherd.title"),
        "Salvage Sherd"
    );
    assert_eq!(
        fallback_label("advancements.adventure.salvage_sherd.description"),
        "Salvage Sherd"
    );
    let lookup = |key: &str| -> Option<String> {
        match key {
            "advancements.adventure.root.title" => Some("Adventure".into()),
            _ => None,
        }
    };
    assert_eq!(
        resolve_display(r#"{"translate":"advancements.adventure.root.title"}"#, &lookup),
        "Adventure"
    );
    assert_eq!(
        resolve_display(
            r#"{"translate":"advancements.adventure.salvage_sherd.title"}"#,
            &lookup
        ),
        "Salvage Sherd"
    );
    // A successful lookup is used as-is; a miss is humanized, not left as the raw key.
    assert_ne!(
        resolve_display(
            r#"{"translate":"advancements.nether.root.title"}"#,
            &lookup
        ),
        "advancements.nether.root.title"
    );
    assert_eq!(
        resolve_display(r#"{"translate":"advancements.nether.root.title"}"#, &lookup),
        "Nether"
    );
    assert_eq!(tab_label("Adventure", 3, 45), "Adventure 3/45");
    assert_eq!(fit_tab_label("Adventure", 3, 45, 20), "Adventure 3/45");
    let fitted = fit_tab_label("Adventure", 3, 45, 10);
    assert!(fitted.ends_with("3/45"), "{fitted}");
    assert!(fitted.chars().count() <= 10, "{fitted}");
}

#[test]
fn progress_counts_percent_and_criteria() {
    let req = vec![vec!["a".into()], vec!["b".into(), "c".into()]];
    let mut have = BTreeSet::new();
    assert!(!is_done(&req, &have));
    have.insert("a".into());
    assert!(!is_done(&req, &have));
    have.insert("c".into());
    assert!(is_done(&req, &have));
    assert_eq!(percent(1, 4), 25);
    assert_eq!(percent(0, 0), 0);
    assert_eq!(percent(3, 2), 100);
    let (done, rest) = criterion_lists(&["a".into(), "b".into()], &have);
    assert_eq!(done, vec!["a".to_string()]);
    assert_eq!(rest, vec!["b".to_string()]);
    let tree = sample_tree();
    let (d, t) = tab_counts(&tree, "minecraft:story/root");
    // story root done, mine and locked not, recipe excluded
    assert_eq!((d, t), (1, 3));
    assert_eq!(percent(d, t), 33);
}

#[test]
fn filter_and_search_hide_the_right_rows() {
    let tree = sample_tree();
    assert_eq!(roots(&tree).len(), 2);
    assert!(matches_query("minecraft:story/mine_stone", "Getting an Upgrade", "upgrade"));
    assert!(matches_query("minecraft:story/mine_stone", "Getting an Upgrade", "mine_stone"));
    assert!(!matches_query("minecraft:story/mine_stone", "Getting an Upgrade", "beacon"));
    assert!(matches_query("minecraft:story/mine_stone", "Getting an Upgrade", "  "));

    let all = shown(&tree, "minecraft:story/root", Filter::All, "");
    let ids: Vec<_> = all.iter().map(|e| e.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "minecraft:story/root",
            "minecraft:story/mine_stone",
            "minecraft:story/smelt_iron"
        ]
    );
    assert!(all.iter().all(|e| !e.recipe));
    assert!(all.iter().all(|e| in_tab(&tree, e, "minecraft:story/root")));

    let done = shown(&tree, "minecraft:story/root", Filter::Done, "");
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].id, "minecraft:story/root");

    let active = shown(&tree, "minecraft:story/root", Filter::Active, "");
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "minecraft:story/mine_stone");

    let locked = shown(&tree, "minecraft:story/root", Filter::Locked, "");
    assert_eq!(locked.len(), 1);
    assert_eq!(locked[0].id, "minecraft:story/smelt_iron");

    let q = shown(&tree, "minecraft:story/root", Filter::All, "hardware");
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].id, "minecraft:story/smelt_iron");

    let nether = shown(&tree, "minecraft:nether/root", Filter::All, "");
    assert_eq!(nether.len(), 1);
    assert_eq!(nether[0].id, "minecraft:nether/root");
}

#[test]
fn prefs_round_trip_and_cycles() {
    let mut p = Prefs::default();
    assert!(p.plaques);
    assert_eq!(p.duration_s, 5);
    assert!(p.show_task && p.show_goal && p.show_challenge);
    assert!(!p.show_recipe);
    assert_eq!(p.position, PlaqueSide::Top);
    assert_eq!(p.view, ViewMode::Tree);
    assert_eq!(p.filter, Filter::All);
    assert_eq!(p.duration_ms(), 5000.0);
    assert!(p.shows_frame(FrameKind::Task));

    p.plaques = false;
    p.cycle_duration();
    assert_eq!(p.duration_s, 8);
    p.cycle_duration();
    assert_eq!(p.duration_s, 3);
    p.show_task = false;
    p.show_recipe = true;
    p.position = p.position.next();
    p.view = p.view.next();
    p.filter = p.filter.next();
    let text = dump_prefs(&p);
    assert!(text.contains("format=1"));
    assert!(text.contains("plaques=false"));
    assert!(text.contains("duration=3"));
    assert!(text.contains("show_recipe=true"));
    assert!(text.contains("position=bottom"));
    assert!(text.contains("view=list"));
    assert!(text.contains("filter=done"));
    let loaded = load_prefs(&text);
    assert_eq!(loaded, p);

    let extras = load_prefs("format=1\nduration=99\nplaques=yes\nposition=sideways\nview=unknown\n");
    assert_eq!(extras.duration_s, 12);
    assert!(extras.plaques);
    assert_eq!(extras.position, PlaqueSide::Top);
    assert_eq!(extras.view, ViewMode::Tree);
}

#[test]
fn close_state_follows_the_statsplus_lessons() {
    assert_eq!(apply_take_over(false, false), (true, false));
    assert_eq!(apply_take_over(true, false), (true, false));
    assert_eq!(apply_take_over(true, true), (true, true));
    assert_eq!(apply_take_over(false, true), (false, true));

    assert_eq!(
        decide_close(true, true, true),
        Some(Destination::Pause)
    );
    assert_eq!(
        decide_close(true, true, false),
        Some(Destination::Playing)
    );
    assert_eq!(decide_close(true, false, true), None);
    assert_eq!(decide_close(false, true, false), None);

    assert_eq!(toggle_from_claimed(false), Toggle::Open);
    assert_eq!(toggle_from_claimed(true), Toggle::Close);
    assert!(should_drop_claim(true, false));
    assert!(!should_drop_claim(true, true));
    assert!(!should_drop_claim(false, false));
    assert!(auto_return_to_pause(false, true));
    assert!(!auto_return_to_pause(true, false));
    assert!(auto_return_to_pause(true, true));
}

#[test]
fn layout_stays_inside_the_panel_at_common_scaled_sizes() {
    const EPS: f32 = 0.05;
    let sizes = [
        (320.0, 240.0),
        (427.0, 240.0),
        (480.0, 270.0),
        (512.0, 288.0),
        (640.0, 360.0),
        (854.0, 480.0),
        (960.0, 540.0),
        (1280.0, 720.0),
        (1920.0, 1080.0),
        (256.0, 200.0),
        (400.0, 180.0),
        (300.0, 500.0),
    ];
    for (sw, sh) in sizes {
        let ui = compute(sw, sh);
        let panel = ui.panel();
        assert!(
            panel_inside_screen(panel, sw, sh),
            "panel {panel:?} leaves screen {sw}x{sh}"
        );
        assert!(panel[0] >= MARGIN - EPS);
        assert!(panel[1] >= MARGIN - EPS);
        assert!(panel[0] + panel[2] <= sw - MARGIN + EPS);
        assert!(panel[1] + panel[3] <= sh - MARGIN + EPS);
        for (name, r) in [
            ("done", ui.done),
            ("tabs", ui.tabs_bar),
            ("body", ui.body),
            ("search", ui.search),
            ("filter", ui.filter),
            ("mode", ui.mode),
            ("view", ui.view),
            ("bar", ui.bar),
        ] {
            assert!(
                rect_inside(r, panel),
                "{name} {r:?} escapes panel {panel:?} at {sw}x{sh}"
            );
            assert!(r[2] > 0.0 && r[3] > 0.0, "{name} empty at {sw}x{sh}: {r:?}");
        }
        // The view sits below the toolbar, so list/tree content cannot share the search row.
        assert!(
            ui.view[1] + 0.01 >= ui.search[1] + ui.search[3]
                || ui.tools_wrapped && ui.view[1] + 0.01 >= ui.filter[1] + ui.filter[3],
            "view overlaps tools at {sw}x{sh}: view={:?} search={:?} filter={:?}",
            ui.view,
            ui.search,
            ui.filter
        );
        let n = 5;
        for i in 0..n {
            let t = ui.tab_rect(i, n);
            assert!(rect_inside(t, panel), "tab {i} {t:?} escapes {panel:?}");
        }
        assert!(ui.view[3] >= 1.0, "no view height at {sw}x{sh}");
    }
}

#[test]
fn list_rows_clip_to_the_viewport() {
    let list = [20.0, 40.0, 200.0, ROW * 3.5];
    let total = 10;
    let view_h = list[3];
    let max = max_scroll(total, view_h, ROW);
    let offsets = [
        0.0,
        8.0,
        16.0,
        ROW * 0.5,
        ROW,
        ROW + 16.0,
        ROW * 1.5,
        max,
        max + 24.0,
        -4.0,
        f32::NAN,
    ];
    for raw in offsets {
        let scroll = clamp_scroll(raw, total, view_h, ROW);
        let (start, end) = shown_rows(scroll, total, view_h, ROW);
        for i in start..end {
            let unclipped = row_box(list, i, scroll, ROW);
            if let Some(vis) = visible_row_rect(list, i, scroll, ROW) {
                assert!(
                    rect_inside(vis, list),
                    "visible row {i} {vis:?} escapes {list:?} at scroll={scroll}"
                );
                assert!(vis[3] <= ROW + 0.05);
            } else {
                let below = unclipped[1] >= list[1] + list[3] - 0.05;
                let above = unclipped[1] + unclipped[3] <= list[1] + 0.05;
                assert!(above || below, "row {i} missing visible rect but overlaps at {scroll}");
            }
        }
        let above = (list[0] + 4.0, list[1] - 1.0);
        for i in start..end {
            if let Some(vis) = visible_row_rect(list, i, scroll, ROW) {
                assert!(
                    !(above.0 >= vis[0]
                        && above.0 < vis[0] + vis[2]
                        && above.1 >= vis[1]
                        && above.1 < vis[1] + vis[3]),
                    "click above the list hit row {i} at scroll={scroll}"
                );
            }
        }
    }
    assert!(last_row_fully_visible(total, view_h, ROW));
    let t = thumb(0.0, total, view_h, view_h, ROW).expect("thumb");
    assert!(t.h >= 16.0 && t.y >= 0.0);
}

#[test]
fn wheel_moves_one_row_per_notch() {
    assert_eq!(wheel_delta(1.0, ROW), ROW);
    assert_eq!(wheel_delta(-1.0, ROW), -ROW);
    // Later rows arrive as dy = -1.
    let next = apply_wheel(0.0, -1.0, 12, ROW * 4.0, ROW);
    assert_eq!(next, ROW);
    let back = apply_wheel(next, 1.0, 12, ROW * 4.0, ROW);
    assert_eq!(back, 0.0);
    let clamped = apply_wheel(0.0, 1.0, 12, ROW * 4.0, ROW);
    assert_eq!(clamped, 0.0);
}

#[test]
fn pan_and_nodes_stay_in_the_tree_viewport() {
    let all = sample_tree();
    let tree: Vec<_> = all
        .iter()
        .filter(|e| in_tab(&all, e, "minecraft:story/root") && !e.recipe)
        .cloned()
        .collect();
    let view = [10.0, 20.0, 240.0, 180.0];
    let origin = clamp_pan([0.0, 0.0], &tree, view, 1.0);
    let far = clamp_pan([10_000.0, -10_000.0], &tree, view, 1.0);
    assert!(far[0].is_finite() && far[1].is_finite());
    assert_ne!(far, [10_000.0, -10_000.0]);
    let zoomed = clamp_pan(origin, &tree, view, 2.0);
    assert!(zoomed[0].is_finite() && zoomed[1].is_finite());
    for e in &tree {
        if let Some(vis) = visible_node_rect(view, origin, 1.0, e.x, e.y) {
            assert!(rect_inside(vis, view), "node {vis:?} escapes {view:?}");
        }
        let r = node_rect(view, origin, 1.0, e.x, e.y);
        assert!((r[2] - NODE).abs() < 0.01);
    }
    // A node panned far away has no visible rect.
    let hidden = visible_node_rect(view, [800.0, 800.0], 1.0, 0.0, 0.0);
    // After clamp it may still be visible; test the unclamped helper path with an extreme pan
    // of a solitary far node.
    let lonely = {
        let mut e = tree[0].clone();
        e.x = 0.0;
        e.y = 0.0;
        vec![e]
    };
    assert!(visible_node_rect(view, [0.0, 0.0], 1.0, lonely[0].x, lonely[0].y).is_some());
    assert!(visible_node_rect(view, [400.0, 400.0], 1.0, lonely[0].x, lonely[0].y).is_none());
    let _ = hidden;
}

#[test]
fn plaque_phases_and_rect_stay_on_screen() {
    assert!(matches!(phase(0.0, 5000.0, FADE_MS), Phase::In(_)));
    assert!(matches!(phase(FADE_MS + 10.0, 5000.0, FADE_MS), Phase::Hold));
    assert!(matches!(phase(FADE_MS + 5000.0 + 10.0, 5000.0, FADE_MS), Phase::Out(_)));
    assert!(matches!(phase(FADE_MS + 5000.0 + FADE_MS, 5000.0, FADE_MS), Phase::Done));
    assert_eq!(offset(Phase::Hold, PlaqueSide::Top), 0.0);
    let top = plaque_rect(640.0, 360.0, PlaqueSide::Top, Phase::Hold);
    assert!((top[2] - WIDTH).abs() < 0.01 || top[2] <= 640.0 - 16.0);
    assert!((top[3] - HEIGHT).abs() < 0.01);
    assert!(top[0] >= 8.0);
    assert!(top[0] + top[2] <= 640.0);
    assert!(top[1] >= 0.0 && top[1] < 360.0);
    let bottom = plaque_rect(640.0, 360.0, PlaqueSide::Bottom, Phase::Hold);
    assert!(bottom[1] + bottom[3] <= 360.0);
    let incoming = plaque_rect(640.0, 360.0, PlaqueSide::Top, Phase::In(0.0));
    assert!(incoming[1] < top[1]);
}

#[test]
fn pause_advancements_button_sits_left_of_stats() {
    let r = pause_advancements_rect(854.0, 480.0, 0);
    assert_eq!(r[2], 98.0);
    assert_eq!(r[3], 20.0);
    // Left column of the 2x2 mid-grid (stats is x0+110).
    assert!(r[0] < 854.0 * 0.5);
}

#[test]
fn horizontal_tidy_layout_does_not_overlap_and_grows_right() {
    let all = sample_tree();
    let mut tree: Vec<_> = all
        .iter()
        .filter(|e| !e.recipe && in_tab(&all, e, "minecraft:story/root"))
        .cloned()
        .collect();
    tidy_layout(&mut tree, "minecraft:story/root");
    let root = tree.iter().find(|e| e.id == "minecraft:story/root").unwrap();
    let mine = tree.iter().find(|e| e.id == "minecraft:story/mine_stone").unwrap();
    let iron = tree.iter().find(|e| e.id == "minecraft:story/smelt_iron").unwrap();
    assert_eq!(root.x, 0.0);
    assert!(mine.x > root.x, "child should sit to the right of the root (got root.x={}, child.x={})", root.x, mine.x);
    assert!(iron.x > mine.x, "grandchild should continue to the right");
    let view = [0.0, 0.0, 400.0, 300.0];
    assert!(
        !any_nodes_overlap(&tree, view, [0.0, 0.0], 1.0),
        "tidy layout produced overlapping nodes"
    );
    // Packet-style vanilla positions (grid units) also must not overlap at our node size.
    let mut vanilla = sample_tree();
    vanilla.retain(|e| !e.recipe);
    // already laid out left-to-right in sample_tree
    let story: Vec<_> = vanilla
        .into_iter()
        .filter(|e| in_tab(&sample_tree(), e, "minecraft:story/root") && !e.recipe)
        .collect();
    assert!(!any_nodes_overlap(&story, view, [0.0, 0.0], 1.0));
    assert!(frame_shape(FrameKind::Task) != frame_shape(FrameKind::Goal));
    assert!(frame_shape(FrameKind::Goal) != frame_shape(FrameKind::Challenge));
    assert!(frame_shape(FrameKind::Task) != frame_shape(FrameKind::Challenge));
}

#[test]
fn edges_only_connect_visible_parent_child_pairs() {
    let tree = sample_tree();
    let story: Vec<_> = tree
        .iter()
        .filter(|e| in_tab(&tree, e, "minecraft:story/root") && !e.recipe)
        .cloned()
        .collect();
    let all = edges(&story);
    assert!(all.contains(&("minecraft:story/root".into(), "minecraft:story/mine_stone".into())));
    assert!(all.contains(&("minecraft:story/mine_stone".into(), "minecraft:story/smelt_iron".into())));
    assert_eq!(all.len(), 2);
    // No edge to a node that isn't shown, and no skip-to-grandparent when the parent is hidden.
    let hidden_mid: Vec<_> = story
        .iter()
        .filter(|e| e.id != "minecraft:story/mine_stone")
        .cloned()
        .collect();
    let e = edges(&hidden_mid);
    assert!(e.is_empty(), "expected no dangling or skip edges, got {e:?}");
    // A lone root has no edges.
    let only_root: Vec<_> = story
        .iter()
        .filter(|e| e.id == "minecraft:story/root")
        .cloned()
        .collect();
    assert!(edges(&only_root).is_empty());
}

#[test]
fn elbow_runs_from_parent_right_to_child_left() {
    let parent = [10.0, 20.0, 26.0, 26.0];
    let child = [60.0, 40.0, 26.0, 26.0];
    let pts = elbow(parent, child);
    let end = elbow_child_end(child);
    // Just inside the parent's right edge (it runs flush into the frame), mid-height.
    assert!((pts[0][0] - (36.0 - LINK_INSET)).abs() < 0.01);
    assert!((pts[0][1] - 33.0).abs() < 0.01);
    // Mid x sits between parent right and child left.
    assert!(pts[1][0] > pts[0][0] && pts[1][0] < child[0]);
    assert!((pts[1][1] - pts[0][1]).abs() < 0.01);
    // Vertical drop/rise at mid x, then into the child's left edge.
    assert!((pts[2][0] - pts[1][0]).abs() < 0.01);
    assert!((pts[2][1] - end[1]).abs() < 0.01);
    assert!((end[0] - (child[0] + LINK_INSET)).abs() < 0.01);
}

#[test]
fn opening_pan_fits_or_keeps_the_root_in_view() {
    let mut tree: Vec<_> = sample_tree()
        .into_iter()
        .filter(|e| !e.recipe && in_tab(&sample_tree(), e, "minecraft:story/root"))
        .collect();
    tidy_layout(&mut tree, "minecraft:story/root");
    let root = tree.iter().find(|e| e.id == "minecraft:story/root").unwrap();
    let view = [0.0, 0.0, 320.0, 200.0];
    let pan = initial_pan(&tree, Some(root), view, 1.0);
    let r = node_rect(view, pan, 1.0, root.x, root.y);
    // The root's box intersects the view.
    assert!(r[0] + r[2] > view[0] && r[0] < view[0] + view[2]);
    assert!(r[1] + r[3] > view[1] && r[1] < view[1] + view[3]);
    // Children extend to the right of the root.
    let child = tree.iter().find(|e| e.id == "minecraft:story/mine_stone").unwrap();
    let c = node_rect(view, pan, 1.0, child.x, child.y);
    assert!(c[0] > r[0], "child should be drawn to the right of the root");
    let _ = STEP_X;
}

#[test]
fn tidy_layout_is_left_to_right_for_children_and_grandchildren() {
    // A single-child chain stays on one row; siblings share a column and spread on y.
    let mut chain = vec![
        entry("root", None, "Root", &["a"], &["a"], true),
        entry("child", Some("root"), "Child", &["a"], &["a"], true),
        entry("grand", Some("child"), "Grand", &["a"], &["a"], true),
    ];
    tidy_layout(&mut chain, "root");
    let root = chain.iter().find(|e| e.id == "root").unwrap();
    let child = chain.iter().find(|e| e.id == "child").unwrap();
    let grand = chain.iter().find(|e| e.id == "grand").unwrap();
    assert!(child.x > root.x, "child.x ({}) should be greater than parent.x ({})", child.x, root.x);
    assert!(grand.x > child.x, "grandchild.x ({}) should be greater than child.x ({})", grand.x, child.x);
    assert!((child.y - root.y).abs() < 0.01, "single child should share the parent's row, got parent.y={} child.y={}", root.y, child.y);
    assert!((grand.y - child.y).abs() < 0.01, "single grandchild should share the child's row, got child.y={} grand.y={}", child.y, grand.y);

    let mut tree = vec![
        entry("root", None, "Root", &["a"], &["a"], true),
        entry("child", Some("root"), "Child", &["a"], &["a"], true),
        entry("grand", Some("child"), "Grand", &["a"], &["a"], true),
        entry("sib", Some("root"), "Sib", &["a"], &["a"], true),
    ];
    tidy_layout(&mut tree, "root");
    let child = tree.iter().find(|e| e.id == "child").unwrap();
    let grand = tree.iter().find(|e| e.id == "grand").unwrap();
    let sib = tree.iter().find(|e| e.id == "sib").unwrap();
    let root = tree.iter().find(|e| e.id == "root").unwrap();
    assert!(child.x > root.x && sib.x > root.x);
    assert!((child.x - sib.x).abs() < 0.01, "siblings should share a column");
    assert!((child.y - sib.y).abs() > 0.01, "siblings should be spread on y");
    assert!(grand.x > child.x);
    assert!((grand.y - child.y).abs() < 0.01);
}


#[test]
fn adaptive_layout_keeps_chain_heavy_trees_left_to_right() {
    let mut chain = vec![
        entry("root", None, "Root", &["a"], &["a"], true),
        entry("a", Some("root"), "A", &["a"], &["a"], true),
        entry("b", Some("a"), "B", &["a"], &["a"], true),
        entry("c", Some("b"), "C", &["a"], &["a"], true),
        entry("d", Some("c"), "D", &["a"], &["a"], true),
        entry("side", Some("b"), "Side", &["a"], &["a"], true),
    ];
    let view = DEFAULT_VIEW;
    let orient = layout_tab(&mut chain, "root", view);
    assert_eq!(orient, Orient::Ltr, "chain-heavy story-like trees should stay LTR");
    assert!(max_fanout(&chain, "root") <= 6);
    let root = chain.iter().find(|e| e.id == "root").unwrap();
    let a = chain.iter().find(|e| e.id == "a").unwrap();
    let d = chain.iter().find(|e| e.id == "d").unwrap();
    assert!(a.x > root.x);
    assert!(d.x > a.x);
    assert!(
        !any_nodes_overlap(&chain, view, [0.0, 0.0], 1.0),
        "LTR chain layout overlapped"
    );
    let e = edges(&chain);
    assert!(e.contains(&("root".into(), "a".into())));
    assert!(e.contains(&("b".into(), "side".into())));
    assert_eq!(e.len(), 5, "edges must be parent-child only, got {e:?}");

    let mut story: Vec<_> = sample_tree()
        .into_iter()
        .filter(|e| !e.recipe && in_tab(&sample_tree(), e, "minecraft:story/root"))
        .collect();
    let orient = layout_tab(&mut story, "minecraft:story/root", view);
    assert_eq!(orient, Orient::Ltr);
    assert!(!any_nodes_overlap(&story, view, [0.0, 0.0], 1.0));
}

fn adventure_like() -> Vec<Entry> {
    let mut tree = vec![entry("root", None, "Adventure", &["a"], &["a"], true)];
    for i in 0..40 {
        let id = format!("c{i:02}");
        tree.push(entry(&id, Some("root"), &id, &["a"], &["a"], true));
    }
    tree.push(entry("c00a", Some("c00"), "c00a", &["a"], &["a"], true));
    tree.push(entry("c00b", Some("c00"), "c00b", &["a"], &["a"], true));
    tree.push(entry("c07a", Some("c07"), "c07a", &["a"], &["a"], true));
    tree
}

#[test]
fn adaptive_layout_sends_high_fanout_trees_top_down_and_wraps() {
    let mut tree = adventure_like();
    let view = DEFAULT_VIEW;
    assert_eq!(max_fanout(&tree, "root"), 40);
    let orient = layout_tab(&mut tree, "root", view);
    assert_eq!(orient, Orient::Ttb, "Adventure-like fanout should pick TTB");
    let root = tree.iter().find(|e| e.id == "root").unwrap();
    let child = tree.iter().find(|e| e.id == "c00").unwrap();
    let grand = tree.iter().find(|e| e.id == "c00a").unwrap();
    assert!(
        child.y > root.y + 0.4,
        "children should sit below the root, got root.y={} child.y={}",
        root.y,
        child.y
    );
    assert!(
        grand.y > child.y + 0.4,
        "grandchildren should continue downward"
    );
    // Children of the root must occupy more than one x column (spread out)
    // and more than one y row (wrapped).
    let kids: Vec<_> = tree.iter().filter(|e| e.parent.as_deref() == Some("root")).collect();
    let xs: Vec<i32> = kids.iter().map(|e| (e.x * 10.0).round() as i32).collect();
    let ys: Vec<i32> = kids.iter().map(|e| (e.y * 10.0).round() as i32).collect();
    let uniq_x = xs.iter().copied().collect::<std::collections::BTreeSet<_>>().len();
    let uniq_y = ys.iter().copied().collect::<std::collections::BTreeSet<_>>().len();
    assert!(uniq_x > 4, "wrapped children should spread across x, got {uniq_x} columns");
    assert!(uniq_y > 1, "forty children should wrap into multiple rows, got {uniq_y} rows");
    let aspect = bbox_aspect(&tree);
    assert!(
        (0.5..=3.0).contains(&aspect),
        "Adventure-like bbox aspect should stay near the viewport, got {aspect}"
    );
    assert!(
        !any_nodes_overlap(&tree, view, [0.0, 0.0], 1.0),
        "TTB layout overlapped"
    );
    // Also check a pure TTB call and LTR call independently have no overlaps.
    let mut ltr = adventure_like();
    tidy_layout(&mut ltr, "root");
    assert!(!any_nodes_overlap(&ltr, view, [0.0, 0.0], 1.0));
    let mut ttb = adventure_like();
    tidy_layout_ttb(&mut ttb, "root", view);
    assert!(!any_nodes_overlap(&ttb, view, [0.0, 0.0], 1.0));
    let e = edges(&tree);
    assert_eq!(e.len(), 43, "40 children + 3 grandchildren, got {e:?}");
    assert!(e.iter().all(|(p, c)| tree.iter().any(|n| n.id == *c && n.parent.as_deref() == Some(p.as_str()))));
}

#[test]
fn ttb_connectors_are_vertical_elbows_and_wrap_buses() {
    let parent = [100.0, 10.0, 26.0, 26.0];
    let children = vec![[80.0, 60.0, 26.0, 26.0], [140.0, 60.0, 26.0, 26.0]];
    let segs = ttb_segments(parent, &children);
    // Just inside the parent's bottom (113, 36) down to a mid bus, then a drop into each child.
    let pb = [113.0, 36.0 - LINK_INSET];
    assert!(
        segs.iter().any(|s| (s[0][0] - pb[0]).abs() < 0.1 && (s[0][1] - pb[1]).abs() < 0.1
            || (s[1][0] - pb[0]).abs() < 0.1 && (s[1][1] - pb[1]).abs() < 0.1),
        "expected a segment at the parent bottom, got {segs:?}"
    );
    for c in &children {
        let top = [c[0] + c[2] * 0.5, c[1] + LINK_INSET];
        assert!(
            segs.iter().any(|s| {
                ((s[0][0] - top[0]).abs() < 0.1 && (s[0][1] - top[1]).abs() < 0.1)
                    || ((s[1][0] - top[0]).abs() < 0.1 && (s[1][1] - top[1]).abs() < 0.1)
            }),
            "expected a drop to child top {top:?}, got {segs:?}"
        );
    }

    // Wrapped rows share a left spine and per-row buses.
    let row2 = vec![[80.0, 110.0, 26.0, 26.0], [140.0, 110.0, 26.0, 26.0]];
    let mut both = children.clone();
    both.extend(row2);
    let segs = ttb_segments(parent, &both);
    let leftmost = 80.0;
    // Halfway across the corridor left of the leftmost child (16 px at 100 %).
    let spine = leftmost - corridor(1.0) * 0.5;
    assert!(
        segs.iter().any(|s| (s[0][0] - spine).abs() < 0.1 || (s[1][0] - spine).abs() < 0.1),
        "wrapped children should share a left bus/spine, got {segs:?}"
    );
}

#[test]
fn opening_camera_prefers_100_percent_then_75_and_fits() {
    // A short chain fits at 100%.
    let mut chain = vec![
        entry("root", None, "Root", &["a"], &["a"], true),
        entry("a", Some("root"), "A", &["a"], &["a"], true),
        entry("b", Some("a"), "B", &["a"], &["a"], true),
    ];
    let view = DEFAULT_VIEW;
    let orient = layout_tab(&mut chain, "root", view);
    let root = chain.iter().find(|e| e.id == "root").cloned();
    let (pan, zoom) = opening_camera(&chain, root.as_ref(), view, orient);
    assert!((zoom - 1.0).abs() < 0.001, "small tree should open at 100%, got {zoom}");
    let r = node_rect(view, pan, zoom, root.as_ref().unwrap().x, root.as_ref().unwrap().y);
    assert!(r[0] + r[2] > view[0] && r[0] < view[0] + view[2]);
    assert!(r[1] + r[3] > view[1] && r[1] < view[1] + view[3]);

    // A huge Adventure-like tree should fall back to 75% if it cannot fit at 100%.
    let mut tree = adventure_like();
    let orient = layout_tab(&mut tree, "root", view);
    let root = tree.iter().find(|e| e.id == "root").cloned();
    let (pan, zoom) = opening_camera(&tree, root.as_ref(), view, orient);
    assert!(zoom == 1.0 || zoom == 0.75, "opening zoom must be 100% or 75%, got {zoom}");
    let r = node_rect(view, pan, zoom, root.as_ref().unwrap().x, root.as_ref().unwrap().y);
    assert!(r[0] + r[2] > view[0] && r[0] < view[0] + view[2]);
    assert!(r[1] + r[3] > view[1] && r[1] < view[1] + view[3]);
    let _ = STEP_Y;
}

#[test]
fn wheel_zoom_steps_around_the_cursor_and_clamps() {
    assert!((snap_zoom(1.0) - 1.0).abs() < 0.001);
    assert!((zoom_step(1.0, 1.0) - 1.25).abs() < 0.001);
    assert!((zoom_step(1.25, 1.0) - 1.5).abs() < 0.001);
    assert!((zoom_step(1.5, 1.0) - 1.5).abs() < 0.001);
    assert!((zoom_step(1.0, -1.0) - 0.75).abs() < 0.001);
    assert!((zoom_step(0.75, -1.0) - 0.75).abs() < 0.001);
    assert!((zoom_step(0.5, -1.0) - 0.75).abs() < 0.001);
    assert!((snap_zoom(0.7) - 0.75).abs() < 0.001);
    assert!((snap_zoom(2.0) - 1.5).abs() < 0.001);
    assert_eq!(ZOOM_STEPS, [0.75, 1.0, 1.25, 1.5]);

    let view = [0.0, 0.0, 200.0, 100.0];
    let cursor = [150.0, 25.0];
    let pan = [10.0, 5.0];
    let z0 = 1.0;
    let z1 = 1.25;
    let new_pan = zoom_pan(pan, z0, z1, view, cursor);
    let cx = view[0] + view[2] * 0.5;
    let cy = view[1] + view[3] * 0.5;
    let world_x = pan[0] + (cursor[0] - cx) / z0;
    let world_y = pan[1] + (cursor[1] - cy) / z0;
    let world_x2 = new_pan[0] + (cursor[0] - cx) / z1;
    let world_y2 = new_pan[1] + (cursor[1] - cy) / z1;
    assert!((world_x - world_x2).abs() < 0.001);
    assert!((world_y - world_y2).abs() < 0.001);
}

#[test]
fn frame_and_icon_scale_together_at_every_zoom() {
    for z in ZOOM_STEPS {
        let frame = frame_size(z);
        let icon = icon_size(z);
        let pad = icon_pad(z);
        assert!((frame - (NODE * z).round()).abs() < 0.001);
        assert!((icon - ICON * z).abs() < 0.001);
        assert!(icon + pad * 2.0 <= frame + 0.001, "icon overflows frame at {z}");
        assert!(pad > 0.0, "expected padding inside the frame at {z}");
        let r = node_rect([0.0, 0.0, 400.0, 300.0], [0.0, 0.0], z, 0.0, 0.0);
        assert!((r[2] - frame).abs() < 0.001);
        assert!((r[3] - frame).abs() < 0.001);
    }
}

#[test]
fn single_tab_width_is_capped() {
    let ui = compute(960.0, 540.0);
    // One visible root plus Settings must not stretch across the bar.
    let t0 = ui.tab_rect(0, 2);
    let t1 = ui.tab_rect(1, 2);
    assert!(t0[2] <= TAB_W_MAX + 0.05, "tab 0 width {} exceeds cap {}", t0[2], TAB_W_MAX);
    assert!(t1[2] <= TAB_W_MAX + 0.05, "tab 1 width {} exceeds cap {}", t1[2], TAB_W_MAX);
    assert!(
        t0[2] + t1[2] + 4.0 < ui.tabs_bar[2],
        "capped tabs should not fill the bar: {}+{} vs {}",
        t0[2],
        t1[2],
        ui.tabs_bar[2]
    );
    let lone = ui.tab_rect(0, 1);
    assert!(lone[2] <= TAB_W_MAX + 0.05, "a single tab must not stretch, got {}", lone[2]);
}

// ---- Branch lines: every connector runs flush into its frames and its buses sit in clear
// corridors, for normal and Challenge frames, left-to-right and top-down, at every zoom. ----

/// A Challenge frame's outer ring: the frame grown 2 px on each side (`paint_node_frame`).
fn challenge_outer(r: [f32; 4]) -> [f32; 4] {
    [r[0] - 2.0, r[1] - 2.0, r[2] + 4.0, r[3] + 4.0]
}

fn right(r: [f32; 4]) -> f32 {
    r[0] + r[2]
}
fn bottom(r: [f32; 4]) -> f32 {
    r[1] + r[3]
}
fn whole(v: f32) -> bool {
    (v - v.round()).abs() < 1e-4
}

#[test]
fn branch_corridor_is_clear_at_every_zoom() {
    assert!(STEP_X - NODE >= 12.0 && STEP_Y - NODE >= 12.0, "frames need a 12-16 px corridor");
    for &z in &ZOOM_STEPS {
        // A bus (2 px) plus the Challenge ring (2 px) on both sides still leaves open space.
        assert!(corridor(z) >= LINK_THICK + 2.0 * 2.0 + 4.0, "corridor at {z} is {}", corridor(z));
    }
}

#[test]
fn node_rects_and_segments_sit_on_whole_pixels() {
    let view = [3.0, 7.0, 333.0, 211.0];
    for &z in &ZOOM_STEPS {
        for pan in [[0.0, 0.0], [13.37, -7.21], [101.5, 44.25]] {
            for (x, y) in [(0.0, 0.0), (1.0, 0.5), (2.0, 3.0), (-1.5, 2.25)] {
                let r = node_rect(view, pan, z, x, y);
                assert!(r.iter().all(|v| whole(*v)), "node rect {r:?} at zoom {z}");
            }
            let a = node_rect(view, pan, z, 0.0, 0.0);
            let b = node_rect(view, pan, z, 1.0, 1.0);
            let pts = elbow(a, b);
            let end = elbow_child_end(b);
            for (p, q) in [(pts[0], pts[1]), (pts[1], pts[2]), (pts[2], end)] {
                let s = seg_rect(p, q);
                assert!(s.iter().all(|v| whole(*v)), "segment {s:?} at zoom {z}");
                assert!((s[2] - LINK_THICK).abs() < 1e-4 || (s[3] - LINK_THICK).abs() < 1e-4);
            }
        }
    }
}

#[test]
fn ltr_branches_attach_flush_to_normal_and_challenge_frames() {
    let view = DEFAULT_VIEW;
    for &z in &ZOOM_STEPS {
        for (cx, cy) in [(1.0, 0.0), (1.0, 1.0), (1.0, -2.0), (1.0, 0.5)] {
            let parent = node_rect(view, [0.0, 0.0], z, 0.0, 0.0);
            let child = node_rect(view, [0.0, 0.0], z, cx, cy);
            let pts = elbow(parent, child);
            let end = elbow_child_end(child);
            let first = seg_rect(pts[0], pts[1]);
            let bus = seg_rect(pts[1], pts[2]);
            let last = seg_rect(pts[2], end);
            // Runs into the parent's frame and into the child's frame: no floating gap.
            assert!(first[0] < right(parent) - 1.0, "first leg starts inside the parent: {first:?} vs {parent:?}");
            assert!(first[1] >= parent[1] && bottom(first) <= bottom(parent));
            assert!(right(last) > child[0] + 1.0, "last leg ends inside the child: {last:?} vs {child:?}");
            assert!(last[1] >= child[1] && bottom(last) <= bottom(child));
            // The vertical bus is in the corridor, clear of both frames and both Challenge rings.
            let (po, co) = (challenge_outer(parent), challenge_outer(child));
            assert!(bus[0] > right(po), "bus {bus:?} clear of the parent ring {po:?} at {z}");
            assert!(right(bus) < co[0], "bus {bus:?} clear of the child ring {co:?} at {z}");
            // Centred in the corridor (within a pixel of rounding).
            let mid = (right(parent) + child[0]) * 0.5;
            assert!((bus[0] + bus[2] * 0.5 - mid).abs() <= 1.0);
            // Each leg crosses the Challenge ring completely (no gap between ring and line).
            assert!(first[0] < right(parent) && right(first) > right(po));
            assert!(last[0] < co[0] && right(last) > child[0]);
            // The corners meet: legs and bus overlap where they join.
            assert!(right(first) >= bus[0] + bus[2] - 0.01 && first[0] <= bus[0]);
            assert!(last[0] <= bus[0] && right(last) >= right(bus) - 0.01);
        }
    }
}

#[test]
fn ttb_branches_attach_flush_and_buses_sit_in_corridors() {
    let view = DEFAULT_VIEW;
    for &z in &ZOOM_STEPS {
        let parent = node_rect(view, [0.0, 0.0], z, 1.0, 0.0);
        let row1: Vec<[f32; 4]> = (0..3).map(|i| node_rect(view, [0.0, 0.0], z, i as f32, 1.0)).collect();
        // One row: a drop from the parent, a bus, a drop into each child.
        let segs = ttb_segments(parent, &row1);
        let rects: Vec<[f32; 4]> = segs.iter().map(|s| seg_rect(s[0], s[1])).collect();
        let down = rects
            .iter()
            .find(|r| r[1] < bottom(parent) && r[0] < right(parent) && right(**r) > parent[0])
            .expect("a drop out of the parent");
        assert!(down[1] < bottom(parent) - 1.0, "parent drop starts inside the frame at {z}");
        for c in &row1 {
            let drop = rects
                .iter()
                .find(|r| r[2] <= LINK_THICK + 0.01 && r[0] >= c[0] && right(**r) <= right(*c) && bottom(**r) > c[1])
                .unwrap_or_else(|| panic!("a drop into child {c:?} at {z}: {rects:?}"));
            assert!(bottom(*drop) > c[1] + 1.0, "drop runs into the child's frame");
            assert!(bottom(*drop) < bottom(*c));
        }
        let bus = rects
            .iter()
            .find(|r| r[3] <= LINK_THICK + 0.01 && r[2] > LINK_THICK + 0.01)
            .expect("a horizontal bus");
        assert!(bus[1] > bottom(challenge_outer(parent)), "bus clear of the parent ring at {z}");
        for c in &row1 {
            assert!(bottom(*bus) < challenge_outer(*c)[1], "bus clear of the child rings at {z}");
        }

        // Wrapped rows: a left spine in the corridor and a bus in each row's corridor.
        let row2: Vec<[f32; 4]> = (0..3).map(|i| node_rect(view, [0.0, 0.0], z, i as f32, 2.0)).collect();
        let mut all = row1.clone();
        all.extend(row2.iter().copied());
        let segs = ttb_segments(parent, &all);
        let rects: Vec<[f32; 4]> = segs.iter().map(|s| seg_rect(s[0], s[1])).collect();
        let leftmost = all.iter().map(|c| c[0]).fold(f32::INFINITY, f32::min);
        let spine = rects
            .iter()
            .filter(|r| r[2] <= LINK_THICK + 0.01 && right(**r) <= leftmost)
            .copied()
            .collect::<Vec<_>>();
        assert!(!spine.is_empty(), "a spine left of the children at {z}");
        for s in &spine {
            assert!(right(*s) < leftmost - 2.0, "spine clear of the Challenge rings at {z}");
            assert!(s[0] > leftmost - corridor(z) + 1.0, "spine stays in the corridor at {z}");
        }
        let row2_top = row2.iter().map(|c| c[1]).fold(f32::INFINITY, f32::min);
        let row1_bottom = row1.iter().map(|c| bottom(*c)).fold(f32::NEG_INFINITY, f32::max);
        let bus2 = rects
            .iter()
            .find(|r| r[3] <= LINK_THICK + 0.01 && r[1] > row1_bottom && bottom(**r) < row2_top)
            .unwrap_or_else(|| panic!("a bus in the corridor above row 2 at {z}: {rects:?}"));
        assert!(bus2[1] > row1_bottom + 2.0 && bottom(*bus2) < row2_top - 2.0);
        for c in &row2 {
            assert!(
                rects.iter().any(|r| r[0] >= c[0] && right(*r) <= right(*c) && bottom(*r) > c[1] + 1.0 && r[1] < c[1]),
                "a drop into wrapped child {c:?} at {z}"
            );
        }
    }
}
