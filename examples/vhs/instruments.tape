# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/instruments.gif"
Set Theme "Aardvark Blue"
Set Width 900
Set Height 520
Set FontSize 16
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cargo build --release --features theme --example instruments 2>&1 | tail -1"
Enter
Wait+Screen /Finished|error/
Type "clear && cargo run --release --features theme --example instruments"
Enter
Sleep 3s
Show

Sleep 2s
Tab
Sleep 2s
Tab
Sleep 2s
Tab
Sleep 2s
Tab
Sleep 2s
Tab
Sleep 2s

Hide
Type "q"
Sleep 0.3s
