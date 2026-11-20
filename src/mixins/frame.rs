// SPDX-License-Identifier: MIT
fn advplus_frame(game: &mut crate::game::Game<'_>, elapsed: std::time::Duration) {
    crate::game::native_hooks::client_tick_post::advplus::on_frame(game, elapsed);
}
