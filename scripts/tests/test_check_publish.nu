#!/usr/bin/env nu
# Tests for scripts/check_publish.nu — required-file list and failure reporting.

use std/assert
use runner.nu *
use helpers.nu *
use ../check_publish.nu [required_files]

def "test check_publish: required files list" [] {
    let files = (required_files)
    for f in ["README.md" "LICENSE" "Cargo.toml" "CHANGELOG.md" "cliff.toml"] {
        assert ($f in $files) $"missing from list: ($f)"
    }
}

def "test check_publish: real repo has every required file" [] {
    let root = ((scripts-dir) | path join "..")
    for f in (required_files) {
        assert ($root | path join $f | path exists) $"repo is missing ($f)"
    }
}

def "test check_publish: empty dir fails and reports missing files" [] {
    let d = (make-tmp)
    let r = (run-script $d "check_publish.nu")
    assert not equal $r.exit_code 0
    assert ($r.stdout | str contains "missing")
    assert ($r.stdout | str contains "failed")
    assert ($r.stdout | str contains "pre-publish check")
}

def "test check_publish: fixture crate lacking LICENSE and cliff.toml fails" [] {
    let d = (make-tmp)
    write-fixture $d
    let r = (run-script $d "check_publish.nu")
    assert not equal $r.exit_code 0
    assert ($r.stdout | str contains "checking LICENSE")
    assert ($r.stdout | str contains "checking cliff.toml")
    assert ($r.stdout | str contains "missing")
}

def main [] { run-tests }
