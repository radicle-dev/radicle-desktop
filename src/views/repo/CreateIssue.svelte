<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Issue } from "@bindings/cob/issue/Issue";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";

  import { invoke } from "@app/lib/invoke";

  import { issueStatusColor } from "@app/lib/utils";
  import * as router from "@app/lib/router";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import Layout from "./Layout.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import OutlineButton from "@app/components/OutlineButton.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import Textarea from "@app/components/Textarea.svelte";
  import Markdown from "@app/components/Markdown.svelte";

  export let repo: RepoInfo;
  export let issues: Issue[];
  export let config: Config;

  let title: string = "";
  let description: string = "";
  let preview: boolean = false;
  const announce = false;

  const labels: string[] = [];
  const assignees: Author[] = [];
  const embeds: { name: string; content: string }[] = [];

  async function createIssue() {
    const response: Issue = await invoke("create_issue", {
      rid: repo.rid,
      new: { title, description, labels, assignees, embeds },
      opts: { announce },
    });
    void router.push({
      resource: "repo.issue",
      rid: repo.rid,
      issue: response.id,
    });
  }

  $: project = repo.payloads["xyz.radicle.project"]!;
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
    height: calc(100% - 8rem);
  }
  .body {
    background-color: var(--color-background-float);
    padding: 1rem;
    min-height: calc(100% + 2px);
    clip-path: var(--2px-corner-fill);
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
    <Link route={{ resource: "repo.issues", rid: repo.rid, status: "open" }}>
      Issues
    </Link>
    <Icon name="chevron-right" />
    New Issue
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
    {#if preview}
      <div class="title">
        <InlineTitle content={title} fontSize="medium" />
      </div>
    {:else}
      <div style:margin-bottom="0.35rem">
        <TextInput placeholder="Title" autofocus bind:value={title} />
      </div>
    {/if}
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
        placeholder="Description"
        bind:value={description}
        size="fixed-height"
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
