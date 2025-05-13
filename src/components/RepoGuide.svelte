<script lang="ts">
  import { z } from "zod";

  import useLocalStorage from "@app/lib/useLocalStorage.svelte";

  import clone from "@app/components/RepoGuide/clone.md?raw";
  import publish from "@app/components/RepoGuide/publish.md?raw";

  import Border from "@app/components/Border.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import Tab from "@app/components/Tab.svelte";

  const tab = useLocalStorage(
    "repoGuideTab",
    z.union([z.literal("clone"), z.literal("publish")]),
    "publish",
    !window.localStorage,
  );
</script>

<style>
  .container {
    overflow: scroll;
  }
  .tab {
    height: 1.5rem;
    color: var(--color-foreground-contrast);
  }
</style>

{#snippet tabSnippet(name: typeof tab.value, content: string)}
  <Tab
    active={tab.value === name}
    onclick={() => {
      tab.value = name;
    }}>
    <span class="tab">{content}</span>
  </Tab>
{/snippet}

<Border
  stylePosition="relative"
  variant="ghost"
  flatBottom
  styleDisplay="flex"
  styleWidth="100%"
  styleGap="1rem"
  stylePadding="0 1rem">
  {@render tabSnippet("clone", "Clone a repo from the network")}
  {@render tabSnippet("publish", "Publish existing repo")}
</Border>

<Border
  variant="ghost"
  flatTop
  stylePadding="1rem"
  styleDisplay="block"
  styleFlexDirection="column"
  styleAlignItems="flex-start">
  <div class="container txt-small">
    {#if tab.value === "clone"}
      <Markdown content={clone} />
    {:else if tab.value === "publish"}
      <Markdown content={publish} />
    {/if}
  </div>
</Border>
