// SPDX-License-Identifier: MIT
fn advplus_draw_advancements(
    game: &mut crate::game::Game<'_>,
    batch: &mut sunrise_client::hud::HudBatch,
) -> core::ops::ControlFlow<()> {
    crate::game::native_hooks::client_tick_post::advplus::take_over(game);
    sunrise_client::widgets::begin_screen(|| {
        sunrise_client::widgets::ScreenInfo::new(
            "advancements",
            crate::game::screen_info::tr("gui.advancements", &[]),
        )
    });
    let (w, h) = (batch.width, batch.height);
    sunrise_client::ui::menu_background(batch, &game.gui.atlas, 0.0, 0.0, w, h, true);
    core::ops::ControlFlow::Break(())
}
