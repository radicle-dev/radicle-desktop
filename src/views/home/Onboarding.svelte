<script lang="ts">
  import ActionItem from "@app/components/ActionItem.svelte";
  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Tab from "@app/components/Tab.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import Textarea from "@app/components/Textarea.svelte";
  import {
    GETTING_STARTED_REPO_RID,
    SEED_RADICLE_GARDEN_ADDRESS,
    SEED_RADICLE_GARDEN_DID,
  } from "@app/lib/init.svelte";
  import { invoke } from "@app/lib/invoke";

  type Actions = "done" | "running";

  const { reload }: { reload: () => Promise<void> } = $props();

  let tab = $state<"fetch" | "create" | "init">("init");
  let repoName = $state<string>("");
  let repoDescription = $state<string>("");
  const fetchStatus = $state<{
    start?: Actions;
    connect?: Actions;
    fetch?: Actions;
    stop?: Actions;
  }>({
    start: undefined,
    connect: undefined,
    fetch: undefined,
    stop: undefined,
  });

  async function fetchDemoRepo() {
    try {
      fetchStatus.start = "running";
      await invoke("start_node");
      fetchStatus.start = "done";
      fetchStatus.connect = "running";
      await invoke("connect_node", {
        from: SEED_RADICLE_GARDEN_DID,
        address: SEED_RADICLE_GARDEN_ADDRESS,
      });
      fetchStatus.connect = "done";
      fetchStatus.fetch = "running";
      await invoke("fetch_repo", {
        rid: GETTING_STARTED_REPO_RID,
        from: SEED_RADICLE_GARDEN_DID,
        timeoutSeconds: 10,
      });
      fetchStatus.fetch = "done";
      fetchStatus.stop = "running";
      await invoke("stop_node");
      fetchStatus.stop = "done";
      await reload();
    } catch (err) {
      console.error(err);
    }
  }
</script>

<style>
  .container {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .label {
    margin-bottom: 0.5rem;
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  code {
    font-family: var(--font-family-monospace);
    font-size: var(--font-size-small);
    background-color: var(--color-fill-ghost);
    padding: 0.125rem 0.25rem;
  }

  pre code {
    background: none;
    padding: 0;
  }

  :not(pre) > code {
    font-size: inherit;
  }

  pre {
    font-family: var(--font-family-monospace);
    font-size: var(--font-size-regular);
    background-color: var(--color-fill-ghost);
    padding: 1rem !important;
    overflow: scroll;
    scrollbar-width: none;
    clip-path: var(--2px-corner-fill);
  }
  ul {
    margin: 0.5rem;
  }
</style>

<div class="txt-missing txt-small" style:margin-bottom="1.5rem">
  You don't have any repositories in your Radicle storage yet. To get started,
  try one of the options below.
</div>
<div class="global-flex" style:gap="0.5rem" style:margin-top="1rem">
  <Border stylePosition="relative" variant="ghost" flatBottom>
    <div
      class="global-flex"
      style:z-index="10"
      style:gap="1rem"
      style:padding="0 1rem"
      style:width="100%">
      <Tab
        active={tab === "fetch"}
        onclick={() => {
          tab = "fetch";
        }}>
        <span
          style:height="24px"
          style:color="var(--color-foreground-contrast)">
          Fetch a demo repo
        </span>
      </Tab>
      <Tab
        active={tab === "create"}
        onclick={() => {
          tab = "create";
        }}>
        <span
          style:height="24px"
          style:color="var(--color-foreground-contrast)">
          Create a new repo
        </span>
      </Tab>
      <Tab
        active={tab === "init"}
        onclick={() => {
          tab = "init";
        }}>
        <span
          style:height="24px"
          style:color="var(--color-foreground-contrast)">
          Initialize existing repo
        </span>
      </Tab>
    </div>
  </Border>
</div>

<Border
  variant="ghost"
  flatTop
  stylePadding="1rem"
  styleDisplay="block"
  styleFlexDirection="column"
  styleAlignItems="flex-start">
  {#if tab === "fetch"}
    <div class="container txt-small">
      <div>To fetch your first repo, we'll execute the following steps:</div>
      <Border
        styleBackgroundColor="var(--color-background-float)"
        stylePadding="1rem"
        variant="ghost"
        styleGap="1rem"
        styleFlexDirection="column"
        styleAlignItems="flex-start">
        <ActionItem status={fetchStatus.start} position={1}>
          <div class="txt-monospace">Start node</div>
        </ActionItem>
        <ActionItem status={fetchStatus.connect} position={2}>
          <div class="txt-monospace">Connect to seed.radicle.garden</div>
        </ActionItem>
        <ActionItem status={fetchStatus.fetch} position={3}>
          <div class="txt-monospace">
            Fetch <code>getting-started</code>
            repo
          </div>
        </ActionItem>
        <ActionItem status={fetchStatus.stop} position={4}>
          <div class="txt-monospace">Stop node</div>
        </ActionItem>
      </Border>

      <div style:width="max-content">
        <Button
          variant="secondary"
          disabled={Boolean(fetchStatus.start)}
          onclick={fetchDemoRepo}>
          <span style:text-align="center">Fetch the demo repo</span>
        </Button>
      </div>
    </div>
  {:else if tab === "create"}
    <div class="txt-small" style="display: flex; gap: 2rem;">
      <div class="container" style="width: 50%;">
        <div>Create a new repo initialized with Radicle.</div>

        <div class="form">
          <div style:text-align="left">
            <div class="label txt-tiny">Repository name</div>
            <TextInput bind:value={repoName} placeholder="Name of your repo" />
          </div>

          <div style:text-align="left">
            <div class="label txt-tiny">Description</div>
            <Textarea
              borderVariant="ghost"
              placeholder="Add description"
              submit={async () => {
                await invoke("create_repo", {
                  name: repoName,
                  description: repoDescription,
                  defaultBranch: "master",
                });
                void reload();
              }}
              bind:value={repoDescription} />
          </div>
          <div style:width="max-content">
            <Button
              variant="secondary"
              onclick={async () => {
                await invoke("create_repo", {
                  name: repoName,
                  description: repoDescription,
                  defaultBranch: "master",
                });
                void reload();
              }}>
              Create new repo
            </Button>
          </div>
        </div>
      </div>
      <Border
        styleHeight="max-content"
        styleDisplay="flex"
        styleFlexDirection="column"
        styleAlignItems="flex-start"
        stylePadding="1rem"
        styleGap="1rem"
        variant="float"
        styleBackgroundColor="var(--color-background-float)">
        <div>👾</div>
        <div class="txt-bold txt-regular">Did you know?</div>
        <div>
          This repository will be stored directly in Radicle, instead of a
          folder on your computer.
          <br />
          That means you don't need to choose a location or manually create a folder.
          <br />
          Want to learn more about how Radicle storage works compared to a regular
          Git working copy?
          <br />
          Check out the
          <a
            class="txt-missing"
            href="https://radicle.xyz/guides/protocol#local-first-storage">
            Protocol Guide
          </a>
          .
        </div>
      </Border>
    </div>
  {:else}
    <div class="container txt-small">
      <ol style:margin="0">
        <li class="txt-semibold">Install Radicle CLI</li>
        <div>Run this command in your terminal to install Radicle:</div>
        <pre><code>curl -sSf https://radicle.xyz/install | sh</code></pre>
        <li class="txt-semibold">Verify the installation:</li>
        <div>Check if the Radicle CLI is installed correctly:</div>
        <pre><code>rad --version</code></pre>
        <li class="txt-semibold">Initialize Your Repository:</li>
        <div>
          Navigate to your existing Git repository and initialize it with
          Radicle:
        </div>
        <pre><code>cd path/to/your/repository
rad init</code></pre>
        <li class="txt-semibold">Follow the Setup Prompts:</li>
        <div>You'll be prompted to provide:</div>
        <ul>
          <li>
            <strong>Repository Name</strong>
            - Give your repository a name.
          </li>
          <li>
            <strong>Description:</strong>
            - A short summary of what your repository does.
          </li>
          <li>
            <strong>Default branch:</strong>
            Usually
            <code>main</code>
            or
            <code>master</code>
            .
          </li>
          <li>
            <strong>Visibility:</strong>
            Choose
            <code>public</code>
            to share with others or
            <code>private</code>
            to keep it restricted.
          </li>
        </ul>
        <li class="txt-semibold">Get Your Repository Identifier (RID):</li>
        <div>
          After setup, your repository will be assigned a unique ID. You can
          view it anytime with:
        </div>
        <div>To view your repository's RID at any time:</div>
        <pre><code>rad .</code></pre>
        <div>
          That's it! Your repository is now part of the Radicle network. 🚀
        </div>
      </ol>
    </div>
  {/if}
</Border>
