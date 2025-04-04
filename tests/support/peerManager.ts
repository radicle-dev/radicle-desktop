import type * as Execa from "execa";
import type { Config } from "@tests/support/fixtures";

import * as Fs from "node:fs/promises";
import * as Os from "node:os";
import * as Path from "node:path";
import * as Stream from "node:stream";
import getPort from "get-port";
import waitOn from "wait-on";
import { defaultConfig } from "@tests/support/fixtures";
import { execa } from "execa";
import { logPrefix } from "@tests/support/logPrefix.js";

interface PeerManagerParams {
  dataPath: string;
  radSeed: string;
  // Id for easy identification. Used on file system and in logs.
  id: string;
  gitOptions?: Record<string, string>;
  outputLog: Stream.Writable;
}

export interface PeerManager {
  createPeer(params: {
    id: string;
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
        id: params.id,
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

  #sshAgentPid?: string;
  #sshAgentAuthSock?: string;
  #radSeed: string;
  #socket: string;
  #radHome: string;
  #outputLog: Stream.Writable;
  #gitOptions?: Record<string, string>;
  #listenSocketAddr?: string;
  #httpdBaseUrl?: BaseUrl;
  #nodeProcess?: SpawnResult;
  // Id for easy identification. Used on file system and in logs.
  #id: string;
  #childProcesses: SpawnResult[] = [];

  private constructor(props: {
    checkoutPath: string;
    radSeed: string;
    socket: string;
    gitOptions?: Record<string, string>;
    radHome: string;
    logFile: Stream.Writable;
    id: string;
  }) {
    this.checkoutPath = props.checkoutPath;
    this.#gitOptions = props.gitOptions;
    this.#radSeed = props.radSeed;
    this.#socket = props.socket;
    this.#radHome = props.radHome;
    this.#outputLog = props.logFile;
    this.#id = props.id;
  }

  public static async create({
    dataPath,
    id,
    gitOptions,
    radSeed: node,
    outputLog: logFile,
  }: PeerManagerParams): Promise<RadiclePeer> {
    const checkoutPath = Path.join(dataPath, id, "copy");
    await Fs.mkdir(checkoutPath, { recursive: true });
    const radHome = Path.join(dataPath, id, "home");
    await Fs.mkdir(radHome, { recursive: true });

    const socketDir = await Fs.mkdtemp(Path.join(Os.tmpdir(), `radicle-${id}`));
    const socket = Path.join(socketDir, "control.sock");

    return new RadiclePeer({
      checkoutPath,
      gitOptions,
      radSeed: node,
      socket,
      radHome,
      logFile,
      id,
    });
  }

  /**
   Removes the peers identity file from the `ssh-agent`
   **/
  public async logOut(): Promise<void> {
    await this.spawn("ssh-add", ["-d", `${this.#radHome}/keys/radicle.pub`]);
  }

  public async startHttpd(port: number): Promise<void> {
    this.#httpdBaseUrl = {
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
      `${this.#httpdBaseUrl.hostname}:${this.#httpdBaseUrl.port}`,
    ]);

    await waitOn({
      resources: [
        `tcp:${this.#httpdBaseUrl.hostname}:${this.#httpdBaseUrl.port}`,
      ],
      timeout: 20_000,
    });
  }

  public async startSSHAgent() {
    const { stdout } = await this.spawn("ssh-agent", ["-s"]);
    const match = stdout.match(/SSH_AUTH_SOCK=([^;]+);.*SSH_AGENT_PID=(\d+)/s);
    if (match) {
      this.#sshAgentAuthSock = match[1];
      this.#sshAgentPid = match[2];
    } else {
      throw new Error("Could not start a new ssh-agent");
    }

    await waitOn({
      resources: [`socket:${this.#sshAgentAuthSock}`],
      timeout: 2000,
    });
  }

  public async stopSSHAgent() {
    if (this.#sshAgentPid) {
      process.kill(Number(this.#sshAgentPid), "SIGTERM");
    }

    await waitOn({
      resources: [`socket:${this.#sshAgentAuthSock}`],
      reverse: true,
      timeout: 2000,
    });
  }

  public async startNode(config: Partial<Config> = defaultConfig) {
    const listenPort = await getPort();
    this.#listenSocketAddr = `0.0.0.0:${listenPort}`;

    await updateConfig(this.#radHome, config);

    this.#nodeProcess = this.spawn("radicle-node", [
      "--listen",
      this.#listenSocketAddr,
    ]);

    await waitOn({
      resources: [`socket:${this.#socket}`],
      timeout: 2000,
    });

    const { stdout } = this.rad(["node", "events"], {
      cwd: this.#radHome,
    });

    if (!stdout) {
      throw new Error("Could not get stdout to track events");
    }
  }

  public async stopNode() {
    // Don’t leak unhandled rejections when forcefully killing the process
    // eslint-disable-next-line @typescript-eslint/no-empty-function
    this.#nodeProcess?.catch(() => {});
    this.#nodeProcess?.kill("SIGTERM");

    await waitOn({
      resources: [`socket:${this.#socket}`],
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
  public get httpdBaseUrl(): BaseUrl {
    if (!this.#httpdBaseUrl) {
      throw new Error("No httpd service running");
    }

    return this.#httpdBaseUrl;
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
    const prefix = logPrefix(`${this.#id} ${cmd}`);
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
        RAD_PASSPHRASE: "asdf",
        RAD_LOCAL_TIME: "1671125284",
        RAD_KEYGEN_SEED: this.#radSeed,
        RAD_SOCKET: this.#socket,
        SSH_AUTH_SOCK: this.#sshAgentAuthSock,
        SSH_AGENT_PID: this.#sshAgentPid,
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

async function updateConfig(radHome: string, configParams: Partial<Config>) {
  const configPath = Path.join(radHome, "config.json");
  const configFile = await Fs.readFile(configPath, "utf-8");
  const config = {
    defaultConfig,
    ...JSON.parse(configFile),
  };
  config.preferredSeeds = [];
  config.web = { ...config.web, ...configParams.web };
  config.node = {
    ...config.node,
    ...configParams.node,
    network: "test",
  };
  await Fs.writeFile(configPath, JSON.stringify(config), "utf-8");
}
