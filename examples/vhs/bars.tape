# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/bars.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --example bars"
Enter
Sleep 5s
Show

Sleep 1s
Type "t"
Sleep 0.7s
Type "v"
Sleep 0.7s
Type "s"
Sleep 0.7s
Type "d"
Sleep 0.7s
Type "o"
Sleep 0.7s
Sleep 1s

Hide
Type "q"
Sleep 0.3s
