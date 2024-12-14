<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { IssueStatus } from "./router";
  import type { Embed } from "@bindings/cob/thread/Embed";

  import { invoke } from "@app/lib/invoke";

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

  const { repo, issues, config, status }: Props = $props();

  let preview: boolean = $state(false);
  let title: string = $state("");

  let assignees: Author[] = $state([]);
  let labels: string[] = $state([]);

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
    height: 100%;
    padding: 0 1rem 1rem 1rem;
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
    <IssueSecondColumn {repo} {issues} {status} />
  {/snippet}

  <div class="content">
    {#if preview}
      <div class="title">
        <InlineTitle content={title} fontSize="medium" />
      </div>
    {:else}
      <div style:margin-bottom="0.35rem">
        <TextInput placeholder="Title" autofocus bind:value={title} />
      </div>
    {/if}

    <div style:margin-bottom="0.35rem">
      <Border variant="ghost" styleGap="0">
        <div class="metadata-section" style:flex="1">
          <LabelInput
            allowedToEdit={true}
            {labels}
            submitInProgress={false}
            save={newLabels => {
              labels = newLabels;
            }} />
        </div>

        <div class="metadata-divider"></div>

        <div class="metadata-section" style:flex="1">
          <AssigneeInput
            allowedToEdit={true}
            bind:assignees
            submitInProgress={false}
            save={newAssignees => {
              assignees = newAssignees;
            }} />
        </div>
      </Border>
    </div>

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
