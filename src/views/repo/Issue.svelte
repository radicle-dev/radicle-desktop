<script lang="ts">
  import type { IssueStatus } from "./router";
  import type { Author } from "@bindings/cob/Author";
  import type { Action } from "@bindings/cob/issue/Action";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { Operation } from "@bindings/cob/Operation";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { Thread } from "@bindings/cob/thread/Thread";
  import type { Config } from "@bindings/config/Config";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import partial from "lodash/partial";

  import { nodeRunning } from "@app/lib/events";
  import { invoke } from "@app/lib/invoke";
  import { show } from "@app/lib/modal";
  import * as roles from "@app/lib/roles";
  import * as router from "@app/lib/router";
  import {
    explorerUrl,
    issueStatusBackgroundColor,
    issueStatusColor,
    issueStatusLabel,
    publicKeyFromDid,
  } from "@app/lib/utils";

  import { announce } from "@app/components/AnnounceSwitch.svelte";
  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Button from "@app/components/Button.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import Discussion from "@app/components/Discussion.svelte";
  import EditableTitle from "@app/components/EditableTitle.svelte";
  import ExternalLink from "@app/components/ExternalLink.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Id from "@app/components/Id.svelte";
  import IssueStateButton from "@app/components/IssueStateButton.svelte";
  import IssueTimeline from "@app/components/IssueTimeline.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import ScrollArea from "@app/components/ScrollArea.svelte";
  import Topbar from "@app/components/Topbar.svelte";
  import CreateIssueModal from "@app/modals/CreateIssue.svelte";

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

  // Parent reuses this component across issue navigations; a sibling $effect
  // resets local state when issue.id changes.
  // svelte-ignore state_referenced_locally
  let issues = $state(initialIssues);
  // The initial status filter is captured at mount so it stays fixed while
  // navigating between sibling issues.
  // svelte-ignore state_referenced_locally
  const status = initialStatus;
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

    hideTimeline = true;
  });

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
    [issue, activity, threads, issues] = await Promise.all([
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
      invoke<Issue[]>("list_issues", {
        rid: repo.rid,
        status,
      }),
    ]);
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

  async function updateTitle(newTitle: string) {
    try {
      await invoke("edit_issue", {
        rid: repo.rid,
        cobId: issue.id,
        action: {
          type: "edit",
          id: issue.id,
          title: newTitle,
        },
        opts: { announce: $nodeRunning && $announce },
      });
      // Update second column issue title without reloading the whole issue list.
      const issueIndex = issues.findIndex(i => i.id === issue.id);
      if (issueIndex !== -1) {
        issues[issueIndex].title = newTitle;
      }
    } catch (error) {
      console.error("Issue title editing failed: ", error);
    } finally {
      await reload();
    }
  }

  async function reactOnComment(
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
            ({ did }) => publicKeyFromDid(did) === config.publicKey,
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
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
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
    display: grid;
    grid-template-columns: 1fr 22rem;
  }
  .main {
    padding: 1.5rem 2rem;
    min-width: 0;
  }
  .title {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-bottom: 1rem;
  }
  .status-chip {
    padding: 0;
    height: 2rem;
    width: 2rem;
    flex-shrink: 0;
  }
  .issue-body {
    margin: 1rem 0;
    position: relative;
  }
  .sidebar {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    border-left: 1px solid var(--color-border-subtle);
    height: 100%;
    padding: 1.5rem 1rem;
  }
  .sidebar-section {
    padding: 0.5rem;
    font: var(--txt-body-m-regular);
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }
  @media (max-width: 1349.98px) {
    .content {
      grid-template-columns: 1fr;
    }
    .sidebar {
      order: -1;
      border-left: none;
      border-bottom: 1px solid var(--color-border-subtle);
      flex-direction: row;
      align-items: flex-start;
    }
    .sidebar-section {
      flex: 1;
    }
  }
</style>

<Layout>
  <div class="page">
    <Topbar>
      <div class="breadcrumb">
        <Icon name={issue.state.status === "open" ? "issue" : "issue-closed"} />
        <button
          class="breadcrumb-link"
          onclick={() =>
            router.push({
              resource: "repo.issues",
              rid: repo.rid,
              status: issue.state.status,
            })}>
          {issueStatusLabel[issue.state.status]}
        </button>
        <Icon name="chevron-right" />
        <Id id={issue.id} clipboard={issue.id} placement="bottom-start" />
        <ExternalLink
          href={explorerUrl(`${repo.rid}/issues/${issue.id}`)}
          title="Open in radicle.network" />
      </div>
      <div style:margin-left="auto">
        <Button
          styleHeight="2rem"
          variant="ghost"
          onclick={() =>
            show({
              component: CreateIssueModal,
              props: { repo },
            })}>
          <Icon name="plus" />New issue
        </Button>
      </div>
    </Topbar>

    <ScrollArea style="flex: 1; min-height: 0;">
      <div class="content">
        <div class="main">
          <div class="title">
            <div
              class="global-chip status-chip"
              style:color={issueStatusColor[issue.state.status]}
              style:background-color={issueStatusBackgroundColor[
                issue.state.status
              ]}>
              <Icon
                name={issue.state.status === "open"
                  ? "issue"
                  : "issue-closed"} />
            </div>
            <EditableTitle
              {updateTitle}
              allowedToEdit={roles.isDelegateOrAuthor(
                config.publicKey,
                repo.delegates.map(delegate => delegate.did),
                issue.body.author.did,
              )}
              title={issue.title}
              cobId={issue.id} />
          </div>

          <div class="issue-body">
            <CommentComponent
              rid={repo.rid}
              currentUserNid={config.publicKey}
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
              reactOnComment={partial(reactOnComment, issue.body.id)}>
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

          <div class="global-flex" style:margin-top="1rem">
            <Button
              variant="naked"
              onclick={() => (hideTimeline = !hideTimeline)}>
              <Icon name={hideTimeline ? "chevron-right" : "chevron-down"} />
            </Button>
            <div class="txt-body-m-regular global-flex">Timeline</div>
          </div>
          <div
            style:display={hideTimeline ? "none" : "revert"}
            style:margin-top="1rem">
            <IssueTimeline {activity} />
          </div>
        </div>

        <div class="sidebar">
          <div class="sidebar-section">
            <IssueStateButton
              selectedState={issue.state}
              onSelect={saveState}
              disabled={!roles.isDelegate(
                config.publicKey,
                repo.delegates.map(d => d.did),
              )} />
          </div>
          <div class="sidebar-section">
            <LabelInput
              allowedToEdit={!!roles.isDelegate(
                config.publicKey,
                repo.delegates.map(delegate => delegate.did),
              )}
              labels={issue.labels}
              submitInProgress={labelSaveInProgress}
              save={saveLabels} />
          </div>
          <div class="sidebar-section">
            <AssigneeInput
              allowedToEdit={!!roles.isDelegate(
                config.publicKey,
                repo.delegates.map(delegate => delegate.did),
              )}
              assignees={issue.assignees}
              submitInProgress={assigneesSaveInProgress}
              save={saveAssignees} />
          </div>
        </div>
      </div>
    </ScrollArea>
  </div>
</Layout>
