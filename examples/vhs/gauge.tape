# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/gauge.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cargo build --release --features theme --example gauge 2>&1 | tail -1"
Enter
Wait+Screen@300s /Finished|error/
Type "clear && ./target/release/examples/gauge"
Enter
Sleep 1s
Show

Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Type "s"
Sleep 1.5s
Up
Sleep 0.5s
Up
Sleep 0.5s
Up
Sleep 0.5s
Down
Sleep 0.5s
Type "s"
Sleep 1.5s
Type "o"
Sleep 1.2s
Type "o"
Sleep 0.8s
Type "t"
Sleep 1s
Sleep 1s

Hide
Type "q"
Sleep 0.3s
