#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# Memory budget for this machine, sourced by the other scripts.
#
# The developer laptop has 15 GiB of RAM and 8 GiB of zram and no cgroup limit, so a
# single runaway process can take the whole session down — which is exactly what
# happened on 2026-11-06, when a drawing test allocated in a loop until the kernel
# killed it at 13 GiB resident. These two numbers are the answer: how many compilers run
# at once, and how much virtual memory a test binary may claim before it is stopped.
#
# Both are derived from the machine, so a bigger machine gets a bigger budget, and both
# can be overridden from the environment.
#
#   PUNCHI_BUILD_JOBS=4            parallel rustc jobs
#   PUNCHI_TEST_MEMORY_KB=6000000  per test binary, in KiB
#
# Sourced, never executed.

# Total physical memory, in MiB, from the kernel rather than from a guess.
_punchi_mem_mib() {
    awk '/^MemTotal:/ { printf "%d", $2 / 1024 }' /proc/meminfo 2>/dev/null || printf '4096'
}

# Two jobs on this machine: each rustc peak is roughly 1 GiB and the linker needs room.
: "${PUNCHI_BUILD_JOBS:=$(( $(nproc) / 4 > 0 ? $(nproc) / 4 : 1 ))}"
((PUNCHI_BUILD_JOBS > 4)) && PUNCHI_BUILD_JOBS=4
export PUNCHI_BUILD_JOBS
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-$PUNCHI_BUILD_JOBS}"

# A quarter of RAM, never below 1 GiB. The heaviest test binary here peaks at tens of
# megabytes, so this only ever catches a defect.
: "${PUNCHI_TEST_MEMORY_KB:=$(( $(_punchi_mem_mib) * 1024 / 4 ))}"
((PUNCHI_TEST_MEMORY_KB < 1048576)) && PUNCHI_TEST_MEMORY_KB=1048576
export PUNCHI_TEST_MEMORY_KB

# run_capped <command...>
#
# Runs a command with the virtual-memory ceiling applied. Rust reports the refusal as an
# allocation failure with a backtrace, which names the defect, instead of the kernel
# killing whichever process it picks — often the editor.
run_capped() {
    (
        ulimit -v "$PUNCHI_TEST_MEMORY_KB" 2>/dev/null || true
        exec "$@"
    )
}
