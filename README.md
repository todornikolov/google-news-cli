# Google News CLI

The top ten Google News results for a query, in your terminal, with one short command: `gnews`. Move with the arrow keys, press Enter to open a story in your browser. No account, no API key.

```sh
gnews rust                  # search, newest stories first
gnews --since day rust      # only the last 24 hours
gnews --relevance rust      # Google's own ranking instead
```

Works on Linux, macOS and Windows. You build it once from source; it takes a few minutes.

---

## Install

Pick your system. Each one has the same three parts: **1.** install Rust (and a C compiler), **2.** get the code, **3.** install the `gnews` command.

### Linux (Ubuntu, Debian, Fedora, Arch...)

**1. Install a C compiler, git and Rust**

```sh
# Ubuntu / Debian
sudo apt update && sudo apt install -y build-essential curl git xdg-utils

# Fedora
sudo dnf install -y gcc curl git xdg-utils

# Arch
sudo pacman -S --needed base-devel curl git xdg-utils

# Rust (all distros) - accept the defaults
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
```

**2. Get the code**

```sh
git clone <repo-url> gnews-cli
cd gnews-cli
```

**3. Install the command**

```sh
cargo install --path .
```

### macOS

**1. Install the developer tools, then Rust**

```sh
xcode-select --install      # a window opens; click Install (skip if already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
```

(Prefer Homebrew? `brew install rust` also works, provided it's version 1.88 or newer.)

**2 and 3.** Same as Linux:

```sh
git clone <repo-url> gnews-cli
cd gnews-cli
cargo install --path .
```

### Windows 10 / 11

Use **Windows Terminal** with PowerShell. It's built into Windows 11; on Windows 10 get it from the Microsoft Store.

**1. Install Rust and the C++ build tools**

Download and run `rustup-init.exe` from <https://rustup.rs>. If it says the Visual Studio C++ Build Tools are missing, accept its offer to install them (choose **Desktop development with C++**) and then continue with the default Rust install. This is a big download, but you only do it once.

Also install [Git for Windows](https://git-scm.com/download/win) if you don't have it.

**2 and 3.** Open a **new** PowerShell window (so it sees the new `PATH`) and run:

```powershell
git clone <repo-url> gnews-cli
cd gnews-cli
cargo install --path .
```

### Check that it worked

```sh
gnews --version
gnews rust
```

`cargo install` puts the program in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` on Windows). The Rust installer adds that folder to your `PATH`; if `gnews` is "not found", open a new terminal window.

### Update or remove

```sh
git pull && cargo install --path .    # update (run inside the project folder)
cargo uninstall gnews               # remove
```

### Just build it, no install

```sh
cargo build --release
```

The program is then at `target/release/gnews` (`target\release\gnews.exe` on Windows). It's a single file you can copy anywhere on the same type of machine, and it needs nothing else installed to run.

---

## Use it

```sh
gnews rust tui                       # several words, no quotes needed
gnews --since day rust               # only the last day: hour, day, week or month
gnews --relevance rust               # Google's own ranking instead of newest first
gnews -s hour rust                   # last hour only, newest first
gnews --country bg --lang bg софия   # Bulgarian edition
```

| Option | Values | What it does |
|---|---|---|
| `-s`, `--since` | `hour`, `day`, `week`, `month` | Only stories from that long ago until now (`month` means the last 30 days) |
| `--relevance` | | Use Google's own ranking instead of newest first (the default) |
| `--country` | e.g. `us`, `gb`, `de`, `bg` (default `us`) | Country edition of Google News |
| `--lang` | e.g. `en`, `de`, `bg` (default `en`) | Language of the results |
| `-h`, `--help` | | Show help |

**Keys:** `↑` `↓` or `j` `k` move · `Enter` opens the selected story in your browser · `q` or `Esc` quits. The selected link is shown at the bottom of the screen.

`--since` uses Google's own `when:` search operator, so Google does the filtering. Results are sorted newest first by default. The sort happens *before* picking the top ten, so you get the ten newest stories, not just the newest of Google's first ten. Stories without a date go last. Add `--relevance` to keep Google's own order instead.

---

## Requirements

| What | Needed for | Notes |
|---|---|---|
| **Rust 1.88 or newer** | building | `rustup update` if yours is older. Tested with 1.92. |
| **A C compiler** | building | `build-essential`/`gcc` on Linux, Xcode Command Line Tools on macOS, Visual Studio C++ Build Tools on Windows. One of the dependencies (`ring`, for HTTPS) compiles a little C. |
| **git** | getting the code | Or download the ZIP from the repository page. |
| **Internet access** | building and running | Building downloads dependencies from crates.io; running reaches `news.google.com`. HTTP(S) proxies set by the usual `HTTPS_PROXY` / `ALL_PROXY` environment variables are honored. |
| **A web browser, plus an opener** | pressing Enter | Windows and macOS have one built in. Linux needs `xdg-open` (package `xdg-utils`). |
| **An interactive terminal** | running | Any modern one. UTF-8 recommended. |

You do **not** need OpenSSL, Node, Python, or any API key. HTTPS is handled by pure-Rust `rustls`.

## Not supported, or known limits

- **News only.** It searches Google News, not the general web. Google's web search API is closed to new users and its pages need JavaScript, so there's no key-free way to do it.
- **Needs a real, interactive terminal.** It is a full-screen app. Piping its output (`gnews rust | grep ...`), running it from cron or CI, or running it with no terminal won't work. There's no plain-text output mode.
- **No browser, no Enter.** On a headless Linux server, a bare container, or over SSH with no `xdg-open`, search and browsing work, but pressing Enter exits the app with `Error: No such file or directory (os error 2)`. Copy the link shown at the bottom of the screen instead.
- **Old Rust.** Anything older than 1.88 fails to build. Cargo tells you so, and `rustup update` fixes it.
- **Offline or blocked networks.** If `news.google.com` can't be reached you get an error and no results.
- **Ten results, as Google returns them.** The feed holds roughly 100 stories at most. A narrow `--since hour` may return fewer than ten, or none ("No results.").
- **`--since` depends on an undocumented Google operator (`when:`).** It works today, but Google could change it.
- **Links go through Google.** Story links are `news.google.com/rss/articles/...` redirects that take you to the article in your browser.
- **Display.** Needs a UTF-8 terminal and a font with arrows and block characters (`↑ ↓ ▌ ·`). The old Windows `cmd.exe` console window may show boxes; use Windows Terminal.
- **CPUs.** Expected to work on 64-bit Intel/AMD and ARM (including Apple Silicon and 64-bit Raspberry Pi OS). Other architectures are untested.
- **WSL.** Links open in the Windows browser through PowerShell, which usually works out of the box.

> **Tested on:** Ubuntu Linux. macOS and Windows should work, since every dependency is cross-platform, but they haven't been tried by the author. If something fails there, please open an issue with the error message.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `gnews: command not found` | Open a new terminal. If it persists, add `~/.cargo/bin` to your `PATH`. |
| `error: linker 'cc' not found` or `link.exe not found` | Install the C compiler from the table above, then run the build again. |
| `requires rustc 1.88 or newer` | Run `rustup update`. |
| `Error: ... dns error` / timeout | No route to `news.google.com`. Check your connection or proxy variables. |
| `No results.` | Try a wider `--since` or no `--since`. |
| `Error: No such file or directory (os error 2)` after pressing Enter | No browser opener found. Install `xdg-utils` (Linux), or copy the link from the bottom line of the screen next time. |

---

## For developers

```sh
cargo test                 # 29 tests: parsing, ordering, key handling, screen rendering, CLI arguments
cargo run -- rust          # run without installing
cargo clippy --all-targets && cargo fmt --check
```

```
src/
  main.rs   command-line arguments (clap), the `Hit` type, runs the search, shows the results
  news.rs   Google News: one HTTP GET (ureq), XML into items (rss), sorting, items into `Hit`s
  ui.rs     the ratatui screen: list, selected link, keys
```

Unit tests live next to the code they test, in a `#[cfg(test)] mod tests` block at the bottom of each file. The screen is rendered into an in-memory terminal (`TestBackend`) to check what's on it, and the key handling is a pure function (`App::on_key`) so it needs no real terminal or browser.

### Ideas for next steps

- Press `/` to type a new query without leaving the app (try the `tui-input` crate)
- Remember recent queries in a file (`dirs` + `serde_json`)
- A plain-text output mode (`--print`) for scripts and headless machines
