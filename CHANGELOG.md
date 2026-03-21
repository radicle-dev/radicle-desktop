### v0.9.0

- Redesigned the entire app with a new UI kit — typography, colors, icons,
  avatars, and border radii have all been updated for a more polished,
  consistent appearance matching the rest of the ecosystem
- Replaced blockies-based avatars with a new DID-based avatar system
- Moved Settings and Create Issue into fullscreen modals
- Default theme now follows the system theme
- Removed window decorations on macOS for a cleaner, more native feel
- Updated app icon
- Rewrote app navigation and layouts
- Added a sidebar that gives you access to all your repos
- App now opens directly to the notification inbox on startup
- Getting started guide shown to new users on first launch
- Added a setting to disable the notification badge
- Added a code font setting
- Fixed reactions and edits incorrectly targeting the root comment instead of the intended reply
- Restricted issue/patch state, labels, and assignees to delegates only

**Shout-out to our contributors 🙏✨**

- [brandonhaslegs](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MkpwnLQxFBQXyMK3Es91s8A7Ew7G11BFFyng1dqZR8QhG3)
- [julien](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MkgvaRdahQZfnf7ccMHByeHtTYrMrM3MuSGYawF6ZL6DNj)


### v0.8.0

- CI job run statuses are now shown for the default branch and in patch revision commit lists
- Fixed an issue where commit listings reversed when switching between patches
- Fixed a bug that prevented replying to review comments
- Fixed app version check feature
- Fixed incorrect IDs (e.g., patch revision, comments) being copied to clipboard
- Improved accessibility and keyboard navigation in repository filters on the home view

**Shout-out to our contributors 🙏✨**

- [geigerzaehler](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6Mki9XNNHeVRnYS88U59iCBzKUp2xWM3f4zvA3cXuKJFvWF)


### v0.7.1

- Fixed a bug in patch and notification lists caused by a library incompatibility


### v0.7.0

- Source code can now be browsed directly within the app
- Draft reviews and their comments can now be deleted
- Issue state changes no longer affect the issue list in the sidebar
- Repo cards now support keyboard navigation for easier selection

**Shout-out to our contributors 🙏✨**

- [geigerzaehler](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6Mki9XNNHeVRnYS88U59iCBzKUp2xWM3f4zvA3cXuKJFvWF)


### v0.6.1

- Fixed a bug where the app wouldn't onboard new users or locked identities
- Fixed regression where the current version of the app wasn't including the commit it was built from


### v0.6.0

- UI polish: scrollbars now auto-hide across the app
- Fixed button overflow in issue and patch sidebars
- Draft reviews can now be composed before publishing
- Added opt-in automatic app update notifications
- Reworked patch view for clearer, more efficient code reviews

**Shout-out to our contributors 🙏✨**

- [geigerzaehler](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6Mki9XNNHeVRnYS88U59iCBzKUp2xWM3f4zvA3cXuKJFvWF)


### v0.5.0

- The app is now officially available via Arch packaging
- Fuzzy search now supports repo IDs, descriptions, and issue/patch IDs
- Updated Heartwood crates to support Radicle 1.2.0
- Bundled Radicle tooling and Node in Nix for smoother setup
- Added a startup spinner if launch takes more than 1 second
- Fullscreen error shown if COB schemas need upgrading
- UI improvements: themed scrollbars that auto-hide and match the color scheme
- Repos can now be seeded by RID directly from the app
- Added warning and one-click recovery when COB cache is out of date, fixing mismatched patch/issue counters
- Fixed regression where single patch view didn’t load additional items on scroll

**Shout-out to our contributors 🙏✨**

- [geigerzaehler](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6Mki9XNNHeVRnYS88U59iCBzKUp2xWM3f4zvA3cXuKJFvWF)
- [Kuehle](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MkgFWv1K9Lqvi5VHm4mtfDo5MvanWVNYU1gkvT9Rwr1Lu7)
- [lorenz](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MkkPvBfjP4bQmco5Dm7UGsX2ruDBieEHi8n9DVJWX5sTEz)
- [tippfehlr](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MksSTAFPKk7eQju9aGzK64qpfiSnKQ1tJWv1rdzRd5bMHU)


### v0.4.1

- Fixed a bug in the release pipeline to update the latest artifacts
- Added WSL2 installation instructions to https://radworks.garden


### v0.4.0

- Notification inbox is now accessible from any view without navigating away
- App icon now shows a badge with the unread notification count and polls for updates
- Added global breadcrumbs for easier navigation
- Added quick copy actions for IDs and links to https://app.radicle.xyz
- Syntax highlighting added for diffs in markdown code blocks (` ```diff `)
- Improved dropdown UI — triggers are now visually distinct when active
- Fixed AppImage bug affecting Arch, Fedora, and Red Hat users
- App is now officially available for Nix users via https://radworks.garden

**Shout-out to our contributors 🙏✨**

- [fintohaps](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MkireRatUThvd3qzfKht1S44wpm4FEWSSa4PRMTSQZ3voM)
- [geigerzaehler](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6Mki9XNNHeVRnYS88U59iCBzKUp2xWM3f4zvA3cXuKJFvWF)
- [lorenz](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MkkPvBfjP4bQmco5Dm7UGsX2ruDBieEHi8n9DVJWX5sTEz)
- [tshepang](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MkfPSKW7AgQqXSi8fgEJMduHpm9ABmsPYwPhMeF7PssonK)
- [youthlic](https://app.radicle.xyz/nodes/iris.radicle.xyz/users/did:key:z6MktsSuE4bVYbuTtEPjmhrQWA7dMri7GUg9Qp9o8tRCsmhu)


### v0.3.0

- New onboarding guide to help users get started faster
- Repo homepage now shows README and project info for quick overview
- UI improvements, including clearer buttons and a new "New Patch" button
- Settings now show version and commit hash for easier support
- Fixed notification count and comment placement bugs


### v0.2.0

- Added font size controls for enhanced readability and accessibility
- Introduced contextual explainers in key areas to guide users and improve overall user experience
- Enhanced contrast in both dark and light modes to improve legibility and visual clarity


### v0.1.0

- First public release
