import type { Root } from "hast";

import onigurumaWASMUrl from "vscode-oniguruma/release/onig.wasm?url";
import sourceAsciiDoc from "@wooorm/starry-night/text.html.asciidoc";
import sourceDockerfile from "@wooorm/starry-night/source.dockerfile";
import sourceErlang from "@wooorm/starry-night/source.erlang";
import sourceSolidity from "@wooorm/starry-night/source.solidity";
import sourceSvelte from "@wooorm/starry-night/source.svelte";
import sourceSass from "@wooorm/starry-night/source.sass";
import sourceToml from "@wooorm/starry-night/source.toml";
import sourceTsx from "@wooorm/starry-night/source.tsx";
import sourceNix from "@wooorm/starry-night/source.nix";
import sourceGitconfig from "@wooorm/starry-night/source.gitconfig";
import sourceGitignore from "@wooorm/starry-night/source.gitignore";
import sourceGitrevlist from "@wooorm/starry-night/source.git-revlist";
import sourceGitattributes from "@wooorm/starry-night/source.gitattributes";
import sourceJson from "@wooorm/starry-night/source.json";
import sourceNpmrc from "@wooorm/starry-night/source.ini.npmrc";
import sourceGradle from "@wooorm/starry-night/source.groovy.gradle";
import sourceBatchfile from "@wooorm/starry-night/source.batchfile";
import sourceEditorconfig from "@wooorm/starry-night/source.editorconfig";
import sourceHaproxyConfig from "@wooorm/starry-night/source.haproxy-config";
import sourceDotenv from "@wooorm/starry-night/source.dotenv";
import sourceZig from "@wooorm/starry-night/source.zig";
import textHtmlVue from "@wooorm/starry-night/text.html.vue";
import textHtmlDjango from "@wooorm/starry-night/text.html.django";
import textRobotsTxt from "@wooorm/starry-night/text.robots-txt";
import textZoneFile from "@wooorm/starry-night/text.zone_file";
import etc from "@wooorm/starry-night/etc";
import goMod from "@wooorm/starry-night/go.mod";
import goSum from "@wooorm/starry-night/go.sum";

import { createStarryNight, common, type Grammar } from "@wooorm/starry-night";

export { type Root };

export const grammars = [
  ...common,
  sourceAsciiDoc,
  sourceToml,
  sourceErlang,
  sourceSolidity,
  sourceSvelte,
  sourceSass,
  sourceTsx,
  sourceDockerfile,
  sourceNix,
  sourceGitconfig,
  sourceGitignore,
  sourceGitrevlist,
  sourceGitattributes,
  sourceNpmrc,
  sourceGradle,
  sourceBatchfile,
  sourceEditorconfig,
  sourceHaproxyConfig,
  sourceDotenv,
  sourceZig,
  textHtmlVue,
  textHtmlDjango,
  textRobotsTxt,
  textZoneFile,
  etc,
  goMod,
  goSum,
  {
    extensions: [".hintrc"],
    names: ["json"],
    patterns: [sourceJson],
    scopeName: "source.json",
  },
  {
    extensions: [
      ".npmignore",
      ".eslintignore",
      ".dockerignore",
      ".nuxtignore",
      ".vscodeignore",
    ],
    names: ["ignore"],
    patterns: [sourceGitignore],
    scopeName: "source.gitignore",
  },
  {
    extensions: [".sample", ".example", ".template"],
    names: [".env.sample", ".env.example", ".env.template"],
    patterns: [sourceDotenv],
    scopeName: "source.dotenv",
  },
  {
    extensions: [".mod"],
    names: ["go.mod"],
    patterns: [goMod],
    scopeName: "go.mode",
  },
  {
    extensions: [".sum"],
    names: ["go.sum"],
    patterns: [goSum],
    scopeName: "go.sum",
  },
  // A grammar that doesn't do any parsing, but needed for files without a known filetype.
  {
    extensions: [""],
    names: ["raw-format"],
    patterns: [],
    scopeName: "text.raw",
  },
] satisfies Grammar[];

let starryNight: Awaited<ReturnType<typeof createStarryNight>>;

export async function highlight(
  content: string,
  grammar: string,
): Promise<Root> {
  if (starryNight === undefined) {
    starryNight = await createStarryNight(grammars, {
      getOnigurumaUrlFetch: () => new URL(onigurumaWASMUrl, import.meta.url),
    });
  }
  const scope = starryNight.flagToScope(grammar);
  return starryNight.highlight(content, scope ?? "text.raw");
}
