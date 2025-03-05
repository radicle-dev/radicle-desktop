<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/Error";
  import type { Action } from "@app/components/ActionItem.svelte";

  import ActionItem from "@app/components/ActionItem.svelte";
  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Tab from "@app/components/Tab.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import Textarea from "@app/components/Textarea.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import Icon from "@app/components/Icon.svelte";

  const GETTING_STARTED_REPO_RID = "rad:z3Wu3dtimpnoMAveMDFGty9UCdv9b";
  const SEED_RADICLE_GARDEN_DID =
    "did:key:z6MkrLMMsiPWUcNPHcRajuMi9mDfYckSoJyPwwnknocNYPm7";
  const SEED_RADICLE_GARDEN_ADDRESS = "seed.radicle.garden:8776";

  const { reload }: { reload: () => Promise<void> } = $props();

  let passphrase = $state("");
  let error = $state<ErrorWrapper>();
  let tab = $state<"fetch" | "create" | "init">("fetch");
  let repoName = $state<string>("");
  let repoDescription = $state<string>("");
  let existingRunningNode = $state(false);
  const fetchStatus = $state<{
    start?: Action;
    connect?: Action;
    fetch?: Action;
    stop?: Action;
  }>({
    start: undefined,
    connect: undefined,
    fetch: undefined,
    stop: undefined,
  });

  const disableFetchButton = $derived(
    Boolean(fetchStatus.start) || (!$nodeRunning && passphrase.length === 0),
  );

  async function fetchDemoRepo() {
    try {
      if ($nodeRunning) {
        existingRunningNode = true;
      } else {
        updateStatus("start", "running");
        await invoke("start_node", { passphrase });
      }
      updateStatus("start", "done");
      updateStatus("connect", "running");
      await invoke("connect_node", {
        from: SEED_RADICLE_GARDEN_DID,
        address: SEED_RADICLE_GARDEN_ADDRESS,
      });
      updateStatus("connect", "done");
      updateStatus("fetch", "running");
      await invoke("fetch_repo", {
        rid: GETTING_STARTED_REPO_RID,
        from: SEED_RADICLE_GARDEN_DID,
        timeoutSeconds: 10,
      });
      updateStatus("fetch", "done");
      // We only stop radicle-node processes we spawned ourselves.
      if (!existingRunningNode) {
        updateStatus("stop", "running");
        await invoke("stop_node");
      }
      updateStatus("stop", "done");
      await reload();
    } catch (err) {
      if (fetchStatus.start === "running") fetchStatus.start = "error";
      if (fetchStatus.connect === "running") fetchStatus.connect = "error";
      if (fetchStatus.fetch === "running") fetchStatus.fetch = "error";
      if (fetchStatus.stop === "running") fetchStatus.stop = "error";
      error = err as ErrorWrapper;
      console.error(err);
    }
  }

  function updateStatus(key: keyof typeof fetchStatus, status: Action) {
    fetchStatus[key] = status;
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

  .grid {
    display: grid;
    grid-auto-flow: column;
    grid-template-columns: "1fr 1fr";
    column-gap: 3rem;
    grid-template-rows: repeat(4, 2rem);
  }
  .tab {
    height: 24px;
    color: var(--color-foreground-contrast);
  }
  .error {
    color: var(--color-foreground-red);
    gap: 0.25rem;
  }
  .create-repo-container {
    display: flex;
    gap: 2rem;
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
<Border
  stylePosition="relative"
  variant="ghost"
  flatBottom
  styleDisplay="flex"
  styleWidth="100%"
  styleGap="1rem"
  stylePadding="0 1rem">
  <Tab
    active={tab === "fetch"}
    onclick={() => {
      tab = "fetch";
    }}>
    <span class="tab">Fetch a demo repo</span>
  </Tab>
  <Tab
    active={tab === "create"}
    onclick={() => {
      tab = "create";
    }}>
    <span class="tab">Create a new repo</span>
  </Tab>
  <Tab
    active={tab === "init"}
    onclick={() => {
      tab = "init";
    }}>
    <span class="tab">Initialize existing repo</span>
  </Tab>
</Border>

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
        variant="ghost">
        <div class="grid">
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
          {#each Object.values(fetchStatus) as key}
            {#if key === "error"}
              <div class="global-flex error">
                {#if error}
                  <Icon name="info" />{error.message}
                {/if}
              </div>
            {:else}
              <div class="global-flex error"></div>
            {/if}
          {/each}
        </div>
      </Border>

      <div style:width="23rem">
        {#if !$nodeRunning}
          <div>
            <div class="label txt-tiny">Passphrase</div>
            <TextInput
              placeholder="Enter passphrase to be able to start your node."
              type="password"
              bind:value={passphrase} />
          </div>
        {/if}
      </div>
      <div style:width="max-content">
        <Button
          variant="secondary"
          disabled={disableFetchButton}
          onclick={fetchDemoRepo}>
          <span style:text-align="center">Fetch the demo repo</span>
        </Button>
      </div>
    </div>
  {:else if tab === "create"}
    <div class="txt-small create-repo-container">
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
