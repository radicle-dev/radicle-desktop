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
- Publish Arch Linux package (See `arch/README.md` for more information)
  - Update `pkgver` in `arch/PKGBUILD` to match the release version
  - Update `_commit` in `arch/PKGBUILD` to the release commit created above
  - Regenerate `.SRCINFO` with

    ```bash
    cd arch && ./generate-srcinfo.sh
    ```

  - Create a patch with the changes and wait for CI to pass

- Create a patch on [radworks-product][1] to [update the download links][2].
- Once merged, publish the website: `git push github main`.
- Publish the Arch package by pushing changes to the [Arch User Repository][4]:

  ```bash
  cd arch && ./publish.sh
  ```

  You need to be a maintainer of the AUR package to push.
- Announce the release on Zulip, following the [previous announcement format][3].
- Resolve the previous release topic on Zulip.

[0]: https://minio.radworks.garden/browser/radworks-releases/radicle-desktop%2F
[1]: https://app.radicle.xyz/nodes/seed.radicle.garden/rad:z4Rxw3J2gX8SJgUYJ3h1KvQCAYoKS
[2]: https://app.radicle.xyz/nodes/seed.radicle.garden/rad:z4Rxw3J2gX8SJgUYJ3h1KvQCAYoKS/commits/17c25b85b123d9766c774cc414e3ffdfc7b69771
[3]: https://radicle.zulipchat.com/#narrow/channel/409174-announcements/topic/radicle-desktop.20v0.2E2.2E0.20.28early.20preview.29/with/514356912
[4]: https://aur.archlinux.org/packages/radicle-desktop
