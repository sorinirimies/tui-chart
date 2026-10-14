#!/usr/bin/env nu
# Tests for scripts/migrate_to_gitea.nu — local bare repo as "Gitea", no network.

use std/assert
use runner.nu *
use helpers.nu *
use ../migrate_to_gitea.nu [gitea_host ask]

def "test migrate: gitea_host from ssh url" [] {
    assert equal (gitea_host "git@gitea.example.com:user/tui-chart.git") "gitea.example.com"
}

def "test migrate: gitea_host from https url" [] {
    assert equal (gitea_host "https://gitea.example.com/user/tui-chart.git") "gitea.example.com"
}

def "test migrate: gitea_host with port" [] {
    assert equal (gitea_host "https://git.example.com:3000/u/r.git") "git.example.com"
}

def "test migrate: ask returns empty without a terminal" [] {
    assert equal (ask "q? ") ""
}

def "test migrate: fails outside a git repository" [] {
    let d = (make-tmp)
    let r = (run-script $d "migrate_to_gitea.nu" ["--gitea-url" "https://g.example.com/u/r.git"])
    assert equal $r.exit_code 1
    assert ($r.stdout | str contains "Not a git repository")
}

def "test migrate: missing url without terminal fails" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    let r = (run-script $d "migrate_to_gitea.nu" ["--project-dir" $d])
    assert equal $r.exit_code 1
    assert ($r.stdout | str contains "Gitea URL is required")
}

def "test migrate: adds remote via --project-dir" [] {
    let d = (make-tmp)
    write-fixture $d
    init-repo $d
    let bare = (make-tmp "bare")
    git-in $bare init -q --bare | ignore
    let other = (make-tmp "elsewhere")
    let r = (run-script $other "migrate_to_gitea.nu" ["--project-dir" $d "--gitea-url" $bare])
    assert equal $r.exit_code 0 $r.stdout
    assert ($r.stdout | str contains "Gitea remote added")
    assert equal ((git-in $d remote get-url gitea).stdout | str trim) $bare
    assert ($r.stdout | str contains "Migration Complete")
}

def "test migrate: updates existing remote and keeps existing workflows" [] {
    let d = (make-tmp)
    write-fixture $d
    mkdir ($d | path join ".gitea" "workflows")
    "name: x\n" | save ($d | path join ".gitea" "workflows" "ci.yml")
    init-repo $d
    git-in $d remote add gitea "https://old.example.com/u/r.git" | ignore
    let bare = (make-tmp "bare")
    git-in $bare init -q --bare | ignore
    let r = (run-script $d "migrate_to_gitea.nu" ["--gitea-url" $bare])
    assert equal $r.exit_code 0 $r.stdout
    assert ($r.stdout | str contains "already exists")
    assert ($r.stdout | str contains "ci.yml")
    assert equal ((git-in $d remote get-url gitea).stdout | str trim) $bare
}

def "test migrate: does not create workflows or push without confirmation" [] {
    let d = (make-tmp)
    write-fixture $d
    mkdir ($d | path join ".github" "workflows")
    "name: x\n" | save ($d | path join ".github" "workflows" "ci.yml")
    init-repo $d
    let bare = (make-tmp "bare")
    git-in $bare init -q --bare | ignore
    run-script $d "migrate_to_gitea.nu" ["--gitea-url" $bare] | ignore
    assert (not ($d | path join ".gitea" | path exists))
    assert equal ((git-in $bare branch --list).stdout | str trim) ""
}

def main [] { run-tests }
