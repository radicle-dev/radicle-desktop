<script lang="ts">
  import type { PatchStatus } from "@app/views/repo/router";
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Revision } from "@bindings/cob/patch/Revision";
  import type { Config } from "@bindings/config/Config";

  import orderBy from "lodash/orderBy";
  import uniqBy from "lodash/uniqBy";

  import { authorForNodeId } from "@app/lib/utils";

  import DropdownListItem from "@app/components/DropdownListItem.svelte";
  import Icon from "@app/components/Icon.svelte";
  import NodeId from "@app/components/NodeId.svelte";
  import RevisionBadges from "@app/components/RevisionBadges.svelte";
  import RevisionReviews from "@app/components/RevisionReviews.svelte";

  interface Props {
    config: Config;
    patch: Patch;
    revisions: Revision[];
    rid: string;
    selectRevision: (revision: Revision) => void;
    selectedRevision: Revision;
    status: PatchStatus | undefined;
  }

  const {
    config,
    patch,
    revisions,
    rid,
    selectRevision,
    selectedRevision,
    status,
  }: Props = $props();

  const revisionAuthors = $derived(
    orderBy(
      uniqBy(
        revisions.map(r => r.author),
        "did",
      ),
      [o => o.did === patch.author.did],
      ["desc"],
    ),
  );
</script>

<style>
  .author-revisions:not(:last-of-type) {
    margin-bottom: 1.5rem;
  }
</style>

{#each revisionAuthors as author}
  <div class="author-revisions">
    <div style:padding-bottom="0.5rem">
      <span class="global-flex txt-small" style:gap="0">
        <NodeId
          {...authorForNodeId(author)}
          styleFontWeight="var(--font-weight-regular)" />'s revisions
      </span>
    </div>
    {#each orderBy( revisions.filter(r => {
        return r.author.did === author.did;
      }), "timestamp", ["asc"], ) as revision}
      <div style:margin="0.5rem 0">
        <DropdownListItem
          selected={revision.id === selectedRevision.id}
          onclick={() => {
            selectRevision(revision);
          }}>
          <div class="global-flex txt-overflow" style:width="100%">
            {#if patch.state.status === "merged" && patch.state.revision === revision.id}
              <div style:color="var(--color-fill-primary)">
                <Icon name="patch-merged" />
              </div>
            {:else}
              <Icon name="revision" />
            {/if}
            <span class="global-oid">
              {revision.id.substring(0, 4)}
            </span>
            <RevisionBadges {revision} {revisions} />
            <span class="txt-overflow">
              {#if revision.description[0].body.trim()}
                {revision.description[0].body}
              {:else}
                <span
                  class="txt-missing"
                  style:font-weight="var(--font-weight-regular)">
                  No description.
                </span>
              {/if}
            </span>
            <div class="global-flex" style:margin-left="auto">
              {#if revision.discussion && revision.discussion.length > 0}
                <div
                  class="global-flex"
                  style:font-weight="var(--font-weight-regular)"
                  style:gap="0.25rem">
                  <Icon name="comment" />{revision.discussion.length}
                </div>
              {/if}
            </div>
          </div>
        </DropdownListItem>
      </div>
      <RevisionReviews {config} {rid} {status} {revision} {patch} />
    {/each}
  </div>
{/each}
