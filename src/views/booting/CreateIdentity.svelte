<script lang="ts">
  import type { ErrorWrapper } from "@bindings/error/ErrorWrapper";

  import * as router from "@app/lib/router";
  import { createEventEmittersOnce } from "@app/lib/startup.svelte";
  import { invoke } from "@app/lib/invoke";

  import debounce from "lodash/debounce";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import logo from "/radicle.svg?url";

  let passphrase = $state("");
  let notMatchingPassphrases = $state<boolean>();
  let passphraseRepeat = $state("");
  let alias = $state("");
  const errors: { alias: ErrorWrapper[]; passphrase: ErrorWrapper[] } = {
    alias: [],
    passphrase: [],
  };

  const validatePassphraseRepeat = debounce(() => {
    if (passphrase !== passphraseRepeat && passphraseRepeat.length !== 0) {
      notMatchingPassphrases = true;
    }
  }, 400);

  function validateInput(field: "alias" | "passphrase") {
    if (field === "alias" && alias.length === 0) {
      errors.alias.push({ code: "AliasError.EmptyAlias" });
    }
    if (field === "alias" && alias.length > 32) {
      errors.alias.push({ code: "AliasError.TooLongAlias" });
    }
    if (field === "alias" && alias.includes(" ")) {
      errors.alias.push({ code: "AliasError.InvalidAlias" });
    }
    if (field === "passphrase" && passphrase.length === 0) {
      errors.passphrase.push({ code: "PassphraseError.InvalidPassphrase" });
    }
  }

  const validAlias = $derived(
    alias.length > 0 && alias.length <= 32 && !alias.includes(" "),
  );
  const validPassphrase = $derived(
    passphrase.length > 0 && passphrase === passphraseRepeat,
  );

  async function handleKeydown() {
    if (passphrase !== passphraseRepeat) {
      notMatchingPassphrases = true;

      return;
    }
    try {
      await invoke("init", { passphrase, alias });
      await invoke("startup");
      await invoke("authenticate", { passphrase });
      // Clearing the passphrases from memory.
      passphrase = "";
      passphraseRepeat = "";

      if (window.__TAURI_INTERNALS__) {
        await createEventEmittersOnce();
      }

      void router.loadFromLocation();
    } catch (err) {
      const e = err as ErrorWrapper;
      if (e.code.startsWith("AliasError")) {
        errors.alias = [e];
      } else if (e.code.startsWith("PassphraseError")) {
        errors.passphrase = [e];
      }
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
        autofocus
        onSubmit={handleKeydown}
        oninput={() => {
          errors.alias = [];
          if (alias.length > 0) {
            validateInput("alias");
          }
        }}
        placeholder="Enter desired alias"
        type="text"
        bind:value={alias}></TextInput>
      {#if errors.alias.some(e => e.code.startsWith("AliasError"))}
        {#each errors.alias as error}
          <div
            style="color: var(--color-foreground-red);"
            class="hint txt-small global-flex">
            <Icon name="warning" />
            {#if error.code === "AliasError.EmptyAlias"}
              <span>Alias cannot be empty.</span>
            {:else if error.code === "AliasError.TooLongAlias"}
              <span>Alias is too long, make it less than 32 characters.</span>
            {:else if error.code === "AliasError.InvalidAlias"}
              <span>Alias cannot contain whitespace.</span>
            {/if}
          </div>
        {/each}
      {:else}
        <div class="hint txt-small txt-missing global-flex">
          <Icon name="info" /> Max 32 characters, no whitespace.
        </div>
      {/if}
    </div>
    <div>
      <div style:text-align="left" style:margin-bottom="0.5rem">
        <div class="label txt-tiny">Passphrase (required)</div>
        <TextInput
          onSubmit={handleKeydown}
          oninput={() => {
            errors.passphrase = [];
            notMatchingPassphrases = false;
            if (passphrase.length > 0) {
              validateInput("passphrase");
            }
          }}
          placeholder="Enter passphrase to protect your keys"
          type="password"
          bind:value={passphrase}></TextInput>
        {#if errors.passphrase.some(e => e.code.startsWith("PassphraseError"))}
          {#each errors.passphrase as error}
            <div
              style="color: var(--color-foreground-red);"
              class="hint txt-small global-flex">
              <Icon name="warning" />
              {#if error.code === "PassphraseError.InvalidPassphrase"}
                <span>Passphrase cannot be empty.</span>
              {:else}
                <span>{error.message}</span>
              {/if}
            </div>
          {/each}
        {/if}
      </div>
      <div style:text-align="left">
        <TextInput
          onSubmit={handleKeydown}
          oninput={() => {
            errors.passphrase = [];
            notMatchingPassphrases = false;
            validatePassphraseRepeat();
          }}
          placeholder="Repeat passphrase"
          type="password"
          bind:value={passphraseRepeat}></TextInput>
        {#if notMatchingPassphrases}
          <div
            style="color: var(--color-foreground-red);"
            class="hint txt-small global-flex">
            <Icon name="warning" /> Passphrases don't match
          </div>
        {/if}
      </div>
    </div>
    <Button
      disabled={!(validAlias && validPassphrase)}
      variant="secondary"
      onclick={handleKeydown}>
      <div class="global-flex text-center">
        <Icon name="seedling" />Create new identity
      </div>
    </Button>
  </div>
</div>
