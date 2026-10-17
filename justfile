# tui-chart - A collection of TUI chart widgets library for ratatui
#
# Prerequisites: cargo install just && cargo install nu
# Usage: just <task> or just --list
# Patterns: See docs/JUSTFILE_PATTERNS.md for best practices

# Default task - show available commands
default:
    @just --list

# Install required tools (just, git-cliff)
install-tools:
    @echo "Installing required tools..."
    @command -v just >/dev/null 2>&1 || cargo install just
    @command -v git-cliff >/dev/null 2>&1 || cargo install git-cliff
    @echo "✅ All tools installed!"

# Build the project
build:
    cargo build --all-features

# Build release version
build-release:
    cargo build --release

# Run line chart example
run:
    cargo run --features theme --example line

# Run bars example
example-bars:
    cargo run --features theme --example bars

# Run boxplot example
example-boxplot:
    cargo run --features theme --example boxplot

# Run candlestick example
example-candlestick:
    cargo run --features theme --example candlestick

# Run depth example
example-depth:
    cargo run --features theme --example depth

# Run gantt example
example-gantt:
    cargo run --features theme --example gantt

# Run gauge example
example-gauge:
    cargo run --features theme --example gauge

# Run heatmap example
example-heatmap:
    cargo run --features theme --example heatmap

# Run histogram example
example-histogram:
    cargo run --features theme --example histogram

# Run line example
example-line:
    cargo run --features theme --example line

# Run radar example
example-radar:
    cargo run --features theme --example radar

# Run renko example
example-renko:
    cargo run --features theme --example renko

# Run showcase example
example-showcase:
    cargo run --features theme --example showcase

# Run sparkline example
example-sparkline:
    cargo run --features theme --example sparkline

# Run chartstack example
example-chartstack:
    cargo run --features theme --example chartstack

# Run funnel example
example-funnel:
    cargo run --features theme --example funnel

# Run bullet example
example-bullet:
    cargo run --features theme --example bullet

# Run bubble example
example-bubble:
    cargo run --features theme --example bubble

# Run stream example
example-stream:
    cargo run --features theme --example stream

# Run treemap example
example-treemap:
    cargo run --features theme --example treemap

# Run waterfall example
example-waterfall:
    cargo run --features theme --example waterfall

# Run all examples (sequentially)
examples: example-themes example-bars example-boxplot example-candlestick example-chartstack example-bubble example-bullet example-funnel example-depth example-gantt example-gauge example-heatmap example-histogram example-line example-radar example-renko example-showcase example-sparkline example-stream example-treemap example-waterfall

# Run themes example
example-themes:
    cargo run --features theme --example themes

# Run tests
test:
    cargo test --all-features

# Run the Nushell script tests (scripts/tests)
test-nu:
    nu scripts/tests/run_all.nu

# Run Rust and Nushell tests
test-all: test test-nu
    @echo "✅ All Rust and Nu tests passed!"

# Audit dependencies for known vulnerabilities (cargo install cargo-audit)
audit:
    cargo audit

# Check advisories, licenses, bans and sources (cargo install cargo-deny)
deny:
    cargo deny check

# Build all fuzz targets (needs nightly + cargo install cargo-fuzz)
fuzz-build:
    cargo +nightly fuzz build

# Fuzz one target (render_line, render_candles, render_misc, timeaxis, indicators, theme_json); SECONDS bounds the run
fuzz target seconds="30":
    cargo +nightly fuzz run {{target}} -- -max_total_time={{seconds}}

# Smoke-fuzz every target (SECONDS each)
fuzz-all seconds="30":
    for t in $(cargo +nightly fuzz list); do cargo +nightly fuzz run "$t" -- -max_total_time={{seconds}} || exit 1; done

# Run tests with coverage
test-coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Check code without building
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Check if code is formatted
fmt-check:
    cargo fmt --check

# Run clippy linter
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run all checks (fmt, clippy, test)
check-all: fmt-check clippy test test-nu
    @echo "✅ All checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Check if git-cliff is installed
check-git-cliff:
    @command -v git-cliff >/dev/null 2>&1 || { echo "❌ git-cliff not found. Install with: cargo install git-cliff"; exit 1; }

# Generate full changelog from all tags
changelog: check-git-cliff
    @echo "Generating full changelog..."
    git-cliff -o CHANGELOG.md
    @echo "✅ Changelog generated!"

# Generate changelog for unreleased commits only
changelog-unreleased: check-git-cliff
    @echo "Generating unreleased changelog..."
    git-cliff --unreleased --prepend CHANGELOG.md
    @echo "✅ Unreleased changelog generated!"

# Generate changelog for specific version tag
changelog-version version: check-git-cliff
    @echo "Generating changelog for version {{version}}..."
    git-cliff --tag v{{version}} -o CHANGELOG.md
    @echo "✅ Changelog generated for version {{version}}!"

# Preview changelog without writing to file
changelog-preview: check-git-cliff
    @git-cliff

# Preview unreleased changes
changelog-preview-unreleased: check-git-cliff
    @git-cliff --unreleased

# Generate changelog for latest tag only
changelog-latest: check-git-cliff
    @echo "Generating changelog for latest tag..."
    git-cliff --latest -o CHANGELOG.md
    @echo "✅ Latest changelog generated!"

# Update changelog with all commits (force regenerate)
changelog-update: check-git-cliff
    @echo "Regenerating complete changelog from all tags..."
    git-cliff --output CHANGELOG.md
    @echo "✅ Changelog updated from all git history!"

# Bump version (usage: just bump 0.2.0)
# Note: Runs check-all first to ensure code quality before version bump (fail early)
bump version: check-all check-git-cliff
    @echo "Bumping version to {{version}}..."
    nu scripts/bump_version.nu {{version}}

# Quick release: format, check, test, and build
release-check: fmt clippy test build-release
    @echo "✅ Ready for release!"

# Publish to crates.io (dry run)
publish-dry:
    cargo publish --dry-run

# Publish to crates.io
publish:
    cargo publish

# Run pre-publish readiness checks
check-publish:
    nu scripts/check_publish.nu

# Upgrade dependencies (nightly-style: upgrade pins, sync lock, quality gate)
upgrade-deps:
    nu scripts/upgrade_deps.nu --dry-run

# Update dependencies
update:
    cargo update

# Show outdated dependencies
outdated:
    cargo outdated

# Generate documentation
doc:
    cargo doc --no-deps --open

# Watch and auto-run on file changes (requires cargo-watch)
watch:
    cargo watch -x "run --example showcase"

# Git: commit current changes
commit message:
    git add .
    git commit -m "{{message}}"

# Git: pull from GitHub (origin)
pull:
    git pull origin "$(git branch --show-current)"

# Git: pull from Gitea Microlab
pull-gitea-microlab:
    git pull gitea-microlab "$(git branch --show-current)"

# Git: pull from Gitea (nexus-lab instance)
pull-gitea-nexus-lab:
    git pull gitea-nexus-lab "$(git branch --show-current)"

# Git: pull from Gitea Starscream
pull-gitea-starscream:
    git pull gitea-starscream "$(git branch --show-current)"

# Git: pull from all remotes (Gitea first, then GitHub)
pull-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in gitea-microlab gitea-starscream gitea-nexus-lab origin; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ pulling from $r"
        if git pull "$r" "$(git branch --show-current)"; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Pulled from:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Git: push to GitHub (origin)
push:
    git push origin "$(git branch --show-current)"

# Git: push to Gitea Microlab
push-gitea-microlab:
    git push gitea-microlab "$(git branch --show-current)"

# Git: push to Gitea (nexus-lab instance)
push-gitea-nexus-lab:
    git push gitea-nexus-lab "$(git branch --show-current)"

# Git: push to Gitea Starscream
push-gitea-starscream:
    git push gitea-starscream "$(git branch --show-current)"

# Git: push to GitHub, Gitea, and Gitea (nexus-lab)
push-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in origin gitea-microlab gitea-starscream gitea-nexus-lab; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ pushing to $r"
        if git push "$r" "$(git branch --show-current)"; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Pushed to:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Git: force push (with lease) to GitHub (origin)
push-force:
    git push --force-with-lease origin "$(git branch --show-current)"

# Git: force push (with lease) to Gitea Microlab
push-force-gitea-microlab:
    git push --force-with-lease gitea-microlab "$(git branch --show-current)"

# Git: force push (with lease) to Gitea (nexus-lab instance)
push-force-gitea-nexus-lab:
    git push --force-with-lease gitea-nexus-lab "$(git branch --show-current)"

# Git: force push (with lease) to Gitea Starscream
push-force-gitea-starscream:
    git push --force-with-lease gitea-starscream "$(git branch --show-current)"

# Git: force push (with lease) to every configured remote
alias push-all-force := push-force-all

# Git: force push (with lease) to every configured remote
push-force-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in origin gitea-microlab gitea-starscream gitea-nexus-lab; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ force pushing to $r"
        if git push --force-with-lease "$r" "$(git branch --show-current)"; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Force pushed to:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Git: push tags to GitHub
push-tags:
    git push origin --tags

# Git: push tags to all remotes
push-tags-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in origin gitea-microlab gitea-starscream gitea-nexus-lab; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ pushing tags to $r"
        if git push "$r" --tags; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Tags pushed to:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Full release workflow: bump version and push to GitHub
release version: (bump version)
    @echo "Pushing to GitHub..."
    git push origin "$(git branch --show-current)"
    git push origin v{{version}}
    @echo "✅ Release v{{version}} complete on GitHub!"

# Full release workflow: bump version and push to Gitea Microlab
release-gitea-microlab version: (bump version)
    @echo "Pushing to Gitea Microlab..."
    git push gitea-microlab "$(git branch --show-current)"
    git push gitea-microlab v{{version}}
    @echo "✅ Release v{{version}} complete on Gitea Microlab!"

# Full release workflow: bump version and push to Gitea (nexus-lab instance)
release-gitea-nexus-lab version: (bump version)
    @echo "Pushing to Gitea (nexus-lab)..."
    git push gitea-nexus-lab "$(git branch --show-current)"
    git push gitea-nexus-lab v{{version}}
    @echo "✅ Release v{{version}} complete on Gitea (nexus-lab)!"

# Full release workflow: bump version and push to Gitea Starscream
release-gitea-starscream version: (bump version)
    @echo "Pushing to Gitea Starscream..."
    git push gitea-starscream "$(git branch --show-current)"
    git push gitea-starscream v{{version}}
    @echo "✅ Release v{{version}} complete on Gitea Starscream!"

# Full release workflow: bump version and push to GitHub, Gitea, and Gitea (nexus-lab)
release-all version: (bump version)
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in origin gitea-microlab gitea-starscream gitea-nexus-lab; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ releasing to $r"
        if git push "$r" "$(git branch --show-current)" && git push "$r" v{{version}}; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Release done on:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Push release to all remotes (without bumping)
push-release-all:
    #!/usr/bin/env sh
    failed=""; ok_=""; skipped=""
    for r in origin gitea-microlab gitea-starscream gitea-nexus-lab; do
        if ! git remote get-url "$r" >/dev/null 2>&1; then
            echo "↷ skipping $r (not configured here; add with: git remote add $r <url>)"
            skipped="$skipped $r"; continue
        fi
        echo "→ releasing to $r"
        if git push "$r" "$(git branch --show-current)" && git push "$r" --tags; then ok_="$ok_ $r"; else failed="$failed $r"; fi
    done
    echo ""
    [ -n "$ok_" ] && echo "✅ Release done on:$ok_"
    [ -n "$skipped" ] && echo "ℹ️  Skipped (not configured):$skipped"
    if [ -n "$failed" ]; then echo "⚠️  Failed:$failed"; exit 1; fi

# Sync Gitea Microlab with GitHub (force)
sync-gitea-microlab:
    @echo "Syncing Gitea Microlab with GitHub..."
    git push gitea-microlab "$(git branch --show-current)" --force
    git push gitea-microlab --tags --force
    @echo "✅ Gitea Microlab synced!"

# Sync Gitea Starscream with GitHub (force)
sync-gitea-starscream:
    @echo "Syncing Gitea Starscream with GitHub..."
    git push gitea-starscream "$(git branch --show-current)" --force
    git push gitea-starscream --tags --force
    @echo "✅ Gitea Starscream synced!"

# Sync Gitea (nexus-lab instance) with GitHub (force)
sync-gitea-nexus-lab:
    @echo "Syncing Gitea (nexus-lab) with GitHub..."
    git push gitea-nexus-lab "$(git branch --show-current)" --force
    git push gitea-nexus-lab --tags --force
    @echo "✅ Gitea (nexus-lab) synced!"

# Show configured remotes
remotes:
    @echo "Configured git remotes:"
    @git remote -v

# Setup Gitea remote (provide your Gitea URL)
setup-gitea url:
    @echo "Adding Gitea remote..."
    git remote add gitea-microlab {{url}}
    @echo "✅ Gitea remote added!"
    @echo "Test with: git push gitea-microlab main"

# Show current version
version:
    @nu scripts/version.nu

# Show git-cliff info
cliff-info:
    @echo "Git-cliff configuration:"
    @echo "  Config file: cliff.toml"
    @echo "  Installed: $(command -v git-cliff >/dev/null 2>&1 && echo '✅ Yes' || echo '❌ No (run: just install-tools)')"
    @command -v git-cliff >/dev/null 2>&1 && git-cliff --version || true

# Show project info
info:
    @echo "Project: tui-chart"
    @echo "Version: $(just version)"
    @echo "Author: Sorin Albu-Irimies"
    @echo "License: MIT"

# View changelog
view-changelog:
    @cat CHANGELOG.md

# Generate demo GIF for bars
vhs-bars:
    vhs examples/vhs/bars.tape
    @echo "✅ examples/vhs/output/bars.gif"

# Generate demo GIF for boxplot
vhs-boxplot:
    vhs examples/vhs/boxplot.tape
    @echo "✅ examples/vhs/output/boxplot.gif"

# Generate demo GIF for candlestick
vhs-candlestick:
    vhs examples/vhs/candlestick.tape
    @echo "✅ examples/vhs/output/candlestick.gif"

# Generate demo GIF for depth
vhs-depth:
    vhs examples/vhs/depth.tape
    @echo "✅ examples/vhs/output/depth.gif"

# Generate demo GIF for gantt
vhs-gantt:
    vhs examples/vhs/gantt.tape
    @echo "✅ examples/vhs/output/gantt.gif"

# Generate demo GIF for gauge
vhs-gauge:
    vhs examples/vhs/gauge.tape
    @echo "✅ examples/vhs/output/gauge.gif"

# Generate demo GIF for heatmap
vhs-heatmap:
    vhs examples/vhs/heatmap.tape
    @echo "✅ examples/vhs/output/heatmap.gif"

# Generate demo GIF for histogram
vhs-histogram:
    vhs examples/vhs/histogram.tape
    @echo "✅ examples/vhs/output/histogram.gif"

# Generate demo GIF for line
vhs-line:
    vhs examples/vhs/line.tape
    @echo "✅ examples/vhs/output/line.gif"

# Generate demo GIF for radar
vhs-radar:
    vhs examples/vhs/radar.tape
    @echo "✅ examples/vhs/output/radar.gif"

# Generate demo GIF for renko
vhs-renko:
    vhs examples/vhs/renko.tape
    @echo "✅ examples/vhs/output/renko.gif"

# Generate demo GIF for showcase
vhs-showcase:
    vhs examples/vhs/tui_chart.tape
    @echo "✅ examples/vhs/output/tui_chart.gif"

# Generate demo GIF for sparkline
vhs-sparkline:
    vhs examples/vhs/sparkline.tape
    @echo "✅ examples/vhs/output/sparkline.gif"

# Generate demo GIF for chartstack
vhs-chartstack:
    vhs examples/vhs/chartstack.tape
    @echo "✅ examples/vhs/output/chartstack.gif"

# Generate demo GIF for funnel
vhs-funnel:
    vhs examples/vhs/funnel.tape
    @echo "✅ examples/vhs/output/funnel.gif"

# Generate demo GIF for bullet
vhs-bullet:
    vhs examples/vhs/bullet.tape
    @echo "✅ examples/vhs/output/bullet.gif"

# Generate demo GIF for bubble
vhs-bubble:
    vhs examples/vhs/bubble.tape
    @echo "✅ examples/vhs/output/bubble.gif"

# Generate demo GIF for stream
vhs-stream:
    vhs examples/vhs/stream.tape
    @echo "✅ examples/vhs/output/stream.gif"

# Generate demo GIF for treemap
vhs-treemap:
    vhs examples/vhs/treemap.tape
    @echo "✅ examples/vhs/output/treemap.gif"

# Generate demo GIF for waterfall
vhs-waterfall:
    vhs examples/vhs/waterfall.tape
    @echo "✅ examples/vhs/output/waterfall.gif"

# Generate demo GIF for themes
vhs-themes:
    vhs examples/vhs/themes.tape
    @echo "✅ examples/vhs/output/themes.gif"

# Generate all demo GIFs
vhs-all:
    nu scripts/generate_all_tapes.nu
