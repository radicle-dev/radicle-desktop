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
</style>

<Popover popoverPadding="0" popoverPositionTop="2.5rem" popoverPositionLeft="0">
  {#snippet toggle(onclick)}
    <NakedButton variant="ghost" {onclick}>
      <Icon name="chevron-down" />
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
                    <span class="global-oid">
                      {formatOid(revision.id)}
                    </span>
                    <RevisionBadges {revision} {revisions} />
                    <span class="txt-overflow">
                      {revision.description[0].body}
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
