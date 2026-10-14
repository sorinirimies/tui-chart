# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/heatmap.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cargo run --release --example heatmap"
Enter
Sleep 5s
Show

Sleep 1s
Type "t"
Sleep 0.7s
Type "v"
Sleep 0.7s
Type "g"
Sleep 0.7s
Sleep 1s

Hide
Type "q"
Sleep 0.3s
