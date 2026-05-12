# Radicle Desktop

[![lint](https://github.com/radicle-dev/radicle-desktop/actions/workflows/lint.yaml/badge.svg?branch=main)](https://github.com/radicle-dev/radicle-desktop/actions/workflows/lint.yaml) [![unit-test](https://github.com/radicle-dev/radicle-desktop/actions/workflows/unit-test.yaml/badge.svg?branch=main)](https://github.com/radicle-dev/radicle-desktop/actions/workflows/unit-test.yaml) [![e2e](https://github.com/radicle-dev/radicle-desktop/actions/workflows/e2e.yaml/badge.svg?branch=main)](https://github.com/radicle-dev/radicle-desktop/actions/workflows/e2e.yaml)

This desktop application lets you interact with [Radicle][rad], a peer-to-peer code collaboration and publishing stack.

## Installation

See [radicle.dev/desktop#install](https://radicle.dev/desktop#install).

### From source

**Prerequisites:**

- [Node.js][nod] (24 or higher) and [npm][npm]
- [Rust][rus] toolchain (1.77 or higher)
- [Tauri system dependencies][tau]

On Windows, NPM's script shell must be set to a POSIX compatible shell such as
[Git Bash][gfw]:

```
npm config set script-shell "C:\\Program Files\\git\\bin\\bash.exe"
```

Run the following commands to build the desktop app locally:

```
git clone https://seed.radicle.dev/z4D5UCArafTzTQpDZNQRuqswh3ury.git radicle-desktop
cd radicle-desktop
npm install
npm run tauri build
```

Then run one of the builds that the script outputs at the end.

## Development

To start the app in development mode with hot reload:

```
npm run dev
```

## Getting in touch

To get in touch with the maintainers, sign up to our [official chat on Zulip][zul].

## License

The UI is distributed under the terms of GPLv3. See [LICENSE][lic] for details.

[gfw]: https://gitforwindows.org
[lic]: ./LICENSE
[rad]: https://radicle.dev
[nod]: https://nodejs.org
[npm]: https://www.npmjs.com
[rus]: https://www.rust-lang.org/
[tau]: https://v2.tauri.app/start/prerequisites/#system-dependencies
[zul]: https://radicle.zulipchat.com/#narrow/stream/444463-desktop
