<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";

  import initialize from "@app/views/home/initialize.md?raw";
  import { invoke } from "@app/lib/invoke";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import Tab from "@app/components/Tab.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import Textarea from "@app/components/Textarea.svelte";

  const { reload }: { reload: () => Promise<void> } = $props();

  const errors = $state<{
    repoName: ErrorWrapper[];
    repoDescription: ErrorWrapper[];
  }>({
    repoName: [],
    repoDescription: [],
  });
  let tab = $state<"create" | "init">("init");
  let repoName = $state<string>("");
  let repoDescription = $state<string>("");

  function validateInput(field: "name" | "description") {
    if (field === "name" && !validRepoName) {
      errors.repoName.push({ code: "ProjectError.InvalidName" });
    }
    if (field === "description" && !validRepoDescription) {
      errors.repoDescription.push({ code: "ProjectError.InvalidDescription" });
    }
  }

  const validRepoName = $derived(/^[a-zA-Z0-9._-]+$/.test(repoName));
  const validRepoDescription = $derived(repoDescription.length <= 255);

  async function createRepo() {
    try {
      await invoke("create_repo", {
        name: repoName,
        description: repoDescription,
      });
      void reload();
    } catch (err) {
      const e = err as ErrorWrapper;
      if (e.code === "ProjectError.InvalidName") {
        errors.repoName.push(e);
      } else if (e.code === "ProjectError.InvalidDescription") {
        errors.repoDescription.push(e);
      }
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
  .tab {
    height: 24px;
    color: var(--color-foreground-contrast);
  }
  .hint {
    padding: 0.25rem 0 0 0.25rem;
    gap: 0.25rem;
  }
  .create-repo-container {
    display: flex;
    gap: 2rem;
  }
</style>

{#snippet tabSnippet(name: typeof tab, content: string)}
  <Tab
    active={tab === name}
    onclick={() => {
      tab = name;
    }}>
    <span class="tab">{content}</span>
  </Tab>
{/snippet}

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
  {@render tabSnippet("create", "Create a new repo")}
  {@render tabSnippet("init", "Initialize existing repo")}
</Border>

<Border
  variant="ghost"
  flatTop
  stylePadding="1rem"
  styleDisplay="block"
  styleFlexDirection="column"
  styleAlignItems="flex-start">
  {#if tab === "create"}
    <div class="txt-small create-repo-container">
      <div class="container" style="width: 50%;">
        <div>Create a new repo initialized with Radicle.</div>

        <div class="form">
          <div style:text-align="left">
            <div class="label txt-tiny">Repository name (required)</div>
            <TextInput
              bind:value={repoName}
              oninput={() => {
                errors.repoName = [];
                validateInput("name");
              }}
              placeholder="Name of your repo" />
            {#if errors.repoName.length > 0}
              {#each errors.repoName as error}
                {#if error.code === "ProjectError.InvalidName" && repoName.length > 0}
                  <div
                    style="color: var(--color-foreground-red);"
                    class="hint txt-small global-flex">
                    <Icon name="warning" />
                    <span>
                      Only alphanumeric characters, '-', '_' and '.' are
                      allowed.
                    </span>
                  </div>
                {/if}
              {/each}
            {/if}
          </div>

          <div style:text-align="left">
            <div class="label txt-tiny">Description</div>
            <Textarea
              borderVariant="ghost"
              placeholder="Add description"
              oninput={() => {
                errors.repoDescription = [];
                validateInput("description");
              }}
              submit={async () => {
                await invoke("create_repo", {
                  name: repoName,
                  description: repoDescription,
                  defaultBranch: "master",
                });
                void reload();
              }}
              bind:value={repoDescription} />
            {#if errors.repoDescription.length > 0}
              {#each errors.repoDescription as error}
                <div
                  style="color: var(--color-foreground-red);"
                  class="hint txt-small global-flex">
                  <Icon name="warning" />
                  {#if error.code === "ProjectError.InvalidDescription"}
                    <span>Description cannot exceed 255 characters.</span>
                  {:else}
                    <span>{error.message}</span>
                  {/if}
                </div>
              {/each}
            {/if}
          </div>
          <div style:width="max-content">
            <Button
              disabled={!(validRepoDescription && validRepoName)}
              variant="secondary"
              onclick={createRepo}>
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
        styleGap="0.5rem"
        variant="float"
        styleBackgroundColor="var(--color-background-float)">
        <div>👾</div>
        <div class="txt-bold txt-regular">Did you know?</div>
        <div>
          This repository will be stored in Radicle's local storage as a bare
          Git repository, so you don’t need to choose a folder or manually
          create one. Later, you can create a checkout to work with the
          repository as needed.
          <p>
            Want to learn more about how Radicle storage works compared to a
            regular Git working copy?
          </p>
          <!-- For handling whitespace -->
          <!-- prettier-ignore -->
          <span>Check out the <a target="_blank" class="txt-missing global-link" href="https://radicle.xyz/guides/protocol#local-first-storage">Protocol Guide</a>.</span>
        </div>
      </Border>
    </div>
  {:else}
    <div class="container txt-small">
      <Markdown rid="" content={initialize} />
    </div>
  {/if}
</Border>
