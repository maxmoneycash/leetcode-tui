# Leetcode TUI

# Use Leetcode in your terminal.

![Demo](https://vhs.charm.sh/vhs-6HARsZDGe7mSVwTepHFUJv.gif)

### Why this TUI:

My motivation for creating leetcode-tui stemmed from my preference for tools that are lightweight and consume fewer system resources. When I explored existing leetcode CLI tools on GitHub, I came across a few raw command-line interfaces, but they lacked the interactivity I desired.
To address this, I decided to develop leetcode-tui, a Text-based User Interface, that provides an interactive and user-friendly experience for solving LeetCode problems.

> **Warning**
> This TUI is currently under active development. Please feel free to open an issue if you find errors.

## Installation

```sh
cargo install leetcode-tui-rs
```

Post installation

```sh
leetui

# This is going to create `~/.config/leetcode_tui/config.toml`.

# Get the Cookies from the browser `LEETCODE_SESSION` and `csrftoken` and paste it in `~/.config/leetcode_tui/config.toml`

# run the command again to populate db
leetui
```

## Features

- Question grouped by categories
- Read Question
- Jump to question using vim like keybinding (123G).
- Open question in `EDITOR`
- Solve question in multiple languages
- Submit and run solution in multiple languages
- Read Stats of your performance
- Solved questions are marked with ✔️
- Neetcode 75
- For Fuzzy search questions in the question list use `/`
- Grind mode: offline typing trainer with a live WPM candlestick chart (`leetui grind`)

## Grind mode

`leetui grind` launches a typing trainer that needs no leetcode session,
config, or database. Pick a classic problem, type out its solution, and watch
your words-per-minute chart like a stock ticker — every 2 seconds of typing
closes an OHLC candle: green when your pace is climbing, red when it drops.
Finish the run to get your average WPM, keystroke accuracy, session high, and
a bullish/bearish verdict on the session.

```sh
leetui grind
```

Keys: `↑/↓`/`j/k` pick a problem, `Enter` start, `Esc` back to menu,
`Ctrl+r` restart the run, `Enter` (on results) next problem, `Ctrl+c` quit.
Wrong keys don't advance the cursor — they flash red and count against
accuracy. Indentation after a newline is skipped automatically.

Few related projects:

- [https://github.com/skygragon/leetcode-cli](https://github.com/skygragon/leetcode-cli)
- [https://github.com/clearloop/leetcode-cli](https://github.com/clearloop/leetcode-cli)
