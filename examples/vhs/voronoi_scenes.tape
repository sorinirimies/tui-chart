# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/voronoi_scenes.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme --example voronoi"
Enter
Sleep 5s
Show
Sleep 1s
Type "v"
Sleep 1.6s
Type "v"
Sleep 1.6s
Type "v"
Sleep 1.6s
Type "v"
Sleep 1.6s
Type "v"
Sleep 1.6s
Type "d"
Type "n"
Sleep 0.8s
Type "n"
Sleep 0.8s
Type "n"
Sleep 1s

Hide
Type "q"
Sleep 0.3s
