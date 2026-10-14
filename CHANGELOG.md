# Changelog
All notable changes to this project will be documented in this file.

## [0.1.1] - 2026-10-14

### Bug Fixes
- Doc code blocks in lib.rs

### Documentation
- Generated demo GIFs, 2-row help footer
- Re-record demos showing results only; reference tui-nuances version

### Features
- Tui-chart widgets, examples, tapes, docs
- Sparkline, gauge, radar, boxplot, waterfall, gantt, depth, renko, treemap
- Annotations, crosshair, log/spline/error bars, indicators, stream, time axis, stack
- Themes, styling, serde feature, theme-switching examples, docs, negative bars, KDE
- Braille candles/bars, Marker everywhere, sextant, overlap policy; restyle tapes like tui-rating
- Macros/architecture, robustness tests, fuzz, audit+deny workflows, nushell tests
- Theme::from_base for app-provided themes
- Shared theming via tui-nuances
- Make theming an optional "theme" feature

### Miscellaneous
- Stub new chart modules
- Remove gitea migration script
- Remove gitea migration script
- Depend on tui-nuances 0.1.5
- Depend on tui-nuances 0.1.6
- Depend on tui-nuances 0.1.6 from crates.io

### Ci
- Prebuilt Nushell, cargo-deny + cargo-audit security workflow

