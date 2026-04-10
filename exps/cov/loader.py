"""Helpers for invoking llvm-cov export and loading the JSON."""
import json
import os
import subprocess
import sys


def llvm_cov_export(binary, profdata=None, llvm_cov="llvm-cov-21"):
    """Run `llvm-cov export` and return the parsed JSON.

    If profdata is None, uses -empty-profile (returns the structural list of
    all functions/branches with zero counts).
    """
    if not os.path.exists(binary):
        sys.exit(f"Error: binary not found: {binary}")

    cmd = [llvm_cov, "export", binary]
    if profdata:
        if not os.path.exists(profdata):
            sys.exit(f"Error: profdata not found: {profdata}")
        cmd.append(f"-instr-profile={profdata}")
    else:
        cmd.append("-empty-profile")

    try:
        out = subprocess.check_output(cmd, stderr=subprocess.DEVNULL)
    except subprocess.CalledProcessError as e:
        sys.exit(f"Error: {' '.join(cmd)} failed: {e}")
    return json.loads(out)


def basename(path):
    """Strip directory from a path (last component only)."""
    return path.rsplit("/", 1)[-1]
