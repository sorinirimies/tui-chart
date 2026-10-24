# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/radial.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme --example radial"
Enter
Sleep 5s
Show

Sleep 2s
Type "m"
Sleep 1.5s
Type "f"
Sleep 1.5s
Type "m"
Sleep 1.5s
Type "f"
Sleep 1.5s
Type "l"
Sleep 1.5s
Type "l"
Sleep 1.5s
Type "t"
Sleep 1.5s

Hide
Type "q"
Sleep 0.3s
