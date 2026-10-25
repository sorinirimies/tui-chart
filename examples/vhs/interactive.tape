# A VHS tape. See https://github.com/charmbracelet/vhs

# for running against the tui-chart git repo https://github.com/sorinirimies/tui-chart
Output "examples/vhs/output/interactive.gif"
Set Theme "Aardvark Blue"
Set Width 1200
Set Height 700
Set FontSize 22
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "clear && cargo run --release --features theme --example interactive"
Enter
Sleep 5s
Show

Sleep 1s
Right 10
Sleep 1s
Right 5
Sleep 1s
Space
Sleep 1.5s
Right 20
Up 6
Left 2
Sleep 1s
Space
Sleep 1.5s
Space
Sleep 1s
Type "+++"
Sleep 1.5s
Type "]"
Sleep 1.5s
Type "r"
Sleep 1s
Tab
Sleep 0.5s
Left 25
Down 3
Sleep 1.5s
Right 8
Sleep 1.5s
Right 8
Sleep 1.5s
Tab
Sleep 0.5s
Left 10
Sleep 1.5s
Right 12
Sleep 1.5s
Tab 3
Sleep 0.5s
Left 8
Sleep 1.5s
Right 10
Sleep 1.5s

Hide
Type "q"
Sleep 0.3s
