#!/usr/bin/env nu
# Shared helpers for the script tests (not a test file itself).

# Absolute path of the scripts/ directory.
export def scripts-dir []: nothing -> string {
    $env.CURRENT_FILE | path dirname | path join ".." | path expand
}

# Absolute path of a script in scripts/.
export def script [name: string]: nothing -> string {
    scripts-dir | path join $name
}

# Fresh empty temp directory.
export def make-tmp [prefix: string = "tuichart"]: nothing -> string {
    let d = (mktemp --directory --tmpdir $"($prefix).XXXXXX")
    $d | path expand
}

# Minimal single-crate Cargo.toml (no dependencies, so cargo works offline).
export def fixture-cargo [version: string = "0.1.0"]: nothing -> string {
    $'[package]
name = "tui-chart"
version = "($version)"
edition = "2021"
description = "fixture"
license = "MIT"

[dependencies]
'
}

# Write a fixture crate (Cargo.toml, src/lib.rs, README.md, CHANGELOG.md) to `dir`.
export def write-fixture [dir: string, version: string = "0.1.0"] {
    mkdir ($dir | path join "src")
    fixture-cargo $version | save --force ($dir | path join "Cargo.toml")
    "//! fixture\n" | save --force ($dir | path join "src" "lib.rs")
    $"# fixture\n\n![v]\(https://img.shields.io/badge/version-($version)-blue\)\n" | save --force ($dir | path join "README.md")
    "# Changelog\n" | save --force ($dir | path join "CHANGELOG.md")
}

# Git environment so commits/tags never depend on the user's global config.
export def git-env []: nothing -> record {
    {
        GIT_AUTHOR_NAME: "Test"
        GIT_AUTHOR_EMAIL: "test@example.com"
        GIT_COMMITTER_NAME: "Test"
        GIT_COMMITTER_EMAIL: "test@example.com"
        GIT_CONFIG_GLOBAL: "/dev/null"
        GIT_CONFIG_SYSTEM: "/dev/null"
    }
}

# Run git in `dir`; returns the `complete` record.
export def --wrapped git-in [dir: string, ...args: string]: nothing -> record {
    with-env (git-env) { do { ^git -C $dir ...$args } | complete }
}

# `git init` + initial commit of everything in `dir`.
export def init-repo [dir: string] {
    git-in $dir init -q -b main | ignore
    git-in $dir add -A | ignore
    git-in $dir commit -q -m "init" | ignore
}

# Run `nu <script> ...args` inside `dir` with optional stdin text. Returns `complete` record.
export def run-script [dir: string, script_name: string, args: list<string> = [], --stdin: string = ""]: nothing -> record {
    let path = (script $script_name)
    with-env (git-env) {
        do {
            cd $dir
            $stdin | ^nu --no-config-file $path ...$args
        } | complete
    }
}
