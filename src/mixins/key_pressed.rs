// SPDX-License-Identifier: MIT
fn advplus_key_pressed(game: &crate::game::Game<'_>, screen: &mod_api::ScreenView<'_>, key: &str, canceled: &mut bool) {
    if crate::game::native_hooks::client_tick_post::advplus_panel::key(game, screen, key) {
        *canceled = true;
    }
}
