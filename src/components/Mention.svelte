<script lang="ts">
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Config } from "@bindings/config/Config";
  import type { ComponentProps } from "svelte";

  import {
    cachedConfig,
    cachedIssueById,
    cachedPatchById,
    cachedRepoById,
    cachedRepoCommit,
  } from "@app/lib/invoke";
  import type { MentionTarget } from "@app/lib/mentions";
  import { mentionUrl } from "@app/lib/mentions";
  import type { Route } from "@app/lib/router";
  import { push, routeToPath } from "@app/lib/router";
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
  // Set once a lookup has finished and come back empty: the repo is not
  // replicated locally, or the object is gone.
  let missing = $state(false);
  let config: Config | undefined = $state(undefined);

  // Every chip is a real link to the web explorer. When the target is here,
  // the click is intercepted and handled in-app; when it is not, following the
  // link is the only thing that can still work.
  const explorerHref = $derived(
    config ? mentionUrl(target, config) : undefined,
  );

  $effect(() => {
    if (config) return;
    let cancelled = false;
    void cachedConfig()
      .then(result => {
        if (!cancelled) config = result;
      })
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  const repoName = $derived(resolvedRepoName ?? fallback);

  // A reference can point at a repo or COB this node does not have, so a
  // failed lookup is expected and leaves the chip on its fallback label.
  $effect(() => {
    if (target.type !== "repo") return;
    let cancelled = false;
    void cachedRepoById(target.rid)
      .then(result => {
        if (cancelled) return;
        if (!result) {
          missing = true;
          return;
        }
        resolvedRepoName = result.payloads["xyz.radicle.project"]?.data.name;
      })
      .catch(() => {
        if (!cancelled) missing = true;
      });
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (target.type !== "cob" || target.kind !== "issue") return;
    let cancelled = false;
    void cachedIssueById(target.rid, target.oid)
      .then(result => {
        if (cancelled) return;
        if (result) issue = result;
        else missing = true;
      })
      .catch(() => {
        if (!cancelled) missing = true;
      });
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (target.type !== "cob" || target.kind !== "patch") return;
    let cancelled = false;
    void cachedPatchById(target.rid, target.oid)
      .then(result => {
        if (cancelled) return;
        if (result) patch = result;
        else missing = true;
      })
      .catch(() => {
        if (!cancelled) missing = true;
      });
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (target.type !== "commit") return;
    let cancelled = false;
    void cachedRepoCommit(target.rid, target.oid)
      .then(result => {
        if (cancelled) return;
        if (result) commitSummary = result.summary;
        else missing = true;
      })
      .catch(() => {
        if (!cancelled) missing = true;
      });
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

  const route: Route | undefined = $derived.by(() => {
    // A person is rendered by `NodeId`, which brings its own hover card and
    // does not use any of this.
    if (target.type === "node") return undefined;
    if (target.type === "repo") {
      return { resource: "repo.home", rid: target.rid };
    }
    if (target.type === "commit") {
      return { resource: "repo.commit", rid: target.rid, commit: target.oid };
    }
    if (target.kind === "issue") {
      return {
        resource: "repo.issue",
        rid: target.rid,
        issue: target.oid,
        status: "all",
      };
    }

    return {
      resource: "repo.patch",
      rid: target.rid,
      patch: target.oid,
      status: undefined,
      reviewId: undefined,
    };
  });

  // An in-app path while the target is here, the explorer only once a lookup
  // has come back empty. The webview opens an external href itself, ahead of
  // any handler here, so a chip that should navigate in-app must never carry
  // one.
  const href = $derived(missing ? explorerHref : route && routeToPath(route));

  function handleClick(event: MouseEvent) {
    // Nothing to open in-app, so let the webview follow the link out to the
    // explorer rather than swallowing the click and appearing to do nothing.
    if (missing || !route) return;

    event.preventDefault();

    // A reference to the page you are already on would otherwise re-render
    // the same view and look like a dead link. Scrolling back to the top is
    // the honest answer: you are already here.
    if (isCurrentPage(route)) {
      scrollToTop(event.currentTarget as HTMLElement);
      return;
    }

    void push(route);
  }

  /**
   * Scroll whatever actually scrolls around the chip. Found by walking up from
   * the chip rather than by class name: which element scrolls differs between
   * views, and the issue page's container carries no class at all.
   */
  function scrollToTop(from: HTMLElement) {
    let node = from.parentElement;
    while (node) {
      const overflow = getComputedStyle(node).overflowY;
      if (
        /auto|scroll/.test(overflow) &&
        node.scrollHeight > node.clientHeight
      ) {
        node.scrollTo({ top: 0, behavior: "smooth" });
        return;
      }
      node = node.parentElement;
    }
    window.scrollTo({ top: 0, behavior: "smooth" });
  }

  /**
   * Whether a route points at the page currently open, compared by path only:
   * the same issue viewed under a different status filter is still the same
   * issue. Both sides go through `URL` so their escaping matches.
   */
  function isCurrentPage(candidate: Route): boolean {
    return (
      new URL(routeToPath(candidate), window.origin).pathname ===
      window.location.pathname
    );
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
    /* One step darker than the canvas the content sits on, in both themes:
       light #f8f9fa on #ffffff, dark #00060f on #0a1018. */
    background-color: var(--color-surface-base);
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
    /* A step further from the canvas: deeper in light, lifted in dark, since
       there is nothing darker than `surface-base` to reach for there. */
    background-color: var(--color-surface-subtle);
  }
  .mention-label {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  /* Only the family, not the `--txt-code-*` shorthands: those carry their own
     line-height, which would break the chip's baseline alignment. */
  .mention-oid {
    flex-shrink: 0;
    color: var(--color-text-secondary);
    font-family: var(--font-family-code);
    font-weight: var(--font-weight-regular);
  }
  /* A node with no known alias falls back to its raw id, which reads as an
     identifier rather than a name. */
  .mention-node :global(.no-alias) {
    font-family: var(--font-family-code);
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
    /* Matching `.mention`, so a person reads as the same kind of chip as a
       repo or an issue rather than as bare text with an avatar. */
    gap: 0.25rem;
    padding: 0 0.25rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-base);
  }
  .mention-node:hover {
    background-color: var(--color-surface-subtle);
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
  <a
    class="mention"
    {href}
    target={missing ? "_blank" : undefined}
    rel={missing ? "noreferrer" : undefined}
    onclick={handleClick}
    title={missing ? `${target.rid} — not replicated locally` : target.rid}>
    <span class="mention-status">
      {#if resolvedRepoName}
        <RepoAvatar
          rid={target.rid}
          name={resolvedRepoName}
          styleWidth="1rem" />
      {:else}
        <Icon name="repository" />
      {/if}
    </span>
    <span class="mention-label">{repoName}</span>
  </a>
{:else if target.type === "commit"}
  <a
    class="mention"
    {href}
    target={missing ? "_blank" : undefined}
    rel={missing ? "noreferrer" : undefined}
    onclick={handleClick}
    title={missing
      ? `${target.oid} — not replicated locally`
      : (commitSummary ?? target.oid)}>
    <span class="mention-status">
      <Icon name="commit" />
    </span>
    <span class="mention-label">{commitSummary ?? fallback}</span>
    {#if commitSummary}
      <!-- Only alongside a summary. Unresolved, the label is already the id,
           and repeating it reads as "commit 8c5df5e 8c5df5e". -->
      <span class="mention-oid">{formatOid(target.oid)}</span>
    {/if}
  </a>
{:else if target.kind === "issue"}
  <a
    class="mention"
    {href}
    target={missing ? "_blank" : undefined}
    rel={missing ? "noreferrer" : undefined}
    onclick={handleClick}
    title={missing
      ? `${fallback} · ${target.oid} — not replicated locally`
      : `${issue?.title ?? fallback} · ${target.oid}`}>
    <span
      class="mention-status"
      style:color={issue ? issueStatusColor[issue.state.status] : undefined}>
      <Icon name={issue ? issueIcon[issue.state.status] : "issue"} />
    </span>
    <span class="mention-label">{issue?.title ?? fallback}</span>
  </a>
{:else}
  <a
    class="mention"
    {href}
    target={missing ? "_blank" : undefined}
    rel={missing ? "noreferrer" : undefined}
    onclick={handleClick}
    title={missing
      ? `${fallback} · ${target.oid} — not replicated locally`
      : `${patch?.title ?? fallback} · ${target.oid}`}>
    <span
      class="mention-status"
      style:color={patch ? patchStatusColor[patch.state.status] : undefined}>
      <Icon name={patch ? patchIcon[patch.state.status] : "patch"} />
    </span>
    <span class="mention-label">{patch?.title ?? fallback}</span>
  </a>
{/if}
