# Radicle Desktop

This desktop application lets you interact with [Radicle][rad], a peer-to-peer code collaboration and publishing stack.

## Local-first

**Prerequisites:**

- Recent versions of [Node.js][nod] (22.11.0 or higher) and [npm][npm] installed
- Requires the [Rust][rus] toolchain (1.77 or higher).

Also make sure to have the following [Tauri system dependencies][tau] installed.

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
