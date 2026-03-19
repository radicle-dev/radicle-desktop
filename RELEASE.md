# Release process

- In your working shell set the following variables

  ```bash
  VERSION="X.Y.Z"
  RADICLE_DESKTOP_DIR="$(pwd)"                   # absolute path to this repo
  SIGNING_KEY="$HOME/work/apt-signing/rudolfs"   # adjust to your signing key path
  ```

- Create a new release branch

  ```bash
  rad sync && git fetch
  git checkout -b release/v${VERSION}
  ```

- Update the version in `crates/radicle-tauri/tauri.conf.json`.
- Update `CHANGELOG.md` — only include changes relevant to users.
- Commit the changes `git commit -a -m "Release v${VERSION}"`. This commit is the
  _Release Commit_.
- In your working shell, set the release commit SHA

  ```bash
  RELEASE_SHA="$(git rev-parse HEAD)"
  ```

- Update Arch Linux package info
  - Update `pkgver` in `arch/radicle-desktop/PKGBUILD` to match the release version
  - Update `_commit` in `arch/radicle-desktop/PKGBUILD` to the release commit
    created above.
  - Regenerate `.SRCINFO` with

    ```bash
    cd arch && ./generate-srcinfo.sh
    ```

  - Commit the changes (from the repo root):

    ```bash
    git add arch/radicle-desktop/.SRCINFO
    git add arch/radicle-desktop/PKGBUILD
    git commit -m "Update arch package to v${VERSION}"
    ```

- Create a _Release Patch_ with `git push rad HEAD:refs/patches`
- Wait for CI of the Release Patch to pass
- Wait for approval of the Release Patch and merge it into `main`

- Build all release artifacts

  ```bash
  scripts/release
  ```

  This builds the macOS DMG, Linux amd64 deb and AppImage.
  Artifacts are written to `releases/v${VERSION}/`.

  To build a single artifact, pass the corresponding flag:

  ```bash
  scripts/release --only-dmg        # macOS aarch64 DMG
  scripts/release --only-deb        # Linux amd64 deb
  scripts/release --only-appimage   # Linux amd64 AppImage
  ```

  The macOS DMG is built natively. The Linux builds run in an ARM64
  Podman container (`Dockerfile.release`) that cross-compiles Rust to
  x86_64. The AppImage additionally requires x86_64 emulation via
  binfmt_misc + QEMU to run linuxdeploy inside the ARM64 container — see
  `scripts/appimage-build` for details.

  The Linux builds cache Rust artifacts in named Podman volumes across
  runs. To start fully from scratch:

  ```bash
  scripts/release --clean && scripts/release
  ```

  `--clean` wipes `target-release/`, `node_modules/`, all Linux build
  volumes, and the container image. To only rebuild the container image
  while keeping the cached volumes (e.g. after updating
  `Dockerfile.release`):

  ```bash
  scripts/release --rebuild-image
  ```

  After the build, install the macOS DMG and start the app to verify that it works.

- Sign the Debian package from `radicle-apt-repo`

  ```bash
  cd radicle-apt-repo
  rad sync && git fetch

  podman build -t apt-import scripts
  podman run --rm \
    -v "$SIGNING_KEY:/src/keys/signing-key" \
    -v "$(pwd):/src/apt" \
    -v "$RADICLE_DESKTOP_DIR/releases/v${VERSION}/radicle-desktop_${VERSION}_amd64.deb:/src/tmp.deb" \
    apt-import

  git checkout -b release-v${VERSION}
  git add -A
  git commit -m "Update radicle-desktop to ${VERSION}"
  git push rad HEAD:refs/patches

  # Once you get peer approval, merge the patch
  git checkout main
  git merge --ff-only release-v${VERSION}
  git push
  ```

- Publish the Debian package to the APT repository from `radicle-apt-repo`

  ```bash
  scp -r -i "$(rad path)/keys/radicle" \
    dists "release@files.radicle.xyz:/mnt/radicle/files/apt/"
  scp -r -i "$(rad path)/keys/radicle" \
    pool "release@files.radicle.xyz:/mnt/radicle/files/apt/"
  ```

- Publish release files from `radicle-desktop`

  ```bash
  cd radicle-desktop

  scp -i "$(rad path)/keys/radicle" \
    releases/v${VERSION}/* "release@files.radicle.xyz:/mnt/radicle/files/releases/radicle-desktop/latest/"
  ```

- Publish the Arch package by pushing changes to the [Arch User Repository][1]
  (You need to be a maintainer of the AUR package to push)

  ```bash
  cd arch && ./publish.sh "Release v${VERSION}"
  ```

- Verify the Arch package version on the [AUR](https://aur.archlinux.org/packages/radicle-desktop)
- Resolve the previous release topic on Zulip
- Announce the release on Zulip, following the [previous announcement format][2]



[1]: https://aur.archlinux.org/packages/radicle-desktop
[2]: https://radicle.zulipchat.com/#narrow/channel/409174-announcements/topic/radicle-desktop.20v0.2E2.2E0.20.28early.20preview.29/with/514356912
