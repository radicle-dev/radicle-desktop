# Radicle Desktop

This desktop application lets you interact with [Radicle][rad], a peer-to-peer code collaboration and publishing stack, directly from your web browser.

## Local-first

**Prerequisites:**

- Recent versions of [Node.js][nod] (20.9.0 or higher) and [npm][npm] installed
- Requires the [Rust][rus] toolchain (1.77 or higher).

If you are running any Linux distro make sure to also have at least the following dependencies installed:

- libgtk-3-dev
- libjavascriptcoregtk-4.1-dev
- libsoup-3.0-dev
- libwebkit2gtk-4.1-dev

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
[zul]: https://radicle.zulipchat.com/#narrow/stream/444463-desktop
