<script lang="ts" module>
  export interface ActivityItem<T = unknown> {
    key: string;
    timestamp: number;
    data: T;
    /// Renders as a card or a filled band rather than a bare row of text.
    /// Grouped rows are packed tight against each other, which reads as one
    /// element when the things being packed have their own edges, so these are
    /// never folded into a run.
    standalone?: boolean;
  }
</script>

<script lang="ts" generics="A">
  import type { Author } from "@bindings/cob/Author";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { Snippet } from "svelte";

  import partial from "lodash/partial";

  import * as roles from "@app/lib/roles";
  import { authorForNodeId } from "@app/lib/utils";

  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import ThreadComponent from "@app/components/Thread.svelte";

  interface Props {
    cobId: string;
    commentThreads: Thread[];
    config: Config;
    repoDelegates: Author[];
    createComment: (
      body: string,
      embeds: Embed[],
      replyTo?: string,
    ) => Promise<void>;
    editComment: (
      commentId: string,
      body: string,
      embeds: Embed[],
    ) => Promise<void>;
    reactOnComment: (
      commentId: string,
      authors: Author[],
      reaction: string,
    ) => Promise<void>;
    // Optional: a discussion whose host has no way to redact a comment simply
    // does not offer the action.
    deleteComment?: (commentId: string) => Promise<void>;
    rid: string;
    activityItems?: ActivityItem<A>[];
    /// Drops the outer margin, for a host that owns the spacing between its own
    /// blocks. The default keeps the spacing the timeline pages rely on.
    flush?: boolean;
    renderActivity?: Snippet<[A, { hideAuthor: boolean }]>;
    authorOf?: (data: A) => Author | undefined;
    afterActivity?: Snippet;
  }

  /* eslint-disable prefer-const */
  let {
    cobId,
    commentThreads,
    config,
    repoDelegates,
    createComment,
    editComment,
    reactOnComment,
    deleteComment,
    rid,
    activityItems,
    flush = false,
    renderActivity,
    authorOf,
    afterActivity,
  }: Props = $props();
  /* eslint-enable prefer-const */

  // svelte-ignore state_referenced_locally
  let previousCobId = cobId;
  let focusReply: boolean = $state(false);
  let commentFormKey = $state(0);

  type TimelineEntry =
    | { kind: "thread"; key: string; timestamp: number; thread: Thread }
    | {
        kind: "activity";
        key: string;
        timestamp: number;
        data: A;
        standalone: boolean;
      };

  const timeline: TimelineEntry[] = $derived(
    [
      ...commentThreads.map(
        thread =>
          ({
            kind: "thread",
            key: thread.root.id,
            timestamp: thread.root.edits[0].timestamp,
            thread,
          }) satisfies TimelineEntry,
      ),
      ...(activityItems ?? []).map(
        item =>
          ({
            kind: "activity",
            key: item.key,
            timestamp: item.timestamp,
            data: item.data,
            standalone: item.standalone === true,
          }) satisfies TimelineEntry,
      ),
    ].sort((a, b) => a.timestamp - b.timestamp),
  );

  function entryAuthor(entry: TimelineEntry): Author | undefined {
    if (entry.kind === "thread") {
      return entry.thread.root.author;
    }
    return authorOf?.(entry.data);
  }

  type Run =
    | { kind: "thread"; entry: Extract<TimelineEntry, { kind: "thread" }> }
    | {
        kind: "single";
        entry: Extract<TimelineEntry, { kind: "activity" }>;
        repeatsAuthor: boolean;
      }
    | {
        kind: "group";
        author: Author;
        entries: Extract<TimelineEntry, { kind: "activity" }>[];
        repeatsAuthor: boolean;
      };

  function runAuthor(run: Run): Author | undefined {
    if (run.kind === "thread") return run.entry.thread.root.author;
    if (run.kind === "single") return entryAuthor(run.entry);
    return run.author;
  }

  const runs: Run[] = $derived.by(() => {
    const result: Run[] = [];
    for (const entry of timeline) {
      if (entry.kind === "thread") {
        result.push({ kind: "thread", entry });
        continue;
      }
      const author = entryAuthor(entry);
      const last = result[result.length - 1];
      const groupable =
        !entry.standalone &&
        !(last?.kind === "single" && last.entry.standalone);
      if (
        groupable &&
        author &&
        last &&
        ((last.kind === "single" &&
          entryAuthor(last.entry)?.did === author.did) ||
          (last.kind === "group" && last.author.did === author.did))
      ) {
        if (last.kind === "single") {
          result[result.length - 1] = {
            kind: "group",
            author,
            entries: [last.entry, entry],
            repeatsAuthor: last.repeatsAuthor,
          };
        } else {
          last.entries.push(entry);
        }
      } else {
        result.push({ kind: "single", entry, repeatsAuthor: false });
      }
    }
    // A run only names its author when the run before it was someone else's.
    // A comment breaks a run, so without this the same person's actions are
    // re-attributed on the other side of their own comment.
    for (let i = 1; i < result.length; i++) {
      const run = result[i];
      if (run.kind === "thread") continue;
      const previous = runAuthor(result[i - 1]);
      const current = runAuthor(run);
      run.repeatsAuthor = Boolean(
        previous && current && previous.did === current.did,
      );
    }
    return result;
  });

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    cobId;

    if (cobId !== previousCobId) {
      previousCobId = cobId;
      focusReply = false;
      commentFormKey += 1;
    }
  });
</script>

<style>
  .discussion {
    margin: 1.5rem 0 2.5rem;
  }
  .discussion.flush {
    margin: 0;
  }
  .timeline-rail {
    position: relative;
  }
  .activity-stream {
    position: relative;
  }
  .activity-stream.has-runs::before {
    content: "";
    position: absolute;
    top: 0.5rem;
    bottom: -1rem;
    left: 1rem;
    width: 1px;
    background-color: var(--color-border-subtle);
    pointer-events: none;
    z-index: -1;
  }
  .timeline-rail :global(.icon) {
    background-color: var(--color-surface-canvas);
  }
  .timeline-rail :global(.timeline-item.toggleable:hover .icon),
  .timeline-rail :global(.timeline-item.toggleable:focus-visible .icon),
  .timeline-rail :global(.older-revisions:hover .icon),
  .timeline-rail :global(.older-revisions:focus-visible .icon) {
    background-color: var(--color-surface-subtle);
  }
  .timeline-rail :global(.verdict-accept .icon),
  .timeline-rail :global(.verdict-reject .icon),
  .timeline-rail :global(.verdict-comment .icon),
  .timeline-rail :global(.merge-badge .icon) {
    background-color: transparent;
  }
  .timeline-rail :global(.replies-wrapper) {
    margin-left: 1.5rem;
  }
  .timeline-rail :global(.replies-wrapper)::before {
    display: none;
  }
  .connector {
    height: 0.5rem;
  }
  /* Grouped actions are bare rows whose own min-height already spaces them;
     the connector on top of that leaves more air between them than between
     the header and the first row. The last one is kept: it separates the
     group from whatever follows. */
  .run-children > .connector:not(:last-child) {
    height: 0;
  }
  .run-header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 1.5rem 0.5rem 0.5rem;
    color: var(--color-text-tertiary);
    min-height: 2.5rem;
  }
  /* The first group's top padding is redundant with the space the timeline
     container already leaves below the tabs, so drop it to tighten the gap
     between the patch nav and the first author. */
  .activity-stream > .run-header:first-child {
    padding-top: 0;
  }
  /* A first child with its own top margin (e.g. a revision card) would push
     that margin out above the group, leaving the rail to span a wider gap than
     the rest of the group's spacing. */
  .run-children > :global(:first-child) {
    margin-top: 0;
  }
  .reply-wrapper {
    margin-top: 1rem;
  }
  /* Nothing above it to connect to, so the host's own spacing is the whole gap. */
  .activity-stream:not(.has-runs) + .reply-wrapper {
    margin-top: 0;
  }
</style>

<div class="discussion" class:flush>
  <div class="timeline-rail">
    <div class="activity-stream" class:has-runs={runs.length > 0}>
      {#each runs as run, runIndex (runIndex)}
        {#if run.kind === "thread"}
          <ThreadComponent
            thread={run.entry.thread}
            {rid}
            currentUserNid={config.publicKey}
            canModifyComment={partial(
              roles.isDelegateOrAuthor,
              config.publicKey,
              repoDelegates.map(delegate => delegate.did),
            )}
            {editComment}
            {deleteComment}
            createReply={createComment}
            {reactOnComment} />
          <div class="connector"></div>
        {:else if run.kind === "single" && renderActivity}
          {@render renderActivity(run.entry.data, {
            hideAuthor: run.repeatsAuthor,
          })}
          <div class="connector"></div>
        {:else if run.kind === "group" && renderActivity}
          {#if !run.repeatsAuthor}
            <div class="run-header">
              <NodeId
                {...authorForNodeId(run.author)}
                styleFont="var(--txt-body-m-medium)" />
            </div>
          {/if}
          <div class="run-children">
            {#each run.entries as entry (entry.key)}
              {@render renderActivity(entry.data, { hideAuthor: true })}
              <div class="connector"></div>
            {/each}
          </div>
        {/if}
      {/each}

      {@render afterActivity?.()}
    </div>

    <div id={`reply-${cobId}`} class="reply-wrapper">
      {#key commentFormKey}
        <ExtendedTextarea
          disallowEmptyBody
          {rid}
          focus={focusReply}
          borderVariant="ghost"
          stylePadding="0.5rem 0.75rem"
          hideDiscard
          placeholder="Leave a comment"
          submitActiveVariant="secondary"
          close={() => {
            focusReply = false;
            commentFormKey += 1;
          }}
          submit={async ({ comment, embeds }) => {
            try {
              await createComment(comment, Array.from(embeds.values()));
            } finally {
              focusReply = false;
              commentFormKey += 1;
            }
          }} />
      {/key}
    </div>
  </div>
</div>
