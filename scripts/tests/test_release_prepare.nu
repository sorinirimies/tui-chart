#!/usr/bin/env nu
# Tests for scripts/release_prepare.nu — release-note builder and error paths.

use std/assert
use runner.nu *
use helpers.nu *
use ../release_prepare.nu [build_release_notes]

def "test release_prepare: notes contain title and version" [] {
    let n = (build_release_notes "1.2.3" "v1.2.2" "- fix: a thing")
    assert ($n | str starts-with "# tui-chart 1.2.3")
    assert ($n | str contains 'tui-chart = "1.2.3"')
}

def "test release_prepare: notes embed changes and last tag" [] {
    let n = (build_release_notes "1.2.3" "v1.2.2" "- fix: a thing")
    assert ($n | str contains "Changes since v1.2.2")
    assert ($n | str contains "- fix: a thing")
    assert (not ($n | str contains "Initial Release"))
}

def "test release_prepare: initial release without last tag" [] {
    let n = (build_release_notes "0.1.0" "" "- feat: first")
    assert ($n | str contains "Initial Release")
    assert (not ($n | str contains "Changes since"))
}

def "test release_prepare: notes reference real tui-chart API" [] {
    let n = (build_release_notes "0.1.0" "" "x")
    assert ($n | str contains "use tui_chart::{LineChart, Series};")
    assert (not ($n | str contains "Slider"))
}

def "test release_prepare: code fences are balanced" [] {
    let n = (build_release_notes "0.1.0" "" "x")
    let fences = ($n | lines | where {|l| $l | str starts-with '```' } | length)
    assert equal ($fences mod 2) 0
}

def "test release_prepare: requires a tag argument" [] {
    let d = (make-tmp)
    write-fixture $d
    let r = (run-script $d "release_prepare.nu")
    assert not equal $r.exit_code 0
}

def "test release_prepare: --help shows usage" [] {
    let d = (make-tmp)
    let r = (run-script $d "release_prepare.nu" ["--help"])
    assert equal $r.exit_code 0
    assert ($r.stdout | str contains "Release tag")
}

def "test release_prepare: full run in temp repo writes notes" [] {
    if (which git-cliff | is-empty) { return }   # optional tool, skipped when absent
    let d = (make-tmp)
    write-fixture $d "0.1.0"
    cp (scripts-dir | path join ".." "cliff.toml") ($d | path join "cliff.toml")
    do { cd $d; ^cargo generate-lockfile --offline } | complete | ignore
    init-repo $d
    let r = (run-script $d "release_prepare.nu" ["v0.9.0"])
    assert equal $r.exit_code 0 $"($r.stdout)\n($r.stderr)"
    assert equal (open ($d | path join "Cargo.toml") | get package.version) "0.9.0"
    let notes = (open ($d | path join "RELEASE_NOTES.md") --raw)
    assert ($notes | str contains "# tui-chart 0.9.0")
    assert ($notes | str contains "Initial Release")
    assert (not ($d | path join "CLIFF_CHANGES.md" | path exists))
}

def main [] { run-tests }
