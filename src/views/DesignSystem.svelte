<script lang="ts">
  import Border from "@app/components/Border.svelte";
  import Button from "@app/components/Button.svelte";
  import Fill from "@app/components/Fill.svelte";
  import Icon from "@app/components/Icon.svelte";

  let theme = "dark";

  $: document.documentElement.setAttribute("data-theme", theme);
</script>

<div style="display: flex; gap: 1rem;">
  <div style="display: flex; gap: 1rem;">
    <Border variant="secondary">
      <Fill
        stylePadding="0 0.5rem"
        variant={theme === "dark" ? "secondary" : "transparent"}
        on:click={() => {
          theme = "dark";
        }}>
        <Icon name="seedling" />
        Dark
      </Fill>

      <Fill
        stylePadding="0 0.5rem"
        variant={theme === "light" ? "secondary" : "transparent"}
        on:click={() => {
          theme = "light";
        }}>
        <Icon name="seedling" />
        Light
      </Fill>
    </Border>
  </div>

  <div style="display: flex; gap: 1rem;">
    <Button variant="primary"><Icon name="seedling" />Press me</Button>
    <Button variant="secondary"><Icon name="seedling" /> Press me</Button>
    <Button variant="ghost"><Icon name="seedling" /> Press me</Button>
  </div>
</div>
