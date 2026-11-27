# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/transitions.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme,tachyonfx --example transitions"
Enter
Sleep 6s
Show

Sleep 2s
Type "2"
Sleep 2.2s
Type "3"
Sleep 2.2s
Type "4"
Sleep 2.2s
Type "5"
Sleep 2.2s
Type "v"
Sleep 2.2s
Type "1"
Sleep 2.2s

Hide
Type "q"
Sleep 0.3s
