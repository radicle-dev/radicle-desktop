<script lang="ts" module>
  export const addRepoPopoverToggleId = "add-repo-popover-toggle";
</script>

<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { z } from "zod";

  import useLocalStorage from "@app/lib/useLocalStorage.svelte";
  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import { closeFocused } from "./Popover.svelte";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import { parseRepositoryId, twemoji } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import Button from "./Button.svelte";
  import Command from "./Command.svelte";
  import ExternalLink from "./ExternalLink.svelte";
  import Icon from "./Icon.svelte";
  import Popover from "./Popover.svelte";
  import Tab from "./Tab.svelte";
  import TextInput from "./TextInput.svelte";

  interface Props {
    onOpen: () => void;
    reload: () => Promise<void>;
    repos: RepoInfo[];
    seededNotReplicated: string[];
  }

  const { onOpen, reload, repos, seededNotReplicated }: Props = $props();

  let popoverExpanded: boolean = $state(false);
  let rid = $state("");
  let validationMessage: string | undefined = $state(undefined);

  // Clear validation message when changing the input.
  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    rid;

    validationMessage = undefined;
  });

  // Clear input when the popover is closed.
  $effect(() => {
    if (!popoverExpanded) {
      rid = "";
    }
  });

  const tab = useLocalStorage(
    "addRepoPopoverSelectedTab",
    z.union([z.literal("seed"), z.literal("publish")]),
    "seed",
    !window.localStorage,
  );

  async function submit() {
    const trimmedRid = rid.trim();

    if (trimmedRid === "") {
      return;
    }

    validationMessage = validate(trimmedRid);

    if (validationMessage === undefined) {
      await seed(trimmedRid);
      await reload();
      rid = "";
      closeFocused();
    }
  }

  async function seed(rid: string) {
    try {
      await invoke<null>("seed", {
        rid: rid,
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Seeding failed", error);
    }
  }

  function validate(rid: string): string | undefined {
    const parsedRid = parseRepositoryId(rid);
    if (parsedRid === undefined) {
      return "RID is not valid";
    }

    if (seededNotReplicated.includes(rid)) {
      return "This repo is already queued for fetching";
    }
    if (repos.map(r => r.rid).includes(rid)) {
      return "This repo is already seeded";
    }
  }
</script>

<style>
  li {
    padding: 0;
  }
</style>

<Popover
  popoverPositionRight="0"
  popoverPositionTop="3rem"
  bind:expanded={popoverExpanded}>
  {#snippet toggle(onclick)}
    <Button
      id={addRepoPopoverToggleId}
      styleHeight="2.5rem"
      variant="secondary"
      onclick={() => {
        onOpen();
        onclick();
      }}
      active={popoverExpanded}>
      <Icon name="add" />Add repo
    </Button>
  {/snippet}

  {#snippet popover()}
    <Border
      stylePosition="relative"
      variant="ghost"
      flatBottom
      styleDisplay="flex"
      styleWidth="100%"
      styleGap="1rem"
      styleMinWidth="27rem"
      stylePadding="0 1rem">
      <Tab
        active={tab.value === "seed"}
        onclick={() => {
          tab.value = "seed";
        }}>
        Seed a repo
      </Tab>
      <Tab
        active={tab.value === "publish"}
        onclick={() => {
          tab.value = "publish";
        }}>
        Publish existing repo
      </Tab>
    </Border>

    <div style:margin-top="-2px">
      <Border
        variant="ghost"
        flatTop
        stylePadding="1rem"
        styleDisplay="block"
        styleFlexDirection="column"
        styleAlignItems="flex-start">
        <div class="txt-small" style:line-height="1.625rem">
          {#if tab.value === "seed"}
            <!-- prettier-ignore -->
            <div style:margin-bottom="1rem">
              You can search for Radicle repos by name or description at
              <ExternalLink href="https://search.radicle.xyz">
                search.radicle.xyz
              </ExternalLink>.
            </div>
            <div style:width="100%">
              <div class="txt-semibold" style:margin-bottom="0.5rem"></div>
              <div
                class="global-flex"
                style:flex-direction="column"
                style:align-items="flex-start"
                style:gap="1rem">
                <div style:width="100%">
                  <div class="global-flex" style:width="100%">
                    <TextInput
                      autofocus
                      valid={validationMessage === undefined}
                      bind:value={rid}
                      onSubmit={submit}
                      placeholder="RID, e.g. rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5" />
                    <Button
                      variant="ghost"
                      styleHeight="2.5rem"
                      onclick={submit}
                      disabled={rid.trim() === ""}>
                      Seed
                    </Button>
                  </div>
                  {#if validationMessage}
                    <div
                      class="txt-small global-flex"
                      style:color="var(--color-foreground-red)"
                      style:padding="0.25rem 0 0 0.25rem"
                      style:gap="0.25rem">
                      <Icon name="warning" />
                      {validationMessage}
                    </div>
                  {/if}
                </div>
              </div>
            </div>
            <div
              class="global-flex txt-missing"
              style:align-items="flex-start"
              style:margin-top="2rem">
              <span style:margin-top="0.25rem">
                <Icon name="info" />
              </span>
              By seeding a repository, your node fetches it from the network, allowing
              you to interact with it locally while also making it available to others.
            </div>
            {#if !$nodeRunning}
              <div
                class="global-flex txt-missing"
                style:align-items="flex-start"
                style:margin-top="1rem">
                <span style:margin-top="0.25rem">
                  <Icon name="bulb" />
                </span>

                <div>
                  Your node is
                  <span class="txt-semibold">
                    <span
                      style:display="inline-block"
                      style:vertical-align="text-top">
                      <Icon name="offline" />
                    </span>
                    Offline.
                  </span>
                  You can still add repos, but they will only be fetched once your
                  node is back online.
                </div>
              </div>
            {/if}
          {:else if tab.value === "publish"}
            <p style="margin: 0 0 1rem 0">
              Navigate to an existing Git repo in your terminal
              <code
                style:white-space="nowrap"
                style:padding="0.125rem 0.25rem"
                style:background-color="var(--color-fill-ghost)">
                cd path/to/your/repo
              </code>
              and run the following command:
            </p>

            <Command styleWidth="fit-content" command="rad init" />

            <p style="margin: 1rem 0 0 0">
              Follow the setup prompts to initialize the repo and publish it on
              the Radicle network:
            </p>

            <ul style:padding="0 1rem">
              <li>
                <strong>Repository Name:</strong>
                The name of your repo.
              </li>
              <li>
                <strong>Description:</strong>
                A brief summary of what your repo does.
              </li>
              <!-- prettier-ignore -->
              <li>
                <strong>Default Branch:</strong>
                Typically
                <strong>main</strong>
                or
                <strong>master</strong>.
              </li>
              <li>
                <strong>Visibility:</strong>
                Choose
                <strong>public</strong>
                to share with others or
                <strong>private</strong>
                to not publish it to the network yet.
              </li>
            </ul>
            <p use:twemoji style:margin="2rem 0 0 0">
              That's it! Your repo is now on the Radicle network. 🚀
            </p>
          {/if}
        </div>
      </Border>
    </div>
  {/snippet}
</Popover>
