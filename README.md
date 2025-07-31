# `zenohi`: Zenoh Interactive Environment

An experimental REPL for interacting with [Zenoh](https://zenoh.io/) systems
built on [Nushell](https://www.nushell.sh/). To get started:

```bash
cargo install --git https://github.com/ZettaScaleLabs/zenohi.git zenohi
zenohi
```

A REPL instance supports multiple Zenoh sessions each identified by a name (string).
On startup, a session named `default` is created. All commands use this session unless
the argument `-s, --session` is supplied:

```console
> zenoh session list
╭───┬─────────┬──────────────────────────────────╮
│ # │  name   │               zid                │
├───┼─────────┼──────────────────────────────────┤
│ 0 │ default │ 41aa8953ad1abda60a9149e25c54067d │
╰───┴─────────┴──────────────────────────────────╯
> zenoh zid -s default
41aa8953ad1abda60a9149e25c54067d
```

To get the list of available commands:

```console
> zenoh --help
```
