<script lang="ts">
  import type { Commit } from "@bindings/repo/Commit";
  import type { Snippet } from "svelte";

  import { writeToClipboard } from "@app/lib/invoke";
  import * as utils from "@app/lib/utils";

  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";

  interface Props {
    children: Snippet;
    commit: Commit;
  }

  const { children, commit }: Props = $props();

  // Which address was last copied, so the two rows keep independent feedback.
  let copied: string | undefined = $state();

  // Copies the git identity as it appears in a commit trailer, so the result
  // can be pasted straight into `Co-authored-by:` and the like.
  // Co-authors are not part of the commit's git identity fields; they are a
  // convention carried in the message trailers.
  const coAuthors = $derived(
    utils
      .coAuthors(commit.message)
      .filter(
        who =>
          who.email !== commit.author.email &&
          who.email !== commit.committer.email,
      ),
  );

  // Everyone credited on the commit, in the order the popover lists them.
  const people = $derived([
    commit.author,
    ...(commit.committer.email === commit.author.email
      ? []
      : [commit.committer]),
    ...coAuthors,
  ]);
  // The stack fans out on hover, which widens the row, so only the first few
  // get an avatar and the rest are counted. The popover lists them all.
  const STACK_LIMIT = 3;
  const stacked = $derived(people.slice(0, STACK_LIMIT));
  const overflow = $derived(people.length - stacked.length);

  async function copyIdentity(who: { name: string; email: string }) {
    await writeToClipboard(`${who.name} <${who.email}>`);
    copied = who.email;
    setTimeout(() => {
      if (copied === who.email) copied = undefined;
    }, 1000);
  }
</script>

<style>
  .authorship {
    display: flex;
    font: var(--txt-body-m-regular);
    column-gap: 0.5rem;
    align-items: center;
    white-space: nowrap;
  }
  .avatar {
    width: 1rem;
    height: 1rem;
    border-radius: var(--border-radius-sm);
  }
  .avatars {
    display: flex;
    align-items: center;
  }
  /* Author and committer sit stacked, and fan apart on hover. The ring keeps
     the top avatar readable against the one behind it on any row background. */
  .avatars .avatar + .avatar {
    margin-left: -0.375rem;
    transition: margin-left 150ms ease;
  }
  .avatars:hover .avatar + .avatar {
    margin-left: 0.25rem;
  }
  /* Each avatar overlaps the next, so every one needs the ring and its own
     stacking order rather than only the first. */
  .avatars .avatar {
    position: relative;
    box-shadow: 0 0 0 1px var(--color-border-subtle);
  }
  .avatars .more {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background-color: var(--color-surface-mid);
    color: var(--color-text-secondary);
    font: var(--txt-body-s-medium);
    font-size: 0.625rem;
  }
  @media (prefers-reduced-motion: reduce) {
    .avatars .avatar + .avatar {
      transition: none;
    }
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    max-width: 22rem;
  }
  .role + .role {
    padding-top: 0.625rem;
    border-top: 1px solid var(--color-border-subtle);
  }
  .role-label {
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .role-person {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 0.25rem;
    min-width: 0;
  }
  .role-person + .role-person {
    margin-top: 0.375rem;
  }
  .role-person .avatar {
    flex-shrink: 0;
    object-fit: cover;
  }
  /* The avatar centres against the pair, while the name and the smaller email
     share a baseline rather than each being centred on its own box. */
  .role-text {
    display: flex;
    align-items: baseline;
    gap: 0.25rem;
    min-width: 0;
  }
  /* The name keeps its full width and the email gives way first, so a long
     address never squeezes out the person it belongs to. */
  .name {
    flex: 0 0 auto;
    white-space: nowrap;
    color: var(--color-text-primary);
  }
  .email-row {
    display: inline-flex;
    align-items: baseline;
    gap: 0.25rem;
    flex: 0 1 auto;
    min-width: 0;
    padding: 0.125rem 0.25rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background: none;
    cursor: pointer;
  }
  .email-row:hover,
  .email-row:focus-visible {
    background-color: var(--color-surface-mid);
  }
  .email-row:hover .email,
  .email-row:focus-visible .email,
  .email-row:hover .email-copy,
  .email-row:focus-visible .email-copy {
    color: var(--color-text-primary);
  }
  .email {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-text-tertiary);
    font: var(--txt-code-regular);
    font-size: 0.75rem;
  }
  .email-copy {
    display: inline-flex;
    align-self: center;
    flex-shrink: 0;
    color: var(--color-text-tertiary);
  }
</style>

{#snippet role(label: string, people: { name: string; email: string }[])}
  <div class="role">
    <div class="role-label txt-body-s-medium">{label}</div>
    {#each people as who (who.email)}
      <div class="role-person">
        <img class="avatar" alt="" src={utils.gravatarURL(who.email)} />
        <div class="role-text">
          <span class="name txt-body-m-regular">{who.name}</span>
          <button
            type="button"
            class="email-row"
            title={`Copy "${who.name} <${who.email}>"`}
            onclick={event => {
              event.stopPropagation();
              void copyIdentity(who);
            }}>
            <span class="email">{who.email}</span>
            <span class="email-copy">
              <Icon name={copied === who.email ? "checkmark" : "copy"} />
            </span>
          </button>
        </div>
      </div>
    {/each}
  </div>
{/snippet}

<div class="authorship">
  <HoverPopover placement="top-start" stylePadding="0.75rem">
    {#snippet toggle()}
      <div class="avatars">
        {#each stacked as who, index (who.email)}
          <img
            class="avatar"
            alt="avatar"
            title={who.name}
            style:z-index={people.length - index}
            src={utils.gravatarURL(who.email)} />
        {/each}
        {#if overflow > 0}
          <span class="avatar more">+{overflow}</span>
        {/if}
      </div>
    {/snippet}

    {#snippet popover()}
      <div class="card">
        {#if commit.author.email === commit.committer.email}
          {@render role("Author & committer", [commit.committer])}
        {:else}
          {@render role("Author", [commit.author])}
          {@render role("Committer", [commit.committer])}
        {/if}
        {#if coAuthors.length > 0}
          {@render role(
            coAuthors.length === 1 ? "Co-author" : "Co-authors",
            coAuthors,
          )}
        {/if}
      </div>
    {/snippet}
  </HoverPopover>
  {@render children()}
  <div title={utils.absoluteTimestamp(commit.committer.time * 1000)}>
    {utils.formatTimestamp(commit.committer.time * 1000)}
  </div>
</div>
