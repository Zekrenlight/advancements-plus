// SPDX-License-Identifier: MIT
fn advplus_char(game: &crate::game::Game<'_>, screen: &mod_api::ScreenView<'_>, c: char, canceled: &mut bool) {
    if crate::game::native_hooks::client_tick_post::advplus_panel::typed(game, screen, c) {
        *canceled = true;
    }
}
