<script lang="ts">
  import type { ActionWithAuthor } from "@bindings/cob/inbox/ActionWithAuthor";
  import type { Action as IssueAction } from "@bindings/cob/issue/Action";
  import type { Action as PatchAction } from "@bindings/cob/patch/Action";
  import type { ComponentProps } from "svelte";
  import type { RepoRoute } from "@app/views/repo/router";
  import type { NotificationItem } from "@bindings/cob/inbox/NotificationItem";

  import uniqWith from "lodash/uniqWith";
  import {
    authorForNodeId,
    formatTimestamp,
    issueStatusBackgroundColor,
    issueStatusColor,
    patchStatusBackgroundColor,
    patchStatusColor,
  } from "@app/lib/utils";
  import { closeFocused } from "./Popover.svelte";
  import { compressActions } from "@app/lib/notification";
  import { push } from "@app/lib/router";

  import Icon from "./Icon.svelte";
  import InlineTitle from "./InlineTitle.svelte";
  import NakedButton from "./NakedButton.svelte";
  import NodeId from "./NodeId.svelte";

  interface Props {
    rid: string;
    kind: "issue" | "patch";
    oid: string;
    clearByIds: (ids: string[]) => Promise<void>;
    notificationItems: NotificationItem[];
    selected?: boolean;
  }

  const {
    clearByIds,
    notificationItems,
    rid,
    oid,
    kind,
    selected = false,
  }: Props = $props();

  type Action = ActionWithAuthor<IssueAction> | ActionWithAuthor<PatchAction>;

  const uniqueActions = $derived.by(() => {
    return compressActions(
      uniqWith(
        notificationItems.flatMap<Action>(n => n.actions),
        (a, b) =>
          Boolean(
            a.oid === b.oid &&
              a.type === b.type &&
              a.author.did &&
              b.author.did,
          ),
      ).sort((a, b) => b.timestamp - a.timestamp),
      kind,
      oid,
    );
  });

  const clearIcon = $derived(
    uniqueActions.length > 1 ? "broom-double" : "broom",
  );

  const title = $derived.by(() => {
    const lastDetail = notificationItems.at(-1);
    if (lastDetail && "title" in lastDetail) {
      return lastDetail.title;
    }
  });

  const icon: ComponentProps<typeof Icon>["name"] = $derived.by(() => {
    const lastDetail = notificationItems.at(-1);
    if (lastDetail?.type === "issue" && lastDetail.status.status !== "open") {
      return `issue-${lastDetail.status.status}` as const;
    } else if (lastDetail?.type === "issue") {
      return "issue" as const;
    } else if (
      lastDetail?.type === "patch" &&
      lastDetail.status.status !== "open"
    ) {
      return `patch-${lastDetail.status.status}` as const;
    } else {
      return "patch" as const;
    }
  });

  const statusColor = $derived.by(() => {
    const lastDetail = notificationItems.at(-1);
    if (lastDetail?.type === "patch") {
      return {
        color: patchStatusColor[lastDetail.status.status],
        background: patchStatusBackgroundColor[lastDetail.status.status],
      };
    } else if (lastDetail?.type === "issue") {
      return {
        color: issueStatusColor[lastDetail.status.status],
        background: issueStatusBackgroundColor[lastDetail.status.status],
      };
    } else {
      return {
        color: "var(--color-foreground-dim)",
        background: "var(--color-fill-ghost)",
      };
    }
  });

  const route = $derived.by(() => {
    const lastDetail = notificationItems.at(-1);
    switch (lastDetail?.type) {
      case "patch":
        return {
          resource: "repo.patch",
          rid,
          patch: lastDetail.id,
          status: undefined,
        } as RepoRoute;
      case "issue":
        return {
          resource: "repo.issue",
          rid,
          issue: lastDetail.id,
          status: "all",
        } as RepoRoute;
    }

    return undefined;
  });
</script>

<style>
  .notification-teaser {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.25rem;
    min-height: 5rem;
    background-color: var(--color-background-float);
    padding: 1rem;
    cursor: pointer;
    font-size: var(--font-size-regular);
    word-break: break-word;
  }
  .clear-icon {
    display: none;
  }
  .notification-teaser:hover .clear-icon {
    display: flex;
  }
  .selected {
    background-color: var(--color-fill-float-hover);
  }
  .notification-teaser:hover {
    background-color: var(--color-fill-float-hover);
  }
  .status {
    padding: 0;
    margin-right: 1rem;
  }
  .notification-teaser:first-of-type {
    clip-path: var(--3px-top-corner-fill);
  }
  .notification-teaser:last-of-type {
    clip-path: var(--3px-bottom-corner-fill);
  }
  .notification-teaser:only-of-type {
    clip-path: var(--3px-corner-fill);
  }
</style>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  tabindex="0"
  role="button"
  class:selected
  class="notification-teaser"
  onclick={() => {
    if (route) {
      void push(route);
      closeFocused();
    }
  }}>
  <div
    class="global-flex"
    style:justify-content="space-between"
    style:align-items="flex-start"
    style:width="100%">
    <div class="global-flex">
      <div
        class="global-counter status"
        style:align-self="flex-start"
        style:color={statusColor.color}
        style:background-color={statusColor.background}>
        <Icon name={icon} />
      </div>
      <div
        class="global-flex"
        style:flex-direction="column"
        style:align-items="flex-start">
        {#if title}
          <InlineTitle content={title} />
        {/if}
        <div class="txt-small">
          {#each uniqueActions as action}
            <div
              class="global-flex"
              style:gap="0.25rem"
              style:min-height="2rem"
              style:flex-wrap="wrap">
              <NodeId {...authorForNodeId(action.items[0].author)} />
              <span>{@html action.summary}</span>
              <span>{formatTimestamp(action.items[0].timestamp)}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>
    <div class="clear-icon">
      <NakedButton
        stylePadding="0 0.25rem"
        variant="ghost"
        onclick={e => {
          e.stopPropagation();
          void clearByIds(notificationItems.map(n => n.rowId));
        }}>
        <Icon name={clearIcon} />
      </NakedButton>
    </div>
  </div>
</div>
