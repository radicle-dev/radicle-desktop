<script lang="ts">
  import Button from "@app/components/Button.svelte";
  import Header from "@app/components/Header.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Link from "@app/components/Link.svelte";
</script>

<Header currentPage="Design System" />

<div style="display: flex; gap: 1rem;">
  <div style="display: flex; gap: 1rem;">
    <Button variant="primary"><Icon name="seedling" />Press me</Button>
    <Button variant="secondary"><Icon name="seedling" /> Press me</Button>
    <Button variant="ghost"><Icon name="seedling" /> Press me</Button>
  </div>
</div>

👉 <Link route={{ resource: "home" }}>Home</Link>
