<script lang="ts">
  import type { Action } from "@bindings/cob/issue/Action";
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { IssueStatus } from "./router";
  import type { Operation } from "@bindings/cob/Operation";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Thread } from "@bindings/cob/thread/Thread";

  import partial from "lodash/partial";

  import * as roles from "@app/lib/roles";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";
  import {
    issueStatusBackgroundColor,
    issueStatusColor,
    publicKeyFromDid,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";

  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Border from "@app/components/Border.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Discussion from "@app/components/Discussion.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import IssueSecondColumn from "@app/components/IssueSecondColumn.svelte";
  import IssueStateBadge from "@app/components/IssueStateBadge.svelte";
  import IssueStateButton from "@app/components/IssueStateButton.svelte";
  import IssueTimeline from "@app/components/IssueTimeline.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import NakedButton from "@app/components/NakedButton.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    issue: Issue;
    issues: Issue[];
    activity: Operation<Action>[];
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
    status: initialStatus,
  }: Props = $props();
  /* eslint-enable prefer-const */

  let issues = $state(initialIssues);
  let status = $state(initialStatus);
  let editingTitle = $state(false);
  let updatedTitle = $state("");
  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);
  let hideTimeline = $state(true);

  $effect(() => {
    // The component doesn't get destroyed when we switch between different
    // issues in the second column and because of that the top-level state
    // gets retained when the issue changes. This reactive statement makes
    // sure we always reset the state to defaults.

    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    issue.id;

    editingTitle = false;
    updatedTitle = issue.title;
    hideTimeline = true;
  });

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  async function loadIssues(filter: IssueStatus) {
    try {
      issues = await invoke<Issue[]>("list_issues", {
        rid: repo.rid,
        status: filter,
      });
      status = filter;
    } catch (error) {
      console.error("Loading issue list failed", error);
    }
  }

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
          assignees,
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

  async function reload() {
    [issue, activity, threads] = await Promise.all([
      invoke<Issue>("issue_by_id", {
        rid: repo.rid,
        id: issue.id,
      }),
      invoke<Operation<Action>[]>("activity_by_issue", {
        rid: repo.rid,
        id: issue.id,
      }),
      invoke<Thread[]>("comment_threads_by_issue_id", {
        rid: repo.rid,
        id: issue.id,
      }),
    ]);

    editingTitle = false;
  }

  async function createComment(
    body: string,
    embeds: Embed[],
    replyTo?: string,
  ) {
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
      console.error("Comment creation failed: ", error);
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
      console.error("Issue title editing failed: ", error);
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
      console.error("Changing issue state failed", error);
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
    word-break: break-word;
    min-height: 40px;
  }
  .status {
    padding: 0;
    margin-right: 0.75rem;
    height: 2rem;
    width: 2rem;
  }
  .issue-body {
    margin: 1rem 0;
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
  .title-icons {
    display: flex;
    gap: 1rem;
    margin-left: 1rem;
    align-items: center;
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
  .hide {
    display: none;
  }
</style>

<Layout publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={issue.id} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab="issues" rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <IssueSecondColumn
      {repo}
      selectedIssueId={issue.id}
      {issues}
      {status}
      changeFilter={async filter => {
        await loadIssues(filter);
      }}
      title={project.data.name} />
  {/snippet}

  <div class="content">
    <div style:margin-bottom="0.5rem">
      {#if editingTitle}
        <div class="title">
          <div
            class="global-counter status"
            style:color={issueStatusColor[issue.state.status]}
            style:background-color={issueStatusBackgroundColor[
              issue.state.status
            ]}>
            {#if issue.state.status === "open"}
              <Icon name="issue" />
            {:else}
              <Icon name="issue-closed" />
            {/if}
          </div>
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
          <div class="global-flex" style:gap="0">
            <div
              class="global-counter status"
              style:color={issueStatusColor[issue.state.status]}
              style:background-color={issueStatusBackgroundColor[
                issue.state.status
              ]}>
              {#if issue.state.status === "open"}
                <Icon name="issue" />
              {:else}
                <Icon name="issue-closed" />
              {/if}
            </div>
            <InlineTitle content={issue.title} fontSize="medium" />
          </div>
          {#if roles.isDelegateOrAuthor( config.publicKey, repo.delegates.map(delegate => delegate.did), issue.body.author.did, )}
            <div class="title-icons">
              <Icon name="pen" onclick={() => (editingTitle = !editingTitle)} />
              <IssueStateButton issueState={issue.state} save={saveState} />
            </div>
          {/if}
        </div>
      {/if}
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
      </CommentComponent>
    </div>

    <Discussion
      cobId={issue.id}
      commentThreads={threads}
      {config}
      {createComment}
      {editComment}
      {reactOnComment}
      repoDelegates={repo.delegates}
      rid={repo.rid} />

    <div class="global-flex">
      <NakedButton
        variant="ghost"
        onclick={() => (hideTimeline = !hideTimeline)}>
        <Icon name={hideTimeline ? "chevron-right" : "chevron-down"} />
        <div class="txt-semibold global-flex txt-regular">Timeline</div>
      </NakedButton>
    </div>
    <div class:hide={hideTimeline} style:margin-top="1rem">
      <IssueTimeline {activity} />
    </div>
  </div>
</Layout>
