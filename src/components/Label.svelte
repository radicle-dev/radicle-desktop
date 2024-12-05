<script lang="ts">
  interface Props {
    label: string;
  }

  const { label }: Props = $props();
</script>

<div class="global-counter txt-small" style:max-width="10rem">
  <div class="txt-overflow" title={label}>{label}</div>
</div>
