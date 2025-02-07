<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { IssueStatus } from "./router";
  import type { Embed } from "@bindings/cob/thread/Embed";

  import { invoke } from "@app/lib/invoke";

  import * as roles from "@app/lib/roles";
  import * as router from "@app/lib/router";
  import { nodeRunning } from "@app/lib/events";

  import { announce } from "@app/components/AnnounceSwitch.svelte";

  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Border from "@app/components/Border.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import IssueSecondColumn from "@app/components/IssueSecondColumn.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import TextInput from "@app/components/TextInput.svelte";

  import Layout from "./Layout.svelte";
  import ExtendedTextarea from "@app/components/ExtendedTextarea.svelte";

  interface Props {
    repo: RepoInfo;
    issues: Issue[];
    config: Config;
    status: IssueStatus;
  }

  const {
    repo,
    issues: initialIssues,
    config,
    status: initialStatus,
  }: Props = $props();

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  let preview: boolean = $state(false);
  let title: string = $state("");
  let status = $state(initialStatus);
  let issues = $state(initialIssues);

  let assignees: Author[] = $state([]);
  let labels: string[] = $state([]);

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

  async function createIssue(
    description: string,
    embeds: Embed[],
  ): Promise<Issue> {
    return invoke("create_issue", {
      rid: repo.rid,
      new: {
        title,
        description,
        labels: $state.snapshot(labels),
        assignees: $state.snapshot(assignees.map(a => a.did)),
        embeds,
      },
      opts: { announce: $nodeRunning && $announce },
    });
  }
</script>

<style>
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    margin-top: 0.35rem;
    margin-bottom: 1rem;
  }
  .content {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    padding: 1rem 1rem 1rem 0;
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
</style>

<Layout publicKey={config.publicKey}>
  {#snippet sidebar()}
    <Sidebar activeTab={{ type: "issues", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <IssueSecondColumn
      {repo}
      {issues}
      {status}
      title={project.data.name}
      changeFilter={async filter => {
        await loadIssues(filter);
      }} />
  {/snippet}

  <div class="content">
    {#if preview}
      <div class="title">
        {#if title.trim().length === 0}
          <span class="txt-missing">No title</span>
        {:else}
          <InlineTitle content={title} fontSize="medium" />
        {/if}
      </div>
    {:else}
      <div style:margin-bottom="1rem">
        <TextInput placeholder="Title" autofocus bind:value={title} />
      </div>
    {/if}

    {#if !!roles.isDelegate( config.publicKey, repo.delegates.map(delegate => delegate.did), )}
      <div style:margin-bottom="1rem">
        <Border variant="ghost" styleGap="0">
          <div class="metadata-section" style:flex="1">
            <LabelInput
              allowedToEdit={!!roles.isDelegate(
                config.publicKey,
                repo.delegates.map(delegate => delegate.did),
              )}
              {labels}
              submitInProgress={false}
              save={newLabels => {
                labels = newLabels;
              }} />
          </div>

          <div class="metadata-divider"></div>

          <div class="metadata-section" style:flex="1">
            <AssigneeInput
              allowedToEdit={!!roles.isDelegate(
                config.publicKey,
                repo.delegates.map(delegate => delegate.did),
              )}
              bind:assignees
              submitInProgress={false}
              save={newAssignees => {
                assignees = newAssignees;
              }} />
          </div>
        </Border>
      </div>
    {/if}

    <ExtendedTextarea
      textAreaSize="fixed-height"
      disableSubmit={title.trim() === ""}
      disallowEmptyBody
      submitCaption="Save"
      close={() => window.history.back()}
      submit={async ({ comment, embeds }) => {
        try {
          const response = await createIssue(
            comment,
            Array.from(embeds.values()),
          );
          void router.push({
            resource: "repo.issue",
            rid: repo.rid,
            issue: response.id,
            status,
          });
        } catch {
          console.error("Not able to create issue.");
        }
      }}
      rid={repo.rid}
      bind:preview
      borderVariant="ghost"
      placeholder="Description" />
  </div>
</Layout>
