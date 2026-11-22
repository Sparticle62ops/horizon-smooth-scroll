# smooth-scroll (1:1 port)

Exact 1:1 Rust translation of [Smooth Scrolling](https://modrinth.com/mod/smooth-scroll)
(upstream Java: SmajloSlovakian/Minecraft-Smooth-Scrolling @639d45b), which makes
scrolling in lists, inventories and the chat smooth instead of jumping slot to slot.

Upstream copyright (C) SmajloSlovakian, licensed under GPL-3.0-only:
https://www.gnu.org/licenses/gpl-3.0.txt

The Rust translation carries the same GPL-3.0-only license and credits the
upstream author in the LICENSE shipped alongside. It is a Horizon native mod built on the current SDK (Horizon f00125f61f794aaa3d4e950d301f353b35b6ee3d,
Horizon Mixins e59da9094f0c2fdda6c627fe9eff22dec6a906f0, mod API 0.2). `src/lib.rs` is the real
crate root and mod entry: the original's config fields are registered as Horizon settings with the
same defaults and ranges (`Options > Mods`, labels in `assets/smooth_scroll/lang`), and
`/smoothscroll` (short form `/ss`) reads and writes them. Honest status: the scrolling itself is
not wired yet - the client mod API has no mouse-wheel / scroll-event surface for container,
creative-screen, chat or entry-list screens, and no text-field cursor surface, so the 1:1 logic
runs its tests and stays unwired until that lands.

Ported by sparticle.
