# The review: what is proved and what is recorded

A visual change is reviewed against the canonical document, not against taste. This page is
the checklist and the evidence. The task rows are in [the skill](../SKILL.md); the canonical
homes are [ui-ux.md](../../../../docs/ui-ux.md) and the documents its sections link.

## Machine-checked

Run the commands the task row names and paste their result into the pull request. Each one
fails on a specific condition; do not claim more than it checks.

| Command | Fails on | Does not prove |
|---|---|---|
| `--check-theme` | a pairing below the contrast contract (`contrast.passes()`) | the unit and the motion (printed, not asserted) or a literal colour, size or radius in a widget |
| `--check-qml` | a component that does not load or a broken binding | the visual result |
| `--check-layout` | a region, a mode or a ribbon row outside its measurement | the font's rendering on another desktop |
| `--selftest` | a named gesture, panel or window step | a path the step does not touch |
| `--check-icons` | a command with no drawing, a fallback gap or a use outside the family | that the icon reads well at the small size |
| `./scripts-dev/check-docs.sh` | a relative link outside the tree or missing, a skill folder without a catalog row, a mismatched `name`, a wrong `#fragment` | that the linked prose is still true |
| `./scripts-dev/check.sh` | the full gate the branch must pass | that an unconfigured window was checked |

`--check-theme` prints the unit and the motion values and asserts only the contrast. The
no-literal-colour rule is likewise manual: the command cannot see a literal in a widget. If
the review records “unit and motion verified” or “no literal”, it needs the line or the
manual observation, not just a green command. `--icon-sheet <file> --theme light|dark` is
not in the table because it is not a check: it renders a sheet for a human and its exit
status proves nothing.

## Manual, with evidence

Mark each item and link the capture, the measurement or the line:

- the token is used and no widget holds the literal; the accent follows
  [Visual language](../../../../docs/ui-ux.md#visual-language);
- the spacing unit and the motion come from the theme; reduced motion keeps the state
  change and drops the animation;
- every state exists and none is colour-only;
- the control is reachable by keyboard and exposes its accessible name and role;
- the icon is one concept at the small render size;
- the chrome uses the spacing unit and the drawing uses millimetres;
- the change stays inside the visual language: no diagram, no ISO 1219 symbol, no exported
  page and no symbology review record is edited as “style”.

## Link resolution is a security surface

A skill is a file that authors links, and the documentation check resolves those links. The
review is recorded in the pull request that adds or changes a link.

- **Surface**: a relative link inside `.agents/skills/**/*.md` (and the documentation files
  the check already scanned), resolved by `scripts-dev/check-docs.sh`.
- **Control**: the target is resolved from the containing file and bounded to `ROOT`; an
  absolute path, a `../` that leaves the tree and a symlink whose real target leaves the tree
  all fail. The `#fragment` is validated against the headings of the destination document,
  so a renamed heading fails instead of passing silently.
- **Hostile case**: a link to `../../../../etc/passwd`, an absolute path such as `/etc/passwd`,
  or an in-tree symlink that points outside. None is followed: the check reports the link
  and exits non-zero.
- **Record**: the pull request carries this note, the `check-docs.sh` result and the
  `test-check-docs.sh` fixture, which builds the failures and the path escapes on a
  temporary tree.

The fixture is the executable record: it proves a broken relative link, a missing catalog row,
a wrong anchor and the three path escapes (`../` outside the tree, an absolute path and a
symlink) on a tree that is not the repository, and it passes on the real tree.

## What the pull request carries

- the command results, with the exact commands and their exit status;
- the captures, named and linked, in both schemes when the shell changes;
- the manual checklist marked with the evidence for each item;
- the security note above when a link or a path resolution changed;
- a statement of what was **not** verified: an unconfigured window, another desktop, another
  scale or another font.
