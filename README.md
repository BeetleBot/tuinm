# tuinm

This is just a simple NetworkManager TUI i made in Rust because i always felt nmtui looked a bit ugly and dated. I really liked how the impala tool looks for iwd, so i decided to build something similar but specifically for NetworkManager instead.

It is mostly for my own use but i figured i would put it here on GitHub if anyone else wants to use it or find it helpful for their linux setup.

It uses ratatui for the interface and zbus to talk to NetworkManager.

### What it does
- Shows available wifi networks around you
- list your saved connections and you can delete them if you dont need them anymore
- Connects to new networks and asks for a password if its not saved yet
- Has a dynamic status bar so u know if u are online or not

### Running it
You need to have NetworkManager running on your system for this to work. also you need Nerd Fonts in your terminal if you want the icons to look right.

First clone the repo:
```bash
git clone https://github.com/BeetleBot/tuinm.git
cd tuinm
cargo run
```

I probably wont be updating this much unless i need a new feature myself but feel free to check it out.
