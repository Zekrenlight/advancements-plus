// SPDX-License-Identifier: MIT
fn advplus_toast(game: &mut crate::game::Game<'_>, toast: &Toast) -> core::ops::ControlFlow<()> {
    // Sunrise's toast carries no advancement id: find the advancement whose
    // frame and title the toast shows (the description line is its title).
    let taken = if toast.is_recipe() {
        crate::game::native_hooks::client_tick_post::advplus::enqueue_plaque(game, "recipe", "")
    } else {
        let shown = toast.description.as_ref().map(|t| t.plain()).unwrap_or_default();
        let id = game.adv.tree.list.iter().find_map(|a| {
            let d = a.display.as_ref()?;
            (d.frame == toast.frame && text_of(&d.title).plain() == shown).then(|| a.id.clone())
        });
        match id {
            Some(id) => crate::game::native_hooks::client_tick_post::advplus::enqueue_plaque(game, "advancement", &id),
            None => false,
        }
    };
    if taken {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
