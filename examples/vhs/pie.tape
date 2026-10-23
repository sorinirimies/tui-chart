# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/pie.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme --example pie"
Enter
Sleep 5s
Show

Sleep 1s
Type "b"
Sleep 1.2s
Type "r"
Sleep 1.2s
Type "l"
Sleep 1.2s
Type "l"
Sleep 1.2s
Type "b"
Sleep 1.2s
Sleep 1s

Hide
Type "q"
Sleep 0.3s
