# Advancements+

Version 1.0.0 · Minecraft Java 26.3 · Sunrise API 0.2 · Mod SDK `9425651` · MIT licence

Advancements+ is written in Rust for Sunrise. It needs no Java or Fabric.
It replaces the vanilla Advancements screen with a full-window tree (or a list), and replaces advancement toasts with short on-screen plaques.

This is an independent implementation. It makes no network requests. It has no author API, news, credits or announcements. It does not collect, hash or store machine or hardware identifiers.

## Features

- Click **Advancements** on the pause menu, or press the existing Advancements key (L by default), to open Advancements+. `/advplus panel` does the same. There is no extra keybind.
- The stock Advancements widgets are not drawn. The world stays blurred and dimmed, and only the Advancements+ card appears.
- Tabs for each advancement root, labelled with the translated title and that tab’s progress (for example “Adventure 3/45”). Labels truncate and clip so they stay inside the tab, and tab width is capped so a single visible tab is not stretched. A Settings tab lives beside them.
- A large tree view that fills the card, laid out from parent links. Chain-heavy tabs (such as Minecraft) stay left-to-right; wide tabs with many siblings of the root (Adventure, Husbandry, Nether) switch to a top-down tree so the root sits at the top and children branch down and out, wrapping a very wide level into a grid with a shared bus line. The opening view prefers 100% zoom, or 75% if that is what fits, and centers the tree when it can. Drag to pan; the mouse wheel over the tree steps zoom through 75%, 100%, 125%, and 150% around the cursor. Node frames and item icons scale together so the icon stays inside the frame. Elbow lines join a child to its visible parent only, drawn under the icons.
- Nodes have distinct frame shapes: a square for tasks, a clipped octagon for goals, and a double frame with pips for challenges, with clearer done/undone colouring.
- A list mode alternative, with one-row-per-notch scrolling and a scrollbar that shares the same viewport as painting and hit-testing.
- Search by title or id, and a filter that cycles All / Done / Active / Locked.
- Titles, descriptions, tab names, and plaques are resolved through the game’s language lookup. If a translation is missing, the key is humanized (for example `advancements.adventure.root.title` becomes “Adventure”). Hover tooltips show the title, description, frame type, and criteria already completed or still remaining.
- Advancement toasts are replaced with a short top- or bottom-of-screen plaque (icon, title, frame-type accent, slide in and out). Vanilla toast sounds are used. Recipes can be hidden. Plaques can be turned off, in which case the game’s own toasts remain.
- Settings on the card persist in `advplus.cfg` in `sunrise-mods/.sunrise-mod-data/advplus/`. They cover
  - plaques on or off
  - duration
  - which frame types to show
  - recipe plaques
  - plaque position
  - the default tree or list view

  The earlier build's mods-list button is gone, because Sunrise has no hook for it.
- The card stays inside the window with margins. The search field, toolbar, and tabs shrink, wrap, or abbreviate to fit, and the tree or list uses the space that remains. Tree and list drawing is clipped to the viewport so content cannot paint over the toolbar.
- **Done** or Escape (or the Advancements key again) closes the card on the next frame, including while an unpublished singleplayer world is paused.


## Look and settings

Advancements+ uses the shared Sunrise dark look: a `#0F1118` card with a 2 px accent edge, tabs and
charcoal chips. The accent is Sunrise orange `#F9801D` at first; change it with **Accent colour**
in the settings card (eight presets, or Reset).
The Advancements screen, its tree and the plaques use the same look.

The settings card opens from SunnyMods' **Settings** button, from Sunrise's own Mods list, or
from the mod's settings key. **Key binds › Rebind** switches to Sunrise's classic page to change keys.

## Install (local)

You need the Sunrise launcher with native mods allowed. Use either way below.

- **From the file:** copy `advplus-1.0.0.smod` into the instance's `sunrise-native-mods/` folder, next to `options.txt`. Turn it on in the launcher's **Mods** page, or add the line `mod.advplus:true` to `sunrise-mods.txt`. The launcher then compiles it for that instance (`sunrise-launcher native-build`).
- **From GitHub:** once the repository is public, use **Add from GitHub** on the launcher's Mods page, or run `sunrise-launcher mods add-github https://github.com/Zekrenlight/advancements-plus --trust-native-code`.

Native mods run with full access to your computer. Read the source before you install one.

Then open the pause menu and click **Advancements**, press L in the world, or type `/advplus panel`.

## Commands

There is no dedicated keybind. The existing Advancements key (L) and the pause-menu Advancements button open this screen.

Chat commands use the prefix `/advplus`: `panel` and `help`. `/advplus panel` toggles the screen closed if it is already open.

Settings are stored in `advplus.cfg` in `sunrise-mods/.sunrise-mod-data/advplus/` (format version 1). There is no automatic import of another mod's config.

## Menu

The screen is a centered tabbed card, in the same family as Hookline's panel (navy, parchment text, gold accents — not a copy of another advancements menu). It shrinks and reflows to stay on screen:

- One tab per advancement root, labelled with the translated title and that tab’s progress (for example “Adventure 3/45”)
- **Settings** — plaques, duration, frame types, recipes, plaque position, and the default view
- A search field, All/Done/Active/Locked, and Tree/List above the view
- Tree nodes (or list rows) use a warm parchment accent for tasks, a cool blue for goals, and violet for challenges

**Done** or Escape closes the screen and returns to the pause menu if you opened it from there, or to the world otherwise. `/advplus panel` toggles the screen closed if it is already open.


## How it fits into Sunrise

Advancements+ takes over the game's own Advancements screen and advancement toasts. It reads the advancement tree, its layout and progress. So the game side is native:

- **Support modules**, placed beside `native_hooks/client_tick_post.rs`:
  - `src/native/session.rs`
  - `src/native/panel.rs`
- **Cancellable `HEAD` mixins**:
  - `Game::draw_advancements`: hides the stock widgets.
  - `Game::add_advancement_toast`: turns an advancement or recipe toast into a plaque. The toast carries no advancement id, so the plaque is matched to its advancement by frame and title.
- **`HEAD` mixins** on these Sunrise hook points:
  - `native_command` (cancellable)
  - `native_client_tick_post`
  - `native_render_frame_pre`
  - `native_input_key`
  - `native_render_gui_post`, which draws the plaques
  - `native_screen_render_post`
  - the screen mouse, wheel, key and char `_pre` hooks

The rules in `src/logic.rs` are plain Rust and are tested with `cargo test --lib`. A mod with a mixin works only on the exact build it was made for.


## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/advplus.smod
```

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## Compatibility

| Piece | Pin |
| --- | --- |
| Minecraft Java | 26.3 |
| Sunrise API | 0.2 |
| Mod SDK (`[native].sunrise`) | `94256510f74e6843754e0a93fd9e4da7e45b1056` (`9425651`, 19 Nov 2026) |
| Mixins (`[native].mixins`) | `e59da9094f0c2fdda6c627fe9eff22dec6a906f0` |

Advancements+ works on its own and alongside the other Sunrise mods by Zekrenlight. Its settings-card
hooks act only on its own Options page, so several mods can be installed together without any
conflicting replacements. A mod with native hooks works only on the exact SDK build it was made for.

## Building

```sh
cargo sunrise-mod           # fetches the pinned SDK into .sunrise/ (gitignored)
cargo sunrise-mod check     # type-checks against the pinned SDK
cargo test --lib            # unit tests, no game needed
cargo sunrise-mod build     # target/sunrise-mod/advplus.smod
```

To move to a newer SDK, run `cargo sunrise-mod update --sunrise <build>` (it rewrites the
`sunrise` and `mixins` lines in `sunrise-mod.toml`), then check and rebuild.

Install the `.smod` with sunrise-launcher (**Add from file**), or point **Add from GitHub** at
this repository. Native mods run with full access to your computer; read the source first.

## License

MIT © 2026 Zekrenlight. See [LICENSE](LICENSE).
