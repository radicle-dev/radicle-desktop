# Release process

**Note:** We release every second Thursday, before the end of the cycle.

- In your working shell set the following variable

  ```bash
  VERSION="X.Y.Z"
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
  SHORT_RELEASE_SHA="$(git rev-parse --short=8 HEAD)"
  ```

- Create a _Release Patch_ with `git push rad HEAD:refs/patches`
- Build the macOS app locally

  ```bash
  cargo clean
  rm -rf node_modules
  npm install
  npm exec -- tauri build --bundles dmg
  ```

  This creates a file `target/release/bundle/dmg/Radicle_X.Y.Z_aarch64.dmg`.

- Install the build macOS DMG and start the app to verify that it works.
- Wait for CI of the Release Commit in the Release Patch to pass

- Collect release artifacts

  ```bash
  rm -rf release-artifacts && mkdir release-artifacts
  curl -fL \
    "https://minio-api.radworks.garden/radworks-releases/radicle-desktop/pre-release/${VERSION}_${SHORT_RELEASE_SHA}/radicle-desktop_${VERSION}_amd64.AppImage" \
    --output release-artifacts/radicle-desktop-amd64.AppImage
  cp -a "target/release/bundle/dmg/Radicle_${VERSION}_aarch64.dmg" \
    release-artifacts/radicle-desktop-aarch64.dmg
  echo -n "{\"sha\": \"${RELEASE_SHA}\", \"version\": \"${VERSION}\"}" \
    > release-artifacts/latest.json
  ```

  The content of `release-artifacts` should look like this:

  ```plain
  release-artifacts
    latest.json
    radicle-desktop-aarch64.dmg
    radicle-desktop-amd64.AppImage
  ```

- Update Arch Linux package info
  - Update `pkgver` in `arch/radicle-desktop/PKGBUILD` to match the release version
  - Update `_commit` in `arch/radicle-desktop/PKGBUILD` to the release commit
    created above.
  - Regenerate `.SRCINFO` with

    ```bash
    cd arch && ./generate-srcinfo.sh
    ```

  - Commit and push the changes to `arch/radicle-desktop/PKGBUILD` and
    `arch/radicle-desktop/.SRCINFO`:

    ```bash
    git add radicle-desktop/.SRCINFO
    git add radicle-desktop/PKGBUILD
    git commit -m "Update arch package to v${VERSION}"
    git push rad
    ```

- Wait for CI of the Release Patch to pass
- Wait for approval of the Release Patch and merge it into `main`

- Sign the Debian package from `radicle-apt-repo`

  ```bash
  cd radicle-apt-repo
  rad sync && git fetch

  curl -fLO \
    "https://minio-api.radworks.garden/radworks-releases/radicle-desktop/pre-release/${VERSION}_${SHORT_RELEASE_SHA}/radicle-desktop_${VERSION}_amd64.deb"

  podman build -t apt-import scripts
  podman run --rm -v ~/work/apt-signing/rudolfs:/src/keys/signing-key -v $(pwd):/src/apt -v ./radicle-desktop_${VERSION}_amd64.deb:/src/tmp.deb apt-import

  rm -rf ./radicle-desktop_${VERSION}_amd64.deb

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

    - Upload the signed deb packages to files.radicle.xyz
    ```bash
    scp -r -i "$(rad path)/keys/radicle" \
      dists "release@files.radicle.xyz:/mnt/radicle/files/apt/"
    scp -r -i "$(rad path)/keys/radicle" \
      pool "release@files.radicle.xyz:/mnt/radicle/files/apt/"
    ```

    TODO: can we make scp skip the files that are already uploaded?

- Publish release files from `radicle-desktop`

  ```bash
  cd radicle-desktop

  scp -i "$(rad path)/keys/radicle" \
    release-artifacts/* "release@files.radicle.xyz:/mnt/radicle/files/releases/radicle-desktop/latest/"
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
