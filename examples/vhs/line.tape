# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/line.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme --example line"
Enter
Sleep 5s
Show

Sleep 1s
Type "t"
Sleep 0.7s
Type "m"
Sleep 0.7s
Type "m"
Sleep 0.7s
Type "m"
Sleep 0.7s
Type "k"
Sleep 0.7s
Type "k"
Sleep 0.7s
Type "g"
Sleep 0.7s
Type "s"
Sleep 0.7s
Type "e"
Sleep 0.7s
Type "a"
Sleep 0.7s
Type "y"
Sleep 0.7s
Type "c"
Sleep 0.7s
Right
Sleep 0.7s
Right
Sleep 0.7s
Left
Sleep 0.7s
Sleep 1s

Hide
Type "q"
Sleep 0.3s
