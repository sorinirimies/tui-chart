# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Features

- Add ChartStack example, tape and gallery entry ([92c42dc](https://github.com/sorinirimies/tui-chart/commit/92c42dc1f6e964cdc107e41583d5edb0de76b716))
- Funnel, Bullet, BubbleChart; faster line/area rendering; force-push just recipes ([4d613a6](https://github.com/sorinirimies/tui-chart/commit/4d613a6e75e6425f7a68308699b19826fa99e2ba))

### Documentation

- Add chart gallery to README and regenerate demo GIFs ([c4c886e](https://github.com/sorinirimies/tui-chart/commit/c4c886e70a1f2d0421761924092e90b12b1d952f))
- Drop duplicate showcase GIF from README ([4125b98](https://github.com/sorinirimies/tui-chart/commit/4125b9845593e555ead6538a018939b597626137))

### Miscellaneous

- **justfile:** Skip unconfigured remotes in *-all recipes ([e5cd0fa](https://github.com/sorinirimies/tui-chart/commit/e5cd0fa5f448e286ebcf40f1cf1949d1a10fb4f7))
- **justfile:** Push/pull current branch instead of hardcoded main ([000270e](https://github.com/sorinirimies/tui-chart/commit/000270e47b453fa03b1fe06118827c2fd820da16))
- Remove duplicate showcase tape/gif; vhs-showcase builds tui_chart.gif ([b69dc71](https://github.com/sorinirimies/tui-chart/commit/b69dc710c093c94d4ede5fbcfbd45bd44ad12565))
- Alias push-all-force ([0886ec2](https://github.com/sorinirimies/tui-chart/commit/0886ec2ab53bdb9c3d59b8a715dec3a88baf2e82))
- **just:** Align push/force/sync/release recipes with sibling repos ([7cc8d31](https://github.com/sorinirimies/tui-chart/commit/7cc8d31802b220351388b8192c13aaa7e59792be))

## [0.1.1](https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.1) - 2026-10-14

### Features

- Tui-chart widgets, examples, tapes, docs ([90fa8cd](https://github.com/sorinirimies/tui-chart/commit/90fa8cdf17fd16b03ce251fcf53706fba919f06e))
- Sparkline, gauge, radar, boxplot, waterfall, gantt, depth, renko, treemap ([c4b7658](https://github.com/sorinirimies/tui-chart/commit/c4b7658c131f7a68d9acd20f3878fce7b8e7f7af))
- Annotations, crosshair, log/spline/error bars, indicators, stream, time axis, stack ([7f0ca48](https://github.com/sorinirimies/tui-chart/commit/7f0ca483568cb67b21791fd542e70b57b8e41f60))
- Themes, styling, serde feature, theme-switching examples, docs, negative bars, KDE ([8ea79b9](https://github.com/sorinirimies/tui-chart/commit/8ea79b969e7b97675177beb0e156ab8108a627cd))
- Braille candles/bars, Marker everywhere, sextant, overlap policy; restyle tapes like tui-rating ([fc55baf](https://github.com/sorinirimies/tui-chart/commit/fc55baf6a821e346f619b54c7d9cfd71c9e43d85))
- Macros/architecture, robustness tests, fuzz, audit+deny workflows, nushell tests ([cc90e1f](https://github.com/sorinirimies/tui-chart/commit/cc90e1ff1c4c57e018f98d11b05df904ace8ee27))
- Theme::from_base for app-provided themes ([eb97986](https://github.com/sorinirimies/tui-chart/commit/eb979869b49ddaa6fb70b6c478c7c8a278370563))
- Shared theming via tui-nuances ([0bb5253](https://github.com/sorinirimies/tui-chart/commit/0bb5253ee0308b42399a5d43cd9f83d503cd3b1d))
- Make theming an optional "theme" feature ([c8e32f9](https://github.com/sorinirimies/tui-chart/commit/c8e32f9f483d5c296fe49e3f5af4a12a1878e761))

### Bug Fixes

- Doc code blocks in lib.rs ([b808a60](https://github.com/sorinirimies/tui-chart/commit/b808a6077df7964d9054378bb51e4c176484ad30))

### Documentation

- Generated demo GIFs, 2-row help footer ([5ecc16e](https://github.com/sorinirimies/tui-chart/commit/5ecc16e9c3499a807d9581e93d8bb39baf9914c5))
- Re-record demos showing results only; reference tui-nuances version ([8d1e90b](https://github.com/sorinirimies/tui-chart/commit/8d1e90b7adf12e5fb62bdda16f0e820fbeef77da))

### CI

- Prebuilt Nushell, cargo-deny + cargo-audit security workflow ([4893c9a](https://github.com/sorinirimies/tui-chart/commit/4893c9ade7614534f97a00a3a237602b5fc3b702))

### Miscellaneous

- Stub new chart modules ([322f1ff](https://github.com/sorinirimies/tui-chart/commit/322f1ff34252181d28130789a9fe21cddd873ab8))
- Remove gitea migration script ([25f9954](https://github.com/sorinirimies/tui-chart/commit/25f995436a1d40800c90b01137ad619e6f8918ae))
- Remove gitea migration script ([606acdf](https://github.com/sorinirimies/tui-chart/commit/606acdf790bb36d6e9035074a87105f28a02b364))
- Depend on tui-nuances 0.1.5 ([67ab26a](https://github.com/sorinirimies/tui-chart/commit/67ab26acbc1368065511d327f82bcf133cba853a))
- Depend on tui-nuances 0.1.6 ([8927821](https://github.com/sorinirimies/tui-chart/commit/89278213e38ec0b99efab66b73724508f10e610b))
- Depend on tui-nuances 0.1.6 from crates.io ([4069a83](https://github.com/sorinirimies/tui-chart/commit/4069a8395f6b99831381fe6b0fc9110255a67494))


[Unreleased]: https://github.com/sorinirimies/tui-chart/compare/v0.1.1...HEAD

[0.1.1]: https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.1


