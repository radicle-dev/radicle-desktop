<script lang="ts">
  import type { Author } from "@bindings/cob/Author";
  import type { Job } from "@bindings/repo/Job";
  import type { Run } from "@bindings/repo/Run";

  import { SvelteSet } from "svelte/reactivity";

  import { cachedListJobs } from "@app/lib/invoke";
  import { authorForNodeId, safeHttpUrl } from "@app/lib/utils";

  import Button from "@app/components/Button.svelte";
  import Icon from "@app/components/Icon.svelte";
  import Popover from "@app/components/Popover.svelte";

  import NodeId from "./NodeId.svelte";

  interface Props {
    commit: string;
    rid: string;
    styleHeight?: "1.75rem" | "2rem" | "2.5rem";
  }

  const { commit, rid, styleHeight = "2rem" }: Props = $props();

  type Status = Run["status"];

  type RunView = {
    run: Run;
    label: string;
    safeLog: string | undefined;
  };

  type HostGroup = {
    host: string;
    runs: RunView[];
  };

  type Counts = { succeeded: number; failed: number; started: number };

  type NodeGroup = {
    nodeKey: string;
    author: Author;
    hosts: HostGroup[];
    flatRuns?: RunView[];
    inlineHost?: string;
    counts: Counts;
    status: Status;
  };

  const collapsed = new SvelteSet<string>();
  let popoverExpanded: boolean = $state(false);

  function parseUrl(s: string): URL | undefined {
    try {
      return new URL(s);
    } catch {
      return undefined;
    }
  }

  function runLabel(run: Run, url: URL | undefined): string {
    if (url && url.host === "github.com") {
      const m = url.pathname.match(/\/actions\/runs\/(\d+)/);
      if (m) return m[1];
    }
    return run.runId.slice(0, 8);
  }

  function isTerminal(status: Status): boolean {
    return status === "succeeded" || status === "failed";
  }

  function aggregateStatus(c: Counts): Status {
    if (c.failed) return "failed";
    if (c.started) return "started";
    return "succeeded";
  }

  function statusLabel(c: Counts): string {
    const parts: string[] = [];
    if (c.succeeded) parts.push(`${c.succeeded} passed`);
    if (c.failed) parts.push(`${c.failed} failed`);
    if (c.started) parts.push(`${c.started} running`);
    return parts.join(" · ");
  }

  function totalCounts(groups: NodeGroup[]): Counts {
    const total: Counts = { succeeded: 0, failed: 0, started: 0 };
    for (const g of groups) {
      total.succeeded += g.counts.succeeded;
      total.failed += g.counts.failed;
      total.started += g.counts.started;
    }
    return total;
  }

  function groupJobs(jobs: Job[]): NodeGroup[] {
    const byNode: Record<
      string,
      {
        author: Author;
        byHost: Record<string, RunView[]>;
        seen: Map<string, RunView>;
      }
    > = {};
    const nodeOrder: string[] = [];
    for (const job of jobs) {
      for (const run of job.runs) {
        const url = parseUrl(run.log);
        const host = url && url.host ? url.host : "(unknown host)";
        const safeLog =
          url?.host !== "no.url.example.com" ? safeHttpUrl(run.log) : undefined;
        const view: RunView = {
          run,
          label: runLabel(run, url),
          safeLog,
        };
        const key = run.node.did;
        let entry = byNode[key];
        if (!entry) {
          entry = { author: run.node, byHost: {}, seen: new Map() };
          byNode[key] = entry;
          nodeOrder.push(key);
        }
        const existing = entry.seen.get(run.runId);
        if (existing) {
          // The same run can appear under multiple job COBs. Keep one view
          // per runId, but let a terminal status override a stale "started"
          // one when COBs polled at different times disagree. We mutate the
          // existing view in place because it is the same object referenced
          // from its host bucket below.
          if (!isTerminal(existing.run.status) && isTerminal(run.status)) {
            existing.run = run;
            existing.label = view.label;
            existing.safeLog = view.safeLog;
          }
          continue;
        }
        entry.seen.set(run.runId, view);
        const bucket = entry.byHost[host] ?? (entry.byHost[host] = []);
        bucket.push(view);
      }
    }
    return nodeOrder.map(nodeKey => {
      const { author, byHost } = byNode[nodeKey];
      const hosts: HostGroup[] = Object.entries(byHost).map(([host, runs]) => ({
        host,
        runs,
      }));
      const counts: Counts = { succeeded: 0, failed: 0, started: 0 };
      for (const { runs } of hosts) {
        for (const v of runs) counts[v.run.status]++;
      }
      const group: NodeGroup = {
        nodeKey,
        author,
        hosts,
        counts,
        status: aggregateStatus(counts),
      };
      if (hosts.length === 1) {
        group.flatRuns = hosts[0].runs;
        if (!aliasMatchesHost(author.alias, hosts[0].host)) {
          group.inlineHost = hosts[0].host;
        }
      }
      return group;
    });
  }

  function aliasMatchesHost(alias: string | undefined, host: string): boolean {
    if (!alias) return false;
    const a = alias.toLowerCase();
    if (host === a) return true;
    if (host.endsWith("." + a)) return true;
    return host.split(".").includes(a);
  }

  function toggleNode(key: string) {
    if (collapsed.has(key)) {
      collapsed.delete(key);
    } else {
      collapsed.add(key);
    }
  }
</script>

<style>
  .chip {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 1rem;
    height: 1rem;
  }
  .chip.succeeded {
    color: var(--color-feedback-success-text);
    background-color: var(--color-feedback-success-bg);
  }
  .chip.failed {
    color: var(--color-feedback-error-text);
    background-color: var(--color-feedback-error-bg);
  }
  .chip.started {
    color: var(--color-text-quaternary);
    background-color: var(--color-surface-mid);
  }
  .popover-body {
    display: flex;
    flex-direction: column;
    min-width: 24rem;
    font: var(--txt-body-m-regular);
  }
  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 2rem;
    padding: 0 0.75rem;
    border-radius: var(--border-radius-sm);
  }
  .node-header {
    cursor: pointer;
    user-select: none;
  }
  .node-header:hover,
  a.run-row:hover {
    background-color: var(--color-surface-subtle);
  }
  .chevron {
    width: 1rem;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    color: var(--color-text-quaternary);
  }
  .node-name {
    min-width: 0;
    flex: 1;
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    overflow: hidden;
  }
  .inline-host {
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .count {
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
    white-space: nowrap;
  }
  .host-row {
    padding-left: 2.25rem;
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
  }
  /*
   * 3.75rem = row padding (0.75) + chevron (1) + gap (0.5) + header chip (1)
   * + gap (0.5), so the run status chip sits under the node avatar.
   */
  .run-row {
    padding-left: 3.75rem;
    text-decoration: none;
    color: var(--color-text-primary);
  }
  a.run-row {
    cursor: pointer;
  }
  .run-label {
    flex: 1;
    white-space: nowrap;
  }
  .run-id {
    font: var(--txt-code-regular);
  }
  .run-affordance {
    color: var(--color-text-quaternary);
    white-space: nowrap;
  }
  .run-affordance.muted {
    font: var(--txt-body-s-regular);
  }
</style>

{#await cachedListJobs(rid, commit) then jobs}
  {#if jobs.length > 0}
    {@const groups = groupJobs(jobs)}
    {@const overallCounts = totalCounts(groups)}
    {@const overall = aggregateStatus(overallCounts)}
    {#snippet statusChip(status: Status)}
      <span class="chip {status}">
        {#if status === "succeeded"}
          <Icon name="checkmark" />
        {:else if status === "failed"}
          <Icon name="close" />
        {:else}
          <Icon name="hourglass" />
        {/if}
      </span>
    {/snippet}
    {#snippet runRow(view: RunView)}
      {#if view.safeLog}
        <a
          class="row run-row"
          href={view.safeLog}
          target="_blank"
          rel="noopener noreferrer">
          {@render statusChip(view.run.status)}
          <span class="run-label">
            run <span class="run-id">{view.label}</span>
          </span>
          <span class="run-affordance">
            <Icon name="open-external" />
          </span>
        </a>
      {:else}
        <div class="row run-row">
          {@render statusChip(view.run.status)}
          <span class="run-label">
            run <span class="run-id">{view.label}</span>
          </span>
          <span class="run-affordance muted">(no log)</span>
        </div>
      {/if}
    {/snippet}
    <Popover placement="bottom-end" bind:expanded={popoverExpanded}>
      {#snippet toggle(onclick)}
        <Button
          variant="naked"
          {styleHeight}
          onclick={e => {
            e.stopPropagation();
            onclick();
          }}
          active={popoverExpanded}>
          {@render statusChip(overall)}
          {statusLabel(overallCounts)}
          <Icon name="chevron-down" />
        </Button>
      {/snippet}

      {#snippet popover()}
        <div
          style:border="1px solid var(--color-border-subtle)"
          style:border-radius="var(--border-radius-sm)"
          style:background-color="var(--color-surface-canvas)"
          style:padding="0.25rem">
          <div class="popover-body">
            {#each groups as group (group.nodeKey)}
              {@const isCollapsed = collapsed.has(group.nodeKey)}
              <div class="node-group">
                <div
                  class="row node-header"
                  role="button"
                  tabindex="0"
                  onclick={() => toggleNode(group.nodeKey)}
                  onkeydown={e => {
                    if (e.key === "Enter" || e.key === " ") {
                      e.preventDefault();
                      toggleNode(group.nodeKey);
                    }
                  }}>
                  <span class="chevron">
                    <Icon
                      name={isCollapsed ? "chevron-right" : "chevron-down"} />
                  </span>
                  {@render statusChip(group.status)}
                  <span class="node-name">
                    <NodeId {...authorForNodeId(group.author)} />
                    {#if group.inlineHost}
                      <span class="inline-host">· {group.inlineHost}</span>
                    {/if}
                  </span>
                  <span class="count">
                    {statusLabel(group.counts)}
                  </span>
                </div>

                {#if !isCollapsed}
                  {#if group.flatRuns}
                    {#each group.flatRuns as view (view.run.runId)}
                      {@render runRow(view)}
                    {/each}
                  {:else}
                    {#each group.hosts as host}
                      <div class="row host-row">
                        <span>{host.host}</span>
                      </div>
                      {#each host.runs as view (view.run.runId)}
                        {@render runRow(view)}
                      {/each}
                    {/each}
                  {/if}
                {/if}
              </div>
            {/each}
          </div>
        </div>
      {/snippet}
    </Popover>
  {/if}
{/await}
