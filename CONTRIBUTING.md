# Contributing to tui-chart

Thank you for your interest in contributing to tui-chart! This document provides guidelines and instructions for contributing.

## Code of Conduct

Be respectful, inclusive, and constructive in all interactions.

## How to Contribute

### Reporting Bugs

Before creating a bug report:
- Check the issue tracker to avoid duplicates
- Collect relevant information (OS, terminal, Rust version)
- Create a minimal reproducible example if possible

When filing a bug report, include:
- Clear description of the issue
- Steps to reproduce
- Expected vs actual behavior
- Environment details
- Code samples or screenshots

### Suggesting Features

For feature requests:
- Check existing issues and discussions
- Clearly describe the use case
- Provide examples of how it would work
- Consider backwards compatibility

### Pull Requests

All contributions must be submitted through GitHub using the forking workflow:

1. **Fork the Repository**
   - Visit https://github.com/sorinirimies/tui-chart
   - Click the "Fork" button to create your own copy of the repository

2. **Clone Your Fork**
   ```bash
   git clone https://github.com/YOUR-USERNAME/tui-chart.git
   cd tui-chart
   ```

3. **Add Upstream Remote**
   ```bash
   git remote add upstream https://github.com/sorinirimies/tui-chart.git
   ```

4. **Create a Branch**
   ```bash
   git checkout -b feature/my-new-feature
   ```

5. **Make Changes**
   - Write clear, idiomatic Rust code
   - Follow existing code style
   - Add tests for new functionality
   - Update documentation as needed

6. **Run Tests**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   ```

7. **Commit Changes**
   - Use conventional commit messages
   - Examples: `feat: add gradient color support`, `fix: handle edge case in animation`

8. **Push to Your Fork**
   ```bash
   git push origin feature/my-new-feature
   ```

9. **Create Pull Request**
   - Go to your fork on GitHub
   - Click "Compare & pull request"
   - Fill in the PR description with details about your changes
   - Submit the PR to the main repository

**Note**: Do not submit patches or contributions through email or other channels. All contributions must go through GitHub's pull request system on your forked repository.

## Architecture

### Module map

| Module | Role |
|---|---|
| `src/lib.rs` | Crate docs, module list, the public `pub use` surface |
| `src/macros.rs` | Internal `macro_rules!` (see catalog below); declared first with `#[macro_use]` |
| `src/theme.rs` | Thin chart glue over the shared `tui_nuances::Theme` (re-exported): `ChartThemeExt` (heatmap gradient, crosshair/tooltip styles), `theme::begin` (background fill + block → inner area). Theme data/built-ins live in `tui-nuances`; every widget also implements `tui_nuances::Themed` |
| `src/canvas.rs` | Sub-cell drawing surface (braille/half-block/etc.) used by line-like widgets |
| `src/layout.rs`, `src/axis.rs` | Plot-area computation, axis drawing, tick/label gutters |
| `src/util.rs` | Public numeric helpers: `nice_ticks`, `nice_bounds`, `format_tick`, `format_compact`, `PALETTE` |
| `src/scale.rs` | Internal numeric mapping (`lerp_u8`, `mix_rgb`) |
| `src/text.rs` | Internal text helpers (`trunc`, clipped `put_str`) |
| `src/timeaxis.rs`, `src/indicator.rs`, `src/stream.rs` | Time ticks/formatting, technical indicators, ring buffer / streaming series |
| `src/annotation.rs`, `src/crosshair.rs`, `src/stack.rs` | Overlays, nearest-point lookup, multi-pane layout |
| `src/<widget>.rs` | One file per widget (`line`, `bar`, `candle`, `histogram`, `heatmap`, `gauge`, `radar`, `treemap`, `boxplot`, `waterfall`, `gantt`, `depth`, `renko`, `sparkline`, `funnel`, `bullet`, `paired_bars`, `bubble`, `lollipop`, `slope`, `calendar`, `violin`, `rose`, `stacked_area`, `sankey`) |
| `src/accessibility.rs` | `Monochrome` wrapper / `strip_color`: render any widget without colour (`NO_COLOR` aware) |

### Macro catalog (`src/macros.rs`)

- `widget_impls!(Type)` – generates `Widget for Type` and `Widget for &Type`
  from an inherent `fn render_into(&self, area: Rect, buf: &mut Buffer)`.
- `setter!(name: Ty)` / `setter!(name => field: Ty)` – consuming builder setter.
- `opt_setter!(...)` – same, but stores `Some(v)` into an `Option` field.
- `theme_setter!(field)` – plain `theme(impl Into<Theme>)` setter.
  Widgets whose theme also styles axes write `theme` by hand.
- `base_style_setter!(field)` – `base_style(Style)` setter.
- `block_setter!(field, 'lt)` – `block(Block<'lt>)` setter.

Put doc comments on the macro call (`setter!(/// doc \n name: Ty)`); `missing_docs`
is enforced.

### Adding a widget

1. Create `src/<name>.rs`: struct with `theme: Option<Theme>`, `base: Option<Style>`,
   `block: Option<Block<'static>>`; builders via the macros above.
2. Implement `fn render_into(&self, area, buf)`. Start with
   `theme::begin(&self.block, &self.theme, self.base, area, buf)` to get the inner
   area, return early for tiny/empty areas, never write outside `area`, and
   tolerate NaN/inf/empty data. Resolve colors as explicit → theme → default
   (helpers: `util::pick(&theme, i)` for palette colors, `util::dim_text(&theme)` for
   grid/label styles, `util::darken`/`mix` for fills).
3. `widget_impls!(Name);`, add `mod name;` and `pub use` in `src/lib.rs`.
4. Add the widget to `all_widgets` in `tests/common/mod.rs` (it is then covered by
   `tests/robustness_test.rs` and `tests/theme_test.rs`), plus a snapshot test in
   `tests/snapshots_test.rs` and an example in `examples/`.
   Add unit tests next to the widget: every toggle/option changes output, explicit
   colors apply, degenerate data/areas don't panic.
5. Document it: a `# Example` doctest on the struct (docs.rs), a row in the README
   table, a `examples/vhs/<name>.tape` and (after `just vhs-<name>`) its GIF + README
   gallery entry, `[[example]]` in `Cargo.toml`, `example-<name>`/`vhs-<name>` recipes.
6. Run `cargo fmt`, `cargo clippy --all-targets --all-features -- -D warnings`,
   `cargo test --all-features`, `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps`.

### Test layout

- `tests/robustness_test.rs` – widget × area × data matrix (no panic, no writes outside area).
- `tests/theme_test.rs` – every theme × every widget; explicit colors beat the theme.
- `tests/snapshots_test.rs` – exact-row buffer paired_barss (inline strings).
- `tests/property_test.rs` – deterministic xorshift property tests (ticks, indicators, ring buffer).

## Development Setup

### Prerequisites

- Rust 1.88.0 or later
- A terminal that supports Unicode (for full visual testing)

### Building

```bash
# Build the library
cargo build

# Build with all features
cargo build --all-features

# Build examples
cargo build --examples
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

Scripts in `scripts/` are covered by Nushell tests (`scripts/tests/test_*.nu`,
one file per script; they use temp directories and local bare git repos, never
the network):

```bash
just test-nu                                  # or: nu scripts/tests/run_all.nu
nu scripts/tests/run_all.nu --filter bump     # only matching files
just test-all                                 # cargo test + Nu tests
```

When you change a script, keep logic in `export def` helpers so tests can
`use ../script.nu [helper]`, and add a `test_<script>.nu` case. Interactive
prompts go through `ask`, which returns `""` without a terminal so tests (and CI)
never block.

### Security & fuzzing

```bash
just audit        # cargo audit   (RUSTSEC advisories; config: .cargo/audit.toml)
just deny         # cargo deny    (advisories, licenses, bans, sources; config: deny.toml)
just fuzz-build   # build all fuzz targets (nightly + cargo-fuzz)
just fuzz render_line 60   # fuzz one target for 60 s
just fuzz-all 30           # every target, 30 s each
```

Fuzz targets live in `fuzz/fuzz_targets/` (own workspace, not part of the crate):
`render_line`, `render_candles`, `render_misc` (all other widgets), `timeaxis`,
`indicators`, `theme_json` (serde round-trip). Inputs are capped (area <= 200x100,
<= 2000 points) so runs stay fast. Rendering must never panic for any input:
if a target crashes, the reproducer is saved to `fuzz/artifacts/<target>/`; fix the
bug, then add a regression test (`cargo fuzz fmt <target> <file>` prints the input).

CI: `security.yml` runs `cargo audit` + `cargo deny check` daily and on Cargo
changes; `fuzz.yml` smoke-fuzzes every target for 60 s daily and on `src/**`
changes (crash artifacts are uploaded). Both exist for GitHub and Gitea
(`.github/workflows`, `.gitea/workflows`). To accept an advisory, add its ID to
`ignore` in `deny.toml` and `.cargo/audit.toml` with a justification comment.

### Running Examples

```bash
# Run specific example
cargo run --example horizontal

# Or use just
just example-horizontal
```

### Using Just

This project includes a `justfile` for common tasks:

```bash
# List all commands
just

# Run checks (format, clippy, test)
just check

# Format code
just fmt

# Run clippy
just clippy

# Build documentation
just doc
```

## Code Style

### Rust Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` to catch common mistakes
- Write idiomatic Rust code

### Documentation

- Add doc comments for all public items
- Include examples in doc comments
- Use `///` for item documentation
- Use `//!` for module documentation
- Run `cargo doc --open` to preview

Example:
```rust
/// Creates a new slider with the given value and bounds
///
/// # Arguments
///
/// * `value` - Initial value
/// * `min` - Minimum value
/// * `max` - Maximum value
///
/// # Example
///
/// ```
/// use tui_chart::Slider;
/// let slider = Slider::new(50.0, 0.0, 100.0);
/// ```
pub fn new(value: f64, min: f64, max: f64) -> Self {
    // ...
}
```

### Testing

- Write unit tests for new functionality
- Add integration tests where appropriate
- Test edge cases and error conditions
- Aim for good test coverage

Example:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_bounds() {
        let slider = Slider::new(150.0, 0.0, 100.0);
        assert_eq!(slider.value(), 100.0);
    }
}
```

## Project Structure

```
tui-chart/
├── src/
│   ├── lib.rs           # Main library entry point
│   ├── slider.rs        # Main slider widget
│   ├── state.rs         # State management
│   ├── orientation.rs   # Orientation types
│   ├── symbols.rs       # Symbol sets
│   ├── styles.rs        # Visual styles
│   ├── colors.rs        # Color gradients
│   └── animation.rs     # Animation system
├── examples/            # Example applications
├── tests/               # Integration tests
└── benches/             # Benchmarks (future)
```

## Adding New Features

### New Slider Style

1. Add the style definition in `src/styles.rs`
2. Create a preset method (e.g., `pub fn my_style()`)
3. Add tests for the new style
4. Document with examples
5. Consider adding an example in `examples/`

### New Symbol Set

1. Add symbols in `src/symbols.rs`
2. Create preset methods for handle and bar
3. Add to the `SliderSymbols` presets
4. Test the symbols render correctly
5. Update documentation

### New Animation Easing

1. Add the easing variant to `AnimationEasing` enum in `src/animation.rs`
2. Implement the easing function in the `apply()` method
3. Add tests for the easing function
4. Document the easing behavior

## Examples

When adding examples:
- Place in `examples/` directory
- Include clear comments explaining functionality
- Add keyboard controls help text
- Register in `Cargo.toml` under `[[example]]`
- Update README with description and run command

## Documentation

### README Updates

When adding significant features:
- Update feature list
- Add examples showing usage
- Update API sections
- Add to appropriate sections

### CHANGELOG

Follow [Keep a Changelog](https://keepachangelog.com/):
- Add entries under `[Unreleased]`
- Use categories: Added, Changed, Deprecated, Removed, Fixed, Security
- Write clear, user-focused descriptions

## Release Process

Maintainers only:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag -a v0.x.x -m "Release v0.x.x"`
4. Push tag: `git push origin v0.x.x`
5. CI will build and publish to crates.io

## Questions?

- Open an issue for questions
- Check existing documentation
- Review examples for usage patterns

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to tui-chart! 🎚️