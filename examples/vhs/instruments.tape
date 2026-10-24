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
Type "clear && cargo run --release --features theme --example instruments"
Enter
Sleep 5s
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
