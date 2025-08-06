#!/usr/bin/env bash
set -euo pipefail

export PACKAGE_VERSION=$(cargo pkgid | cut -d "#" -f2)

export CHECKSUM_SOURCE=$(curl -sL "https://github.com/croissong/gitwatch-rs/archive/v${PACKAGE_VERSION}.tar.gz" | sha256sum | cut -d' ' -f1)
export CHECKSUM_ASSET=$(curl -sL "https://github.com/croissong/gitwatch-rs/releases/download/v${PACKAGE_VERSION}/gitwatch-x86_64-unknown-linux-gnu.tar.gz" | sha256sum | cut -d' ' -f1)

minijinja-cli --env pkg/arch/PKGBUILD.j2 > pkg/arch/PKGBUILD
