# Radicle Desktop

[![status-badge](https://woodpecker.radworks.garden/api/badges/6/status.svg)](https://woodpecker.radworks.garden/repos/6)

This desktop application lets you interact with [Radicle][rad], a peer-to-peer code collaboration and publishing stack.

## Installation

**Requirements:**

* *Linux* or *Unix* based operating system.
* Git 2.34 or later
* OpenSSH 9.1 or later with `ssh-agent`

### From binaries

**Debian**

Add the following to your `sources.list`:

```
deb [trusted=yes] https://radicle.xyz/apt unstable main
```

Run from your shell:

```
sudo apt update
sudo apt install radicle-desktop
```

### From source

**Prerequisites:**

- [Node.js][nod] (22.11.0 or higher) and [npm][npm]
- [Rust][rus] toolchain (1.77 or higher)
- [Tauri system dependencies][tau]

Run the following commands to build the desktop app locally:

```
git clone https://seed.radicle.xyz/z4D5UCArafTzTQpDZNQRuqswh3ury.git radicle-desktop
cd radicle-desktop
npm install
npm run tauri build
```

Then run one of the builds that the script outputs at the end.

## Getting in touch

To get in touch with the maintainers, sign up to our [official chat on Zulip][zul].

## License

The UI is distributed under the terms of GPLv3. See [LICENSE][lic] for details.

[lic]: ./LICENSE
[rad]: https://radicle.xyz
[nod]: https://nodejs.org
[npm]: https://www.npmjs.com
[rus]: https://www.rust-lang.org/
[tau]: https://v2.tauri.app/start/prerequisites/#system-dependencies
[zul]: https://radicle.zulipchat.com/#narrow/stream/444463-desktop
