# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/renko.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cargo build --release --features theme --example renko 2>&1 | tail -1"
Enter
Wait+Screen /Finished|error/
Type "clear && cargo run --release --features theme --example renko"
Enter
Sleep 3s
Show

Sleep 1s
Type "t"
Sleep 0.7s
Type "p"
Sleep 0.7s
Type "+"
Sleep 0.7s
Type "-"
Sleep 0.7s
Sleep 1s

Hide
Type "q"
Sleep 0.3s
