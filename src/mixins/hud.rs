// SPDX-License-Identifier: MIT
fn advplus_hud(game: &crate::game::Game<'_>, batch: &mut sunrise_client::hud::HudBatch) {
    crate::game::native_hooks::client_tick_post::advplus_panel::paint_plaques(game, batch);
}
