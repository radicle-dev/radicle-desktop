// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

export type Commit = {
  id: string;
  author: { name: string; email: string; time: number };
  committer: { name: string; email: string; time: number };
  message: string;
  summary: string;
  parents: Array<string>;
};