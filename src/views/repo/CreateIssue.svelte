<script lang="ts">
  import type { Author } from "@bindings/Author";
  import type { Config } from "@bindings/Config";
  import type { Issue } from "@bindings/Issue";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import { invoke } from "@tauri-apps/api/core";

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

  export let repo: RepoInfo;
  export let issues: Issue[];
  export let config: Config;

  let title: string = "";
  let description: string = "";

  const labels: string[] = [];
  const assignees: Author[] = [];
  const embeds: { name: string; content: string }[] = [];

  async function createIssue() {
    const response: Issue = await invoke("create_issue", {
      rid: repo.rid,
      new: { title, description, labels, assignees, embeds },
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
    margin-bottom: 1rem;
    margin-top: 0.35rem;
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
</style>

<Layout>
  <svelte:fragment slot="breadcrumbs">
    <Link route={{ resource: "home" }}>
      <NodeId
        nodeId={config.publicKey}
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
    <div class="title">
      <TextInput placeholder="Title" autofocus bind:value={title} />
    </div>
    <Textarea placeholder="Description" bind:value={description} />
    <div
      class="global-flex"
      style:justify-content="flex-end"
      style:margin-top="1.5rem">
      <OutlineButton
        variant="ghost"
        onclick={() => {
          window.history.back();
        }}>
        Cancel
      </OutlineButton>
      <Button
        variant="ghost"
        disabled={title.length === 0}
        onclick={createIssue}>
        Save
      </Button>
    </div>
  </div>
</Layout>
