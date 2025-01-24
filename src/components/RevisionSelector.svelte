<script lang="ts">
  import type { Patch } from "@bindings/cob/patch/Patch";
  import type { Revision } from "@bindings/cob/patch/Revision";

  import uniqBy from "lodash/uniqBy";
  import orderBy from "lodash/orderBy";

  import { authorForNodeId, formatOid } from "@app/lib/utils";

  import Border from "./Border.svelte";
  import DropdownList from "./DropdownList.svelte";
  import DropdownListItem from "./DropdownListItem.svelte";
  import Icon from "./Icon.svelte";
  import NakedButton from "./NakedButton.svelte";
  import NodeId from "./NodeId.svelte";
  import Popover, { closeFocused } from "./Popover.svelte";
  import RevisionBadges from "./RevisionBadges.svelte";

  interface Props {
    patch: Patch;
    revisions: Revision[];
    selectedRevision: Revision;
    selectRevision: (revision: Revision) => void;
  }

  /* eslint-disable prefer-const */
  let { patch, revisions, selectedRevision, selectRevision }: Props = $props();
  /* eslint-enable prefer-const */

  const revisionAuthors = $derived(
    orderBy(
      uniqBy(
        revisions.map(r => {
          return r.author;
        }),
        "did",
      ),
      [
        o => {
          return o.did === patch.author.did;
        },
      ],
      ["desc"],
    ),
  );
</script>

<style>
  .dropdown-group:not(:last-of-type) {
    margin-bottom: 1rem;
  }
  .icon {
    min-width: 16px;
  }
</style>

<Popover popoverPadding="0" popoverPositionTop="37px" popoverPositionLeft="0">
  {#snippet toggle(onclick)}
    <NakedButton
      variant="ghost"
      onclick={(e: MouseEvent) => {
        e.stopPropagation();
        e.preventDefault();
        onclick();
      }}
      stylePadding="0 4px">
      <div style:color="var(--color-foreground-contrast)">
        <Icon name="chevron-down" />
      </div>
    </NakedButton>
  {/snippet}

  {#snippet popover()}
    <Border variant="ghost">
      <div style:max-width="20rem" style:padding-top="0.5rem">
        {#each revisionAuthors as author}
          <div class="dropdown-group">
            <div style:padding-left="0.5rem" style:padding-bottom="0.5rem">
              <NodeId {...authorForNodeId(author)} />
            </div>
            <DropdownList
              items={orderBy(
                revisions.filter(r => {
                  return r.author.did === author.did;
                }),
                "timestamp",
                ["desc"],
              )}>
              {#snippet item(revision)}
                <DropdownListItem
                  selected={revision.id === selectedRevision.id}
                  onclick={() => {
                    closeFocused();
                    selectRevision(revision);
                  }}>
                  <div class="global-flex txt-overflow">
                    <div class="icon">
                      {#if patch.state.status === "merged" && patch.state.revision === revision.id}
                        <div style:color="var(--color-fill-primary)">
                          <Icon name="patch-merged" />
                        </div>
                      {:else if revision.reviews && revision.reviews.length > 0 && revision.reviews.every( r => {
                            return r.verdict === "accept";
                          }, )}
                        <div style:color="var(--color-fill-success)">
                          <Icon name="comment-checkmark" />
                        </div>
                      {:else if revision.reviews && revision.reviews.length > 0 && revision.reviews.every( r => {
                            return r.verdict === "reject";
                          }, )}
                        <div style:color="var(--color-foreground-red)">
                          <Icon name="comment-cross" />
                        </div>
                      {:else if revision.reviews && revision.reviews.length}
                        <div style:color="var(--color-foreground-dim)">
                          <Icon name="none" />
                        </div>
                      {/if}
                    </div>
                    <span class="global-oid">
                      {formatOid(revision.id)}
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
                  </div>
                </DropdownListItem>
              {/snippet}
            </DropdownList>
          </div>
        {/each}
      </div>
    </Border>
  {/snippet}
</Popover>
