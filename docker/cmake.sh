#!/usr/bin/env bash

set -x
set -euo pipefail

# shellcheck disable=SC1091
. lib.sh

main() {
    local version=4.0.1

    install_packages curl

    local td
    td="$(mktemp -d)"
    pushd "${td}"

    local cmake_arch
    local cmake_sha256

    local narch
    narch="$(dpkg --print-architecture)"

    case "${narch}" in
    amd64)
        cmake_arch="linux-x86_64"
        cmake_sha256="1b175abad93a117fd9b3b72ecaf44d5d4d158b7c7443e7a6fbee0e1b8c67f697"
        ;;
    arm64)
        cmake_arch="linux-aarch64"
        cmake_sha256="23e0aefd2ee8ef5e4f5a8043f97dad4776847ee900401db319f7a8a278abe1ca"
        ;;
    *)
        echo "Unsupported architecture: ${narch}" 1>&2
        exit 1
        ;;
    esac

    curl --retry 3 -sSfL "https://github.com/Kitware/CMake/releases/download/v${version}/cmake-${version}-${cmake_arch}.sh" -o cmake.sh
    sha256sum --check <<<"${cmake_sha256}  cmake.sh"
    sh cmake.sh --skip-license --prefix=/usr/local
    cmake --version

    popd

    purge_packages

    rm -rf "${td}"
    rm -rf /var/lib/apt/lists/*
    rm "${0}"
}

main "${@}"
