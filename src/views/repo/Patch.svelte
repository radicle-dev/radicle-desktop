<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { Diff } from "@bindings/diff/Diff";
  import type { Embed } from "@bindings/cob/thread/Embed";
  import type { PaginatedQuery } from "@bindings/cob/PaginatedQuery";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { PatchStatus } from "./router";
  import type { RepoInfo } from "@bindings/repo/RepoInfo";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import partial from "lodash/partial";

  import * as roles from "@app/lib/roles";
  import { formatOid, publicKeyFromDid } from "@app/lib/utils";
  import { invoke } from "@app/lib/invoke";
  import { nodeRunning } from "@app/lib/events";

  import { announce } from "@app/components/AnnounceSwitch.svelte";

  import AssigneeInput from "@app/components/AssigneeInput.svelte";
  import Border from "@app/components/Border.svelte";
  import Changeset from "@app/components/Changeset.svelte";
  import CommentComponent from "@app/components/Comment.svelte";
  import CopyableId from "@app/components/CopyableId.svelte";
  import Icon from "@app/components/Icon.svelte";
  import InlineTitle from "@app/components/InlineTitle.svelte";
  import LabelInput from "@app/components/LabelInput.svelte";
  import Layout from "./Layout.svelte";
  import Link from "@app/components/Link.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import PatchStateBadge from "@app/components/PatchStateBadge.svelte";
  import PatchTeaser from "@app/components/PatchTeaser.svelte";
  import Sidebar from "@app/components/Sidebar.svelte";
  import Button from "@app/components/Button.svelte";

  interface Props {
    repo: RepoInfo;
    patch: Patch;
    patches: PaginatedQuery<Patch[]>;
    revisions: Revision[];
    config: Config;
    status: PatchStatus | undefined;
  }

  /* eslint-disable prefer-const */
  let { repo, patch, patches, revisions, config, status }: Props = $props();
  /* eslint-enable prefer-const */

  let cursor = patches.cursor;
  let more = patches.more;
  let items = $state(patches.content);
  let labelSaveInProgress: boolean = $state(false);
  let assigneesSaveInProgress: boolean = $state(false);
  let tab: "patch" | "revisions" = $state("patch");

  $effect(() => {
    items = patches.content;
    cursor = patches.cursor;
    more = patches.more;
  });

  $effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-unused-expressions
    patch.id;

    tab = "patch";
  });

  const project = $derived(repo.payloads["xyz.radicle.project"]!);

  async function loadHighlightedDiff(rid: string, base: string, head: string) {
    return invoke<Diff>("get_diff", {
      rid,
      options: {
        base,
        head,
        unified: 5,
        highlight: true,
      },
    });
  }

  async function loadPatch(rid: string, patchId: string) {
    patch = await invoke<Patch>("patch_by_id", {
      rid: rid,
      id: patchId,
    });
    revisions = await invoke<Revision[]>("revisions_by_patch", {
      rid: rid,
      id: patchId,
    });
  }

  async function loadMoreSecondColumn() {
    if (more) {
      const p = await invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        skip: cursor + 20,
        take: 20,
      });

      cursor = p.cursor;
      more = p.more;
      items = [...items, ...p.content];
    }
  }

  async function saveLabels(labels: string[]) {
    try {
      labelSaveInProgress = true;
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
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
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
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

  async function reload() {
    [config, repo, patches, patch, revisions] = await Promise.all([
      invoke<Config>("config"),
      invoke<RepoInfo>("repo_by_id", {
        rid: repo.rid,
      }),
      invoke<PaginatedQuery<Patch[]>>("list_patches", {
        rid: repo.rid,
        status,
      }),
      invoke<Patch>("patch_by_id", {
        rid: repo.rid,
        id: patch.id,
      }),
      invoke<Revision[]>("revisions_by_patch", {
        rid: repo.rid,
        id: patch.id,
      }),
    ]);
  }

  async function editRevision(
    revisionId: string,
    description: string,
    embeds: Embed[],
  ) {
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "revision.edit",
          revision: revisionId,
          description,
          embeds,
        },
        opts: { announce: $nodeRunning && $announce },
      });
    } catch (error) {
      console.error("Patch revision editing failed: ", error);
    } finally {
      await reload();
    }
  }

  async function reactOnRevision(
    publicKey: string,
    revisionId: string,
    authors: Author[],
    reaction: string,
  ) {
    try {
      await invoke("edit_patch", {
        rid: repo.rid,
        cobId: patch.id,
        action: {
          type: "revision.react",
          revision: revisionId,
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
</script>

<style>
  .title {
    font-size: var(--font-size-medium);
    font-weight: var(--font-weight-medium);
    -webkit-user-select: text;
    user-select: text;
    margin-bottom: 1rem;
  }
  .patch-list {
    margin-top: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding-bottom: 1rem;
  }
  .content {
    padding: 1.5rem 1rem 1rem 0;
  }

  .patch-body {
    margin-top: 1rem;
    position: relative;
  }
  /* We put the background and clip-path in a separate element to prevent
     popovers being clipped in the main element. */
  .patch-body::after {
    position: absolute;
    z-index: -1;
    content: " ";
    background-color: var(--color-background-float);
    clip-path: var(--2px-corner-fill);
    width: 100%;
    height: 100%;
    top: 0;
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

<Layout {loadMoreSecondColumn} publicKey={config.publicKey}>
  {#snippet headerCenter()}
    <CopyableId id={patch.id} />
  {/snippet}

  {#snippet sidebar()}
    <Sidebar activeTab={{ type: "patches", status }} rid={repo.rid} />
  {/snippet}

  {#snippet secondColumn()}
    <div
      style:height="34px"
      class="global-flex txt-medium"
      style:font-weight="var(--font-weight-medium)">
      Patches
    </div>
    <div class="patch-list">
      {#each items as p}
        <PatchTeaser
          compact
          {loadPatch}
          patch={p}
          rid={repo.rid}
          {status}
          selected={patch && p.id === patch.id} />
      {/each}
    </div>
  {/snippet}

  <div class="content">
    <div class="title">
      <InlineTitle content={patch.title} fontSize="medium" />
    </div>

    <div class="breadcrumbs">
      <Link route={{ resource: "home" }}>
        <NodeId
          publicKey={config.publicKey}
          alias={config.alias}
          styleFontFamily="var(--font-family-sans-serif)"
          styleFontSize="var(--font-size-tiny)" />
      </Link>
      <Icon name="chevron-right" />
      <Link
        route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}
        styleColor="var(--color-foreground-dim)">
        {project.data.name}
      </Link>
      <Icon name="chevron-right" />
      <Link
        route={{ resource: "repo.patches", rid: repo.rid, status: "open" }}
        styleColor="var(--color-foreground-dim)">
        Patches
      </Link>
      <Icon name="chevron-right" />
      {patch.title}
    </div>

    <Border variant="ghost" styleGap="0">
      <div class="metadata-section" style:min-width="8rem">
        <div class="metadata-section-title">Status</div>
        <PatchStateBadge state={patch.state} />
      </div>

      <div class="metadata-divider"></div>

      <div class="metadata-section" style:flex="1">
        <LabelInput
          allowedToEdit={!!roles.isDelegateOrAuthor(
            config.publicKey,
            repo.delegates.map(delegate => delegate.did),
            patch.author.did,
          )}
          labels={patch.labels}
          submitInProgress={labelSaveInProgress}
          save={saveLabels} />
      </div>

      <div class="metadata-divider"></div>

      <div class="metadata-section" style:flex="1">
        <AssigneeInput
          allowedToEdit={!!roles.isDelegateOrAuthor(
            config.publicKey,
            repo.delegates.map(delegate => delegate.did),
            patch.author.did,
          )}
          assignees={patch.assignees}
          submitInProgress={assigneesSaveInProgress}
          save={saveAssignees} />
      </div>
    </Border>

    <div class="global-flex" style:gap="0" style:margin-top="1rem">
      <Button
        flatRight
        active={tab === "patch"}
        variant="ghost"
        onclick={() => {
          tab = "patch";
        }}>
        Patch
      </Button>

      <Button
        flatLeft
        variant="ghost"
        active={tab === "revisions"}
        onclick={() => {
          tab = "revisions";
        }}>
        Revision: {formatOid(revisions.slice(-1)[0].id)}
        <span class="global-counter" style:height="22px">latest</span>
      </Button>
    </div>

    {#if tab === "patch"}
      <div class="txt-small patch-body">
        <CommentComponent
          caption="opened"
          rid={repo.rid}
          id={patch.id}
          lastEdit={revisions[0].description.length > 1
            ? revisions[0].description.at(-1)
            : undefined}
          author={revisions[0].author}
          reactions={revisions[0].reactions}
          timestamp={revisions[0].timestamp}
          body={revisions[0].description.slice(-1)[0].body}
          reactOnComment={partial(
            reactOnRevision,
            config.publicKey,
            revisions[0].id,
          )}
          editComment={roles.isDelegateOrAuthor(
            config.publicKey,
            repo.delegates.map(delegate => delegate.did),
            revisions[0].author.did,
          ) && partial(editRevision, revisions[0].id)}>
        </CommentComponent>
      </div>
    {:else}
      {@const revision = revisions.slice(-1)[0]}
      {#await loadHighlightedDiff(repo.rid, revision.base, revision.head) then diff}
        <div style:margin-top="1rem">
          <Changeset {diff} repoId={repo.rid} />
        </div>
      {/await}
    {/if}
  </div>
</Layout>
