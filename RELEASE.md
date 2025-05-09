### Release Checklist

**Note:** We release every second Thursday, before the end of the cycle.

- Bump the minor version in `crates/radicle-tauri/tauri.conf.json`.
- Update `CHANGELOG.md` — only include changes relevant to users.
- Create a version bump patch, push to CI, and request a review.
- Wait for CI to pass and get peer approval.
- Build the macOS app locally: `npm run tauri build`.
- Upload the macOS build to [MinIO][0] in the same folder as the latest Linux build.
- Create a patch on [radworks-product][1] to [update the download links][2].
- Once merged, publish the website: `git push github main`.
- Announce the release on Zulip, following the [previous announcement format][3].
- Resolve the previous release topic on Zulip.

[0]: https://minio.radworks.garden/browser/radworks-releases/radicle-desktop%2F
[1]: https://app.radicle.xyz/nodes/seed.radicle.garden/rad:z4Rxw3J2gX8SJgUYJ3h1KvQCAYoKS
[2]: https://app.radicle.xyz/nodes/seed.radicle.garden/rad:z4Rxw3J2gX8SJgUYJ3h1KvQCAYoKS/commits/17c25b85b123d9766c774cc414e3ffdfc7b69771
[3]: https://radicle.zulipchat.com/#narrow/channel/409174-announcements/topic/radicle-desktop.20v0.2E2.2E0.20.28early.20preview.29/with/514356912
