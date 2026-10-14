#!/usr/bin/env nu
# Tests for scripts/setup_gitea.nu — uses a local bare repo as the "Gitea" remote.

use std/assert
use runner.nu *
use helpers.nu *
use ../setup_gitea.nu [ask]

def "test setup_gitea: ask returns empty without a terminal" [] {
    assert equal (ask "q? ") ""
}

def "test setup_gitea: fails outside a git repository" [] {
    let d = (make-tmp)
    let r = (run-script $d "setup_gitea.nu" ["https://gitea.example.com/u/tui-chart.git"])
    assert equal $r.exit_code 1
    assert ($r.stdout | str contains "Not a git repository")
}

def "test setup_gitea: requires url argument" [] {
    let d = (make-tmp)
    let r = (run-script $d "setup_gitea.nu")
    assert not equal $r.exit_code 0
}

def "test setup_gitea: adds gitea remote" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    let bare = (make-tmp "bare")
    git-in $bare init -q --bare | ignore
    let r = (run-script $d "setup_gitea.nu" [$bare])
    assert equal $r.exit_code 0 $r.stdout
    assert ($r.stdout | str contains "Gitea remote added")
    assert equal ((git-in $d remote get-url gitea).stdout | str trim) $bare
    assert ($r.stdout | str contains "Successfully connected")
}

def "test setup_gitea: updates existing gitea remote" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    git-in $d remote add gitea "https://old.example.com/u/r.git" | ignore
    let bare = (make-tmp "bare")
    git-in $bare init -q --bare | ignore
    let r = (run-script $d "setup_gitea.nu" [$bare])
    assert equal $r.exit_code 0 $r.stdout
    assert ($r.stdout | str contains "Gitea remote URL updated")
    assert equal ((git-in $d remote get-url gitea).stdout | str trim) $bare
}

def "test setup_gitea: unreachable remote is only a warning" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    let r = (run-script $d "setup_gitea.nu" [($d | path join "does-not-exist.git")])
    assert equal $r.exit_code 0 $r.stdout
    assert ($r.stdout | str contains "Could not connect")
}

def "test setup_gitea: nothing is pushed without confirmation" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    let bare = (make-tmp "bare")
    git-in $bare init -q --bare | ignore
    run-script $d "setup_gitea.nu" [$bare] | ignore
    assert equal ((git-in $bare branch --list).stdout | str trim) ""
}

def main [] { run-tests }
