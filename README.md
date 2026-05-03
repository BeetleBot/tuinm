# tuinm

this is just a simple networkmanager tui i made in rust because i wanted something that looks a bit better than the default nmtui. its mostly for my own use but i figured i would put it here on github if anyone else wants to use it or find it helpful for their linux setup.

it uses ratatui for the interface and zbus to talk to networkmanager.

### what it does
- shows available wifi networks around you
- list your saved connections and you can delete them if you dont need them anymore
- connects to new networks and asks for a password if its not saved yet
- has a dynamic status bar so u know if u are online or not

### running it
you need to have networkmanager running on your system for this to work. also you need nerd fonts in your terminal if you want the icons to look right.

just do:
cargo run

i probably wont be updating this much unless i need a new feature myself but feel free to check it out.
