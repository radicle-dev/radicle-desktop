<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { ErrorWrapper } from "@bindings/error/Error";

  import * as router from "@app/lib/router";
  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import logo from "@public/radicle.svg?url";
  import { truncateDid } from "@app/lib/utils";
  import { capitalize } from "lodash";
  import { invoke } from "@app/lib/invoke";
  import { createEventEmitters } from "@app/lib/startup.svelte";

  interface Props {
    profile: Author;
  }

  let error = $state<ErrorWrapper>();
  let passphrase = $state("");
  let disabled = $state(false);
  const { profile }: Props = $props();

  async function handleKeydown() {
    disabled = true;
    try {
      await invoke("authenticate", { passphrase });
      if (window.__TAURI_INTERNALS__) {
        await createEventEmitters();
      }

      void router.push({ resource: "home" });
    } catch (err) {
      disabled = false;
      error = err as ErrorWrapper;
      console.error(err);
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
        onSubmit={handleKeydown}
        placeholder="Enter passphrase to unlock your keys"
        type="password"
        bind:value={passphrase} />
      {#if error?.type.startsWith("IdentityError")}
        <div
          style="color: var(--color-foreground-red);"
          class="hint txt-small global-flex">
          <Icon name="warning" />
          <span>{capitalize(error.message)}</span>
        </div>
      {/if}
    </div>
    <Button {disabled} variant="secondary" onclick={handleKeydown}>
      <div class="global-flex text-center">
        <Icon name="lock" /> Unlock
      </div>
    </Button>
  </div>
</div>
