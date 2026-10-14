#!/usr/bin/env nu
# Tests for scripts/upgrade_deps.nu — helpers and pre-flight guard (no network).

use std/assert
use runner.nu *
use helpers.nu *
use ../upgrade_deps.nu [commit_label all_passed run_check is_dirty do_commit_and_push]

def "test upgrade_deps: commit_label joins basenames" [] {
    assert equal (commit_label ["Cargo.toml" "Cargo.lock"]) "Cargo.toml + Cargo.lock"
    assert equal (commit_label ["a/b/Cargo.lock"]) "Cargo.lock"
}

def "test upgrade_deps: commit_label of empty list" [] {
    assert equal (commit_label []) ""
}

def "test upgrade_deps: all_passed" [] {
    assert (all_passed [true true true])
    assert (not (all_passed [true false true]))
    assert (all_passed [])
}

def "test upgrade_deps: run_check true on success" [] {
    assert (run_check "ok" { ^true })
}

def "test upgrade_deps: run_check false on failure" [] {
    assert (not (run_check "bad" { ^sh -c 'echo boom >&2; exit 3' }))
}

def "test upgrade_deps: is_dirty detects modifications" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    cd $d
    assert (not (is_dirty "Cargo.toml"))
    "# changed\n" | save --append ($d | path join "Cargo.toml")
    assert (is_dirty "Cargo.toml")
    assert (not (is_dirty "README.md"))
}

def "test upgrade_deps: do_commit_and_push dry-run changes nothing" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    cd $d
    "# changed\n" | save --append ($d | path join "Cargo.toml")
    do_commit_and_push ["Cargo.toml"] "msg" "main" "origin" true
    assert equal (git-in $d log --oneline | get stdout | lines | length) 1
    assert (is_dirty "Cargo.toml")
}

def "test upgrade_deps: dirty Cargo.toml aborts before any cargo call" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    "# changed\n" | save --append ($d | path join "Cargo.toml")
    let r = (run-script $d "upgrade_deps.nu" ["--dry-run"])
    assert equal $r.exit_code 1
    assert ($r.stdout | str contains "DRY-RUN")
    assert ($r.stdout | str contains "already has changes")
    assert ((open ($d | path join "Cargo.toml") --raw) | str contains "# changed")
}

def "test upgrade_deps: --help lists flags" [] {
    let d = (make-tmp)
    let r = (run-script $d "upgrade_deps.nu" ["--help"])
    assert equal $r.exit_code 0
    for f in ["--dry-run" "--no-commit" "--bot-name" "--remote"] {
        assert ($r.stdout | str contains $f) $"help missing ($f)"
    }
}

def main [] { run-tests }
