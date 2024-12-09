<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { invoke } from "@app/lib/invoke";

  import * as router from "@app/lib/router";
  import { nodeRunning } from "@app/lib/events";

  import { announce } from "@app/components/AnnounceSwitch.svelte";

  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import IssueSecondColumn from "@app/components/IssueSecondColumn.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import Link from "@app/components/Link.svelte";
  import Markdown from "@app/components/Markdown.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import Textarea from "@app/components/Textarea.svelte";

  import Layout from "./Layout.svelte";

  interface Props {
    repo: RepoInfo;
    issues: Issue[];
    config: Config;
  }

  const { repo, issues, config }: Props = $props();

  let description: string = $state("");
  let preview: boolean = $state(false);
  let title: string = $state("");

  const embeds: { name: string; content: string }[] = [];

  let assignees: Author[] = $state([]);
  let labels: string[] = $state([]);

  async function createIssue() {
    const response: Issue = await invoke("create_issue", {
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
    void router.push({
      resource: "repo.issue",
      rid: repo.rid,
      issue: response.id,
    });
  }

  const project = $derived(repo.payloads["xyz.radicle.project"]!);
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
    padding: 0 1rem 1rem 1rem;
    height: calc(100% - 14rem);
  }
  .body {
    background-color: var(--color-background-float);
    padding: 1rem;
    min-height: calc(100% + 2px);
    clip-path: var(--2px-corner-fill);
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

<Layout>
  {#snippet breadcrumbs()}
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
    <Link route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
      Issues
    </Link>
    <Icon name="chevron-right" />
    New Issue
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab="issues" rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <IssueSecondColumn {repo} {issues} />
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

    {#if preview}
      <div class="txt-small body">
        {#if description.trim() === ""}
          <span class="txt-missing" style:line-height="1.625rem">
            No description.
          </span>
        {:else}
          <Markdown rid={repo.rid} content={description} breaks />
        {/if}
      </div>
    {:else}
      <Textarea
        borderVariant="ghost"
        placeholder="Description"
        bind:value={description}
        size="fixed-height"
        submit={createIssue}
        styleMinHeight="100%" />
    {/if}
    <div
      class="global-flex"
      style:justify-content="space-between"
      style:padding-bottom="1.5rem"
      style:margin-top="1.5rem">
      <OutlineButton
        variant="ghost"
        onclick={() => {
          window.history.back();
        }}>
        <Icon name="cross" />Discard
      </OutlineButton>
      <div class="global-flex">
        <div class="global-flex txt-small txt-missing">
          <Icon name="markdown" />
          Markdown is supported.
        </div>
        <OutlineButton
          variant="ghost"
          disabled={title.length === 0}
          onclick={() => (preview = !preview)}>
          <Icon name={preview ? "pen" : "eye"} />{preview ? "Edit" : "Preview"}
        </OutlineButton>
        <Button
          variant="ghost"
          disabled={title.length === 0}
          onclick={createIssue}>
          <Icon name="checkmark" />Save
        </Button>
      </div>
    </div>
  </div>
</Layout>
