<script lang="ts">
  interface Props {
    fullPath: string;
  }

  const { fullPath }: Props = $props();
</script>

<style>
  .container {
    white-space: nowrap;
    -webkit-touch-callout: initial;
    -webkit-user-select: text;
    user-select: text;
  }
  .path {
    color: var(--color-fill-gray);
    font-weight: var(--font-weight-regular);
  }
  .filename {
    font-weight: var(--font-weight-semibold);
  }
</style>

<!-- prettier-ignore -->
<span class="txt-small container">
  <span class="path">
    {fullPath
      .match(/^.*\/|/)
      ?.values()
      .next().value}
  </span><span class="filename">
    {fullPath.split("/").slice(-1)}
  </span>
</span>
