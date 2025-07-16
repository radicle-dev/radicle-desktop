### Release Checklist

**Note:** We release every second Thursday, before the end of the cycle.

- Bump the minor version in `crates/radicle-tauri/tauri.conf.json`.
- Update `CHANGELOG.md` — only include changes relevant to users.
- Create a version bump patch, push to CI, and request a review.
  - The commit message should start with `Release` followed by `v<version_number>`.
- Wait for CI to pass and get peer approval.
- Build the macOS app locally: `npm run tauri build`.
  - Make sure to clean any transient dependencies with `cargo clean && rm -rf node_modules` before building.
- Upload the macOS build to [MinIO][0] in the same folder as the latest Linux build.
- Update Arch Linux package info
  - Update `pkgver` in `arch/radicle-desktop/PKGBUILD` to match the release version
  - Update `_commit` in `arch/radicle-desktop/PKGBUILD` to the release commit
    created above.
  - Regenerate `.SRCINFO` with

    ```bash
    cd arch && ./generate-srcinfo.sh
    ```

  - Create a patch with the changes and wait for CI to pass

- Publish the Arch package by pushing changes to the [Arch User Repository][4]:

  ```bash
  cd arch && ./publish.sh "Release vX.Y.Z"
  ```

  You’ll get a chance to review the changes. You need to be a maintainer of the
  AUR package to push.
- Verify the Arch package version on the [AUR](https://aur.archlinux.org/packages/radicle-desktop).
- Announce the release on Zulip, following the [previous announcement format][3].
- Resolve the previous release topic on Zulip.

[0]: https://minio.radworks.garden/browser/radworks-releases/radicle-desktop%2F
[3]: https://radicle.zulipchat.com/#narrow/channel/409174-announcements/topic/radicle-desktop.20v0.2E2.2E0.20.28early.20preview.29/with/514356912
[4]: https://aur.archlinux.org/packages/radicle-desktop
