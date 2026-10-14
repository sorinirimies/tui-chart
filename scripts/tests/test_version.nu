#!/usr/bin/env nu
# Tests for scripts/version.nu — prints the version from Cargo.toml.

use std/assert
use runner.nu *
use helpers.nu *

def "test version: prints Cargo.toml version" [] {
    let d = (make-tmp)
    write-fixture $d "1.2.3"
    let r = (run-script $d "version.nu")
    assert equal $r.exit_code 0
    assert equal ($r.stdout | str trim) "1.2.3"
}

def "test version: prints pre-release version" [] {
    let d = (make-tmp)
    write-fixture $d "0.5.0-rc.1"
    let r = (run-script $d "version.nu")
    assert equal ($r.stdout | str trim) "0.5.0-rc.1"
}

def "test version: fails without Cargo.toml" [] {
    let d = (make-tmp)
    let r = (run-script $d "version.nu")
    assert not equal $r.exit_code 0
}

def "test version: real repo version matches Cargo.toml" [] {
    let root = ((scripts-dir) | path join "..")
    let r = (run-script $root "version.nu")
    assert equal $r.exit_code 0
    assert equal ($r.stdout | str trim) (open ($root | path join "Cargo.toml") | get package.version)
}

def main [] { run-tests }
