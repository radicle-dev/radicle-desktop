<script lang="ts">
  import type { Repo } from "@app/lib/api/repo";

  import repoFixture from "@app/lib/api/fixtures/heartwood-repo.json";

  import Header from "@app/components/Header.svelte";
  import Link from "@app/components/Link.svelte";
  import RepoCard from "@app/components/RepoCard.svelte";

  const selfDid = "did:key:z6MksFqXN3Yhqk8pTJdUGLwATkRfQvwZXPqR2qMEhbS9wzpT";
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
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />

    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />

    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />

    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
    <RepoCard repo={repoFixture as Repo} {selfDid} />
  </div>

  <div style:margin-top="1rem">
    👉 <Link route={{ resource: "designSystem" }}>Design System</Link>
  </div>
</div>
