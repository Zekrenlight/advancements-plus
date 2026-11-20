// SPDX-License-Identifier: MIT
// The wheel on Advancements+'s settings card.
fn advplus_card_options_scroll(
    game: &mut crate::game::Game<'_>,
    amount: f32,
) -> ::core::ops::ControlFlow<Result<()>> {
    if !crate::game::advplus_card::active(game) {
        return ::core::ops::ControlFlow::Continue(());
    }
    ::core::ops::ControlFlow::Break(crate::game::advplus_card::scroll(game, amount))
}
