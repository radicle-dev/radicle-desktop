<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";

  import * as router from "@app/lib/router";
  import { createEventEmittersOnce } from "@app/lib/startup.svelte";
  import { invoke } from "@app/lib/invoke";
  import { truncateDid } from "@app/lib/utils";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Spinner from "@app/components/Spinner.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import logo from "/radicle.svg?url";

  interface Props {
    profile: Author;
  }

  let error = $state<ErrorWrapper>();
  let passphrase = $state("");
  let authInProgress = $state(false);
  const { profile }: Props = $props();

  async function authenticate() {
    if (passphrase.length > 0) {
      authInProgress = true;
      error = undefined;
      try {
        await invoke("authenticate", { passphrase });
        if (window.__TAURI_INTERNALS__) {
          await createEventEmittersOnce();
        }
        passphrase = " ".repeat(passphrase.length);
        await router.push({ resource: "home", activeTab: "all" });
      } catch (err) {
        error = err as ErrorWrapper;
      } finally {
        authInProgress = false;
      }
    }
  }
</script>

<style>
  .container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    text-align: center;
    padding-top: 10rem;
  }
  .logo {
    height: 3rem;
  }
  .text-center {
    text-align: center;
    margin: auto;
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    margin-top: 1.5rem;
    width: 23rem;
  }
  .label {
    margin-bottom: 0.5rem;
  }
  .hint {
    padding: 0.25rem 0 0 0.25rem;
    gap: 0.25rem;
  }
</style>

<div class="container">
  <img src={logo} alt="Radicle Space Invader" class="logo" />

  <div class="txt-medium txt-bold">Unlock keys</div>
  <div class="txt-small">Your passphrase is needed to unlock your keys.</div>
  <div class="form">
    <div>
      <Border stylePadding="0.5rem 0.75rem" variant="ghost" flatBottom={true}>
        <div style:text-align="left">
          <div class="label txt-tiny">Alias</div>
          <div>
            {profile.alias}
          </div>
        </div>
      </Border>
      <Border stylePadding="0.5rem 0.75rem" variant="ghost" flatTop={true}>
        <div style:text-align="left">
          <div class="label txt-tiny">DID</div>
          <div>
            {truncateDid(profile.did)}
          </div>
        </div>
      </Border>
    </div>
    <div style:text-align="left">
      <div class="label txt-tiny">Passphrase</div>
      <TextInput
        autofocus
        onSubmit={authenticate}
        oninput={() => {
          error = undefined;
        }}
        placeholder="Enter passphrase to unlock your keys"
        type="password"
        bind:value={passphrase} />
      {#if error?.code === "PassphraseError.InvalidPassphrase"}
        <div
          style="color: var(--color-foreground-red);"
          class="hint txt-small global-flex">
          <Icon name="warning" />
          <span>Not able to decrypt keys with provided passphrase.</span>
        </div>
      {/if}
    </div>
    <Button
      disabled={authInProgress || passphrase.length === 0}
      variant="secondary"
      onclick={authenticate}>
      <div class="global-flex text-center">
        {#if authInProgress}
          <Spinner /> Unlocking…
        {:else}
          <Icon name="lock" /> Unlock
        {/if}
      </div>
    </Button>
  </div>
</div>
