#!/usr/bin/env nu
# Tests for scripts/generate_all_tapes.nu — discovery helpers and runs against a fake `vhs`.

use std/assert
use runner.nu *
use helpers.nu *
use ../generate_all_tapes.nu [discover_tapes tape_name]

# Put a fake `vhs` (exit code `code`) first on PATH and run the script in `dir`.
def run-with-fake-vhs [dir: string, code: int] {
    let bin = (make-tmp "fakebin")
    $"#!/bin/sh\necho fake-vhs \"$@\" >> ($bin)/calls.log\nexit ($code)\n" | save ($bin | path join "vhs")
    chmod +x ($bin | path join "vhs")
    let path = ([$bin] | append $env.PATH | str join (char esep))
    let r = (with-env {PATH: $path} { run-script $dir "generate_all_tapes.nu" })
    {result: $r, bin: $bin}
}

def make-project [tapes: list<string>]: nothing -> string {
    let d = (make-tmp)
    mkdir ($d | path join "examples" "vhs")
    for t in $tapes { "Output x.gif\n" | save ($d | path join "examples" "vhs" $"($t).tape") }
    $d
}

def "test tapes: tape_name strips dir and extension" [] {
    assert equal (tape_name "examples/vhs/line.tape") "line"
    assert equal (tape_name "a.tape.tape") "a.tape"
}

def "test tapes: discover_tapes sorted and filtered" [] {
    let d = (make-project ["b" "a" "c"])
    "x" | save ($d | path join "examples" "vhs" "notes.txt")
    let found = (discover_tapes ($d | path join "examples" "vhs") | each {|p| tape_name $p })
    assert equal $found ["a" "b" "c"]
}

def "test tapes: discover_tapes empty dir" [] {
    let d = (make-project [])
    assert equal (discover_tapes ($d | path join "examples" "vhs")) []
}

def "test tapes: real repo has tapes" [] {
    let root = ((scripts-dir) | path join "..")
    assert ((discover_tapes ($root | path join "examples" "vhs") | length) > 0)
}

def "test tapes: missing vhs dir fails" [] {
    let d = (make-tmp)
    let out = (run-with-fake-vhs $d 0)
    assert not equal $out.result.exit_code 0
    assert ($out.result.stderr | str contains "not found")
}

def "test tapes: no tapes found fails" [] {
    let d = (make-project [])
    let out = (run-with-fake-vhs $d 0)
    assert not equal $out.result.exit_code 0
    assert ($out.result.stderr | str contains "No .tape files")
}

def "test tapes: runs vhs for every tape" [] {
    let d = (make-project ["a" "b"])
    let out = (run-with-fake-vhs $d 0)
    assert equal $out.result.exit_code 0 $out.result.stderr
    assert ($out.result.stdout | str contains "Found 2 tape")
    assert equal (open ($out.bin | path join "calls.log") | lines | length) 2
    assert ($d | path join "examples" "vhs" "output" | path exists)
}

def "test tapes: failing vhs exits non-zero" [] {
    let d = (make-project ["a" "b"])
    let out = (run-with-fake-vhs $d 1)
    assert equal $out.result.exit_code 1
    assert ($out.result.stdout | str contains "Failed: 2")
}

def main [] { run-tests }
