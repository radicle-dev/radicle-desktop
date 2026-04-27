<script lang="ts">
  import type { Diff } from "@bindings/diff/Diff";
  import type { Commit } from "@bindings/repo/Commit";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import * as router from "@app/lib/router";
  import {
    absoluteTimestamp,
    explorerUrl,
    formatOid,
    formatTimestamp,
    gravatarURL,
    pluralize,
  } from "@app/lib/utils";

  import Changeset from "@app/components/Changeset.svelte";
  import DiffStatBadge from "@app/components/DiffStatBadge.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    commit: Commit;
    diff: Diff;
  }

  const { repo, commit, diff }: Props = $props();
</script>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .breadcrumb {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .breadcrumb-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .breadcrumb-link:hover {
    color: var(--color-text-primary);
  }
  .content {
    padding: 1rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
  .meta {
    padding: 0.25rem 0 0.5rem;
  }
  .meta-header {
    display: flex;
    gap: 0.75rem;
    align-items: flex-start;
    justify-content: space-between;
    padding: 0 0 1rem;
    flex-wrap: wrap;
  }
  .meta-title {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    min-width: 0;
  }
  .summary {
    font: var(--txt-body-l-semibold);
    overflow-wrap: anywhere;
  }
  .summary-meta {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    color: var(--color-text-secondary);
    font: var(--txt-body-m-regular);
  }
  .summary-author {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
  .summary-avatar {
    width: 1rem;
    height: 1rem;
    border-radius: 999px;
    flex: none;
  }
  .summary-timestamp {
    color: var(--color-text-quaternary);
  }
  .summary-message {
    white-space: pre-wrap;
    margin: 0;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .summary-parents {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
    align-items: center;
    font: var(--txt-body-m-regular);
    color: var(--color-text-secondary);
  }
  .summary-parents-label {
    font: inherit;
    color: inherit;
  }
  .parent-link {
    cursor: pointer;
    background: none;
    border: none;
    padding: 0;
  }
  .parent-link:hover {
    color: var(--color-text-primary);
  }
  .chips {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    flex-wrap: wrap;
    padding-top: 0.125rem;
  }
  .files-chip {
    padding: 0 0.5rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-sm);
    height: 1.5rem;
    display: flex;
    align-items: center;
    font: var(--txt-code-regular);
    color: var(--color-text-secondary);
  }
</style>

<Layout selfScroll>
  <div class="page">
    <Topbar>
      <div class="breadcrumb">
        <button
          class="breadcrumb-link"
          onclick={() =>
            router.push({
              resource: "repo.commits",
              rid: repo.rid,
            })}>
          All commits
        </button>
        <Icon name="chevron-right" />
        <Id id={commit.id} clipboard={commit.id} placement="bottom-start" />
        <ExternalLink
          href={explorerUrl(`${repo.rid}/commits/${commit.id}`)}
          title="Open in app.radicle.xyz" />
      </div>
    </Topbar>
    <ScrollArea style="height: 100%; min-width: 0;">
      <div class="content">
        <section class="meta">
          <div class="meta-header">
            <div class="meta-title">
              <div class="summary txt-selectable">{commit.summary}</div>
              <div class="summary-meta">
                <span class="summary-author">
                  <img
                    class="summary-avatar"
                    alt=""
                    src={gravatarURL(commit.author.email)} />
                  <span class="txt-selectable">{commit.author.name}</span>
                </span>
                committed
                <Id id={commit.id} clipboard={commit.id} />
                <span
                  class="summary-timestamp"
                  title={absoluteTimestamp(commit.committer.time * 1000)}>
                  {formatTimestamp(commit.committer.time * 1000)}
                </span>
              </div>
              <div class="summary-parents">
                <span class="summary-parents-label">
                  {commit.parents.length === 1 ? "parent" : "parents"}
                </span>
                {#if commit.parents.length === 0}
                  <span>Initial commit</span>
                {:else}
                  {#each commit.parents as parent}
                    <button
                      class="parent-link txt-id"
                      onclick={() => {
                        void router.push({
                          resource: "repo.commit",
                          rid: repo.rid,
                          commit: parent,
                        });
                      }}>
                      {formatOid(parent)}
                    </button>
                  {/each}
                {/if}
              </div>
              <pre class="summary-message txt-selectable">{commit.message
                  .replace(commit.summary, "")
                  .trim()}</pre>
            </div>
            <div class="chips">
              <div class="files-chip">
                {diff.stats.filesChanged}
                {pluralize("file", diff.stats.filesChanged)} changed
              </div>
              <DiffStatBadge stats={diff.stats} />
            </div>
          </div>
        </section>

        <Changeset {diff} head={commit.id} />
      </div>
    </ScrollArea>
  </div>
</Layout>
