# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/voronoi.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cargo build --release --features theme --example voronoi 2>&1 | tail -1"
Enter
Wait+Screen /Finished|error/
Type "clear && cargo run --release --features theme --example voronoi"
Enter
Sleep 3s
Show

Sleep 1s
Type "m"
Sleep 0.8s
Type "m"
Sleep 0.8s
Type "m"
Sleep 0.8s
Type "m"
Sleep 0.8s
Type "w"
Sleep 0.8s
Type "f"
Sleep 0.8s
Type "f"
Sleep 0.8s
Type "r"
Sleep 0.8s
Type "r"
Sleep 0.8s
Type "d"
Sleep 0.8s
Type "d"
Sleep 0.8s
Type "s"
Sleep 0.8s
Type "s"
Sleep 0.8s
Type "a"
Sleep 0.8s

Sleep 1s

Hide
Type "q"
Sleep 0.3s
