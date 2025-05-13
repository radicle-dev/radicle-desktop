#### 1. Find a repo on the Radicle network

You can search for Radicle repos by name or description at [search.radicle.xyz](https://search.radicle.xyz).

To clone a repo, you’ll need its Repository Identifier (RID) — a unique string that begins with `rad:`.

#### 2. Start your node

If you node is Offline, you should start it by running:

```sh
rad node start
```

#### 3. Clone the repo

To clone a repo, use the `rad clone` command followed by the RID of the repo you want to clone.

```sh
rad clone <RID>
```
