# Changelog

All notable changes to this project are documented here. It is generated from
[Conventional Commits](https://www.conventionalcommits.org) by
[git-cliff](https://git-cliff.org) — do not edit it by hand.

## [0.6.5](https://github.com/sorinirimies/tui-chart/compare/v0.6.4...v0.6.5) — 2026-11-10

### 📦 Dependencies

- Bump tui-nuances to 0.1.11 ([`12087b9`](https://github.com/sorinirimies/tui-chart/commit/12087b97053879e3e0d2439dae2f4567ac0b47d4))
- Nightly cargo update 2026-11-10 ([`fe95720`](https://github.com/sorinirimies/tui-chart/commit/fe9572002fdb849036b9c6dc43f2319766d7637b))
- Nightly cargo update 2026-11-09 ([`46abb6a`](https://github.com/sorinirimies/tui-chart/commit/46abb6ae8da5d843cf033fa3307c3a9a230c3588))
- Nightly cargo update 2026-11-07 ([`d48280d`](https://github.com/sorinirimies/tui-chart/commit/d48280d39753c064b14e8588ed38c1e7ab641829))
- Nightly cargo update 2026-11-06 ([`588df1d`](https://github.com/sorinirimies/tui-chart/commit/588df1df5d4a8517b9084042748dbeddf8c2f671))

### 🔄 CI and release

- **gitea:** Check the MSRV on Gitea too, with the same command as GitHub ([`eaf95dd`](https://github.com/sorinirimies/tui-chart/commit/eaf95dd775b8ebd45799799394cd5af4a565de75))
- Download archives to a file so curl retries a dropped transfer, instead of piping into tar ([`c4dab61`](https://github.com/sorinirimies/tui-chart/commit/c4dab61b2cef5554ef35cbb003f78eab9fc1c939))
- The Dependabot sweep asks for a rebase on conflicts and keeps going after a failed merge ([`824016c`](https://github.com/sorinirimies/tui-chart/commit/824016c7b3490330aab6cb6beba901681e1bf3f9))

## [0.6.4](https://github.com/sorinirimies/tui-chart/compare/v0.6.1...v0.6.4) — 2026-11-05

### 🐛 Bug fixes

- **ci:** The version bump does not stage the ignored fuzz/Cargo.lock ([`5b5a672`](https://github.com/sorinirimies/tui-chart/commit/5b5a67206075abf0697030576bbc556951a484cf))
- **ci:** Pin rs-clippy-check to v6.1.1 (no floating v6 tag) ([`900e94e`](https://github.com/sorinirimies/tui-chart/commit/900e94e97eac3e023ebce10ec0e4cb26dc3eb0af))
- **ci:** Drop git-cliff remote lookup that 404s on Gitea ([`b3dacab`](https://github.com/sorinirimies/tui-chart/commit/b3dacab00afcf31148ef52bf0b78a352e3f573ee))

### ♻️ Refactor

- Remove dead_code allows the build doesn't need; gate on unused dependencies ([`1332be7`](https://github.com/sorinirimies/tui-chart/commit/1332be70fed8419cfff002d3e368b604cae79db6))

### 📚 Documentation

- Drop the per-repo agent rules skill; the shared skills cover it ([`5de9440`](https://github.com/sorinirimies/tui-chart/commit/5de944031267fd3927d37dc20d6398c891fc1925))
- This repo's agent rules as a project skill in .agents/skills ([`c05faf8`](https://github.com/sorinirimies/tui-chart/commit/c05faf84f9d1074e342a1f304525dec860e047b1))
- Move agent rules from .zed/rules.md to AGENTS.md ([`1115e96`](https://github.com/sorinirimies/tui-chart/commit/1115e963914c2edbd4864e2221f19c51d7653577))

### 📦 Dependencies

- Upgrade dependencies 2026-11-05 ([`fe50f96`](https://github.com/sorinirimies/tui-chart/commit/fe50f968beecb0919b758e19fd23f08292e583d1))

### 📦 Build

- Align tooling with the library template ([`3677fa1`](https://github.com/sorinirimies/tui-chart/commit/3677fa14def4b19f8e91c240715ab6c8a48914bf))

### 🔄 CI and release

- Bump softprops/action-gh-release from 2 to 3 (#7) ([`25857c0`](https://github.com/sorinirimies/tui-chart/commit/25857c018e25960c80ba59da4d4b0af535e4f7b5))
- Merge green Dependabot PRs, majors included, with a nightly sweep; nightly commits skip ignored lockfiles ([`3ab82dc`](https://github.com/sorinirimies/tui-chart/commit/3ab82dc324d7c9933da70b0a5cd30be89adf98e8))
- Bump dependabot/fetch-metadata from 2 to 3 (#8) ([`4202c33`](https://github.com/sorinirimies/tui-chart/commit/4202c339de3ac612564ef07cb417ce154b9596eb))
- Scan the git history for secrets with gitleaks ([`c640c78`](https://github.com/sorinirimies/tui-chart/commit/c640c78470eea6439434b830805f884eb41ab75d))
- Bump workflow actions ([`729fa76`](https://github.com/sorinirimies/tui-chart/commit/729fa76a30b2b16de084f2f07240dcfdf20128a9))

### 🔧 Chores

- Remove AGENTS.md ([`6accf71`](https://github.com/sorinirimies/tui-chart/commit/6accf71591667628e811a66930bdd97de1b8ecf3))

## [0.6.1](https://github.com/sorinirimies/tui-chart/compare/v0.6.0...v0.6.1) — 2026-10-28

### ✨ Features

- Enable the tui-nuances theme feature by default ([`19a16fe`](https://github.com/sorinirimies/tui-chart/commit/19a16fed02279629ae4cd9a7a0e10a3fc3ba5e02))

### 📦 Dependencies

- Bump tui-nuances to 0.1.10 ([`671d55b`](https://github.com/sorinirimies/tui-chart/commit/671d55bc4d90c464481752e10ed6f9ab0ee703c3))
- Update Cargo.lock ([`d2eee1c`](https://github.com/sorinirimies/tui-chart/commit/d2eee1cd61399b65fcb429c6a9b77eb361d3b2c8))

### 🔄 CI and release

- Stop dependabot bumping the MSRV toolchain pin ([`98c5919`](https://github.com/sorinirimies/tui-chart/commit/98c5919019e755c3708b15f8f89e3e3cce85af53))
- Automerge uses GH_PAT so workflow-file PRs can merge ([`0f4003d`](https://github.com/sorinirimies/tui-chart/commit/0f4003d6b1c05c99bcafc46be443118e50d3664c))
- Bump actions/upload-artifact from 4 to 7 (#2) ([`53c0077`](https://github.com/sorinirimies/tui-chart/commit/53c0077fefed04ff0c60f0f87fd6f31d16f413d3))
- Bump softprops/action-gh-release from 2 to 3 (#6) ([`d1d01a0`](https://github.com/sorinirimies/tui-chart/commit/d1d01a0fab60a9666c3aff6a001750824d297f21))
- Bump actions/checkout from 4 to 7 (#5) ([`3f3a9ff`](https://github.com/sorinirimies/tui-chart/commit/3f3a9ffe6b8fa342dc12c105842f7e099816614a))
- Bump clechasseur/rs-clippy-check from 4 to 5 (#3) ([`9047323`](https://github.com/sorinirimies/tui-chart/commit/9047323e869e134af3d1920429b10c469ef8371c))
- **gitea:** Nightly workflow-action bumps ([`e02a030`](https://github.com/sorinirimies/tui-chart/commit/e02a0306bebf948501db48333f08781a4d2243d2))
- Automerge major dependabot bumps too ([`97461e2`](https://github.com/sorinirimies/tui-chart/commit/97461e200284a0d50ed9616de0f8e6ea960881b5))
- Automerge green PRs (workflow_run) + dependabot for actions ([`fffaae7`](https://github.com/sorinirimies/tui-chart/commit/fffaae7ecc9aac78ef8952e1f8d8f521291c075b))

### 🔧 Chores

- Bump tui-nuances to 0.1.7 ([`0a169a1`](https://github.com/sorinirimies/tui-chart/commit/0a169a172ceb510df78c09a3f6ef4e18056fb3d0))

## [0.6.0](https://github.com/sorinirimies/tui-chart/compare/v0.5.1...v0.6.0) — 2026-10-25

### ✨ Features

- Pie hits, instrument hits, hidden funnel/radial, View zoom/pan, FxExt, Gauge::zones, chamfered treemap; coverage CI; interactivity guide ([`f06e461`](https://github.com/sorinirimies/tui-chart/commit/f06e4612932e8e6fa6f22c47bfb4bd5e953ce514))

## [0.5.1](https://github.com/sorinirimies/tui-chart/compare/v0.5.0...v0.5.1) — 2026-10-25

### 🐛 Bug fixes

- **effects:** Glow skips cells without a background; re-record cyber demo ([`63995af`](https://github.com/sorinirimies/tui-chart/commit/63995afe281d71b5a7b3e65932ab2fad311f1ef4))

### 🧪 Tests

- **effects:** Glow needs an explicit background ([`bbdc2e0`](https://github.com/sorinirimies/tui-chart/commit/bbdc2e0d8ccd883a9b7fa482baffda6c0fd2a5de))

### 🔧 Chores

- Reset version before bump ([`d7f8900`](https://github.com/sorinirimies/tui-chart/commit/d7f8900d0e4db52a9e214ca422c49010088409c7))

## [0.5.0](https://github.com/sorinirimies/tui-chart/compare/v0.4.0...v0.5.0) — 2026-10-24

### ✨ Features

- Mouse interactivity for every chart (HitMap, Interaction, tooltips, legend toggling) (#1) ([`7521260`](https://github.com/sorinirimies/tui-chart/commit/7521260cbfae62e758b226753d76d7d0c3e97e14))

## [0.4.0](https://github.com/sorinirimies/tui-chart/compare/v0.3.0...v0.4.0) — 2026-10-24

### ⚠️ Breaking changes

- Remove shader backdrop; calmer cyber demo and sparser oscilloscope graticule ([`38e4d1b`](https://github.com/sorinirimies/tui-chart/commit/38e4d1ba4117584f7801bd789462a2960c469434))

### ✨ Features

- Remove shader backdrop; calmer cyber demo and sparser oscilloscope graticule ([`38e4d1b`](https://github.com/sorinirimies/tui-chart/commit/38e4d1ba4117584f7801bd789462a2960c469434))

### 📚 Documentation

- Document shader feature ([`5cd5e9f`](https://github.com/sorinirimies/tui-chart/commit/5cd5e9f4ed57337ef3a26d9baf81b1daf6228e1f))
- Move effects section above License ([`b927d24`](https://github.com/sorinirimies/tui-chart/commit/b927d24463c798a3f904dfe974597d9ecf2cb69f))
- Drop duplicate cyber gif ([`490e2f3`](https://github.com/sorinirimies/tui-chart/commit/490e2f3ad884def671721cf161b5e85b9415b8cd))

## [0.3.0](https://github.com/sorinirimies/tui-chart/compare/v0.2.0...v0.3.0) — 2026-10-24

### ✨ Features

- Fx glow/scanlines/chamfer frames and optional GPU ShaderBackdrop (tui-shader) ([`649fe0e`](https://github.com/sorinirimies/tui-chart/commit/649fe0eccc343ad8181295d6e5185023b21f8f61))

## [0.2.0](https://github.com/sorinirimies/tui-chart/compare/v0.1.5...v0.2.0) — 2026-10-24

### ✨ Features

- Oscilloscope, Spectrum, DotMatrix, RadarScope and HexMap instrument widgets ([`3d674a9`](https://github.com/sorinirimies/tui-chart/commit/3d674a9f2bc997db8dd14c46d4bbbb6d5b3565b5))
- **gauge:** 12 new styles (dial, meter, reticle, ring, chevron, smooth, dots, led, battery, level, thermometer, signal); add RadialBar and RingStack ([`38e5cba`](https://github.com/sorinirimies/tui-chart/commit/38e5cba46380523b25e17d2c109a4bfb9a7acc73))
- **examples:** Pie chart demo, tape and gif; embed in README ([`d2dbd7e`](https://github.com/sorinirimies/tui-chart/commit/d2dbd7e8ed646a9927550547f426da9e7f42b557))
- Re-export tui-piechart (default 'piechart' feature) so one dependency covers charts and pies ([`0dfc667`](https://github.com/sorinirimies/tui-chart/commit/0dfc66755eb9b5e35f68a535ff98e8a3f04f2f64))

### 🔧 Chores

- **just:** Release recipes skip the bump confirmation prompt ([`26846b5`](https://github.com/sorinirimies/tui-chart/commit/26846b5a750b56542605953d15e2619e0082fabf))

## [0.1.5](https://github.com/sorinirimies/tui-chart/compare/v0.1.4...v0.1.5) — 2026-10-22

### 🐛 Bug fixes

- **calendar:** Robust labels, flat/NaN/inf handling, zero_empty, wider cells, legend sizing ([`85eef5c`](https://github.com/sorinirimies/tui-chart/commit/85eef5c14113489256456aeb42e2c3808f0ff700))

## [0.1.4](https://github.com/sorinirimies/tui-chart/compare/v0.1.3...v0.1.4) — 2026-10-22

### ⚠️ Breaking changes

- **paired_bars:** Rename Comparison to PairedBars; feat: label_color ([`8ab9e58`](https://github.com/sorinirimies/tui-chart/commit/8ab9e58b97d2b99a301c2ffc343f71a6547279ec))

### ♻️ Refactor

- **paired_bars:** Rename Comparison to PairedBars; feat: label_color ([`8ab9e58`](https://github.com/sorinirimies/tui-chart/commit/8ab9e58b97d2b99a301c2ffc343f71a6547279ec))

## [0.1.3](https://github.com/sorinirimies/tui-chart/compare/v0.1.2...v0.1.3) — 2026-10-22

### ✨ Features

- **comparison:** Comparison, before/after bar pairs per measure, inline or stacked, with a legend ([`186be45`](https://github.com/sorinirimies/tui-chart/commit/186be452b778a8c4ceeb45be1599eb4417f77dac))
- Lollipop, slope, calendar, violin, rose, stacked-area and sankey charts; boxplot mean/notches/jitter; Monochrome accessibility wrapper; golden tests, docs examples ([`3fd18e1`](https://github.com/sorinirimies/tui-chart/commit/3fd18e162aa5ad57e6d10240edae0f1c351f5003))

### 🐛 Bug fixes

- **annotation:** Labels of close vertical lines go on separate rows instead of printing over each other, and over the lines ([`7f87d1f`](https://github.com/sorinirimies/tui-chart/commit/7f87d1f3d561c44d10e03c471f750a3e3bd9a5c0))

### 📚 Documentation

- **comparison:** Demo gif ([`4800427`](https://github.com/sorinirimies/tui-chart/commit/4800427c390321eadc007175b78c0f68c6ca58b5))

### 📦 Dependencies

- Nightly library update (#1) ([`f5efe6f`](https://github.com/sorinirimies/tui-chart/commit/f5efe6f5aa25adb14dd3a59f4996fbb3d516d942))
- Nightly library update (#1) ([`18937b9`](https://github.com/sorinirimies/tui-chart/commit/18937b94aff9135355916caec3deff6eee3bdb3e))

### 🔄 CI and release

- Nightly update pushes to main directly like tui-spinner (no manual-merge PR) ([`1bd98a5`](https://github.com/sorinirimies/tui-chart/commit/1bd98a53208ae5082ec381969b595e0520eab362))

## [0.1.2](https://github.com/sorinirimies/tui-chart/compare/v0.1.1...v0.1.2) — 2026-10-18

### ✨ Features

- **heatmap,radar,treemap,waterfall:** Stretch-to-fit heatmap, filled radar with ring values and legend, treemap gutters and group headers, waterfall grid ([`7dd66b3`](https://github.com/sorinirimies/tui-chart/commit/7dd66b3b02430f0b2f29d33c6858bebdafe06fc3))
- **sparkline,bullet,histogram,depth:** Area/win-loss sparklines with smoothing, 3-row bullets with eighth precision, histogram mean/median markers, gradient depth fills ([`a2e28b2`](https://github.com/sorinirimies/tui-chart/commit/a2e28b2592effe72be99defc1113cfd6d3250a6c))
- **gauge,funnel:** Redesign gauge (solid tracks, rich arc, segmented style) and tapering funnel with drop-off ([`cb3bf03`](https://github.com/sorinirimies/tui-chart/commit/cb3bf03db3cb2eae83d06071f4eb71b9fa7f8db4))
- Funnel, Bullet, BubbleChart; faster line/area rendering; force-push just recipes ([`4d613a6`](https://github.com/sorinirimies/tui-chart/commit/4d613a6e75e6425f7a68308699b19826fa99e2ba))
- Add ChartStack example, tape and gallery entry ([`92c42dc`](https://github.com/sorinirimies/tui-chart/commit/92c42dc1f6e964cdc107e41583d5edb0de76b716))

### 🐛 Bug fixes

- **scripts:** Ask() reads piped stdin instead of blocking on the tty; tests run with empty stdin ([`e9c6841`](https://github.com/sorinirimies/tui-chart/commit/e9c6841f7c73ae749429c60c067d96b00b2e0551))
- **scripts:** Bump skips duplicate clippy/test (--skip-checks), quiet nu test runner ([`718f1ba`](https://github.com/sorinirimies/tui-chart/commit/718f1ba95e94c19872c1e8d4000b79a54a1de752))

### 📚 Documentation

- Drop duplicate showcase GIF from README ([`4125b98`](https://github.com/sorinirimies/tui-chart/commit/4125b9845593e555ead6538a018939b597626137))
- Add chart gallery to README and regenerate demo GIFs ([`c4c886e`](https://github.com/sorinirimies/tui-chart/commit/c4c886e70a1f2d0421761924092e90b12b1d952f))

### 🧪 Tests

- Cover funnel, bullet, bubble options; docs: README feature table; regenerate GIFs ([`8895c75`](https://github.com/sorinirimies/tui-chart/commit/8895c75e1c7ac7297216ec7842641b0f82984c73))

### 📦 Dependencies

- Nightly library update (#1) ([`33feed8`](https://github.com/sorinirimies/tui-chart/commit/33feed82f412d76cf1da531f19bb9eb86a41af27))
- Nightly library update (#1) ([`5ae36e7`](https://github.com/sorinirimies/tui-chart/commit/5ae36e78d42785828d2d7f81d6ece1f0762215b0))

### 🔧 Chores

- **vhs:** Remove themes tape and gif ([`9b74857`](https://github.com/sorinirimies/tui-chart/commit/9b748575dba7ea048a6bd4a1784f9650aee694e6))
- Improve git-cliff config; regenerate changelog; remote-safe push recipes ([`4f2f30e`](https://github.com/sorinirimies/tui-chart/commit/4f2f30e25efb176f314988a68015e7e7690d3bff))
- **just:** Align push/force/sync/release recipes with sibling repos ([`7cc8d31`](https://github.com/sorinirimies/tui-chart/commit/7cc8d31802b220351388b8192c13aaa7e59792be))
- Alias push-all-force ([`0886ec2`](https://github.com/sorinirimies/tui-chart/commit/0886ec2ab53bdb9c3d59b8a715dec3a88baf2e82))
- Remove duplicate showcase tape/gif; vhs-showcase builds tui_chart.gif ([`b69dc71`](https://github.com/sorinirimies/tui-chart/commit/b69dc710c093c94d4ede5fbcfbd45bd44ad12565))
- **justfile:** Push/pull current branch instead of hardcoded main ([`000270e`](https://github.com/sorinirimies/tui-chart/commit/000270e47b453fa03b1fe06118827c2fd820da16))
- **justfile:** Skip unconfigured remotes in *-all recipes ([`e5cd0fa`](https://github.com/sorinirimies/tui-chart/commit/e5cd0fa5f448e286ebcf40f1cf1949d1a10fb4f7))

## [0.1.1](https://github.com/sorinirimies/tui-chart/releases/tag/v0.1.1) — 2026-10-14

### ✨ Features

- Make theming an optional "theme" feature ([`c8e32f9`](https://github.com/sorinirimies/tui-chart/commit/c8e32f9f483d5c296fe49e3f5af4a12a1878e761))
- Shared theming via tui-nuances ([`0bb5253`](https://github.com/sorinirimies/tui-chart/commit/0bb5253ee0308b42399a5d43cd9f83d503cd3b1d))
- Theme::from_base for app-provided themes ([`eb97986`](https://github.com/sorinirimies/tui-chart/commit/eb979869b49ddaa6fb70b6c478c7c8a278370563))
- Macros/architecture, robustness tests, fuzz, audit+deny workflows, nushell tests ([`cc90e1f`](https://github.com/sorinirimies/tui-chart/commit/cc90e1ff1c4c57e018f98d11b05df904ace8ee27))
- Braille candles/bars, Marker everywhere, sextant, overlap policy; restyle tapes like tui-rating ([`fc55baf`](https://github.com/sorinirimies/tui-chart/commit/fc55baf6a821e346f619b54c7d9cfd71c9e43d85))
- Themes, styling, serde feature, theme-switching examples, docs, negative bars, KDE ([`8ea79b9`](https://github.com/sorinirimies/tui-chart/commit/8ea79b969e7b97675177beb0e156ab8108a627cd))
- Annotations, crosshair, log/spline/error bars, indicators, stream, time axis, stack ([`7f0ca48`](https://github.com/sorinirimies/tui-chart/commit/7f0ca483568cb67b21791fd542e70b57b8e41f60))
- Sparkline, gauge, radar, boxplot, waterfall, gantt, depth, renko, treemap ([`c4b7658`](https://github.com/sorinirimies/tui-chart/commit/c4b7658c131f7a68d9acd20f3878fce7b8e7f7af))
- Tui-chart widgets, examples, tapes, docs ([`90fa8cd`](https://github.com/sorinirimies/tui-chart/commit/90fa8cdf17fd16b03ce251fcf53706fba919f06e))

### 🐛 Bug fixes

- Doc code blocks in lib.rs ([`b808a60`](https://github.com/sorinirimies/tui-chart/commit/b808a6077df7964d9054378bb51e4c176484ad30))

### 📚 Documentation

- Re-record demos showing results only; reference tui-nuances version ([`8d1e90b`](https://github.com/sorinirimies/tui-chart/commit/8d1e90b7adf12e5fb62bdda16f0e820fbeef77da))
- Generated demo GIFs, 2-row help footer ([`5ecc16e`](https://github.com/sorinirimies/tui-chart/commit/5ecc16e9c3499a807d9581e93d8bb39baf9914c5))

### 🔄 CI and release

- Prebuilt Nushell, cargo-deny + cargo-audit security workflow ([`4893c9a`](https://github.com/sorinirimies/tui-chart/commit/4893c9ade7614534f97a00a3a237602b5fc3b702))

### 🔧 Chores

- Depend on tui-nuances 0.1.6 from crates.io ([`4069a83`](https://github.com/sorinirimies/tui-chart/commit/4069a8395f6b99831381fe6b0fc9110255a67494))
- Depend on tui-nuances 0.1.6 ([`8927821`](https://github.com/sorinirimies/tui-chart/commit/89278213e38ec0b99efab66b73724508f10e610b))
- Depend on tui-nuances 0.1.5 ([`67ab26a`](https://github.com/sorinirimies/tui-chart/commit/67ab26acbc1368065511d327f82bcf133cba853a))
- Remove gitea migration script ([`606acdf`](https://github.com/sorinirimies/tui-chart/commit/606acdf790bb36d6e9035074a87105f28a02b364))
- Remove gitea migration script ([`25f9954`](https://github.com/sorinirimies/tui-chart/commit/25f995436a1d40800c90b01137ad619e6f8918ae))
- Stub new chart modules ([`322f1ff`](https://github.com/sorinirimies/tui-chart/commit/322f1ff34252181d28130789a9fe21cddd873ab8))


