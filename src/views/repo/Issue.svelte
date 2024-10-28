<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { Operation } from "@bindings/cob/issue/Operation";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import partial from "lodash/partial";
  import { onMount, tick } from "svelte";

  import * as roles from "@app/lib/roles";
  import {
    issueStatusColor,
    publicKeyFromDid,
    scrollIntoView,
  } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";

  import { announce } from "@app/components/AnnounceSwitch.svelte";

  import Border from "@app/components/Border.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import IssueMetadata from "@app/components/IssueMetadata.svelte";
  import IssueTimelineLifecycleAction from "@app/components/IssueTimelineLifecycleAction.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import Thread from "@app/components/Thread.svelte";

  import Layout from "./Layout.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  export let repo: RepoInfo;
  export let issue: Issue;
  export let issues: Issue[];
  export let config: Config;

  let topLevelReplyOpen = false;
  let editingTitle = false;
  let updatedTitle: string = issue.title;

  // The view doesn't get destroyed when we switch between different issues in
  // the sidebar and because of that the top-level state gets retained when the
  // issue changes. This reactive statement makes sure we always load the new
  // issue and reset the state to defaults.
  let issueId = issue.id;
  $: if (issueId !== issue.id) {
    issueId = issue.id;
    topLevelReplyOpen = false;
    editingTitle = false;
    updatedTitle = issue.title;
    void loadActivity();
  }

  $: project = repo.payloads["xyz.radicle.project"]!;

  $: threads = issue.discussion
    .filter(
      comment =>
        (comment.id !== issue.discussion[0].id && !comment.replyTo) ||
        comment.replyTo === issue.discussion[0].id,
    )
    .map(thread => {
      return {
        root: thread,
        replies: issue.discussion
          .filter(comment => comment.replyTo === thread.id)
          .sort(
            (a, b) =>
              a.edits.slice(-1)[0].timestamp - b.edits.slice(-1)[0].timestamp,
          ),
      };
    }, []);

  let activity: Operation[];

  async function loadActivity() {
    activity = await invoke("activity_by_id", {
      rid: repo.rid,
      typeName: "xyz.radicle.issue",
      id: issue.id,
    });
  }

  onMount(() => {
    void loadActivity();
  });

  async function toggleReply() {
    topLevelReplyOpen = !topLevelReplyOpen;
    if (!topLevelReplyOpen) {
      return;
    }

    await tick();
    scrollIntoView(`reply-${issue.id}`, {
      behavior: "smooth",
      block: "center",
    });
  }

  async function reload() {
    issue = await invoke("issue_by_id", {
      rid: repo.rid,
      id: issue.id,
    });
    await loadActivity();
  }

  async function createComment(body: string, embeds: Embed[]) {
    try {
      await invoke("create_issue_comment", {
        rid: repo.rid,
        new: { id: issue.id, body, embeds },
        opts: { announce: $announce },
      });
    } catch (error) {
      console.error("Comment creation failed: ", error);
    } finally {
      await reload();
    }
  }

  async function createReply(replyTo: string, body: string, embeds: Embed[]) {
    try {
      await invoke("create_issue_comment", {
        rid: repo.rid,
        new: { id: issue.id, body, embeds, replyTo },
        opts: { announce: $announce },
      });
    } catch (error) {
      console.error("Comment reply creation failed", error);
    } finally {
      await reload();
    }
  }

  async function editComment(id: string, body: string, embeds: Embed[]) {
    try {
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: {
          type: "comment.edit",
          id,
          body,
          embeds,
        },
        opts: { announce: $announce },
      });
    } catch (error) {
      if (error instanceof Error) {
        console.error("Issue comment editing failed: ", error);
      }
    } finally {
      await reload();
    }
  }

  async function editTitle(id: string, title: string) {
    if (issue.title === updatedTitle) {
      editingTitle = false;
      return;
    }

    try {
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: {
          type: "edit",
          id,
          title,
        },
        opts: { announce: $announce },
      });
      issue.title = updatedTitle;
      // Update sidebar issue title without reloading the whole issue list.
      const issueIndex = issues.findIndex(i => i.id === issue.id);
      if (issueIndex !== -1) {
        issues[issueIndex].title = updatedTitle;
      }
      editingTitle = false;
    } catch (error) {
      if (error instanceof Error) {
        console.error("Issue editing failed: ", error);
      }
    }
  }

  async function reactOnComment(
    publicKey: string,
    commentId: string,
    authors: Author[],
    reaction: string,
  ) {
    try {
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: {
          type: "comment.react",
          id: commentId,
          reaction,
          active: !authors.find(
            ({ did }) => publicKeyFromDid(did) === publicKey,
          ),
        },
        opts: { announce: $announce },
      });
    } catch (error) {
      if (error instanceof Error) {
        console.error("Editing reactions failed", error);
      }
    } finally {
      await reload();
    }
  }
</script>

<style>
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    margin-bottom: 20px;
    margin-top: 6px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    word-break: break-all;
  }
  .issue-body {
    margin-top: 1rem;
    position: relative;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .issue-body::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--2px-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
  }
  .issue-teaser {
    max-width: 11rem;
    white-space: nowrap;
  }
  .issue-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding-bottom: 1rem;
  }
  .content {
    padding: 0 1rem 1rem 1rem;
  }
  .connector {
    width: 2px;
    height: 1.5rem;
    margin-left: 1.25rem;
    background-color: var(--color-background-float);
  }

  .title-icons {
    display: flex;
    gap: 0.5rem;
    margin-left: 1rem;
  }
</style>

<Layout>
  <svelte:fragment slot="breadcrumbs">
    <Link route={{ resource: "home" }}>
      <NodeId
        publicKey={config.publicKey}
        alias={config.alias}
        styleFontFamily="var(--font-family-sans-serif)"
        styleFontSize="var(--font-size-tiny)" />
    </Link>
    <Link route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
      <div class="global-flex">
        <Icon name="chevron-right" />
        {project.data.name}
      </div>
    </Link>
    <Icon name="chevron-right" />
    Issues
  </svelte:fragment>

  <svelte:fragment slot="header-center">
    <CopyableId id={issue.id} />
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <Border
      hoverable={false}
      variant="ghost"
      styleWidth="100%"
      styleHeight="32px">
      <div style:margin-left="0.5rem">
        <Icon name="issue" />
      </div>
      <span class="txt-small txt-semibold">Issues</span>
      <div class="global-flex txt-small" style:margin-left="auto">
        <div
          class="global-counter"
          style:padding="0 6px"
          style:background-color="var(--color-fill-ghost)"
          style:gap="4px">
          {project.meta.issues.open + project.meta.issues.closed}
        </div>
      </div>
    </Border>

    <div class="issue-list">
      {#each issues as sidebarIssue}
        <Link
          variant="tab"
          route={{
            resource: "repo.issue",
            rid: repo.rid,
            issue: sidebarIssue.id,
          }}>
          <div class="global-flex">
            <div
              style:color={issueStatusColor[sidebarIssue.state.status]}
              style:margin-left="2px">
              <Icon name="issue" />
            </div>
            <span class="txt-small issue-teaser txt-overflow">
              <InlineTitle content={sidebarIssue.title} fontSize="small" />
            </span>
          </div>
        </Link>
      {/each}
    </div>
  </svelte:fragment>

  <div class="content">
    {#if editingTitle}
      <div class="global-flex" style:margin-bottom="0.5rem">
        <TextInput
          valid={updatedTitle.trim().length > 0}
          bind:value={updatedTitle}
          autofocus
          onSubmit={async () => {
            if (updatedTitle.trim().length > 0) {
              await editTitle(issue.id, updatedTitle);
            }
          }}
          onDismiss={() => {
            updatedTitle = issue.title;
            editingTitle = !editingTitle;
          }} />
        <div class="title-icons">
          <Icon
            name="checkmark"
            onclick={async () => {
              if (updatedTitle.trim().length > 0) {
                await editTitle(issue.id, updatedTitle);
              }
            }} />
          <Icon
            name="cross"
            onclick={() => {
              updatedTitle = issue.title;
              editingTitle = !editingTitle;
            }} />
        </div>
      </div>
    {:else}
      <div class="title">
        <InlineTitle content={issue.title} fontSize="medium" />
        <div class="title-icons">
          <Icon name="pen" onclick={() => (editingTitle = !editingTitle)} />
        </div>
      </div>
    {/if}

    <IssueMetadata {issue} />

    <div class="issue-body">
      <CommentComponent
        rid={repo.rid}
        id={issue.id}
        lastEdit={issue.discussion[0].edits.length > 1
          ? issue.discussion[0].edits.at(-1)
          : undefined}
        author={issue.discussion[0].author}
        reactions={issue.discussion[0].reactions}
        timestamp={issue.discussion[0].edits.slice(-1)[0].timestamp}
        body={issue.discussion[0].edits.slice(-1)[0].body}
        editComment={roles.isDelegateOrAuthor(
          config.publicKey,
          repo.delegates.map(delegate => delegate.did),
          issue.discussion[0].author.did,
        ) && partial(editComment, issue.discussion[0].id)}
        reactOnComment={partial(
          reactOnComment,
          config.publicKey,
          issue.discussion[0].id,
        )}>
        <svelte:fragment slot="actions">
          <Icon styleCursor="pointer" name="reply" onclick={toggleReply} />
        </svelte:fragment>
        <svelte:fragment slot="caption">opened</svelte:fragment>
      </CommentComponent>
    </div>
    <div class="connector"></div>

    <div>
      {#each activity as op}
        {#if op.type === "lifecycle"}
          <IssueTimelineLifecycleAction operation={op} />
          <div class="connector"></div>
        {:else if op.type === "comment"}
          {@const thread = threads.find(t => t.root.id === op.entryId)}
          {#if thread}
            <Thread
              {thread}
              rid={repo.rid}
              canEditComment={partial(
                roles.isDelegateOrAuthor,
                config.publicKey,
                repo.delegates.map(delegate => delegate.did),
              )}
              {editComment}
              createReply={partial(createReply)}
              reactOnComment={partial(reactOnComment, config.publicKey)} />
            <div class="connector"></div>
          {/if}
        {/if}
      {/each}
    </div>

    <div id={`reply-${issue.id}`}>
      <CommentToggleInput
        disallowEmptyBody
        rid={repo.rid}
        focus
        onexpand={toggleReply}
        onclose={topLevelReplyOpen
          ? () => (topLevelReplyOpen = false)
          : undefined}
        placeholder="Leave a comment"
        submit={partial(createComment)} />
    </div>
  </div>
</Layout>
