<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";

  import * as router from "@app/lib/router";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import logo from "@public/radicle.svg?url";
  import { invoke } from "@app/lib/invoke";
  import { capitalize } from "lodash";

  let newPassphrase = $state("");
  let alias = $state("");
  let disabled = $state(false);
  let error = $state<ErrorWrapper>();

  async function handleKeydown() {
    disabled = true;
    try {
      await invoke("init", { passphrase: newPassphrase, alias });
      await invoke("startup");
      await invoke("authenticate", { passphrase: newPassphrase });

      void router.loadFromLocation();
    } catch (err) {
      disabled = false;
      error = err as ErrorWrapper;
      console.error(err);
    }

    return;
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

  <div class="txt-medium txt-bold">Log in to Radicle Desktop</div>
  <div class="txt-small">
    Create a Radicle identity in order to use the app.
  </div>

  <div class="form">
    <div style:text-align="left">
      <div class="label txt-tiny">Alias (required)</div>
      <TextInput
        onSubmit={handleKeydown}
        oninput={() => (error = undefined)}
        placeholder="Enter desired alias"
        type="text"
        bind:value={alias}></TextInput>
      {#if error?.type === "IdentityError.AliasError"}
        <div
          style="color: var(--color-foreground-red);"
          class="hint txt-small global-flex">
          <Icon name="warning" />
          <span>{capitalize(error.message)}</span>
        </div>
      {:else}
        <div class="hint txt-small txt-missing global-flex">
          <Icon name="info" /> Max 32 characters, no whitespace.
        </div>
      {/if}
    </div>
    <div style:text-align="left">
      <div class="label txt-tiny">Passphrase (required)</div>
      <TextInput
        onSubmit={handleKeydown}
        oninput={() => (error = undefined)}
        placeholder="Enter passphrase to protect your keys"
        type="password"
        bind:value={newPassphrase}></TextInput>
      {#if error?.type === "IdentityError.MissingPassphrase"}
        <div
          style="color: var(--color-foreground-red);"
          class="hint txt-small global-flex">
          <Icon name="warning" />
          {capitalize(error.message)}
        </div>
      {/if}
    </div>
    <Button {disabled} variant="secondary" onclick={handleKeydown}>
      <div class="global-flex text-center">
        <Icon name="seedling" /> Create new identity
      </div>
    </Button>
  </div>
</div>
