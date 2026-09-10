<script lang="ts" module>
  export interface MentionInsertion {
    /** Index the replacement starts at, i.e. the trigger character. */
    start: number;
    /** Index the replacement ends at, i.e. the caret. */
    end: number;
    markdown: string;
  }
</script>

<script lang="ts">
  import type { AliasSuggestion } from "@bindings/cob/AliasSuggestion";
  import type { Author } from "@bindings/cob/Author";
  import type { Config } from "@bindings/config/Config";
  import type { ComponentProps } from "svelte";

  import {
    autoUpdate,
    computePosition,
    flip,
    hide,
    shift,
    size,
  } from "@floating-ui/dom";
  import fuzzysort from "fuzzysort";
  import debounce from "lodash/debounce";
  import { useOverlayScrollbars } from "overlayscrollbars-svelte";

  import {
    cachedAlias,
    cachedConfig,
    cachedIssueById,
    cachedListIssueCandidates,
    cachedListPatchCandidates,
    cachedListReposSummary,
    cachedPatchById,
    cachedRepoById,
    cachedRepoCommit,
    cachedSearchAliases,
  } from "@app/lib/invoke";
  import type { MentionTarget } from "@app/lib/mentions";
  import {
    bareOidPattern,
    mentionHref,
    mentionMarkdown,
  } from "@app/lib/mentions";
  import type { MentionTrigger } from "@app/lib/mentionTrigger";
  import { findMentionTrigger } from "@app/lib/mentionTrigger";
  import { portal } from "@app/lib/portal";
  import { repoListScope } from "@app/lib/repoListScope";
  import { caretCoordinates } from "@app/lib/textareaCaret";
  import { formatOid, publicKeyFromDid, truncateId } from "@app/lib/utils";

  import Icon from "@app/components/Icon.svelte";
  import UserAvatar from "@app/components/UserAvatar.svelte";

  type IconName = ComponentProps<typeof Icon>["name"];

  interface Suggestion {
    key: string;
    target: MentionTarget;
    /** Written into the markdown link's label. */
    label: string;
    /** Shown as the row's title. */
    primary: string;
    /** Shown after the title, e.g. an abbreviated object id. */
    secondary?: string;
    /** The name is itself a raw id, so it should be set as one. */
    mono?: boolean;
    icon?: IconName;
    /** Node id of a person, so the row can show their avatar. */
    avatar?: string;
    /**
     * Why this node is trustworthy, when it is. Aliases are self-declared and
     * unverified, so several unrelated nodes routinely share one; this is what
     * lets the user tell them apart.
     */
    badge?: "delegate" | "following" | "you";
    /** What the query is matched against. */
    haystack: string;
  }

  interface Props {
    /** The repo whose issues and patches are offered. */
    rid: string;
    textarea: HTMLTextAreaElement | undefined;
    value: string;
    caret: number;
    onselect: (insertion: MentionInsertion) => void;
    /**
     * Called once with a handler the textarea can pass keys to, so the
     * dropdown takes over the arrow keys, Enter and Tab while it is open. The
     * handler returns true when it consumed the key.
     */
    registerKeydown: (handler: (event: KeyboardEvent) => boolean) => void;
  }

  /**
   * How many rows the dropdown offers. The list scrolls, so this only needs to
   * stay small enough that ranking still means something.
   */
  const maxSuggestions = 50;

  /**
   * How many nodes with no trust signal are offered. Anyone can adopt any
   * alias, so a popular name matches many unrelated nodes from gossip — 28
   * nodes call themselves `rudolfs_seed` on a typical network. Bounded so one
   * popular alias cannot fill the list on its own.
   */
  const maxUnvouchedPeople = 10;

  /** Tallest the dropdown grows before it scrolls. */
  const maxDropdownHeight = 384;

  /** Shortest it is allowed to be squeezed to near a viewport edge. */
  const minDropdownHeight = 160;

  const { rid, textarea, value, caret, onselect, registerKeydown }: Props =
    $props();

  let focused = $state(false);
  let dismissedAt: number | undefined = $state(undefined);
  let activeIndex = $state(0);
  let suggestions: Suggestion[] = $state([]);
  let floatingEl: HTMLDivElement | undefined = $state();
  let scrollEl: HTMLDivElement | undefined = $state();
  // Held so an insertion can write an explorer URL, which is what makes the
  // reference clickable in clients that know nothing of `rad:` hrefs.
  let config: Config | undefined = $state(undefined);
  // Only read imperatively, to scroll the active row into view.
  const rowElements: (HTMLButtonElement | undefined)[] = [];

  const trigger = $derived(findMentionTrigger(value, caret));
  // Escape dismisses the dropdown for the reference being typed, without
  // suppressing it for the next one.
  const active = $derived(
    trigger !== undefined && trigger.start !== dismissedAt,
  );
  // The caret keeps its position when the textarea loses focus, so without
  // this the dropdown would hang around over whatever the user moved on to.
  const open = $derived(focused && active && suggestions.length > 0);

  $effect(() => {
    if (config) return;
    let cancelled = false;
    void cachedConfig()
      .then(result => {
        if (!cancelled) config = result;
      })
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  });

  $effect(() => {
    if (!textarea) return;
    const element = textarea;
    focused = document.activeElement === element;

    const onFocus = () => (focused = true);
    const onBlur = () => (focused = false);
    element.addEventListener("focus", onFocus);
    element.addEventListener("blur", onBlur);

    return () => {
      element.removeEventListener("focus", onFocus);
      element.removeEventListener("blur", onBlur);
    };
  });

  // Debounced so a fast typist does not queue a backend round trip per
  // keystroke; the leading call keeps the first character responsive.
  const loadSuggestions = debounce((requested: MentionTrigger) => {
    const token = triggerToken(requested);
    void collect(requested)
      .then(collected => {
        // A later keystroke may have superseded this trigger while it was in
        // flight, in which case its results are stale.
        if (trigger === undefined || triggerToken(trigger) !== token) return;
        suggestions = collected;
        activeIndex = 0;
      })
      .catch(error => {
        console.warn("Not able to load mention suggestions", error);
        suggestions = [];
      });
  }, 80);

  /** Identifies a trigger, so a slow response can be matched to its request. */
  function triggerToken(value: MentionTrigger): string {
    switch (value.kind) {
      case "query":
        return `query:${value.scope}:${value.query}`;
      case "identifier":
        return `identifier:${mentionHref(value.target)}`;
      case "oid":
        return `oid:${value.oid}`;
    }
  }

  $effect(() => {
    if (!active || !trigger) {
      suggestions = [];
      loadSuggestions.cancel();
      return;
    }
    loadSuggestions(trigger);
  });

  $effect(() => {
    // The dropdown follows the caret rather than the textarea, so a reference
    // typed mid-paragraph does not open a panel somewhere far from the text.
    if (!open || !floatingEl || !textarea) return;
    const element = textarea;
    const position = trigger?.start ?? caret;
    const virtual = {
      // Without this a virtual reference has no place in the DOM, so
      // `autoUpdate` cannot find the scroll containers to watch and the
      // dropdown stays put while the text under it scrolls away.
      contextElement: element,
      getBoundingClientRect: () => {
        const rect = element.getBoundingClientRect();
        const { top, left, height } = caretCoordinates(element, position);

        return new DOMRect(
          rect.left + left - element.scrollLeft,
          rect.top + top - element.scrollTop,
          0,
          height,
        );
      },
    };

    return autoUpdate(virtual, floatingEl, () => {
      void computePosition(virtual, floatingEl!, {
        placement: "bottom-start",
        middleware: [
          flip(),
          shift({ padding: 8 }),
          // `shift` would otherwise pin the dropdown to the viewport edge
          // after the caret itself has scrolled out of sight.
          hide({ padding: 8 }),
          // The list scrolls, so it must never grow past the space left below
          // (or above) the caret, which is tight inside a modal.
          size({
            padding: 8,
            apply({ availableHeight, elements }) {
              elements.floating.style.maxHeight = `${Math.max(
                minDropdownHeight,
                Math.min(availableHeight, maxDropdownHeight),
              )}px`;
            },
          }),
        ],
      }).then(({ x, y, middlewareData }) => {
        if (!floatingEl) return;
        floatingEl.style.left = `${x}px`;
        floatingEl.style.top = `${y}px`;
        floatingEl.style.visibility = middlewareData.hide?.referenceHidden
          ? "hidden"
          : "visible";
      });
    });
  });

  async function collect(requested: MentionTrigger): Promise<Suggestion[]> {
    // A pasted identifier already names exactly one entity, so there is
    // nothing to search: offer to swap it for a link carrying a readable
    // label, which is what other clients will show.
    if (requested.kind === "identifier") {
      return [await resolveIdentifier(requested.target)];
    }

    // A bare oid does not say what it names, so ask the current repo. Nothing
    // is offered when it resolves to nothing, which is the common case for an
    // oid belonging to some other repo.
    if (requested.kind === "oid") {
      return await resolveOid(requested.oid);
    }

    // An oid typed after a trigger character names something in this repo
    // too, most often a commit. Text search cannot find one: a commit has no
    // candidate list to match against, and nothing else in the list is
    // searched by oid. Falls through when the oid resolves to nothing, so a
    // hex-looking query still reaches the ordinary search.
    if (bareOidPattern.test(requested.query)) {
      const resolved = await resolveOid(requested.query);
      if (resolved.length > 0) return resolved;
    }

    // Section order with nothing typed, most-likely target first: the people
    // who own this repo, then the people the user follows, then what is being
    // worked on in this repo. Once something is typed, text relevance takes
    // over and this ordering only breaks ties.
    const collected: Suggestion[] = [];
    if (requested.scope === "all") {
      collected.push(...(await collectPeople(requested.query)));
    }
    collected.push(...(await collectCobs()));

    // Repos are searchable by name or id but are not suggested by default:
    // referring to a whole repo in a comment is rare, and a list of every repo
    // pushes out the people and COBs that are actually being discussed.
    if (requested.scope === "all" && requested.query !== "") {
      collected.push(...(await collectRepos()));
    }

    return rank(collected, requested.query);
  }

  /** Build the single row offered for an identifier that was typed or pasted. */
  async function resolveIdentifier(target: MentionTarget): Promise<Suggestion> {
    const key = `identifier:${mentionHref(target)}`;

    if (target.type === "node") {
      const alias = await cachedAlias(target.nid).catch(() => undefined);
      const [repo, suggestions] = await Promise.all([
        cachedRepoById(rid).catch(() => undefined),
        cachedSearchAliases(alias ?? "").catch(() => []),
      ]);
      const label = alias ?? truncateId(target.nid);

      return {
        key,
        target,
        label: `@${label}`,
        primary: label,
        secondary: truncateId(target.nid),
        mono: alias === undefined,
        avatar: target.nid,
        badge: badgeFor(target.nid, repo?.delegates ?? [], suggestions),
        haystack: label,
      };
    }

    if (target.type === "repo") {
      const repo = await cachedRepoById(target.rid).catch(() => undefined);
      const name = repo?.payloads["xyz.radicle.project"]?.data.name;
      const label = name ?? target.rid;

      return {
        key,
        target,
        label,
        primary: label,
        secondary: truncateId(target.rid.replace("rad:", "")),
        icon: "repository",
        haystack: label,
      };
    }

    if (target.type === "commit") {
      const commit = await cachedRepoCommit(target.rid, target.oid).catch(
        () => undefined,
      );
      const label = commit?.summary ?? `commit ${formatOid(target.oid)}`;

      return {
        key,
        target,
        label,
        primary: label,
        secondary: formatOid(target.oid),
        icon: "commit",
        haystack: label,
      };
    }

    if (target.kind === "issue") {
      const issue = await cachedIssueById(target.rid, target.oid).catch(
        () => undefined,
      );
      const label = issue?.title ?? `issue ${formatOid(target.oid)}`;

      return {
        key,
        target,
        label,
        primary: label,
        secondary: formatOid(target.oid),
        icon: issue?.state.status === "closed" ? "issue-closed" : "issue",
        haystack: label,
      };
    }

    const patch = await cachedPatchById(target.rid, target.oid).catch(
      () => undefined,
    );
    const label = patch?.title ?? `patch ${formatOid(target.oid)}`;

    return {
      key,
      target,
      label,
      primary: label,
      secondary: formatOid(target.oid),
      icon: patch ? patchIcon[patch.state.status] : "patch",
      haystack: label,
    };
  }

  /**
   * Offer whatever a pasted oid turns out to name in this repo. All three
   * lookups run together rather than in sequence: an oid names at most one of
   * them, so this costs one round trip instead of up to three.
   */
  async function resolveOid(oid: string): Promise<Suggestion[]> {
    const [issue, patch, commit] = await Promise.all([
      cachedIssueById(rid, oid).catch(() => undefined),
      cachedPatchById(rid, oid).catch(() => undefined),
      cachedRepoCommit(rid, oid).catch(() => undefined),
    ]);
    const rows: Suggestion[] = [];

    if (issue) {
      rows.push({
        key: `issue:${issue.id}`,
        target: { type: "cob", kind: "issue", rid, oid },
        label: issue.title,
        primary: issue.title,
        secondary: formatOid(oid),
        icon: issue.state.status === "closed" ? "issue-closed" : "issue",
        haystack: issue.title,
      });
    }
    if (patch) {
      rows.push({
        key: `patch:${patch.id}`,
        target: { type: "cob", kind: "patch", rid, oid },
        label: patch.title,
        primary: patch.title,
        secondary: formatOid(oid),
        icon: patchIcon[patch.state.status],
        haystack: patch.title,
      });
    }
    // A COB's id is the oid of the commit that created it, so an issue or
    // patch id also resolves as a commit — one whose summary is an internal
    // "Create issue". When the oid names a COB, that is what was meant.
    if (commit && rows.length === 0) {
      rows.push({
        key: `commit:${oid}`,
        target: { type: "commit", rid, oid },
        label: commit.summary,
        primary: commit.summary,
        secondary: formatOid(oid),
        icon: "commit",
        haystack: commit.summary,
      });
    }

    return rows;
  }

  /** The strongest trust signal available for a node. */
  function badgeFor(
    nid: string,
    delegates: Author[],
    suggestions: AliasSuggestion[],
  ): Suggestion["badge"] {
    if (delegates.some(d => publicKeyFromDid(d.did) === nid)) return "delegate";
    const match = suggestions.find(s => publicKeyFromDid(s.did) === nid);
    if (match?.isSelf) return "you";
    if (match?.followed) return "following";

    return undefined;
  }

  async function collectPeople(query: string): Promise<Suggestion[]> {
    // Delegates are the people most likely to be addressed, and a delegate
    // this node neither follows nor has gossip for would otherwise be missing
    // from the alias stores entirely.
    const [aliases, repo] = await Promise.all([
      cachedSearchAliases(query),
      cachedRepoById(rid).catch(() => undefined),
    ]);
    const delegates = repo?.delegates ?? [];
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- local dedupe, never rendered reactively
    const seen = new Set<string>();

    const rows = [
      ...delegates.map(author => ({
        ...author,
        followed: false,
        isSelf: false,
      })),
      ...aliases,
    ].flatMap(author => {
      const nid = publicKeyFromDid(author.did);
      if (seen.has(nid)) return [];
      seen.add(nid);
      const label = author.alias ?? truncateId(nid);

      return [
        {
          key: `node:${nid}`,
          target: { type: "node", nid } satisfies MentionTarget,
          label: `@${label}`,
          primary: label,
          // Always shown, never only as an alias fallback: a name on its own
          // does not identify anyone, since any node can claim any alias.
          secondary: truncateId(nid),
          avatar: nid,
          badge: badgeFor(nid, delegates, aliases),
          mono: author.alias === undefined,
          named: author.alias !== undefined,
          haystack: `${author.alias ?? ""} ${nid}`,
        },
      ];
    });

    // Within each group, a node whose alias is known comes first: a row whose
    // name is a truncated node id cannot be recognised, so it is the least
    // useful thing to put at the top.
    const group = (badge: Suggestion["badge"]) =>
      rows
        .filter(row => row.badge === badge && row.badge !== "you")
        .sort((a, b) => Number(b.named) - Number(a.named));

    // A common alias matches dozens of unrelated nodes in the gossip address
    // book — 28 nodes call themselves `rudolfs_seed` on a typical network.
    // Left unbounded they crowd out everything else, so they are bounded and
    // always come after the nodes there is some reason to trust.
    const unvouched = group(undefined).slice(0, maxUnvouchedPeople);
    // Mentioning yourself is the least likely thing to want, so it goes after
    // the other people rather than at the very end of a long scrolling list,
    // where it would be out of reach.
    const self = rows.filter(row => row.badge === "you");

    return [
      ...group("delegate"),
      ...group("following"),
      ...unvouched,
      ...self,
    ].map(({ named: _named, ...row }) => row);
  }

  async function collectRepos(): Promise<Suggestion[]> {
    // Same scope the sidebar lists, so a repo the user has chosen not to see
    // is not suggested here either — and it reuses that cache entry rather
    // than provoking a second enumeration of storage.
    const repos = await cachedListReposSummary(repoListScope.value).catch(
      () => [],
    );
    // Only the tie-break matters here, since repos are offered solely in
    // response to a typed query and fuzzy matching does the real ordering.
    const ordered = [...repos].sort(
      (a, b) =>
        Number(b.rid === rid) - Number(a.rid === rid) ||
        a.name.localeCompare(b.name),
    );

    return ordered.map(repo => ({
      key: `repo:${repo.rid}`,
      target: { type: "repo", rid: repo.rid } satisfies MentionTarget,
      label: repo.name,
      primary: repo.name,
      secondary: truncateId(repo.rid.replace("rad:", "")),
      icon: "repository" as IconName,
      haystack: `${repo.name} ${repo.rid}`,
    }));
  }

  async function collectCobs(): Promise<Suggestion[]> {
    const [allIssues, allPatches] = await Promise.all([
      cachedListIssueCandidates(rid).catch(() => []),
      cachedListPatchCandidates(rid).catch(() => []),
    ]);
    // Issues and patches are merged and ordered by recency rather than listed
    // one type after the other: what a comment refers to is whatever is being
    // worked on right now, and which kind it happens to be is incidental. The
    // icons already tell them apart. Fuzzy matching reorders these as soon as
    // anything is typed, so this is what a bare `@` or `#` shows.
    const rows = [
      ...allIssues.map(issue => ({
        timestamp: issue.timestamp,
        key: `issue:${issue.id}`,
        target: {
          type: "cob",
          kind: "issue",
          rid,
          oid: issue.id,
        } satisfies MentionTarget,
        label: issue.title,
        primary: issue.title,
        secondary: formatOid(issue.id),
        icon: (issue.state.status === "open"
          ? "issue"
          : "issue-closed") as IconName,
        haystack: `${issue.title} ${issue.id}`,
      })),
      ...allPatches.map(patch => ({
        timestamp: patch.timestamp,
        key: `patch:${patch.id}`,
        target: {
          type: "cob",
          kind: "patch",
          rid,
          oid: patch.id,
        } satisfies MentionTarget,
        label: patch.title,
        primary: patch.title,
        secondary: formatOid(patch.id),
        icon: patchIcon[patch.state.status],
        haystack: `${patch.title} ${patch.id}`,
      })),
    ];

    return rows
      .sort((a, b) => b.timestamp - a.timestamp)
      .map(({ timestamp: _timestamp, ...row }) => row);
  }

  const patchIcon: Record<string, IconName> = {
    draft: "patch-draft",
    open: "patch",
    archived: "patch-archived",
    merged: "patch-merged",
  };

  function rank(collected: Suggestion[], query: string): Suggestion[] {
    if (query === "") return collected.slice(0, maxSuggestions);

    const order = new Map(collected.map((row, index) => [row.key, index]));

    return (
      fuzzysort
        .go(query, collected, { key: "haystack", threshold: 0.4 })
        .map(result => ({ row: result.obj, score: result.score }))
        // Ten nodes can share one alias and therefore score identically. Falling
        // back to the collected order keeps delegates and followed nodes ahead
        // of anonymous ones instead of leaving it to sort stability.
        .sort(
          (a, b) =>
            b.score - a.score ||
            (order.get(a.row.key) ?? 0) - (order.get(b.row.key) ?? 0),
        )
        .slice(0, maxSuggestions)
        .map(result => result.row)
    );
  }

  function select(suggestion: Suggestion) {
    if (!trigger) return;
    onselect({
      start: trigger.start,
      end: trigger.end,
      markdown: `${mentionMarkdown(suggestion.target, suggestion.label, config)} `,
    });
    suggestions = [];
  }

  $effect(() => {
    registerKeydown(handleKeydown);
  });

  // Match the scrollbar styling the app's other dropdowns use.
  $effect(() => {
    if (!scrollEl) return;
    const [initialize] = useOverlayScrollbars({
      options: () => ({
        scrollbars: {
          theme: "global-os-theme-radicle",
          autoHide: "scroll",
        },
      }),
      defer: true,
    });

    initialize({ target: scrollEl });
  });

  // Arrow keys can move the selection past the visible rows now that the list
  // scrolls, so follow it.
  $effect(() => {
    if (!open) return;
    rowElements[activeIndex]?.scrollIntoView({ block: "nearest" });
  });

  function handleKeydown(event: KeyboardEvent): boolean {
    if (!open) return false;

    switch (event.key) {
      case "ArrowDown":
        activeIndex = (activeIndex + 1) % suggestions.length;
        return true;
      case "ArrowUp":
        activeIndex =
          (activeIndex - 1 + suggestions.length) % suggestions.length;
        return true;
      case "Enter":
      case "Tab": {
        const suggestion = suggestions[activeIndex];
        if (!suggestion) return false;
        select(suggestion);
        return true;
      }
      case "Escape":
        dismissedAt = trigger?.start;
        return true;
      default:
        return false;
    }
  }
</script>

<style>
  .suggestions {
    position: fixed;
    top: 0;
    left: 0;
    visibility: hidden;
    /* Above the fullscreen modal portal (300), so the dropdown is usable when
       a comment box is inside a modal, e.g. the new issue form. */
    z-index: 400;
    display: flex;
    flex-direction: column;
    min-width: 16rem;
    max-width: 28rem;
    padding: 0.25rem;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--border-radius-md);
    /* Matches `ContextMenu`, the app's other floating keyboard-driven menu:
       white in light mode, so the panel reads as lifted off the page rather
       than as a muddy tint of it. */
    background-color: var(--color-surface-canvas);
    box-shadow: var(--elevation-low);
  }
  .suggestions-scroll {
    /* The floating element's max-height is set by floating-ui to whatever
       space the caret leaves, so the list takes it and scrolls inside. */
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }
  .suggestion {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.5rem;
    border: 0;
    border-radius: var(--border-radius-sm);
    background: none;
    color: var(--color-text-primary);
    font: var(--txt-body-m-regular);
    text-align: left;
    cursor: pointer;
  }
  /* One step off the panel, matching `DropdownListItem`. There is no separate
     hover rule: pointing at a row makes it the selected one, so the mouse and
     the arrow keys drive the same single highlight. */
  .suggestion.selected {
    background-color: var(--color-surface-subtle);
  }
  .suggestion-primary {
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
  /* Always an identifier (a truncated node or object id), never a name. */
  .suggestion-secondary {
    margin-left: auto;
    padding-left: 0.5rem;
    color: var(--color-text-secondary);
    font: var(--txt-body-s-regular);
    font-family: var(--font-family-code);
  }
  /* A row whose name is itself a raw id, because no alias is known for it. */
  .suggestion-primary.mono {
    font-family: var(--font-family-code);
  }
  .suggestion-icon {
    display: flex;
    align-items: center;
    color: var(--color-text-secondary);
  }
  .suggestion-badge {
    flex-shrink: 0;
    padding: 0 0.25rem;
    border-radius: var(--border-radius-tiny);
    background-color: var(--color-fill-ghost);
    color: var(--color-text-secondary);
  }
</style>

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="suggestions"
    bind:this={floatingEl}
    use:portal
    onmousedown={event => {
      // Reaching for the scrollbar must not blur the textarea, which would
      // close the very list being scrolled. Rows handle their own selection.
      event.preventDefault();
    }}>
    <div class="suggestions-scroll" bind:this={scrollEl}>
      {#each suggestions as suggestion, index (suggestion.key)}
        <button
          type="button"
          bind:this={rowElements[index]}
          class="suggestion"
          class:selected={index === activeIndex}
          onmouseenter={() => (activeIndex = index)}
          onmousedown={event => {
            // Keep focus in the textarea so the caret does not jump on insert.
            event.preventDefault();
            select(suggestion);
          }}>
          {#if suggestion.avatar}
            <UserAvatar nodeId={suggestion.avatar} styleWidth="1rem" />
          {:else if suggestion.icon}
            <span class="suggestion-icon">
              <Icon name={suggestion.icon} />
            </span>
          {/if}
          <span class="suggestion-primary" class:mono={suggestion.mono}>
            {suggestion.primary}
          </span>
          {#if suggestion.badge}
            <span class="suggestion-badge txt-body-s-medium">
              {suggestion.badge}
            </span>
          {/if}
          {#if suggestion.secondary}
            <span class="suggestion-secondary">{suggestion.secondary}</span>
          {/if}
        </button>
      {/each}
    </div>
  </div>
{/if}
