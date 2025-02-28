<script lang="ts">
  import { onMount } from "svelte";

  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import TextInput from "@app/components/TextInput.svelte";
  import logo from "@public/radicle.svg?url";
  import { invoke } from "@app/lib/invoke";
  import { startup, status } from "@app/lib/init.svelte";
  import { truncateDid } from "@app/lib/utils";

  let passphrase = $state("");
  let newPassphrase = $state("");
  let alias = $state("");
  let disabled = $state(false);

  onMount(async () => {
    try {
      await startup(passphrase);
    } catch (err) {
      console.error(err);
    }
  });

  async function handleKeydown() {
    disabled = true;
    if (status.type === "missingProfile") {
      await invoke("init", { passphrase: newPassphrase, alias });
      void startup(newPassphrase);
      return;
    } else if (status.type === "lockedKeystore") {
      void startup(passphrase);
      return;
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

{#if status.type !== "loading"}
  <div class="container">
    <img src={logo} alt="Radicle Space Invader" class="logo" />

    {#if status.type === "missingProfile"}
      <div class="txt-medium txt-bold">Log in to Radicle Desktop</div>
      <div class="txt-small">
        Create a Radicle identity in order to use the app.
      </div>

      <div class="form">
        <div style:text-align="left">
          <div class="label txt-tiny">Alias</div>
          <TextInput
            onSubmit={handleKeydown}
            placeholder="Enter desired alias"
            type="text"
            bind:value={alias}></TextInput>
        </div>
        <div style:text-align="left">
          <div class="label txt-tiny">Passphrase</div>
          <TextInput
            onSubmit={handleKeydown}
            placeholder="Enter passphrase to protect your keys"
            type="password"
            bind:value={newPassphrase}></TextInput>
        </div>
        <Button {disabled} variant="secondary" onclick={handleKeydown}>
          <div class="global-flex text-center">
            <Icon name="seedling" /> Create new identity
          </div>
        </Button>
      </div>
    {/if}

    {#if status.type === "lockedKeystore"}
      <div class="txt-medium txt-bold">Unlock keys</div>
      <div class="txt-small">
        Your passphrase is needed to unlock your keys.
      </div>
      <div class="form">
        <div>
          <Border
            stylePadding="0.5rem 0.75rem"
            variant="ghost"
            flatBottom={true}>
            <div style:text-align="left">
              <div class="label txt-tiny">Alias</div>
              <div>
                {status.profile.alias}
              </div>
            </div>
          </Border>
          <Border stylePadding="0.5rem 0.75rem" variant="ghost" flatTop={true}>
            <div style:text-align="left">
              <div class="label txt-tiny">DID</div>
              <div>
                {truncateDid(status.profile.did)}
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
        </div>
        <Button {disabled} variant="secondary" onclick={handleKeydown}>
          <div class="global-flex text-center">
            <Icon name="lock" /> Unlock
          </div>
        </Button>
      </div>
    {/if}
  </div>
{/if}
