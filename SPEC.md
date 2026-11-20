# Advancements+ — feature specification

This document describes intended in-game behavior only. It is not source code, and it does not name other mods’ types, files, setting keys, or screen layout numbers.

Advancements+ is an optional advancements browser and toast replacement for Sunrise (Minecraft Java, game 26.3). When it is enabled, the Advancements screen and the existing Advancements key open this implementation’s card. The stock Advancements widgets are not drawn: the world stays blurred and dimmed, and only the Advancements+ card appears. Advancement toasts are replaced with short on-screen plaques unless plaques are turned off. Everything below is the intended behavior of this implementation.

## Opening and closing

- On the pause menu, a left click on the Advancements button (the same place the stock button occupies, including when an extra server-links, additions, or splitscreen row shifts the grid) claims the upcoming advancements screen and lets the game open it. The stock Advancements draw is cancelled: the usual world blur and in-world menu dim remain, then Advancements+ paints its card and consumes input so leftover stock widgets cannot appear or receive clicks.
- There is no dedicated keybind. The existing Advancements key (L by default) claims the screen while the player is in the world; the game then opens the Advancements screen as usual, and this implementation takes it over. `/advplus panel` opens the card, or closes it if it is already open.
- Chat commands start with `/advplus`. Useful subcommands are `panel` and `help`. Unknown input prints a short help blurb.
- **Done** on the card, Escape, or the Advancements key again, closes the card on the next frame (the same hook Sunrise runs every redraw, including while an unpublished singleplayer world is paused). The ordinary client tick hook does not run in that paused state, so close must not wait for a tick. Closing sets the game’s screen to the pause menu if the player arrived from there, or to the world with mouse capture restored otherwise. This does not use the stock Advancements Done widget, which is not drawn. If the claim is still set after the game has left the Advancements screen, it is dropped.
- When the mod is off, Sunrise does not build it in, so the game’s own Advancements screen and toasts are left alone.

## Advancement data

- Opening the card asks the game to load the advancement tree it already holds. Painted nodes follow that tree and the progress the client already received. Layout is computed here from parent links; server-sent display positions are not used for placement. This implementation does not invent a private network protocol.
- A node is treated as visible when the server sent positions for the tree, or when the game’s ordinary visibility walk would show it (completed nodes, their ancestors, and nearby unhidden descendants). Recipe advancements are omitted from the tree and list, and appear as plaques only when recipe plaques are enabled.
- Tabs are the displayed advancement roots. Each tab is labelled with the translated root title and how many of its visible, non-recipe descendants (including the root) are complete, as a count like “Adventure 3/45”. Labels truncate and clip so they stay inside the tab, and tab width is capped so a single visible tab (plus Settings) is not stretched across the bar.
- Titles, descriptions, tab names, and plaques are resolved through the game’s language lookup (the same component-to-text path the stock advancements screen uses). If a translation is missing, the key is humanized instead of shown raw.

## Tree, list, search, and filter

- The default view is a tree laid out from parent links. Both a left-to-right layout (depth is the column, siblings pack down the row) and a top-down layout (depth is the row, siblings pack across, wrapping a very wide sibling group into a grid under the parent) are computed; the one whose bounding box better matches the viewport aspect is used, and a tree whose widest sibling group is larger than six always goes top-down. Chain-heavy tabs such as Minecraft stay left-to-right; wide tabs such as Adventure, Husbandry, and Nether put the root at the top and branch down and out. Connectors are horizontal elbows from a parent’s right edge to a child’s left edge in a left-to-right tree, or vertical elbows (parent bottom to a shared mid-line bus, then down to each child’s top) in a top-down tree. Nodes sit on a 42 px grid, so neighbouring 26 px frames leave a 16 px corridor (12 px at 75%); every bus and spine runs down the middle of a corridor, clear of normal frames and of the 2 px outer ring of Challenge frames. Each connector runs 2 px into the frames it joins, so it meets them flush with no gap, and node frames and connector segments are placed on whole pixels. The opening view prefers 100% zoom, or 75% if the tree does not fit at 100%, and centers the tree when it can; otherwise the root stays in the left side (left-to-right) or near the top (top-down). Pan is clamped to the content bounds. Nodes do not overlap at the default zoom. Drag to pan. The mouse wheel over the tree steps zoom through 75%, 100%, 125%, and 150% around the cursor, then re-clamps the pan. There is no zoom toolbar button. Node frames and item icons scale together (frame 26 × zoom, icon 16 × zoom, centered) so the icon stays inside the frame with padding at every step.
- Connectors are drawn beneath the icons. A line is drawn only when both the advancement and its actual parent are visible in the current filter; there are no lines to hidden nodes and no unconnected branches.
- Frame type chooses a distinct silhouette as well as an accent: a square for tasks, a clipped octagon for goals, and a double frame with edge pips for challenges (warm parchment, cool blue, and violet). Completed nodes use a brighter fill and a stronger accent; incomplete ones are muted.
- List mode shows the same tab’s advancements as a vertical list, one row per item, with a scrollbar. The mouse wheel moves one row per notch. Scroll offset, the painted rows, and hit-testing all share one viewport height, and every row can be reached, including the last one. Rows are clipped to the list viewport so they cannot draw over the toolbar.
- Tree nodes are likewise clipped to the tree viewport.
- A search field matches against an advancement’s title or id. A filter chip cycles All, Done, Active (started or visible but not finished), and Locked. The default view (tree or list) and the last filter are stored in the config when the player changes them.
- Hovering a visible node or row shows a tooltip with the title, a short description when the advancement is visible or complete, the frame type, a progress count, and lists of completed and remaining criteria. Locked, not-yet-visible nodes show a locked label instead of the real title and description.

## Plaques

- When a new advancement toast would be queued, this implementation may cancel it and show a short plaque instead. The plaque sits at the top or bottom of the scaled HUD, with the advancement’s icon, title, and a frame-type accent, and slides in and out over a short interval. Only one plaque is shown at a time; later ones wait.
- Vanilla toast sounds (`ui.toast.in`, `ui.toast.out`, and `ui.toast.challenge_complete` for challenges) are played at the matching moments. No other sound assets are added.
- Settings can turn plaques off (the game’s own toasts then appear as usual), change how long a plaque is held, hide task, goal, or challenge plaques, hide recipe toasts, and choose top or bottom placement.

## Layout and input

- The card is laid out each frame from the scaled screen size. It stays inside the window with a margin on every side, and shrinks if the window or GUI scale leaves less room than the preferred size. The search field and toolbar chips shrink or wrap, tabs abbreviate, and the tree or list uses the space that remains. Painting and hit-testing share the same rectangles.
- Mouse input on the card is consumed so leftover stock widgets cannot receive clicks. Dragging on the tree pans; dragging the list scrollbar moves the list. Closing is requested from Done, Escape, or the Advancements key, and applied on the frame hook as described above.

## Persistence

- Preferences are stored in `advplus.cfg` under the mod data directory (format version 1). The keys are this implementation’s own: whether plaques are enabled, hold duration in seconds, which frame types and recipes to show, plaque position, default view, and filter. There is no automatic import of another mod’s config, and nothing is uploaded.

## Out of scope for 1.0.0

- Custom plaque textures or custom toast sounds
- Importing or converting another mod’s configuration
- Editing, granting, or revoking advancements
- A dedicated keybind in addition to the existing Advancements key
