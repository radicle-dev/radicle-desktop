<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { ComponentProps } from "svelte";

  import {
    cachedIssueById,
    cachedPatchById,
    cachedRepoById,
    cachedRepoCommit,
  } from "@app/lib/invoke";
  import type { MentionTarget } from "@app/lib/mentions";
  import { push } from "@app/lib/router";
  import {
    formatOid,
    issueStatusColor,
    patchStatusColor,
  } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import RepoAvatar from "@app/components/RepoAvatar.svelte";

  type IconName = ComponentProps<typeof Icon>["name"];

  interface Props {
    target: MentionTarget;
    // The label the author wrote, shown until the reference resolves and as a
    // fallback when it cannot be resolved at all.
    fallback: string;
  }

  const { target, fallback }: Props = $props();

  let resolvedRepoName: string | undefined = $state(undefined);
  let issue: Issue | undefined = $state(undefined);
  let patch: Patch | undefined = $state(undefined);
  let commitSummary: string | undefined = $state(undefined);

  const repoName = $derived(resolvedRepoName ?? fallback);

  // A reference can point at a repo or COB this node does not have, so a
  // failed lookup is expected and leaves the chip on its fallback label.
  $effect(() => {
    if (target.type !== "repo") return;
    let cancelled = false;
    void cachedRepoById(target.rid)
      .then(result => {
        if (cancelled || !result) return;
        resolvedRepoName = result.payloads["xyz.radicle.project"]?.data.name;
      })
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (target.type !== "cob" || target.kind !== "issue") return;
    let cancelled = false;
    void cachedIssueById(target.rid, target.oid)
      .then(result => {
        if (!cancelled && result) issue = result;
      })
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (target.type !== "cob" || target.kind !== "patch") return;
    let cancelled = false;
    void cachedPatchById(target.rid, target.oid)
      .then(result => {
        if (!cancelled && result) patch = result;
      })
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (target.type !== "commit") return;
    let cancelled = false;
    void cachedRepoCommit(target.rid, target.oid)
      .then(result => {
        if (!cancelled && result) commitSummary = result.summary;
      })
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  const issueIcon: Record<Issue["state"]["status"], IconName> = {
    open: "issue",
    closed: "issue-closed",
  };
  const patchIcon: Record<Patch["state"]["status"], IconName> = {
    draft: "patch-draft",
    open: "patch",
    archived: "patch-archived",
    merged: "patch-merged",
  };

  function navigate() {
    if (target.type === "repo") {
      void push({ resource: "repo.home", rid: target.rid });
    } else if (target.type === "cob" && target.kind === "issue") {
      void push({
        resource: "repo.issue",
        rid: target.rid,
        issue: target.oid,
        status: "all",
      });
    } else if (target.type === "commit") {
      void push({
        resource: "repo.commit",
        rid: target.rid,
        commit: target.oid,
      });
    } else if (target.type === "cob") {
      void push({
        resource: "repo.patch",
        rid: target.rid,
        patch: target.oid,
        status: undefined,
        reviewId: undefined,
      });
    }
  }
</script>

<style>
  .mention {
    display: inline-flex;
    gap: 0.25rem;
    max-width: 20rem;
    padding: 0 0.25rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-fill-ghost);
    color: var(--color-text-primary);
    font: inherit;
    font-weight: var(--font-weight-medium);
    text-decoration: none;
    /* The label, not the leading icon, defines where the chip sits: an
       inline-flex box takes its baseline from the first item that participates
       in baseline alignment, and an icon has none of its own. Without this the
       baseline is synthesized from the icon's box and the text sits low. */
    align-items: baseline;
    vertical-align: baseline;
    /* The chip is one line: nothing inside it may wrap, or it grows taller
       than the line box and pushes the paragraph apart. */
    white-space: nowrap;
    cursor: pointer;
  }
  .mention:hover,
  .mention:focus-visible {
    background-color: var(--color-fill-ghost-hover);
  }
  .mention-label {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  .mention-oid {
    flex-shrink: 0;
    color: var(--color-text-secondary);
    font-weight: var(--font-weight-regular);
  }
  /* Centred on the line rather than baseline-aligned, which would hang the
     icon below the text. The icon carries the state in its colour alone: a
     filled swatch behind it reads as a second chip inside the chip. */
  .mention-status {
    display: inline-flex;
    align-items: center;
    align-self: center;
    flex-shrink: 0;
  }
  /* A person mention sits in running prose, so its alias has to share the
     baseline of the text around it. An inline-flex box takes its baseline from
     the first flex item that participates in baseline alignment, so the chain
     of wrappers `NodeId` renders has to opt into that: otherwise the baseline
     is synthesized from the avatar's bottom edge and the alias sits low. */
  .mention-node {
    display: inline-flex;
    align-items: baseline;
    vertical-align: baseline;
  }
  .mention-node :global(.avatar-alias) {
    align-items: baseline;
    /* `NodeId` sets a shorthand `font` whose line-height is shorter than body
       text; matching the surrounding line keeps the chip from altering it. */
    line-height: inherit;
  }
  /* The avatar itself is centred on the line rather than baseline-aligned,
     which would hang it below the text. The alias is then the first item that
     participates, so it defines the baseline. */
  .mention-node :global(.avatar-container) {
    align-self: center;
  }
</style>

{#if target.type === "node"}
  <span class="mention-node">
    <NodeId publicKey={target.nid} inline />
  </span>
{:else if target.type === "repo"}
  <button type="button" class="mention" onclick={navigate} title={target.rid}>
    {#if resolvedRepoName}
      <RepoAvatar rid={target.rid} name={resolvedRepoName} styleWidth="1rem" />
    {:else}
      <Icon name="repository" />
    {/if}
    <span class="mention-label">{repoName}</span>
  </button>
{:else if target.type === "commit"}
  <button
    type="button"
    class="mention"
    onclick={navigate}
    title={commitSummary ?? target.oid}>
    <span class="mention-status">
      <Icon name="commit" />
    </span>
    <span class="mention-label">{commitSummary ?? fallback}</span>
    <span class="mention-oid">{formatOid(target.oid)}</span>
  </button>
{:else if target.kind === "issue"}
  <button
    type="button"
    class="mention"
    onclick={navigate}
    title={`${issue?.title ?? fallback} · ${target.oid}`}>
    <span
      class="mention-status"
      style:color={issue ? issueStatusColor[issue.state.status] : undefined}>
      <Icon name={issue ? issueIcon[issue.state.status] : "issue"} />
    </span>
    <span class="mention-label">{issue?.title ?? fallback}</span>
  </button>
{:else}
  <button
    type="button"
    class="mention"
    onclick={navigate}
    title={`${patch?.title ?? fallback} · ${target.oid}`}>
    <span
      class="mention-status"
      style:color={patch ? patchStatusColor[patch.state.status] : undefined}>
      <Icon name={patch ? patchIcon[patch.state.status] : "patch"} />
    </span>
    <span class="mention-label">{patch?.title ?? fallback}</span>
  </button>
{/if}
