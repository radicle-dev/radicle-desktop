<script lang="ts">
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import sortBy from "lodash/sortBy.js";

  import { authorForNodeId } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import Id from "@app/components/Id.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import VisibilityBadge from "@app/components/VisibilityBadge.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    horizontal?: boolean;
    repo: RepoInfo;
  }

  const { horizontal = false, repo }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
</script>

<style>
  .list {
    flex-direction: column;
    align-items: flex-start;
  }
  .horizontal {
    flex-direction: row;
    flex-wrap: wrap;
  }
  .metadata-divider {
    width: 2px;
    background-color: var(--color-fill-ghost);
    height: calc(100% + 4px);
    top: 0;
    position: relative;
  }
  .metadata-divider-horizontal {
    height: 2px;
    background-color: var(--color-fill-ghost);
    width: calc(100% + 4px);
    top: 0;
    left: -2px;
    position: relative;
  }
  .metadata-section {
    padding: 0.5rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    flex: 1;
    white-space: nowrap;
  }
  .metadata-section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
</style>

<Border
  variant="ghost"
  styleGap="0"
  styleFlexDirection="column"
  styleAlignItems="flex-start">
  <div class="metadata-section" style:flex="1" style:width="100%">
    <div class="metadata-section-title">Visibility</div>
    <VisibilityBadge type={repo.visibility.type} />
  </div>

  {#if repo.visibility.type === "private"}
    <div class="metadata-divider-horizontal"></div>

    <div class="metadata-section" style:flex="1" style:width="100%">
      <div class="metadata-section-title">Allow</div>
      {#if repo.visibility.allow}
        <div class="global-flex list" class:horizontal>
          {#each repo.visibility.allow as node}
            <NodeId {...authorForNodeId(node)} />
          {/each}
        </div>
      {:else}
        <div class="global-flex txt-missing" style:gap="0.25rem">
          <Icon name="none" /> None
        </div>
      {/if}
    </div>
  {/if}

  <div class="metadata-divider-horizontal"></div>

  <div class="metadata-section" style:flex="1" style:width="100%">
    <div class="metadata-section-title">Delegates</div>
    <div class="global-flex list" class:horizontal>
      {#each sortBy(repo.delegates, d => {
        return d.alias?.toLowerCase();
      }) as delegate}
        <NodeId {...authorForNodeId(delegate)} />
      {/each}
    </div>
  </div>

  <div class="metadata-divider-horizontal"></div>

  <div
    class="global-flex"
    style:gap="0"
    style:flex-direction="row"
    style:width="100%">
    <div class="metadata-section">
      <div class="metadata-section-title">Default branch</div>
      <span>
        <span class="txt-selectable">{project.data.defaultBranch}</span>
        ->
        <Id id={project.meta.head} variant="commit" />
      </span>
    </div>

    <div class="metadata-divider"></div>

    <div class="metadata-section">
      <div class="metadata-section-title">Threshold</div>
      {repo.threshold}
    </div>
  </div>
</Border>
