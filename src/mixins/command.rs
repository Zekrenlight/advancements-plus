// SPDX-License-Identifier: MIT
fn advplus_command(game: &mut crate::game::Game<'_>, command: &str, ran: &mut bool) -> core::ops::ControlFlow<()> {
    crate::game::native_hooks::client_tick_post::advplus::on_command(game, command, ran);
    if *ran {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
