// SPDX-License-Identifier: MIT
// Dragging on Advancements+'s settings card: its sliders and scrollbar.
fn advplus_card_options_drag(game: &mut crate::game::Game<'_>) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::advplus_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::advplus_card::drag(game))
}
