# Security policy

## Supported versions

`punchi-fluid-lab` is unreleased (`0.x`). Only the current `main` and the latest
`0.x` release receive security fixes.

| Version | Supported |
|---|---|
| `main` | yes |
| latest `0.x` | yes |
| older `0.x` | no |

## Reporting a vulnerability

Report privately through **GitHub → Security → Advisories → Report a vulnerability**
on this repository. Do not open a public issue, and do not attach a real circuit
document if it contains work you need to keep private: a minimal file that reproduces
the behaviour is enough.

Please include the version or commit, the platform, the file that triggers the
problem and what you observed. We acknowledge within seven days and aim to ship a fix
or a documented mitigation within ninety days, coordinating disclosure with you.

## What is in scope

The application parses files that come from outside the program, and exports files
that leave it. That is where the security surface is:

- **Catalog definitions** (`*.toml`): parsing, validation, resource consumption.
- **Circuit documents** (`*.pfluid`): parsing, migrations, malformed or hostile input.
- **Export** (PNG, PDF): page and DPI handling, memory consumption, metadata,
  overwriting files, path handling.
- **Theme, font and icon loading** from the system.
- **Dependency and build**: compromised crates, unsafe code, build scripts.
- **Denial of service by resource exhaustion**: unbounded allocation, unbounded loops,
  a document that makes the solver hang a frame.

## What is out of scope

- Physical accuracy. A model that does not match reality is a bug, filed as a normal
  issue.
- Attacks that require an already compromised machine, a modified binary, or root.
- Anything in a third-party font or icon theme that the application merely displays.
- Reports produced only by a scanner without a reproduction.

## Guarantees the code tries to keep

- No network access at runtime, no telemetry, no auto-update.
- No `unsafe` in this workspace (enforced by a lint).
- No shell-outs and no dynamic loading of native code.
- File size, node count, DPI and pixel budgets checked before allocating.
- Exported metadata carries no path, user name or host name.
- Every file write is atomic (temporary file plus rename).

## Credit

Fix acknowledgements name the reporter unless they prefer otherwise.
