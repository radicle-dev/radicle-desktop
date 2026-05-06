<script lang="ts" module>
  export interface ActivityItem<T = unknown> {
    key: string;
    timestamp: number;
    data: T;
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
    repoDelegates: Author[];
    rid: string;
    activityItems?: ActivityItem<A>[];
    renderActivity?: Snippet<[A, { hideAuthor: boolean }]>;
    authorOf?: (data: A) => Author | undefined;
  }

  /* eslint-disable prefer-const */
  let {
    cobId,
    commentThreads,
    config,
    createComment,
    editComment,
    reactOnComment,
    repoDelegates,
    rid,
    activityItems,
    renderActivity,
    authorOf,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let previousCobId = cobId;
  let focusReply: boolean = $state(false);
  let commentFormKey = $state(0);

  type TimelineEntry =
    | { kind: "thread"; key: string; timestamp: number; thread: Thread }
    | { kind: "activity"; key: string; timestamp: number; data: A };

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
    | { kind: "single"; entry: Extract<TimelineEntry, { kind: "activity" }> }
    | {
        kind: "group";
        author: Author;
        entries: Extract<TimelineEntry, { kind: "activity" }>[];
      };

  const runs: Run[] = $derived.by(() => {
    const result: Run[] = [];
    for (const entry of timeline) {
      if (entry.kind === "thread") {
        result.push({ kind: "thread", entry });
        continue;
      }
      const author = entryAuthor(entry);
      const last = result[result.length - 1];
      if (
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
          };
        } else {
          last.entries.push(entry);
        }
      } else {
        result.push({ kind: "single", entry });
      }
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
  .connector {
    width: 1px;
    height: 1rem;
    margin-left: 1.25rem;
    background-color: var(--color-border-subtle);
  }
  .run-header {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
  .run-ellipsis {
    color: var(--color-text-quaternary);
  }
  .run-children {
    margin-left: 1.5rem;
  }
</style>

<div style:margin="1.5rem 0 2.5rem 0">
  <div>
    {#each runs as run, runIndex (runIndex)}
      {#if run.kind === "thread"}
        <ThreadComponent
          thread={run.entry.thread}
          {rid}
          currentUserNid={config.publicKey}
          canEditComment={partial(
            roles.isDelegateOrAuthor,
            config.publicKey,
            repoDelegates.map(delegate => delegate.did),
          )}
          {editComment}
          createReply={createComment}
          {reactOnComment} />
        <div class="connector"></div>
      {:else if run.kind === "single" && renderActivity}
        {@render renderActivity(run.entry.data, { hideAuthor: false })}
        <div class="connector"></div>
      {:else if run.kind === "group" && renderActivity}
        <div class="run-header txt-body-m-regular">
          <NodeId {...authorForNodeId(run.author)} />
          <span class="run-ellipsis">…</span>
        </div>
        <div class="run-children">
          {#each run.entries as entry (entry.key)}
            {@render renderActivity(entry.data, { hideAuthor: true })}
            <div class="connector"></div>
          {/each}
        </div>
      {/if}
    {/each}

    <div id={`reply-${cobId}`}>
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
