// SPDX-License-Identifier: MIT
fn advplus_command(game: &mut crate::game::Game<'_>, line: &str) {
    let mut ran = false;
    crate::game::native_hooks::client_tick_post::advplus::on_command(game, line, &mut ran);
}
