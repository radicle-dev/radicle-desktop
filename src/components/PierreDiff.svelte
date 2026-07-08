<script lang="ts" module>
  import type { WorkerPoolManager } from "@pierre/diffs/worker";

  import { getOrCreateWorkerPoolSingleton } from "@pierre/diffs/worker";
  // Vite bundles Pierre's highlighting worker (and its Shiki/WASM deps) into a
  // self-contained, same-origin worker module.
  import PierreDiffWorker from "@pierre/diffs/worker/worker.js?worker";

  const themes = { dark: "github-dark", light: "github-light" } as const;

  // Shiki tokenization is the expensive part of rendering a large diff. Run it
  // in a shared worker pool so it stays off the main thread; if the worker
  // cannot be created (e.g. blocked by CSP), fall back to main-thread
  // highlighting rather than failing to render.
  let workerPoolResolved = false;
  let workerPool: WorkerPoolManager | undefined;
  function getWorkerPool(): WorkerPoolManager | undefined {
    if (workerPoolResolved) {
      return workerPool;
    }
    workerPoolResolved = true;
    try {
      workerPool = getOrCreateWorkerPoolSingleton({
        poolOptions: {
          workerFactory: () => new PierreDiffWorker(),
          poolSize: 2,
        },
        highlighterOptions: { theme: themes },
      });
    } catch (error) {
      console.error(
        "PierreDiff: worker pool unavailable; highlighting on the main thread",
        error,
      );
      workerPool = undefined;
    }
    return workerPool;
  }
</script>

<script lang="ts">
  import type {
    CodeViewItem,
    CodeViewSlotSnapshot,
    FileDiffLoadedFiles,
    FileDiffMetadata,
  } from "@pierre/diffs";
  import type { Snippet } from "svelte";

  import { CodeView, CUSTOM_HEADER_SLOT_ID } from "@pierre/diffs";
  import { mount, unmount, untrack } from "svelte";

  import { fontSettings } from "@app/lib/appearance.svelte";
  import { parsePatch } from "@app/lib/pierreParse";

  import DiffFileHeader from "@app/components/DiffFileHeader.svelte";
  import { DiffFileHeaderState } from "@app/components/diffFileHeaderState.svelte";
  import { theme } from "@app/components/ThemeSwitch.svelte";

  interface Props {
    // Unified `git diff` patch text; Pierre renders the diff from this.
    patch: string;
    diffStyle?: "unified" | "split";
    disableBackground?: boolean;
    disableLineNumbers?: boolean;
    wordWrap?: boolean;
    diffIndicators?: "classic" | "bars" | "none";
    lineDiffType?: "word-alt" | "word" | "char" | "none";
    // Content shown above the first file. Handed to CodeView via its
    // `renderCodeViewHeader` API, which renders it non-virtualized at the top of
    // the scroll content, so it scrolls out of view leaving only the sticky file
    // headers pinned. CodeView measures its height itself (ResizeObserver).
    header?: Snippet;
    // Fetch the full old/new contents of a file (by its new-side path) on
    // demand. When provided, it is adapted into Pierre's `loadDiffFiles`
    // loader: the native hunk-expand markers become live, and clicking one
    // makes Pierre hydrate that file's full content lazily (one click, no
    // pre-loading).
    loadFullFile?: (
      path: string,
    ) => Promise<{ oldContents: string; newContents: string }>;
    // Files with no renderable text diff, keyed by new-side path. Pierre has no
    // binary/empty concept, so these get a header note and no expand caret.
    fileNotes?: ReadonlyMap<string, "binary" | "empty">;
    // Per-file change status, keyed by new-side path — renders as a plain text
    // label after the filename (nothing for a plain modification).
    fileStatuses?: ReadonlyMap<
      string,
      "added" | "deleted" | "modified" | "moved" | "copied"
    >;
    // Lazily fetch a single file's raw diff text (by new-side path) for the
    // header's copy-to-clipboard / save-to-disk actions.
    fileDiffText?: (path: string) => Promise<string>;
    // Unique id for this diff (e.g. the commit or revision id). Used as the
    // Pierre `cacheKey` prefix so the shared worker-pool highlight cache does
    // not collide across diffs that share file paths (see `parsePatch`).
    cacheKeyPrefix?: string;
  }

  const {
    patch,
    diffStyle = "unified",
    disableBackground = false,
    disableLineNumbers = false,
    wordWrap = false,
    diffIndicators = "bars",
    lineDiffType = "word-alt",
    header = undefined,
    loadFullFile = undefined,
    fileNotes = undefined,
    fileStatuses = undefined,
    fileDiffText = undefined,
    cacheKeyPrefix = undefined,
  }: Props = $props();

  // Pierre's first-party lazy content loader. It calls this the first time a
  // partial (patch-parsed) file is expanded, hydrates the file's full content
  // in place (flips `isPartial`, reuses the highlight cache), and drives its own
  // expand markers. Stable reference so option updates never re-trigger loader
  // wiring. Pierre only invokes it for changed/renamed files (added/deleted
  // already carry a side).
  const loadDiffFiles = $derived(
    loadFullFile
      ? async (fileDiff: FileDiffMetadata): Promise<FileDiffLoadedFiles> => {
          const { oldContents, newContents } = await loadFullFile(
            fileDiff.name,
          );
          return {
            oldFile: {
              name: fileDiff.prevName ?? fileDiff.name,
              contents: oldContents,
            },
            newFile: { name: fileDiff.name, contents: newContents },
          };
        }
      : undefined,
  );

  // Pierre only emits unprefixed `user-select: none` on its gutters, which
  // WebKit ignores — so with the subtree opted into selection, gutters would
  // become selectable there. Re-assert it with the `-webkit-` prefix inside the
  // shadow root (injected into the highest `@layer unsafe`, so it wins).
  const gutterUnsafeCSS = `
    [data-column-number],
    [data-content-buffer],
    [data-gutter-buffer],
    [data-separator-wrapper],
    [data-separator-content] {
      -webkit-user-select: none;
      user-select: none;
    }
  `;

  // Per-file "card" treatment. Each file is its own `<diffs-container>` custom
  // element (own shadow root), so `:host` — reachable from the per-file
  // `unsafeCSS` — is the whole-file box (header + diff). Horizontal spacing via
  // margin; vertical spacing between files stays on `layout.gap` (vertical
  // margins fight the virtualizer's height math). `background: transparent`
  // gives the "no background" look — unchanged lines then show the app canvas,
  // and only the border + change tints define the card.
  //
  // The border is a `box-shadow`, NOT a real border: the virtualizer computes
  // each file's height from metrics (for `overflow: scroll` it never measures
  // the container), so a real border's 2px would drift the layout model from
  // the DOM — corrupting scroll-to targets and the sticky math for short
  // collapsed files. A box-shadow adds zero layout height. It must be an
  // *outset* ring: the diff surface now equals the view background, so the
  // opaque header/line backgrounds would paint over an inset shadow. (The
  // header/body divider is an inset shadow on `DiffFileHeader`'s own row, so it
  // adds no height and keeps the header matching the metric.)
  //
  // Corners are rounded per-element, not with `overflow: hidden` on `:host`
  // (that would make the header stick within the file box instead of the scroll
  // viewport). The header's opaque sticky background would otherwise square off
  // over the rounded box-shadow, so we round its top corners; the body wrapper
  // (a sibling of the header, not the sticky element) gets rounded + clipped
  // bottom corners — safe because the horizontal scroll lives on the inner
  // `[data-code]`. A header-only card (collapsed, binary, or empty — no rendered
  // body) rounds all its header corners, driven by the `data-app-no-body`
  // attribute we set on the host.
  //
  // Context separators ("N unmodified lines" / "More context" bars) are also
  // restyled here: recessed one surface level below the diff, 4px corners, no
  // hover underline, and Pierre's expand glyph swapped for the app's caret (an
  // app-file chevron painted via a CSS mask, like the tree's folder glyphs).
  const caretMask =
    "url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3E%3Cpath fill='black' d='M4 5.29297L8 9.29297L12 5.29297L12.707 6L8 10.707L3.29297 6L4 5.29297Z'/%3E%3C/svg%3E\")";
  const cardUnsafeCSS = `
    :host {
      margin: 0 0.5rem;
      border-radius: var(--border-radius-md);
      box-shadow: 0 0 0 1px var(--color-border-subtle);
      background: transparent;
      /* Force the diff surface to the app's view background. The Shiki theme
         sets \`--diffs-light-bg\`/\`--diffs-dark-bg\` on \`:host\` directly, so
         overriding those by inheritance fails; overriding the resolved
         \`--diffs-bg\` from this higher \`@layer\` wins. The header, lines and
         gutter read it directly, and the separator/context/change tints are
         \`color-mix\`es off it — so everything sits on the view background with
         only the border, syntax and add/del/mod tints standing out, while the
         separators keep their subtle Pierre tint (now mixed off our base). */
      --diffs-bg: var(--color-surface-canvas);
      /* Lift the context separators one surface level above the diff. */
      --diffs-bg-separator-override: var(--color-surface-subtle);
    }
    [data-diffs-header] {
      border-top-left-radius: var(--border-radius-md);
      border-top-right-radius: var(--border-radius-md);
    }
    :host([data-app-no-body]) [data-diffs-header] {
      border-radius: var(--border-radius-md);
    }
    [data-diffs-header] ~ [data-diff],
    [data-diffs-header] ~ [data-file] {
      border-bottom-left-radius: var(--border-radius-md);
      border-bottom-right-radius: var(--border-radius-md);
      overflow: hidden;
    }
    /* Context separators: 4px corners, no hover underline. */
    [data-separator-content],
    [data-separator="line-info"] [data-separator-wrapper] {
      border-radius: var(--border-radius-md);
    }
    [data-separator-content]:hover {
      text-decoration: none;
    }
    /* Swap Pierre's expand glyph for the app caret (down; flipped when the
       separator expands upward). */
    [data-expand-button] [data-icon] {
      display: none;
    }
    [data-expand-button]::before {
      content: "";
      width: 1rem;
      height: 1rem;
      background-color: currentColor;
      -webkit-mask: center / contain no-repeat ${caretMask};
      mask: center / contain no-repeat ${caretMask};
    }
    [data-expand-button][data-expand-up]::before {
      transform: rotate(180deg);
    }
  `;

  // Kept out of `$state` proxying (it is an external stateful instance), but
  // still reactive on reassignment so the option effects re-run once it exists.
  let container = $state<HTMLElement>();
  let headerEl = $state<HTMLElement>();
  // Handed to CodeView's `renderCodeViewHeader`: it mounts this element at the
  // top of the scroll content and tracks its height itself. Stable reference so
  // option updates do not re-trigger header reconciliation.
  const renderHeader = (): HTMLElement | undefined => headerEl;
  // `DiffFileHeader`'s row height (`2.5rem`), fed to Pierre as the exact
  // `diffHeaderHeight` metric. It must be constant across all files — Pierre
  // estimates each file's position from this single metric (not the measured
  // header), so a varying header height drifts scroll-to-file. Expanded files
  // get a 1px divider above the body but no extra height (inset shadow). The
  // app's root font size is user-configurable (`--font-size`, 14–24px) and one
  // rem equals `fontSettings.size` px, so scale with it rather than hardcoding
  // pixels.
  function fileHeaderHeight(): number {
    return fontSettings.size * 2.5;
  }
  // Diff line height, rounded to a whole pixel. `1.25rem` is fractional at some
  // font sizes (e.g. 17.5px at a 14px root) and WebKit snaps each rendered row
  // to a whole pixel, so a fractional metric drifts scroll-to-file down a long
  // diff. Feed this same integer to both the CSS line height and Pierre's
  // `itemMetrics.lineHeight` so the rendered rows and the virtualization model
  // agree exactly.
  const lineHeightPx = $derived(Math.round(fontSettings.size * 1.25));
  let view = $state.raw<CodeView | undefined>(undefined);
  // The parsed files, kept so `scrollToFile` can map a path to its diff item.
  let parsedFiles = $state.raw<FileDiffMetadata[]>([]);

  // Scroll the diff to a file by its path (item id is its index in the patch).
  export function scrollToFile(path: string): void {
    const index = parsedFiles.findIndex(file => file.name === path);
    if (index < 0) {
      return;
    }
    view?.scrollTo({ type: "item", id: String(index), align: "start" });
  }

  // Collapse or expand every file at once (drives the topbar toggle). Renders
  // coalesce, so one pass over all items is fine.
  export function setAllCollapsed(collapsed: boolean): void {
    const instance = view;
    if (!instance) {
      return;
    }
    for (let index = 0; index < parsedFiles.length; index++) {
      const item = instance.getItem(String(index));
      if (item) {
        instance.updateItem({
          ...item,
          collapsed,
          version: (item.version ?? 0) + 1,
        });
      }
    }
  }

  // `renderCustomHeader` makes Pierre render only our custom-header slot (it
  // drops its own filename/status-icon/stats), so there is no built-in chrome
  // to override with unsafeCSS. A stable no-op marker enables the slot; we fill
  // it ourselves (see `syncHeaderSlots`). Pierre still supplies the sticky
  // header container and the body's context-expand markers.
  const renderCustomHeader = (): undefined => undefined;

  // One `DiffFileHeader` Svelte component per rendered file, keyed by CodeView
  // item id (not by host element). Pierre pools its `<diffs-container>` host
  // elements, and in container-managed mode its `cleanElement` deliberately
  // leaves our slotted light-DOM child attached on release — so a host only
  // re-enters the reuse pool once *we* detach our wrapper (its `isElementClean`
  // gate is `childNodes.length === 0`). We therefore reconcile against every
  // snapshot: mount for newly-rendered ids, and unmount + detach the wrapper for
  // ids that left, which frees the vacated host for reuse and prevents leaking
  // components across scroll. This mirrors Pierre's own React wrapper, which
  // portals slot content per item id and unmounts it when the id drops out.
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- imperative id→component lookup, never rendered reactively
  const mountedHeaders = new Map<
    string,
    {
      state: DiffFileHeaderState;
      instance: ReturnType<typeof mount>;
      // The slot wrapper we appended to the host; detached on removal so the
      // host becomes child-free and poolable again.
      target: HTMLElement;
      // The host this wrapper currently lives in, to detect a recycle.
      host: HTMLElement;
    }
  >();

  function unmountHeaders(): void {
    for (const { instance, target } of mountedHeaders.values()) {
      void unmount(instance);
      target.remove();
    }
    mountedHeaders.clear();
  }

  // The virtualized CodeView is container-managed, so it emits a snapshot of
  // rendered items through a slot coordinator (`{ items, header, footer }`). For
  // each rendered file we ensure a `DiffFileHeader` is mounted into its
  // custom-header slot and push the current file data into its reactive state;
  // afterwards we tear down headers whose file is no longer rendered.
  function syncHeaderSlots(
    snapshot: CodeViewSlotSnapshot<undefined> | undefined,
  ): void {
    const items = snapshot?.items;
    if (!items) {
      return;
    }
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- transient local set, never reactive
    const present = new Set<string>();
    for (const rendered of items) {
      if (rendered.type !== "diff") {
        continue;
      }
      present.add(rendered.id);
      const host = rendered.element;
      let entry = mountedHeaders.get(rendered.id);
      if (!entry) {
        const target = document.createElement("div");
        target.slot = CUSTOM_HEADER_SLOT_ID;
        target.dataset.pierreHeader = "";
        // Transparent wrapper: the component's own `.header` participates in the
        // slot layout directly, without an extra box.
        target.style.display = "contents";
        host.appendChild(target);
        const state = new DiffFileHeaderState();
        const instance = mount(DiffFileHeader, { target, props: { state } });
        entry = { state, instance, target, host };
        mountedHeaders.set(rendered.id, entry);
      } else if (entry.host !== host) {
        // Defensive: if Pierre ever moves an id's content onto a different
        // pooled host without dropping it from the snapshot, follow it (as the
        // React wrapper's portal does when its container prop changes).
        host.appendChild(entry.target);
        entry.host = host;
      }

      const { item } = rendered;
      const fileDiff = item.fileDiff;
      entry.state.fileDiff = fileDiff;
      entry.state.status = fileStatuses?.get(fileDiff.name);
      // Binary comes from the backend (Pierre can't tell binary from empty —
      // both have no hunks). Any other zero-hunk file (empty/mode-only/pure
      // rename adds like `.gitkeep`) is treated as empty regardless of how the
      // backend labelled its diff.
      const note =
        fileNotes?.get(fileDiff.name) ??
        (fileDiff.hunks.length === 0 ? "empty" : undefined);
      entry.state.note = note;
      entry.state.collapsed = item.collapsed === true;
      // Flag header-only cards (collapsed, binary, or empty — anything with no
      // rendered body) so `cardUnsafeCSS` rounds all four corners of the header
      // instead of only the top two.
      host.toggleAttribute(
        "data-app-no-body",
        item.collapsed === true || note !== undefined,
      );
      entry.state.text = fileDiffText
        ? () => fileDiffText(fileDiff.name)
        : undefined;
      entry.state.onToggleCollapse = () => {
        // Bump `version`: CodeView ignores an updateItem whose version is
        // unchanged (see syncItemRecord).
        view?.updateItem({
          ...item,
          collapsed: !(item.collapsed === true),
          version: (item.version ?? 0) + 1,
        });
      };
    }

    // Reconcile removals: unmount headers whose file left the snapshot and
    // detach their slot wrapper so the vacated host is child-free — only then
    // does Pierre's pool promote it for reuse (see the map comment above).
    for (const [id, entry] of mountedHeaders) {
      if (!present.has(id)) {
        void unmount(entry.instance);
        entry.target.remove();
        mountedHeaders.delete(id);
      }
    }
  }

  function options(
    themeType: "dark" | "light",
    style: "unified" | "split",
    headerHeight: number,
  ) {
    return {
      theme: themes,
      themeType,
      diffStyle: style,
      disableBackground,
      disableLineNumbers,
      overflow: (wordWrap ? "wrap" : "scroll") as "wrap" | "scroll",
      diffIndicators,
      lineDiffType,
      stickyHeaders: true,
      unsafeCSS: gutterUnsafeCSS + cardUnsafeCSS,
      layout: { paddingTop: 0, paddingBottom: 8, gap: 8 },
      // Own the whole file header: Pierre renders only our custom-header slot
      // (mounted per file in `syncHeaderSlots`), so no built-in icon/stat chrome
      // to fight with CSS.
      renderCustomHeader,
      // Lazy full-file hydration for context expansion. Pierre shows live
      // expand markers on the partial diff and calls this only when the user
      // expands — one click, nothing loaded up front.
      ...(loadDiffFiles ? { loadDiffFiles } : {}),
      // Commit metadata as a non-virtualized header at the top of the scroll
      // content. It scrolls away with the diff and CodeView measures its height
      // itself, so no `paddingTop` reservation is needed.
      ...(header ? { renderCodeViewHeader: renderHeader } : {}),
      // Pierre estimates each file's position from these two metrics, so they
      // must equal the real rendered heights or scroll-to-file drifts.
      // `diffHeaderHeight` is passed statically (not measured) so it can't lag
      // behind a font-size change. `lineHeight` must match `--diffs-line-height`
      // (`lineHeightPx`); Pierre's default is a fixed 20px, which only matches at
      // the 16px root font.
      itemMetrics: {
        diffHeaderHeight: headerHeight,
        lineHeight: lineHeightPx,
      },
    };
  }

  function buildItems(files: FileDiffMetadata[]): CodeViewItem[] {
    return files.map((fileDiff, index) => ({
      id: String(index),
      type: "diff",
      fileDiff,
    }));
  }

  // Structural rebuild: only when the patch or layout style changes.
  $effect(() => {
    const el = container;
    if (!el) {
      return;
    }
    const p = patch;
    // `diffStyle` is applied via `setOptions` below, so reading it untracked
    // keeps a split/unified toggle from re-parsing and rebuilding the diff.
    const instance = new CodeView(
      untrack(() => options($theme, diffStyle, fileHeaderHeight())),
      // Off-thread highlighting; falls back to the main thread if unavailable.
      getWorkerPool(),
      // The container element is owned by this component.
      true,
    );
    instance.setup(el);
    // Header content in the virtualized path is delivered via a slot
    // coordinator, not the render* options.
    instance.setSlotCoordinator({
      hasHeaderRenderers: true,
      hasAnnotationRenderer: false,
      hasGutterRenderer: false,
      onSnapshotChange: syncHeaderSlots,
    });
    // Mount empty first so the header paints immediately, then fill in the
    // files once the patch has been parsed off the main thread.
    instance.render(true);
    view = instance;

    let cancelled = false;
    // `cacheKeyPrefix` changes in lockstep with `patch` (both derive from the
    // commit), so read it untracked — the `patch` dependency already rebuilds.
    parsePatch(
      p,
      untrack(() => cacheKeyPrefix),
    )
      .then(files => {
        if (cancelled) {
          return;
        }
        parsedFiles = files;
        instance.setItems(buildItems(files));
        instance.render(true);
      })
      .catch((error: unknown) => {
        if (!cancelled) {
          console.error("PierreDiff: failed to parse patch", error);
        }
      });

    return () => {
      cancelled = true;
      view = undefined;
      unmountHeaders();
      instance.cleanUp();
    };
  });

  // Theme toggle, layout style, font-size (header-height), and diff-preference
  // changes: update options in place instead of rebuilding (and re-parsing) the
  // diff.
  $effect(() => {
    const themeType = $theme;
    const style = diffStyle;
    // Read reactively (outside `untrack`) so a font-size change re-runs this
    // effect and pushes the new header height to Pierre.
    const headerHeight = fileHeaderHeight();
    // Touch the preference props so a change re-runs this effect; `options()`
    // reads their current values.
    void disableBackground;
    void disableLineNumbers;
    void wordWrap;
    void diffIndicators;
    void lineDiffType;
    const instance = view;
    if (!instance) {
      return;
    }
    untrack(() => {
      instance.setOptions(options(themeType, style, headerHeight));
      instance.render(true);
    });
  });

  // `lineDiffType` (inline word/char highlighting) is computed in the highlight
  // worker, and the shared pool caches results keyed by a render-options version
  // that only `setRenderOptions` bumps — `CodeView.setOptions` never reaches the
  // worker, so changing it there has no effect (a stale cache hit is served).
  // Push it to the pool instead (which also clears the highlight cache) and
  // re-render to recompute decorations. The main-thread fallback path (no pool)
  // reads `lineDiffType` from the CodeView options set above.
  $effect(() => {
    const ln = lineDiffType;
    const instance = view;
    const pool = getWorkerPool();
    if (!instance || !pool) {
      return;
    }
    void pool.setRenderOptions({ lineDiffType: ln }).then(() => {
      // Guard against a teardown between the async resolve and here.
      if (view === instance) {
        instance.render(true);
      }
    });
  });
</script>

<style>
  .pierre-diff {
    flex: 1;
    min-height: 0;
    min-width: 0;
    /* Feed the app's configured code font and sizing into Pierre (its `:host`
       reads these custom properties, which inherit across the shadow boundary).
       Matches `--txt-code-regular`. The line height is set on the element itself
       (`--diffs-line-height`, below) as a whole-pixel value that mirrors
       `itemMetrics.lineHeight`, so the rendered rows and Pierre's model agree. */
    --diffs-font-family: var(--font-family-code);
    --diffs-font-size: 0.875rem;

    /* Chrome colours from our design tokens. Syntax token colours still come
       from the Shiki theme (github light/dark); these only retint the diff
       surfaces, line numbers, and add/delete/modify accents so the frame
       matches the app without changing the highlighting look and feel. */
    /* Text colour from our tokens. The base *surface* cannot be set here: the
       Shiki theme (github-dark/light) writes `--diffs-light-bg`/`--diffs-dark-bg`
       directly onto the shadow `:host`, which beats any value inherited from this
       light-DOM ancestor. So `--diffs-bg` is forced to the app's view background
       in `cardUnsafeCSS` (a higher `@layer`) instead — every neutral surface and
       the change-tint mixes derive from it. */
    --diffs-light: var(--color-text-primary);
    --diffs-dark: var(--color-text-primary);
    /* Line numbers. */
    --diffs-fg-number-override: var(--color-text-tertiary);
    /* Add / delete / modify accents feed the line-tint mixes. */
    --diffs-addition-color-override: var(--color-feedback-success-text);
    --diffs-deletion-color-override: var(--color-feedback-error-text);
    --diffs-modified-color-override: var(--color-text-brand);
    /* CodeView attaches its scroll listener to this element but does not set
       `overflow` itself, so it must be the scroll viewport. */
    overflow-y: auto;
    /* The app disables selection globally on `html` and opts in per element.
       `user-select` inherits across the shadow boundary, so opt the whole
       Pierre subtree back in; its own CSS keeps gutters/line numbers
       unselectable. */
    -webkit-user-select: text;
    user-select: text;
  }
</style>

<div
  bind:this={container}
  class="pierre-diff"
  style:--diffs-line-height="{lineHeightPx}px">
  {#if header}
    <div bind:this={headerEl} class="pierre-diff-header">
      {@render header()}
    </div>
  {/if}
</div>
