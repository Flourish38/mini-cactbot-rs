# mini-cactbot-rs

This is a discord bot that provides a solver for the Mini Cactpot minigame from Final Fantasy XIV. I put a lot of effort into the user experience, so hopefully it should be pretty intuitive. If you experience any strange behavior, hitting the ↩ UNDO button should correct it, but I implemented lots of double-checks and made sure to inform the user if anything unexpected happens, so it *should* be able to guide you through the process on its own.

### How to use it

Download the code, [make sure Rust is installed](https://www.rust-lang.org/tools/install), and then simply type `cargo build --release` into your terminal from the `mini-cactbot-rs` directory. After a few minutes, this should give you an executable `mini-cactbot` in `./target/release/`, which you can move wherever you like.

Put your token in a file called `config.(ini|json|yaml|toml|ron|json5)` with the key "token".
You can also specify admin users in an array with the key "admins". This is only used for the shutdown command.
**If you do not do this, then any user will be able to shut down your bot.**

For example, a file `config.toml` would look like:
```toml
token = "TOKEN_GOES_HERE"
admins = [ 123456789876543210 ]
```

A default configuration file is provided at `src/config.toml`.
In order to use it, simply move it into the same directory as your executable `mini-cactbot` file.

Alternatively, you can instead provide your token via the environment variable `DISCORD_TOKEN`.
This will override the value provided in the config file, if any.
**If you do this, you will probably still want to provide a list of admin users in the config file. Otherwise, any user will be able to shut down your bot.**

Template made by [Flourish38](https://github.com/Flourish38).
