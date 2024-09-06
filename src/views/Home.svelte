<script lang="ts">
  import type { Config } from "@bindings/Config";
  import type { RepoInfo } from "@bindings/RepoInfo";

  import Header from "@app/components/Header.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let repos: RepoInfo[];
  export let config: Config;

  onMount(async () => {
    const patches = await invoke("list_patches", {
      rid: "rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5",
      status: "open",
    });
    const issues = await invoke("list_issues", {
      rid: "rad:z3gqcJUoA1n9HaHKufZs5FCSGazv5",
      status: "open",
    });

    console.log(patches, issues);
  });
</script>

<style>
  .layout {
    padding: 1rem;
  }
  .repo-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(21rem, 1fr));
    gap: 1rem;
  }
</style>

<Header currentPage="Repositories" />
<div class="layout">
  <div class="repo-grid">
    {#each repos as repo}
      {#if repo.payloads["xyz.radicle.project"]}
        <RepoCard {repo} selfDid={`did:key:${config.publicKey}`} />
      {/if}
    {/each}
  </div>
</div>
