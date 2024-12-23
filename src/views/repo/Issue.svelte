<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { Operation } from "@bindings/cob/issue/Operation";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { IssueStatus } from "./router";

  import partial from "lodash/partial";
  import { tick } from "svelte";

  import * as roles from "@app/lib/roles";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import { publicKeyFromDid, scrollIntoView } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";

  import Border from "@app/components/Border.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import CommentToggleInput from "@app/components/CommentToggleInput.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import IssueSecondColumn from "@app/components/IssueSecondColumn.svelte";
  import IssueStateBadge from "@app/components/IssueStateBadge.svelte";
  import IssueStateButton from "@app/components/IssueStateButton.svelte";
  import IssueTimelineLifecycleAction from "@app/components/IssueTimelineLifecycleAction.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import ThreadComponent from "@app/components/Thread.svelte";

  import Layout from "./Layout.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import AssigneeInput from "@app/components/AssigneeInput.svelte";

  interface Props {
    repo: RepoInfo;
    issue: Issue;
    issues: Issue[];
    activity: Operation[];
    config: Config;
    threads: Thread[];
    status: IssueStatus;
  }

  /* eslint-disable prefer-const */
  let {
    repo,
    issue,
    issues: initialIssues,
    activity,
    config,
    threads,
    status,
  }: Props = $props();
  /* eslint-enable prefer-const */

  const issues = $state(initialIssues);

  let topLevelReplyOpen = $state(false);
  let editingTitle = $state(false);
  let updatedTitle = $state("");
  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);
  let focusReply: boolean = $state(false);

  $effect(() => {
    // The component doesn't get destroyed when we switch between different
    // issues in the second column and because of that the top-level state
    // gets retained when the issue changes. This reactive statement makes
    // sure we always reset the state to defaults.

    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    issue.id;

    topLevelReplyOpen = false;
    editingTitle = false;
    updatedTitle = issue.title;
    focusReply = false;
  });

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  async function saveLabels(labels: string[]) {
    try {
      labelSaveInProgress = true;
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: {
          type: "label",
          labels,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing labels failed", error);
    } finally {
      labelSaveInProgress = false;
      await reload();
    }
  }

  async function saveAssignees(assignees: Author[]) {
    try {
      assigneesSaveInProgress = true;
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: {
          type: "assign",
          assignees: assignees.map(a => a.did),
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing assignees failed", error);
    } finally {
      assigneesSaveInProgress = false;
      await reload();
    }
  }

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
    focusReply = true;
  }

  async function reload() {
    [issue, activity, threads] = await Promise.all([
      invoke<Issue>("issue_by_id", {
        rid: repo.rid,
        id: issue.id,
      }),
      invoke<Operation[]>("activity_by_id", {
        rid: repo.rid,
        typeName: "xyz.radicle.issue",
        id: issue.id,
      }),
      invoke<Thread[]>("comment_threads_by_issue_id", {
        rid: repo.rid,
        id: issue.id,
      }),
    ]);

    topLevelReplyOpen = false;
    editingTitle = false;
  }

  async function createComment(body: string, embeds: Embed[]) {
    try {
      await invoke("create_issue_comment", {
        rid: repo.rid,
        new: { id: issue.id, body, embeds },
        opts: { announce: $nodeRunning && $announce },
      });
      // Update second column issue comment count without reloading the whole
      // issue list.
      const issueIndex = issues.findIndex(i => i.id === issue.id);
      if (issueIndex !== -1) {
        issues[issueIndex].commentCount += 1;
      }
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
        opts: { announce: $nodeRunning && $announce },
      });
      // Update second column issue comment count without reloading the whole
      // issue list.
      const issueIndex = issues.findIndex(i => i.id === issue.id);
      if (issueIndex !== -1) {
        issues[issueIndex].commentCount += 1;
      }
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
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Issue comment editing failed: ", error);
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
        opts: { announce: $nodeRunning && $announce },
      });
      // Update second column issue title without reloading the whole issue list.
      const issueIndex = issues.findIndex(i => i.id === issue.id);
      if (issueIndex !== -1) {
        issues[issueIndex].title = updatedTitle;
      }
      editingTitle = false;
    } catch (error) {
      console.error("Issue editing failed: ", error);
    } finally {
      await reload();
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
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Editing reactions failed", error);
    } finally {
      await reload();
    }
  }

  async function saveState(state: Issue["state"]) {
    try {
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: {
          type: "lifecycle",
          state,
        },
        opts: { announce: $nodeRunning && $announce },
      });
      // Update second column issue icon without reloading the whole issue list.
      const issueIndex = issues.findIndex(i => i.id === issue.id);
      if (issueIndex !== -1) {
        issues[issueIndex].state = state;
      }
    } catch (error) {
      console.error("Editing reactions failed", error);
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
    display: flex;
    align-items: center;
    justify-content: space-between;
    word-break: break-all;
    padding-top: 4px;
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
  .content {
    padding: 1rem 1rem 1rem 0;
  }
  .connector {
    width: 2px;
    height: 1rem;
    margin-left: 1.25rem;
    background-color: var(--color-background-float);
  }
  .title-icons {
    display: flex;
    gap: 1rem;
    margin-left: 1rem;
    align-items: center;
    height: 40px;
  }
  .metadata-divider {
    width: 2px;
    background-color: var(--color-fill-ghost);
    height: calc(100% + 4px);
    top: 0;
    position: relative;
  }
  .metadata-section {
    padding: 0.5rem;
    font-size: var(--font-size-small);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    height: 100%;
  }
  .metadata-section-title {
    margin-bottom: 0.5rem;
    color: var(--color-foreground-dim);
  }
  .breadcrumbs {
    display: flex;
    gap: 0.5rem;
    font-size: var(--font-size-tiny);
    font-weight: var(--font-weight-semibold);
    align-items: center;
    min-height: 1.5rem;
    width: 100%;
    margin-bottom: 1rem;
    color: var(--color-foreground-dim);
  }
</style>

<Layout publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={issue.id} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab={{ type: "issues", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <IssueSecondColumn {repo} selectedIssueId={issue.id} {issues} {status} />
  {/snippet}

  <div class="content">
    <div style:margin-bottom="1rem">
      {#if editingTitle}
        <div class="title">
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
            <IssueStateButton issueState={issue.state} save={saveState} />
          </div>
        </div>
      {:else}
        <div class="title">
          <InlineTitle content={issue.title} fontSize="medium" />
          {#if roles.isDelegateOrAuthor( config.publicKey, repo.delegates.map(delegate => delegate.did), issue.body.author.did, )}
            <div class="title-icons">
              <Icon name="pen" onclick={() => (editingTitle = !editingTitle)} />
              <IssueStateButton issueState={issue.state} save={saveState} />
            </div>
          {/if}
        </div>
      {/if}
    </div>

    <div class="breadcrumbs txt-overflow">
      <Link route={{ resource: "home" }}>
        <NodeId
          publicKey={config.publicKey}
          alias={config.alias}
          styleFontFamily="var(--font-family-sans-serif)"
          styleFontSize="var(--font-size-tiny)" />
      </Link>
      <Icon name="chevron-right" />
      <Link
        route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}
        styleColor="var(--color-foreground-dim)">
        {project.data.name}
      </Link>
      <Icon name="chevron-right" />
      <Link
        route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}
        styleColor="var(--color-foreground-dim)">
        Issues
      </Link>
      <Icon name="chevron-right" />
      <div class="txt-overflow">
        {issue.title}
      </div>
    </div>

    <Border variant="ghost" styleGap="0">
      <div class="metadata-section" style:min-width="8rem">
        <div class="metadata-section-title">Status</div>
        <IssueStateBadge state={issue.state} />
      </div>

      <div class="metadata-divider"></div>

      <div class="metadata-section" style:flex="1">
        <LabelInput
          allowedToEdit={!!roles.isDelegateOrAuthor(
            config.publicKey,
            repo.delegates.map(delegate => delegate.did),
            issue.body.author.did,
          )}
          labels={issue.labels}
          submitInProgress={labelSaveInProgress}
          save={saveLabels} />
      </div>

      <div class="metadata-divider"></div>

      <div class="metadata-section" style:flex="1">
        <AssigneeInput
          allowedToEdit={!!roles.isDelegateOrAuthor(
            config.publicKey,
            repo.delegates.map(delegate => delegate.did),
            issue.body.author.did,
          )}
          assignees={issue.assignees}
          submitInProgress={assigneesSaveInProgress}
          save={saveAssignees} />
      </div>
    </Border>

    <div class="issue-body">
      <CommentComponent
        rid={repo.rid}
        id={issue.id}
        lastEdit={issue.body.edits.length > 1
          ? issue.body.edits.at(-1)
          : undefined}
        author={issue.body.author}
        caption="opened"
        reactions={issue.body.reactions}
        timestamp={issue.body.edits.slice(-1)[0].timestamp}
        body={issue.body.edits.slice(-1)[0].body}
        editComment={roles.isDelegateOrAuthor(
          config.publicKey,
          repo.delegates.map(delegate => delegate.did),
          issue.body.author.did,
        ) && partial(editComment, issue.body.id)}
        reactOnComment={partial(
          reactOnComment,
          config.publicKey,
          issue.body.id,
        )}>
        {#snippet actions()}
          <Icon name="reply" onclick={toggleReply} />
        {/snippet}
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
            <ThreadComponent
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
        focus={focusReply}
        onexpand={toggleReply}
        onclose={topLevelReplyOpen
          ? () => (topLevelReplyOpen = false)
          : undefined}
        placeholder="Leave a comment"
        submit={partial(createComment)} />
    </div>
  </div>
</Layout>
