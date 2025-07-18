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

- Sign the Debian package

  TODO: automate the signing process with a docker image so it can be done from
  the host machine.

  Currently this happens via an Ubuntu vm manually:

  ```bash
  # Set the VERSION and SHORT_RELEASE_SHA variables to the same values here
  # in the VM as on the host system.
  RELEASE_SHA=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
  SHORT_RELEASE_SHA=XXXXXXXX

  cd radicle-apt-repo
  rad sync && git fetch

  curl -fLO \
    "https://minio-api.radworks.garden/radworks-releases/radicle-desktop/pre-release/${VERSION}_${SHORT_RELEASE_SHA}/radicle-desktop_${VERSION}_amd64.deb"

  RADICLE_APT_SIGNING_KEY=~/work/apt-signing/rudolfs ./import-deb ./radicle-desktop_${VERSION}_amd64.deb
  rm -rf ./radicle-desktop_0.7.1_amd64.deb

  git checkout -b release-v${VERSION}
  git commit -a -m "Update radicle-desktop to ${VERSION}"

  # Make sure ssh-agent is started and `rad auth` is working, then publish the patch.

  git push rad HEAD:refs/patches

  # The next step is to merge this patch into main, which effectively publishes
  # the Deb packages. Continue this in the publish section below.
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

- Publish release files

  ```bash
  scp -i "$(rad path)/keys/radicle" \
    release-artifacts/* "release@files.radicle.xyz:/mnt/radicle/files/releases/radicle-desktop/latest/"
  ```

- Publish the Debian package to the APT repository
    - Switch to the `radicle-apt-repo` repository
    ```bash
    cd radicle-apt-repo
    ```

    - Merge the patch that was created on radicle-apt-repo in the VM into main from your host machine as a delegate
    ```bash
    rad sync && git fetch
    rad patch checkout XXXXXXX
    git checkout main
    git merge --ff-only patch/XXXXXXX
    git push
    ```

    - Upload the signed deb packages to files.radicle.xyz
    ```bash
    scp -r -i "$(rad path)/keys/radicle" \
      dists "release@files.radicle.xyz:/mnt/radicle/files/apt/"
    scp -r -i "$(rad path)/keys/radicle" \
      pool "release@files.radicle.xyz:/mnt/radicle/files/apt/"
    ```

    TODO: can we make scp skip the files that are already uploaded?

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
