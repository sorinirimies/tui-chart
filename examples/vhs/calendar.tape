# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/calendar.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme --example calendar"
Enter
Sleep 5s
Show

Sleep 1s
Type "g"
Sleep 0.8s
Type "m"
Sleep 0.8s
Type "c"
Sleep 0.8s
Type "l"
Sleep 0.8s
Type "r"
Sleep 0.8s
Type "t"
Sleep 0.8s
Sleep 1s

Hide
Type "q"
Sleep 0.3s
