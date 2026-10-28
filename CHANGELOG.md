# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.1](https://github.com/sorinirimies/tui-chart/releases/tag/v0.6.1) - 2026-10-28

### Features

- Enable the tui-nuances theme feature by default ([19a16fe](https://github.com/sorinirimies/tui-chart/commit/19a16fed02279629ae4cd9a7a0e10a3fc3ba5e02))

### CI

- Automerge green PRs (workflow_run) + dependabot for actions ([fffaae7](https://github.com/sorinirimies/tui-chart/commit/fffaae7ecc9aac78ef8952e1f8d8f521291c075b))
- Automerge major dependabot bumps too ([97461e2](https://github.com/sorinirimies/tui-chart/commit/97461e200284a0d50ed9616de0f8e6ea960881b5))
- **gitea:** Nightly workflow-action bumps ([e02a030](https://github.com/sorinirimies/tui-chart/commit/e02a0306bebf948501db48333f08781a4d2243d2))
- **deps:** Bump clechasseur/rs-clippy-check from 4 to 5 (#3) ([9047323](https://github.com/sorinirimies/tui-chart/commit/9047323e869e134af3d1920429b10c469ef8371c))
- **deps:** Bump actions/checkout from 4 to 7 (#5) ([3f3a9ff](https://github.com/sorinirimies/tui-chart/commit/3f3a9ffe6b8fa342dc12c105842f7e099816614a))
- **deps:** Bump softprops/action-gh-release from 2 to 3 (#6) ([d1d01a0](https://github.com/sorinirimies/tui-chart/commit/d1d01a0fab60a9666c3aff6a001750824d297f21))
- **deps:** Bump actions/upload-artifact from 4 to 7 (#2) ([53c0077](https://github.com/sorinirimies/tui-chart/commit/53c0077fefed04ff0c60f0f87fd6f31d16f413d3))
- Automerge uses GH_PAT so workflow-file PRs can merge ([0f4003d](https://github.com/sorinirimies/tui-chart/commit/0f4003d6b1c05c99bcafc46be443118e50d3664c))
- Stop dependabot bumping the MSRV toolchain pin ([98c5919](https://github.com/sorinirimies/tui-chart/commit/98c5919019e755c3708b15f8f89e3e3cce85af53))

### Miscellaneous

- Bump tui-nuances to 0.1.7 ([0a169a1](https://github.com/sorinirimies/tui-chart/commit/0a169a172ceb510df78c09a3f6ef4e18056fb3d0))
- **deps:** Update Cargo.lock ([d2eee1c](https://github.com/sorinirimies/tui-chart/commit/d2eee1cd61399b65fcb429c6a9b77eb361d3b2c8))
- **deps:** Bump tui-nuances to 0.1.10 ([671d55b](https://github.com/sorinirimies/tui-chart/commit/671d55bc4d90c464481752e10ed6f9ab0ee703c3))

## [0.6.0](https://github.com/sorinirimies/tui-chart/releases/tag/v0.6.0) - 2026-10-25

### Features

- Pie hits, instrument hits, hidden funnel/radial, View zoom/pan, FxExt, Gauge::zones, chamfered treemap; coverage CI; interactivity guide ([f06e461](https://github.com/sorinirimies/tui-chart/commit/f06e4612932e8e6fa6f22c47bfb4bd5e953ce514))

## [0.5.1](https://github.com/sorinirimies/tui-chart/releases/tag/v0.5.1) - 2026-10-25

### Bug Fixes

- **effects:** Glow skips cells without a background; re-record cyber demo ([63995af](https://github.com/sorinirimies/tui-chart/commit/63995afe281d71b5a7b3e65932ab2fad311f1ef4))

### Testing

- **effects:** Glow needs an explicit background ([bbdc2e0](https://github.com/sorinirimies/tui-chart/commit/bbdc2e0d8ccd883a9b7fa482baffda6c0fd2a5de))

### Miscellaneous

- Reset version before bump ([d7f8900](https://github.com/sorinirimies/tui-chart/commit/d7f8900d0e4db52a9e214ca422c49010088409c7))

## [0.5.0](https://github.com/sorinirimies/tui-chart/releases/tag/v0.5.0) - 2026-10-24

### Features

- Mouse interactivity for every chart (HitMap, Interaction, tooltips, legend toggling) ([7521260](https://github.com/sorinirimies/tui-chart/commit/7521260cbfae62e758b226753d76d7d0c3e97e14))

## [0.4.0](https://github.com/sorinirimies/tui-chart/releases/tag/v0.4.0) - 2026-10-24

### Features

- **BREAKING** Remove shader backdrop; calmer cyber demo and sparser oscilloscope graticule ([38e4d1b](https://github.com/sorinirimies/tui-chart/commit/38e4d1ba4117584f7801bd789462a2960c469434))

### Documentation

- Drop duplicate cyber gif ([490e2f3](https://github.com/sorinirimies/tui-chart/commit/490e2f3ad884def671721cf161b5e85b9415b8cd))
- Move effects section above License ([b927d24](https://github.com/sorinirimies/tui-chart/commit/b927d24463c798a3f904dfe974597d9ecf2cb69f))
- Document shader feature ([5cd5e9f](https://github.com/sorinirimies/tui-chart/commit/5cd5e9f4ed57337ef3a26d9baf81b1daf6228e1f))

## [0.3.0](https://github.com/sorinirimies/tui-chart/releases/tag/v0.3.0) - 2026-10-24

### Features

- Fx glow/scanlines/chamfer frames and optional GPU ShaderBackdrop (tui-shader) ([649fe0e](https://github.com/sorinirimies/tui-chart/commit/649fe0eccc343ad8181295d6e5185023b21f8f61))

## [0.2.0](https://github.com/sorinirimies/tui-chart/releases/tag/v0.2.0) - 2026-10-24

### Features

- Re-export tui-piechart (default 'piechart' feature) so one dependency covers charts and pies ([0dfc667](https://github.com/sorinirimies/tui-chart/commit/0dfc66755eb9b5e35f68a535ff98e8a3f04f2f64))
- **examples:** Pie chart demo, tape and gif; embed in README ([d2dbd7e](https://github.com/sorinirimies/tui-chart/commit/d2dbd7e8ed646a9927550547f426da9e7f42b557))
- **gauge:** 12 new styles (dial, meter, reticle, ring, chevron, smooth, dots, led, battery, level, thermometer, signal); add RadialBar and RingStack ([38e5cba](https://github.com/sorinirimies/tui-chart/commit/38e5cba46380523b25e17d2c109a4bfb9a7acc73))
- Oscilloscope, Spectrum, DotMatrix, RadarScope and HexMap instrument widgets ([3d674a9](https://github.com/sorinirimies/tui-chart/commit/3d674a9f2bc997db8dd14c46d4bbbb6d5b3565b5))

### Miscellaneous

- **just:** Release recipes skip the bump confirmation prompt ([26846b5](https://github.com/sorinirimies/tui-chart/commit/26846b5a750b56542605953d15e2619e0082fabf))

## [0.1.5](https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.5) - 2026-10-22

### Bug Fixes

- **calendar:** Robust labels, flat/NaN/inf handling, zero_empty, wider cells, legend sizing ([85eef5c](https://github.com/sorinirimies/tui-chart/commit/85eef5c14113489256456aeb42e2c3808f0ff700))

## [0.1.4](https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.4) - 2026-10-22

### Refactor

- **paired_bars:** **BREAKING** Rename Comparison to PairedBars; feat: label_color ([8ab9e58](https://github.com/sorinirimies/tui-chart/commit/8ab9e58b97d2b99a301c2ffc343f71a6547279ec))

## [0.1.3](https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.3) - 2026-10-22

### Features

- Lollipop, slope, calendar, violin, rose, stacked-area and sankey charts; boxplot mean/notches/jitter; Monochrome accessibility wrapper; golden tests, docs examples ([3fd18e1](https://github.com/sorinirimies/tui-chart/commit/3fd18e162aa5ad57e6d10240edae0f1c351f5003))
- **comparison:** Comparison, before/after bar pairs per measure, inline or stacked, with a legend ([186be45](https://github.com/sorinirimies/tui-chart/commit/186be452b778a8c4ceeb45be1599eb4417f77dac))

### Bug Fixes

- **annotation:** Labels of close vertical lines go on separate rows instead of printing over each other, and over the lines ([7f87d1f](https://github.com/sorinirimies/tui-chart/commit/7f87d1f3d561c44d10e03c471f750a3e3bd9a5c0))

### Documentation

- **comparison:** Demo gif ([4800427](https://github.com/sorinirimies/tui-chart/commit/4800427c390321eadc007175b78c0f68c6ca58b5))

### CI

- **deps:** Nightly update pushes to main directly like tui-spinner (no manual-merge PR) ([1bd98a5](https://github.com/sorinirimies/tui-chart/commit/1bd98a53208ae5082ec381969b595e0520eab362))

## [0.1.2](https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.2) - 2026-10-18

### Features

- Add ChartStack example, tape and gallery entry ([92c42dc](https://github.com/sorinirimies/tui-chart/commit/92c42dc1f6e964cdc107e41583d5edb0de76b716))
- Funnel, Bullet, BubbleChart; faster line/area rendering; force-push just recipes ([4d613a6](https://github.com/sorinirimies/tui-chart/commit/4d613a6e75e6425f7a68308699b19826fa99e2ba))
- **gauge,funnel:** Redesign gauge (solid tracks, rich arc, segmented style) and tapering funnel with drop-off ([cb3bf03](https://github.com/sorinirimies/tui-chart/commit/cb3bf03db3cb2eae83d06071f4eb71b9fa7f8db4))
- **sparkline,bullet,histogram,depth:** Area/win-loss sparklines with smoothing, 3-row bullets with eighth precision, histogram mean/median markers, gradient depth fills ([a2e28b2](https://github.com/sorinirimies/tui-chart/commit/a2e28b2592effe72be99defc1113cfd6d3250a6c))
- **heatmap,radar,treemap,waterfall:** Stretch-to-fit heatmap, filled radar with ring values and legend, treemap gutters and group headers, waterfall grid ([7dd66b3](https://github.com/sorinirimies/tui-chart/commit/7dd66b3b02430f0b2f29d33c6858bebdafe06fc3))

### Bug Fixes

- **scripts:** Bump skips duplicate clippy/test (--skip-checks), quiet nu test runner ([718f1ba](https://github.com/sorinirimies/tui-chart/commit/718f1ba95e94c19872c1e8d4000b79a54a1de752))
- **scripts:** Ask() reads piped stdin instead of blocking on the tty; tests run with empty stdin ([e9c6841](https://github.com/sorinirimies/tui-chart/commit/e9c6841f7c73ae749429c60c067d96b00b2e0551))

### Documentation

- Add chart gallery to README and regenerate demo GIFs ([c4c886e](https://github.com/sorinirimies/tui-chart/commit/c4c886e70a1f2d0421761924092e90b12b1d952f))
- Drop duplicate showcase GIF from README ([4125b98](https://github.com/sorinirimies/tui-chart/commit/4125b9845593e555ead6538a018939b597626137))

### Testing

- Cover funnel, bullet, bubble options; docs: README feature table; regenerate GIFs ([8895c75](https://github.com/sorinirimies/tui-chart/commit/8895c75e1c7ac7297216ec7842641b0f82984c73))

### Miscellaneous

- **justfile:** Skip unconfigured remotes in *-all recipes ([e5cd0fa](https://github.com/sorinirimies/tui-chart/commit/e5cd0fa5f448e286ebcf40f1cf1949d1a10fb4f7))
- **justfile:** Push/pull current branch instead of hardcoded main ([000270e](https://github.com/sorinirimies/tui-chart/commit/000270e47b453fa03b1fe06118827c2fd820da16))
- Remove duplicate showcase tape/gif; vhs-showcase builds tui_chart.gif ([b69dc71](https://github.com/sorinirimies/tui-chart/commit/b69dc710c093c94d4ede5fbcfbd45bd44ad12565))
- Alias push-all-force ([0886ec2](https://github.com/sorinirimies/tui-chart/commit/0886ec2ab53bdb9c3d59b8a715dec3a88baf2e82))
- **just:** Align push/force/sync/release recipes with sibling repos ([7cc8d31](https://github.com/sorinirimies/tui-chart/commit/7cc8d31802b220351388b8192c13aaa7e59792be))
- Improve git-cliff config; regenerate changelog; remote-safe push recipes ([4f2f30e](https://github.com/sorinirimies/tui-chart/commit/4f2f30e25efb176f314988a68015e7e7690d3bff))
- **vhs:** Remove themes tape and gif ([9b74857](https://github.com/sorinirimies/tui-chart/commit/9b748575dba7ea048a6bd4a1784f9650aee694e6))

### Other

- Revert "chore: bump version to 0.1.0"

This reverts commit 84cc5e27f0e8118c92869d129f38a03b2ff85756. ([aa58de9](https://github.com/sorinirimies/tui-chart/commit/aa58de93cba2fbbc1f9e5222f2d4d8a4ceea2c2a))

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


[0.6.1]: https://github.com/sorinirimies/tui-chart/compare/v0.6.0...v0.6.1

[0.6.0]: https://github.com/sorinirimies/tui-chart/compare/v0.5.1...v0.6.0

[0.5.1]: https://github.com/sorinirimies/tui-chart/compare/v0.5.0...v0.5.1

[0.5.0]: https://github.com/sorinirimies/tui-chart/compare/v0.4.0...v0.5.0

[0.4.0]: https://github.com/sorinirimies/tui-chart/compare/v0.3.0...v0.4.0

[0.3.0]: https://github.com/sorinirimies/tui-chart/compare/v0.2.0...v0.3.0

[0.2.0]: https://github.com/sorinirimies/tui-chart/compare/v0.1.5...v0.2.0

[0.1.5]: https://github.com/sorinirimies/tui-chart/compare/v0.1.4...v0.1.5

[0.1.4]: https://github.com/sorinirimies/tui-chart/compare/v0.1.3...v0.1.4

[0.1.3]: https://github.com/sorinirimies/tui-chart/compare/v0.1.2...v0.1.3

[0.1.2]: https://github.com/sorinirimies/tui-chart/compare/v0.1.1...v0.1.2

[0.1.1]: https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.1


