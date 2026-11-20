// SPDX-License-Identifier: MIT
fn advplus_key(game: &crate::game::Game<'_>, key: &str, action: mod_api::KeyAction) {
    crate::game::native_hooks::client_tick_post::advplus::on_advancements_key(game, key, action == mod_api::KeyAction::Press);
}
