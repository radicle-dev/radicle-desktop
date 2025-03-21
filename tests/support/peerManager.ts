import type * as Execa from "execa";

import * as Fs from "node:fs/promises";
import * as Path from "node:path";
import * as Stream from "node:stream";
import waitOn from "wait-on";
import { execa } from "execa";
import { logPrefix } from "@tests/support/logPrefix.js";

interface PeerManagerParams {
  dataPath: string;
  radSeed: string;
  // Name for easy identification. Used on file system and in logs.
  name: string;
  gitOptions?: Record<string, string>;
  outputLog: Stream.Writable;
}

export interface PeerManager {
  createPeer(params: {
    name: string;
    gitOptions?: Record<string, string>;
  }): Promise<RadiclePeer>;
  /**
   * Kill all processes spawned by any of the peers
   */
  shutdown(): Promise<void>;
}

export async function createPeerManager(createParams: {
  dataDir: string;
  outputLog?: Stream.Writable;
}): Promise<PeerManager> {
  let outputLog: Stream.Writable;
  let outputLogFile: Fs.FileHandle;
  if (createParams.outputLog) {
    outputLog = createParams.outputLog;
  } else {
    outputLogFile = await Fs.open(
      Path.join(createParams.dataDir, "peerManager.log"),
      "a",
    );
    outputLog = outputLogFile.createWriteStream();
  }

  const peers: RadiclePeer[] = [];
  return {
    async createPeer(params) {
      const peer = await RadiclePeer.create({
        dataPath: createParams.dataDir,
        name: params.name,
        gitOptions: params.gitOptions,
        radSeed: Array(64)
          .fill((peers.length + 1).toString())
          .join(""),
        outputLog,
      });
      peers.push(peer);

      return peer;
    },
    async shutdown() {
      await Promise.all(peers.map(peer => peer.shutdown()));
    },
  };
}

// Specialize the return type of `execa()` to guarantee that `stdout` and
// `stderr` are strings.
type SpawnResult = Execa.ResultPromise<
  SpawnOptions & {
    stdout: (line: unknown) => AsyncGenerator<string, void, void>;
    stderr: (line: unknown) => AsyncGenerator<string, void, void>;
    encoding: "utf8";
  }
>;

type SpawnOptions = Omit<
  Execa.Options,
  "stdin" | "stdout" | "stderr" | "lines" | "encoding"
>;

export interface BaseUrl {
  hostname: string;
  port: number;
  scheme: string;
}

export class RadiclePeer {
  public checkoutPath: string;
  public nodeId?: string;
  public sshAgentPid?: string;
  public sshAgentAuthSock?: string;
  public httpdBaseUrl?: BaseUrl;

  #radSeed: string;
  #radHome: string;
  #outputLog: Stream.Writable;
  #gitOptions?: Record<string, string>;
  // Name for easy identification. Used on file system and in logs.
  #name: string;
  #childProcesses: SpawnResult[] = [];

  private constructor(props: {
    checkoutPath: string;
    radSeed: string;
    gitOptions?: Record<string, string>;
    radHome: string;
    logFile: Stream.Writable;
    name: string;
  }) {
    this.checkoutPath = props.checkoutPath;
    this.#gitOptions = props.gitOptions;
    this.#radSeed = props.radSeed;
    this.#radHome = props.radHome;
    this.#outputLog = props.logFile;
    this.#name = props.name;
  }

  public static async create({
    dataPath,
    name,
    gitOptions,
    radSeed: node,
    outputLog: logFile,
  }: PeerManagerParams): Promise<RadiclePeer> {
    const checkoutPath = Path.join(dataPath, name, "copy");
    const radHome = Path.join(dataPath, name, "home");

    return new RadiclePeer({
      checkoutPath,
      gitOptions,
      radSeed: node,
      radHome,
      logFile,
      name,
    });
  }

  /**
   Removes the peers identity file from the `ssh-agent`
   **/
  public async logOut(): Promise<void> {
    await this.spawn("ssh-add", ["-d", `${this.#radHome}/keys/radicle.pub`]);
  }

  public async startHttpd(port: number): Promise<void> {
    this.httpdBaseUrl = {
      hostname: "127.0.0.1",
      port,
      scheme: "http",
    };
    void this.spawn("cargo", [
      "run",
      "--manifest-path",
      "./crates/test-http-api/Cargo.toml",
      "--",
      "--listen",
      `${this.httpdBaseUrl.hostname}:${this.httpdBaseUrl.port}`,
    ]);

    await waitOn({
      resources: [
        `tcp:${this.httpdBaseUrl.hostname}:${this.httpdBaseUrl.port}`,
      ],
      timeout: 20_000,
    });
  }

  public async startSSHAgent() {
    const { stdout } = await this.spawn("ssh-agent", ["-s"]);
    const match = stdout.match(/SSH_AUTH_SOCK=([^;]+);.*SSH_AGENT_PID=(\d+)/s);
    if (match) {
      this.sshAgentAuthSock = match[1];
      this.sshAgentPid = match[2];
    } else {
      throw new Error("Could not start a new ssh-agent");
    }

    await waitOn({
      resources: [`socket:${this.sshAgentAuthSock}`],
      timeout: 2000,
    });
  }

  public async stopSSHAgent() {
    if (this.sshAgentPid) {
      process.kill(Number(this.sshAgentPid), "SIGTERM");
    }

    await waitOn({
      resources: [`socket:${this.sshAgentAuthSock}`],
      reverse: true,
      timeout: 2000,
    });
  }

  /**
   * Kill all child processes created with `spawn()`, including the node and
   * httpd processes.
   */
  public async shutdown() {
    // We don’t care about proper cleanup. We just want to make sure that no
    // processes are running anymore.
    this.#childProcesses.forEach(p => {
      // Don’t leak unhandled rejections when forcefully killing the process
      // eslint-disable-next-line @typescript-eslint/no-empty-function
      p.catch(() => {});
      p.kill("SIGKILL");
    });
  }

  public git(args: string[] = [], opts?: SpawnOptions): SpawnResult {
    return this.spawn("git", args, { ...opts });
  }

  public rad(args: string[] = [], opts?: SpawnOptions): SpawnResult {
    return this.spawn("rad", args, { ...opts });
  }

  public spawn(
    cmd: string,
    args: string[] = [],
    opts?: SpawnOptions,
  ): SpawnResult {
    const prefix = logPrefix(`${this.#name} ${cmd}`);
    const outputLog = this.#outputLog;

    function* logWithPrefix(line: unknown) {
      if (typeof line === "string") {
        outputLog.write(`${prefix} ${line}\n`, "utf8");
      }
      yield line;
    }

    /* eslint-disable @typescript-eslint/naming-convention */
    const childProcess = execa(cmd, args, {
      ...opts,
      env: {
        GIT_CONFIG_GLOBAL: "/dev/null",
        GIT_CONFIG_NOSYSTEM: "1",
        RAD_HOME: this.#radHome,
        RAD_LOCAL_TIME: "1671125284",
        RAD_KEYGEN_SEED: this.#radSeed,
        SSH_AUTH_SOCK: this.sshAgentAuthSock,
        SSH_AGENT_PID: this.sshAgentPid,
        ...opts?.env,
        ...this.#gitOptions,
      },
      encoding: "utf8",
      stdout: logWithPrefix,
      stderr: logWithPrefix,
    });
    /* eslint-enable @typescript-eslint/naming-convention */

    this.#childProcesses.push(childProcess);

    return childProcess;
  }
}
