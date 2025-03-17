### 1. Install Radicle CLI

Run the following command in your terminal to install Radicle:

```sh
$ curl -sSf https://radicle.xyz/install | sh
```

### 2. Verify The Installation

Ensure Radicle CLI is installed correctly by checking its version:

```sh
$ rad --version
rad 1.1.0 (70f0cc35)
```

### 3. Initialize Your Repository

Navigate to your existing Git repository and initialize it with Radicle by following the setup prompts:

- **Repository Name:** Enter a name for your repository.
- **Description:** Provide a brief summary of what your repository does.
- **Default Branch:** Typically **main** or **master**.
- **Visibility:** Choose **public** to share with others or **private** to restrict access.

```sh
$ cd path/to/your/repository
$ rad init
```

### 4. Retrieve Your Repository Identifier (RID)

After initialization, your repository will show up here.  
You can retrieve its unique RID at any time:

```sh
$ rad .
rad:z4D5UCArafTzTQpDZNQRuqswh3ury
```

That's it! Your repository is now a Radicle repo. 🚀
