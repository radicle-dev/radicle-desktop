#!/usr/bin/env bash

exec docker run \
    --user 1000 \
    --rm --volume "$(pwd)/radicle-desktop:/workdir:ro" \
    --env BUILDDIR=/tmp \
    --env PKGDEST=/tmp \
    --env SRCDEST=/tmp \
    --workdir /workdir \
    archlinux:base-devel makepkg --printsrcinfo > radicle-desktop/.SRCINFO

