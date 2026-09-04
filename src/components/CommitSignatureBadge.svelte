<script lang="ts">
  import type { Config } from "@bindings/config/Config";
  import type { CommitSignature } from "@bindings/repo/CommitSignature";

  import { cachedConfig, writeToClipboard } from "@app/lib/invoke";
  import { explorerUrl, publicKeyFromDid, truncateId } from "@app/lib/utils";

  import HoverPopover from "@app/components/HoverPopover.svelte";
  import Icon from "@app/components/Icon.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";

  interface Props {
    signature: CommitSignature;
    /// Icon only, for narrow columns where the label does not fit. The hover
    /// card is unchanged, so nothing is lost but the visible wording.
    compact?: boolean;
  }

  const { signature, compact = false }: Props = $props();

  let copyIcon: "copy" | "checkmark" = $state("copy");
  let config: Config | undefined = $state();

  // `cachedConfig` caches the promise itself, so the many badges on a page
  // share a single request rather than each issuing one.
  $effect(() => {
    let cancelled = false;
    void cachedConfig()
      .then(result => {
        if (!cancelled) config = result;
      })
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  const profileUrl = $derived(
    signature.signer && config
      ? explorerUrl(`users/${signature.signer.did}`, config)
      : undefined,
  );

  async function copy(text: string) {
    await writeToClipboard(text);
    copyIcon = "checkmark";
    setTimeout(() => (copyIcon = "copy"), 1000);
  }

  const label = $derived(
    signature.status === "verified"
      ? "Signed"
      : signature.status === "invalid"
        ? "Bad signature"
        : "Not checked",
  );

  // The headline states what was actually checked. A verified signature proves
  // the key signed these bytes; whether that key speaks for the repository is a
  // separate question, which the delegate line below answers.
  const headline = $derived(
    signature.status === "verified"
      ? signature.known
        ? "Signed with a Radicle key"
        : "Signed with an unrecognized key"
      : signature.status === "invalid"
        ? "Signature does not match this commit"
        : "Signature cannot be checked",
  );

  const explanation = $derived(
    signature.status === "verified"
      ? signature.known
        ? "Git records the author and committer but verifies neither. This signature is the key holder attesting to these exact contents."
        : "The commit carries an SSH signature that verifies against the key below. Your node has no record of that key, so it cannot be tied to a Radicle identity."
      : signature.status === "invalid"
        ? "The commit carries a signature by the key below, but it does not verify against the commit's contents."
        : "The commit is signed with a scheme this app does not check, so nothing is claimed about it.",
  );
</script>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0 0.25rem;
    height: 1.25rem;
    border-radius: var(--border-radius-sm);
    font: var(--txt-body-s-medium);
    white-space: nowrap;
  }
  /* Square, so a column of these lines up regardless of the label they drop. */
  .chip.compact {
    width: 1.25rem;
    padding: 0;
    gap: 0;
    justify-content: center;
  }
  .verified {
    background-color: var(--color-feedback-success-bg);
    color: var(--color-feedback-success-text);
  }
  .invalid {
    background-color: var(--color-feedback-error-bg);
    color: var(--color-feedback-error-text);
  }
  .unsupported {
    background-color: var(--color-surface-subtle);
    color: var(--color-text-tertiary);
  }
  .card {
    display: flex;
    flex-direction: column;
    gap: 0.625rem;
    max-width: 22rem;
  }
  .headline {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    color: var(--color-text-primary);
  }
  .explanation {
    color: var(--color-text-secondary);
  }
  .signer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding-top: 0.625rem;
    border-top: 1px solid var(--color-border-subtle);
  }
  .avatar {
    width: 2.5rem;
    height: 2.5rem;
    flex-shrink: 0;
    overflow: hidden;
  }
  .unknown-key {
    display: grid;
    place-items: center;
    width: 2.5rem;
    height: 2.5rem;
    flex-shrink: 0;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-mid);
    color: var(--color-text-tertiary);
  }
  .avatar :global(img) {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .signer-text {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
  }
  .signer-did-row {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    width: fit-content;
    max-width: 100%;
    padding: 0.125rem 0.25rem;
    margin-left: -0.25rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background: none;
    cursor: pointer;
  }
  .signer-did-row:hover,
  .signer-did-row:focus-visible {
    background-color: var(--color-surface-mid);
  }
  .signer-did-row:hover .signer-did,
  .signer-did-row:focus-visible .signer-did,
  .signer-did-row:hover .signer-copy,
  .signer-did-row:focus-visible .signer-copy {
    color: var(--color-text-primary);
  }
  .signer-did {
    min-width: 0;
    color: var(--color-text-tertiary);
    font: var(--txt-code-regular);
    font-size: 0.75rem;
  }
  /* A fingerprint is compared against `ssh-add -l` or `%GK` output, so it wraps
     rather than truncating: an elided one cannot be checked against anything.
     The row becomes a block so the text and the copy icon share one flow and
     the icon lands after the final character instead of floating beside the
     wrapped block. */
  .signer-did-row.wrapping {
    display: block;
    width: 100%;
    text-align: left;
  }
  .fingerprint {
    display: inline;
    white-space: normal;
    word-break: break-all;
  }
  .wrapping .signer-copy {
    margin-left: 0.25rem;
    vertical-align: text-bottom;
  }
  .signer-copy {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    color: var(--color-text-tertiary);
    opacity: 0;
    transition: opacity 150ms ease;
  }
  .signer-did-row:hover .signer-copy,
  .signer-did-row:focus-visible .signer-copy {
    opacity: 1;
  }
  @media (prefers-reduced-motion: reduce) {
    .signer-copy {
      transition: none;
    }
  }
  .signer-headline {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    min-width: 0;
  }
  .signer-alias {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    flex: 0 1 auto;
    min-width: 0;
    color: var(--color-text-primary);
    text-decoration: none;
  }
  .signer-alias :global(svg) {
    color: var(--color-text-tertiary);
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 150ms ease;
  }
  .signer-alias[href]:hover :global(svg),
  .signer-alias[href]:focus-visible :global(svg) {
    opacity: 1;
  }
  @media (prefers-reduced-motion: reduce) {
    .signer-alias :global(svg) {
      transition: none;
    }
  }
  /* Only a resolved profile URL makes the alias a link, so the affordance
     appears only when there is somewhere to go. */
  .signer-alias[href]:hover span,
  .signer-alias[href]:focus-visible span {
    text-decoration: underline;
  }
  .signer-alias[href]:hover :global(svg) {
    color: var(--color-text-primary);
  }
  /* Pushed to the far edge so the chip and the alias's link icon never sit
     next to each other. */
  .delegate,
  .former-delegate {
    margin-left: auto;
  }
  .delegate {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
    padding: 0.125rem 0.375rem 0.125rem 0.125rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-brand-subtle);
    color: var(--color-text-brand);
  }
  .former-delegate {
    display: inline-flex;
    align-items: center;
    gap: 0.25rem;
    flex-shrink: 0;
    padding: 0.125rem 0.375rem 0.125rem 0.125rem;
    border-radius: var(--border-radius-sm);
    background-color: var(--color-surface-mid);
    color: var(--color-text-secondary);
  }
  .trust {
    color: var(--color-text-secondary);
  }
</style>

<HoverPopover placement="top-end" stylePadding="0.75rem">
  {#snippet toggle()}
    <span
      class="chip"
      class:compact
      class:verified={signature.status === "verified"}
      class:invalid={signature.status === "invalid"}
      class:unsupported={signature.status === "unsupported"}
      aria-label="commit-signature"
      role="img"
      aria-roledescription={label}>
      <Icon name={signature.status === "invalid" ? "warning" : "key"} />
      {#if !compact}
        {label}
      {/if}
    </span>
  {/snippet}

  {#snippet popover()}
    <div class="card">
      <div class="headline txt-body-m-medium">
        <Icon name={signature.status === "invalid" ? "warning" : "key"} />
        {headline}
      </div>
      <div class="explanation txt-body-s-regular">{explanation}</div>

      {#if signature.known && signature.signer}
        {@const did = signature.signer.did}
        {@const key = publicKeyFromDid(did)}
        <div class="signer">
          <div class="avatar">
            <UserAvatar nodeId={key} styleWidth="2.5rem" />
          </div>
          <div class="signer-text">
            <div class="signer-headline">
              <a
                class="signer-alias txt-body-m-medium"
                href={profileUrl}
                title="View profile on radicle.network"
                target="_blank"
                rel="noreferrer"
                onclick={event => event.stopPropagation()}>
                <span class="txt-overflow">
                  {signature.signer.alias ?? truncateId(key)}
                </span>
                {#if profileUrl}
                  <Icon name="open-external" />
                {/if}
              </a>
              {#if signature.delegate}
                <span class="delegate txt-body-s-medium">
                  <Icon name="badge" />
                  <span>Delegate</span>
                </span>
              {:else if signature.formerDelegate}
                <span class="former-delegate txt-body-s-medium">
                  <Icon name="badge" />
                  <span>Former delegate</span>
                </span>
              {/if}
            </div>
            <button
              type="button"
              class="signer-did-row"
              title="Copy DID"
              onclick={event => {
                event.stopPropagation();
                void copy(did);
              }}>
              <span class="signer-did txt-overflow">{truncateId(key)}</span>
              <span class="signer-copy"><Icon name={copyIcon} /></span>
            </button>
          </div>
        </div>

        {#if signature.formerDelegate}
          <!--
            Scoped to the identity history, not to this commit: a commit does
            not record which identity revision it was made under, so we cannot
            say whether the signer still held the role when they signed.
          -->
          <div class="trust txt-body-s-regular">
            This node was in the delegate set at some point, but is not now. It
            is not known whether it still was when this commit was signed.
          </div>
        {:else if !signature.delegate}
          <div class="trust txt-body-s-regular">
            {signature.remote
              ? "This node publishes its own branches in this repo, but has never been a delegate."
              : "This node has never been a delegate and has no branches in this repo."}
          </div>
        {/if}
      {:else if signature.fingerprint}
        {@const fingerprint = signature.fingerprint}
        <!--
          Nothing ties this key to a node, so it is shown as a key: an OpenSSH
          fingerprint, with no avatar, alias or profile link that would imply an
          identity behind it.
        -->
        <div class="signer">
          <div class="unknown-key"><Icon name="key" /></div>
          <div class="signer-text">
            <div class="signer-headline">
              <span class="txt-body-m-medium">Unrecognized key</span>
            </div>
            <button
              type="button"
              class="signer-did-row wrapping"
              title="Copy fingerprint"
              onclick={event => {
                event.stopPropagation();
                void copy(fingerprint);
              }}>
              <span class="signer-did fingerprint">{fingerprint}</span>
              <span class="signer-copy"><Icon name={copyIcon} /></span>
            </button>
          </div>
        </div>
      {/if}
    </div>
  {/snippet}
</HoverPopover>
