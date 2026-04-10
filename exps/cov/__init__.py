"""Coverage utilities for libyaml transpilation experiments.

Wraps llvm-cov export JSON to extract function and branch information.
Each module also has a main() so it can run as a CLI: python3 -m cov.list_functions ...
"""
