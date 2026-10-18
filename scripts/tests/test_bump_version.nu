#!/usr/bin/env nu
# Tests for scripts/bump_version.nu — helper fns, validation/guard error paths
# and one full offline run in a throw-away git repo.

use std/assert
use runner.nu *
use helpers.nu *
use ../bump_version.nu [is_valid_version version_cmp read_version set_version has_version_badge set_badge_version]

def "test bump: valid versions" [] {
    for v in ["0.0.1" "1.2.3" "10.20.30" "0.4.0-beta.1" "1.0.0-rc1"] {
        assert (is_valid_version $v) $"expected valid: ($v)"
    }
}

def "test bump: invalid versions" [] {
    for v in ["1" "1.2" "v1.2.3" "1.2.3.4" "1.2.x" "" "1.2.3-" "a.b.c" "1.2.3 "] {
        assert (not (is_valid_version $v)) $"expected invalid: '($v)'"
    }
}

def "test bump: read_version takes package version" [] {
    assert equal (read_version (fixture-cargo "3.4.5")) "3.4.5"
}

def "test bump: read_version ignores dependency version fields" [] {
    let toml = $"(fixture-cargo "0.1.0")\nserde = { version = \"1\" }\n"
    assert equal (read_version $toml) "0.1.0"
}

def "test bump: set_version rewrites only the version line" [] {
    let out = (set_version (fixture-cargo "0.1.0") "0.2.0")
    assert equal (read_version $out) "0.2.0"
    assert ($out | str contains 'name = "tui-chart"')
    assert ($out | str contains 'edition = "2021"')
}

def "test bump: set_version leaves inline dependency versions alone" [] {
    let toml = $"(fixture-cargo "0.1.0")\nserde = { version = \"1\" }\n"
    let out = (set_version $toml "9.9.9")
    assert ($out | str contains 'serde = { version = "1" }')
}

def "test bump: badge detection and rewrite" [] {
    let readme = "![v](https://img.shields.io/badge/version-0.1.0-blue) and ![w](x/version-0.1.0-blue)"
    assert (has_version_badge $readme)
    let out = (set_badge_version $readme "0.2.0-beta.1")
    assert equal ($out | str replace --all "0.2.0-beta.1" "" | str contains "0.1.0") false
    assert (($out | split row "version-0.2.0-beta.1-blue" | length) == 3)
}

def "test bump: no badge detected" [] {
    assert (not (has_version_badge "# nothing here"))
    assert (not (has_version_badge "version-latest-blue"))
}

def "test bump: rejects invalid version" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    let r = (run-script $d "bump_version.nu" ["--yes" "not-semver"])
    assert not equal $r.exit_code 0
    assert ($r.stderr | str contains "Invalid version format")
    assert equal (read_version (open ($d | path join "Cargo.toml") --raw)) "0.1.0"
}

def "test bump: rejects same version" [] {
    let d = (make-tmp)
    write-fixture $d "0.1.0"
    init-repo $d
    let r = (run-script $d "bump_version.nu" ["--yes" "0.1.0"])
    assert not equal $r.exit_code 0
    assert ($r.stderr | str contains "Already at version")
}

def "test bump: version_cmp orders versions" [] {
    assert equal (version_cmp "0.1.0" "0.1.1") (-1)
    assert equal (version_cmp "0.2.0" "0.1.9") 1
    assert equal (version_cmp "1.0.0" "1.0.0") 0
    assert equal (version_cmp "0.10.0" "0.9.0") 1
    assert equal (version_cmp "1.0.0-rc1" "1.0.0") (-1)
    assert equal (version_cmp "1.0.0" "1.0.0-rc1") 1
}

def "test bump: rejects downgrade" [] {
    let d = (make-tmp)
    write-fixture $d "0.3.0"
    init-repo $d
    let r = (run-script $d "bump_version.nu" ["--yes" "--skip-checks" "0.2.0"])
    assert not equal $r.exit_code 0
    assert ($r.stderr | str contains "lower than current")
    assert equal (read_version (open ($d | path join "Cargo.toml") --raw)) "0.3.0"
}

def "test bump: rejects existing tag" [] {
    let d = (make-tmp)
    write-fixture $d "0.1.0"
    init-repo $d
    git-in $d tag v0.2.0 | ignore
    let r = (run-script $d "bump_version.nu" ["--yes" "--skip-checks" "0.2.0"])
    assert not equal $r.exit_code 0
    assert ($r.stderr | str contains "already exists")
    assert equal (read_version (open ($d | path join "Cargo.toml") --raw)) "0.1.0"
}

def "test bump: aborts when confirmation declined" [] {
    let d = (make-tmp)
    write-fixture $d "0.1.0"
    init-repo $d
    let r = (run-script $d "bump_version.nu" ["0.2.0"] --stdin "n\n")
    assert equal $r.exit_code 0
    assert ($r.stdout | str contains "Aborted")
    assert equal (read_version (open ($d | path join "Cargo.toml") --raw)) "0.1.0"
}

def "test bump: full run in temp repo commits and tags" [] {
    let d = (make-tmp)
    write-fixture $d "0.1.0"
    # fixture needs a lockfile + formatted code so fmt/clippy/test steps pass offline
    do { cd $d; ^cargo generate-lockfile --offline } | complete | ignore
    "//! fixture\n" | save --force ($d | path join "src" "lib.rs")
    init-repo $d
    let r = (run-script $d "bump_version.nu" ["--yes" "--skip-checks" "0.2.0"])
    assert equal $r.exit_code 0 $"stdout: ($r.stdout)\nstderr: ($r.stderr)"
    assert equal (read_version (open ($d | path join "Cargo.toml") --raw)) "0.2.0"
    assert (open ($d | path join "README.md") --raw | str contains "version-0.2.0-blue")
    assert ((git-in $d tag).stdout | str contains "v0.2.0")
    assert ((git-in $d log -1 --format=%s).stdout | str contains "chore: bump version to 0.2.0")
}

def main [] { run-tests }
