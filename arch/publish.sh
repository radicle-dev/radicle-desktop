#!/usr/bin/env bash

set -euo pipefail

push=true

while [[ $# -gt 0 ]]; do
  case "$1" in
    --no-push)
      push=false
      shift
      ;;
    *)
      commit_message="$1"
      shift
      ;;
  esac
done

if [[ -z "${commit_message:-}" ]]; then
  echo "Please provide a commit message as an argument: $0 [--no-push] \"Release vX.Y.Z\""
  exit 1
fi

if [[ ! -d radicle-desktop.git ]]; then
  git clone ssh://aur@aur.archlinux.org/radicle-desktop.git radicle-desktop.git
fi

pushd radicle-desktop.git >/dev/null
export GIT_PAGER=""
git checkout master
git pull
cp -a ../radicle-desktop/* ../radicle-desktop/.* .
git diff
git commit --all --message "$commit_message"
if [[ "$push" == true ]]; then
  git push
fi
popd >/dev/null
