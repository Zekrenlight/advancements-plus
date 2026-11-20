// SPDX-License-Identifier: MIT
//! Advancements+ for Sunrise: a clearer Advancements screen and plaques.
//!
//! This crate holds the pure rules (`logic`: tree layout, filters, search,
//! listing math, plaque timing and prefs). The game side, which replaces the
//! Advancements screen and its toasts, is the support modules in
//! `src/native/` and the HEAD mixins in `src/mixins/`, listed in
//! `sunrise-mod.toml`.

pub mod logic;
#[cfg(test)]
mod rules_tests;

use mod_api::Hooks;

#[derive(Default)]
struct AdvancementsPlus;

#[mod_api::sunrise_mod]
impl Hooks for AdvancementsPlus {
    fn register(&self, registrar: &mut dyn mod_api::Registrar, _cx: &mod_api::RegistrationContext) {
        // The accent colour of advplus's cards (Sunrise orange by default).
        let _ = registrar.register_setting(
            mod_api::SettingSpec::color("accent", "advplus.setting.accent", 0xF9801D)
                .tooltip("advplus.setting.accent.tooltip"),
        );
    }
}
