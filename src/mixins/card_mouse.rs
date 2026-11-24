// SPDX-License-Identifier: MIT
// The mouse buttons on Advancements+'s settings card.
fn advplus_card_options_mouse(
    game: &mut crate::game::Game<'_>,
    pressed: bool,
    right: bool,
) -> ::core::ops::ControlFlow<Result<bool>> {
    if !crate::game::advplus_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::advplus_card::mouse(game, pressed, right).map(|()| true))
}
