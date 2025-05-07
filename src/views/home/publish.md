#### 1. Publish existing repo on Radicle

Navigate to your existing Git repository and publish it to Radicle by following the setup prompts:

- **Repository Name:** Enter a name for your repository.
- **Description:** Provide a brief summary of what your repository does.
- **Default Branch:** Typically **main** or **master**.
- **Visibility:** Choose **public** to share with others or **private** to not publish it to the network yet.

```sh
cd path/to/your/repository
rad init
```

#### 2. Retrieve Your Repository Identifier (RID)

After running `rad init`, your repository will eventually show up in the repo listing.
You can retrieve its unique RID by running the following command in your repo:

```sh
rad .
```

That's it! Your repository is now a Radicle repo. 🚀
