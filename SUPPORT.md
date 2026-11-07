# Support

Punchi Fluid Lab is free software maintained in someone's free time. That sentence is the
whole support policy, and it is here so nobody has to guess what it means.

## Where to ask

| What you have | Where it goes |
|---|---|
| A reproducible bug — something behaves differently from what this documentation says | [Open a bug report](https://github.com/PunchiSoft/punchiFluidLab/issues/new?template=bug_report.yml). Include the version (`git rev-parse --short HEAD` or the tag), the system and the smallest circuit file that shows it |
| An idea, a component or a symbol you miss | [Open a feature request](https://github.com/PunchiSoft/punchiFluidLab/issues/new?template=feature_request.yml) |
| An open question — how to draw something, why a symbol looks like that, whether a circuit is right | [Discussions](https://github.com/PunchiSoft/punchiFluidLab/discussions) |
| A security problem | [Private advisory](https://github.com/PunchiSoft/punchiFluidLab/security/advisories/new), never a public issue. See [SECURITY.md](SECURITY.md) |
| A behaviour problem in the community | [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) |
| A patch | [CONTRIBUTING.md](CONTRIBUTING.md): the gate runs before anything is merged |

## What this project does promise

- An answer to every issue or pull request, even if the answer is "not now, and here is
  why".
- A green gate and a passing test as the bar for merging, never a verbal assurance.
- That the documentation says what is missing instead of implying it exists: the ledger in
  [docs/capabilities.md](docs/capabilities.md) is the list.

## What it does not promise

- **A response time.** The project advances on the free days left by a full-time job: an
  issue may wait days and a review a week. That is the pace, not an oversight.
- **A delivery date for any capability.** There is none, and none will be invented.
- **Commercial support, a warranty, or a fit for a regulated use.** It is a teaching tool.
- **Support for systems it was never run on.** Today: Linux with Qt 6.5 or newer, built
  and tested on Fedora with Plasma and Wayland. Other distributions are expected to work and
  are not verified.

## Before you ask

```bash
./scripts-dev/check.sh                 # the whole gate, including the window when configured
apps/punchi-fluid-qt/build/punchi-fluid --selftest   # the window's own checks, headless
```

A report that includes what the gate said is answered faster than one that does not.
