#!/usr/bin/env nu
# ── tui-chart · Run All Nushell Tests ────────────────────────────────────────
#
# Usage:
#   nu scripts/tests/run_all.nu          # run every test_*.nu file
#   nu scripts/tests/run_all.nu --filter version   # only files matching "version"

def main [
    --filter: string = "",  # only run files whose name matches
    --verbose (-v),         # show per-test output for passing files too
] {
    let green  = (ansi green)
    let red    = (ansi red)
    let bold   = (ansi --escape {attr: b})
    let cyan   = (ansi cyan)
    let reset  = (ansi reset)

    print $"($bold)($cyan)── tui-chart · Nushell Test Suite ──($reset)"
    print ""

    let test_dir = ($env.CURRENT_FILE | path dirname)

    let files = (
        ls $test_dir
        | where { |it| ($it.name | path basename) =~ '^test_.*\.nu$' }
        | get name
        | sort
    )

    let files = if $filter == "" {
        $files
    } else {
        $files | where { |f| ($f | path basename) =~ $filter }
    }

    let total = ($files | length)

    if $total == 0 {
        print $"($red)No test files found.(ansi reset)"
        exit 1
    }

    print $"Found ($total) test file\(s\):"
    for f in $files {
        print $"  ($cyan)($f | path basename)($reset)"
    }
    print ""

    mut passed = 0
    mut failed = 0
    mut failures = []

    for f in $files {
        let name = ($f | path basename)
        let res = (do { nu $f } | complete)

        if $res.exit_code == 0 {
            $passed = $passed + 1
            if $verbose and ($res.stdout | str trim) != "" {
                print $res.stdout
            }
        } else {
            if ($res.stdout | str trim) != "" {
                print $res.stdout
            }
            $failed = $failed + 1
            $failures = ($failures | append $name)
            if ($res.stderr | str trim) != "" {
                print $res.stderr
            }
        }
        print $"  (if $res.exit_code == 0 { $green + "✓" } else { $red + "✗" })($reset) ($name)"
    }

    print $"($bold)── Summary ──($reset)"
    print $"  ($green)($passed) passed($reset) · ($red)($failed) failed($reset) · ($total) total"

    if $failed > 0 {
        print ""
        print $"($red)($bold)Failed files:($reset)"
        for f in $failures {
            print $"  ($red)✗($reset) ($f)"
        }
        exit 1
    }

    print ""
    print $"($green)($bold)✓ All ($total) test files passed!($reset)"
}
