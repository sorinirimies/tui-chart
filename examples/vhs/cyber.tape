# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/cyber.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set Framerate 12
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme,shader --example cyber"
Enter
Sleep 4s
Show

Sleep 2s
Type "b"
Sleep 2s
Type "p"
Sleep 2s
Type "p"
Sleep 2s
Type "g"
Sleep 2s
Type "s"
Sleep 2s

Hide
Type "q"
Sleep 0.3s
