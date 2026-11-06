## What this changes

<!-- One paragraph. Link the issue: Closes #123 -->

## Milestone

<!-- M1 shell / M1.5 symbology / M2 export / M3 document / M4 solver / M5 instruments / M6 release -->
<!-- If it belongs to none, say why it is urgent. -->

## Claim and evidence

<!-- What do you claim, and at which level? Delete the levels that do not apply. -->

- [ ] Unit tests cover the behaviour and fail if it is removed
- [ ] Headless UI test (hit areas, layout, clicks)
- [ ] Pixel or call-list test (export geometry, margins, determinism)
- [ ] Measurement (memory, frame time, solver time) — numbers below
- [ ] Screenshot, light and dark, attached
- [ ] Symbol conformance review attached (`docs/symbology.md`)

## Checklist

- [ ] `./scripts-dev/check.sh` green, no warning silenced
- [ ] No behavioural test deleted, disabled or weakened
- [ ] `CHANGELOG.md` updated under `Unreleased`
- [ ] Docs updated in the same change (README, ADR, spec) if behaviour or a decision changed
- [ ] No hardcoded colour, pixel size or user-visible string outside the theme and translation files
- [ ] Memory and performance budgets still hold, or the change explains why they moved
- [ ] New dependency: licence checked with `cargo deny`, reason written here
- [ ] Commits signed off (`git commit -s`) and in Conventional Commits form

## Known limitations

<!-- What this pull request does NOT do. Reviewers read this part first. -->
