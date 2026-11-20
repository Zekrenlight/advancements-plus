// SPDX-License-Identifier: MIT
fn advplus_tick(game: &mut crate::game::Game<'_>) {
    crate::game::native_hooks::client_tick_post::advplus::close_if_asked(game);
    if !game.native_window() {
        return;
    }
    crate::game::native_hooks::client_tick_post::advplus::ensure_ready(game);
    if crate::game::native_hooks::client_tick_post::advplus::claimed() {
        crate::game::native_hooks::client_tick_post::advplus::sync(game);
    }
}
