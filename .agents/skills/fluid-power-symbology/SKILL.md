---
name: fluid-power-symbology
description: "Author, reconstruct, correct and verify ISO 1219 pneumatic and hydraulic symbols using owner-provided visual references, functional instructions, catalog definitions and geometric conformance reviews. Use when adding or changing a symbol."
license: GPL-3.0-or-later
---

# Fluid-power symbology

## Purpose

This skill defines the workflow for identifying, reconstructing, authoring and validating pneumatic and hydraulic symbols in PunchiFluidLab.

The project implements standardized fluid-power symbology. The symbol's meaning and normative form come from the applicable standards, not from the software displaying it.

The project owner provides the intended reference, functional interpretation and engineering direction. The agent acts as a technical drawing and implementation assistant: it analyzes the available information, identifies uncertainties, constructs the definition and verifies the result against the project's actual schema and rendering behavior.

A symbol must communicate the correct function. A visually plausible but functionally incorrect symbol is not acceptable.

## 1. Authority and responsibilities

Use the following order of authority:

1. **Applicable standards:** normative requirements for symbol form, meaning, connections and diagram conventions.
2. **Project owner:** intended component, supplied references, functional description, engineering requirements and decisions about unresolved interpretations.
3. **Project contracts:** accepted ADRs, `AGENTS.md`, `docs/symbology.md`, catalog schema and renderer behavior.
4. **Existing catalog definitions:** implementation examples, not automatic normative authority.
5. **Agent interpretation:** provisional engineering interpretation that must be identified as such when it is not directly supported by the sources.

The owner is the authority on what component is being requested and what the intended implementation must accomplish.

The agent must not silently replace the owner's functional instructions with an assumption. If an instruction conflicts with a verified normative requirement, explain the conflict and request a decision.

The owner may request a non-standard or custom symbol. Such a symbol must not be represented as ISO-conformant.

### 1.1 Signed symbols are immutable

A symbol is **signed** only when [ADR 0047](../../../docs/adr/0047-the-signed-symbols-are-immutable.md)
holds: `assets/symbols/signed.toml` has one entry for it, the SHA-256 of its TOML, review and
three captures match, the entry carries `signed_by`/`signed_at`, and the detached SSH
signature `signed.toml.sig` verifies against `assets/symbols/owner_signers`. `status:
complete` is the **precondition** of an entry, not the proof: a review that is complete but
has no signed entry is reviewed, not signed. `scripts-dev/check-signed-symbols.sh`, run by
`check.sh` and `check-docs.sh`, is what enforces this.

- **A signed symbol is frozen.** Do not edit its TOML, review or three captures; do not
  regenerate, reformat or swap them. Any change makes the hashes disagree and the gate fail.
  A renderer change that moves the drawing also requires regenerating the captures and
  re-signing, so it is an owner-authorized revision, not a routine edit.
- **A change needs the owner's direct authorization and a new review.** Then `revision`
  increments, `previous_revision` names the old one, the old revision stays in Git, and the
  owner signs the new manifest. The agent prepares the evidence; it never signs, never holds
  the private key, and never edits the manifest or the signature on its own.
- **Exact ID or stop.** Match the symbol the owner named by its exact catalog id. If the
  request is ambiguous, or the only argument is visual similarity, stop and ask; never pick a
  symbol because it looks like the request.
- **The base is being signed.** While `base_open = true`, entries without
  `signed_by`/`signed_at` are **pending**: their hashes are pinned and the gate already
  protects them, but they are not signed. Read the manifest before saying a symbol is signed;
  never call a pending entry signed.
- **The wrong move is to edit first.** If asked to correct a signed symbol, stop and say that
  the change needs the owner's authorization and a new signed revision, with the reasons; do
  not change the file and ask afterwards.

## 2. Visual references supplied by the owner

The owner may provide any visual reference, including:

- Photographs of physical diagrams or components.
- Hand-drawn sketches.
- Screenshots from other software.
- Technical drawings.
- Images from books or technical manuals.
- Figures from applicable standards.
- Annotated images and detail crops.
- Existing pneumatic or hydraulic circuit diagrams.

Do not reject a reference merely because it comes from another application or source.

First determine what the image is intended to communicate and how the owner wants it used.

### 2.1 Classify the reference

Every reference must be classified by its origin and intended purpose.

**A. Normative standard figure**

A figure that defines the symbol in the applicable standard.

It may be used to:

- Identify the normative symbol.
- Read its geometric proportions and dimensions.
- Determine relative positions and orientations.
- Inspect line arrangements and graphical relationships.
- Derive geometric coordinates needed for an independent implementation.
- Compare the authored geometry against the normative requirements.

Record the standard, edition, clause or figure reference, and the measures used in the review.

**B. Owner-provided engineering drawing or sketch**

May be used to:

- Identify the intended component.
- Understand the owner's desired representation.
- Extract owner-specified dimensions when they are provided as engineering requirements.
- Identify ports, connections, annotations and relationships.
- Clarify an intended modification or custom representation.

Do not automatically assume that a sketch is normatively correct.

**C. Image from another application, library or external manual**

May be used to:

- Identify the component the owner is referring to.
- Understand the intended function and arrangement.
- Identify visible features the owner wants to discuss.
- Locate differences that need to be resolved against the applicable standard.

It must not become a source from which third-party graphical assets, proprietary implementations, manual content or application-specific drawing resources are copied into the repository.

Do not confuse the ability to inspect an image with permission to incorporate its original material into the project.

**D. Photograph of a physical component or installation**

May be used to understand:

- Component identity.
- Mechanical arrangement.
- Number and location of physical connections.
- Actuation and return mechanisms.
- Installation context.
- Functional relationships.

A photograph of a physical component does not, by itself, establish its standardized graphical representation.

### 2.2 Analyze the reference

When the owner supplies an image:

1. Inspect the entire image before analyzing individual details.
2. Identify the component or circuit being discussed.
3. Identify visible symbols, ports, actuators, springs, paths and annotations.
4. Separate visible information from functional information supplied by the owner.
5. Determine whether the image is a normative figure, an engineering reference or an identification reference.
6. Ask for clarification only when a relevant detail cannot be established.
7. Select the applicable normative symbol and implementation approach.

Do not invent missing details.

When image quality prevents reliable identification, state which details are unclear rather than fabricating coordinates or geometry.

### 2.3 Geometric reconstruction

Geometric reconstruction is permitted when the source and its use authorize it.

For normative standard figures, the agent may read dimensions, measure proportions, derive coordinates and reconstruct the symbol geometry.

For owner-provided engineering requirements, use explicitly supplied dimensions and geometric constraints.

For third-party application captures and external graphical assets, do not trace or copy their graphical implementation. Use them to understand the requested component and resolve its intended meaning against the applicable standard.

Always construct an independent TOML definition using PunchiFluidLab's own schema and primitives.

The resulting definition must not depend on the original image at runtime.

## 3. Normative standards

Read the project's current normative contract before modifying symbols:

- `docs/symbology.md`
- `AGENTS.md`
- Relevant accepted ADRs.
- Applicable catalog documentation.

The project currently identifies the following standards:

| Standard | Scope |
|---|---|
| ISO 1219-1:2012 | Graphical symbols and conventional use |
| ISO 1219-2:2012 | Circuit diagram conventions |
| ISO 1219-3:2016 | Symbol modules and combinations |
| ISO 6358 | Pneumatic flow through restrictions |
| ISO 5598 | Fluid power vocabulary |

Check the applicable edition and any national adoption documented by the project before making a normative claim.

Do not invent clause numbers, dimensions or requirements.

If the standard's applicable requirement cannot be verified, record the item as unresolved.

### 3.1 Normative identity

The same standardized symbol must retain its normative meaning and form independently of the application that displays it.

PunchiFluidLab must implement the symbol rather than create a software-specific interpretation of its meaning.

Do not introduce graphical variants merely to imitate another application.

A custom representation must be explicitly identified as custom or non-standard.

### 3.2 Functional interpretation

A symbol describes a component or function. The catalog behavior describes how the simulator models it.

These are related but distinct concerns.

For every new component, determine:

- What physical or logical function it represents.
- What its ports represent.
- Which ports communicate in each operating state.
- What conditions change its state.
- Whether it has a mechanical actuator or return mechanism.
- Whether it is pneumatic, hydraulic, electrical or a combination.
- Whether its physical behavior is currently supported by the solver.

A symbol may be graphically valid even when its physical simulation model is not yet implemented. Do not claim simulation support merely because the component can be drawn.

## 4. Workflow: reconstructing a symbol

### Step 1 — Understand the owner's request

Identify the intended component and its purpose.

Write a functional description in plain language.

For example:

"When pressure is present at either inlet, the shuttle selects the higher-pressure inlet and supplies the outlet."

If the owner has supplied a functional description, preserve its meaning.

Do not infer a different behavior from the image alone.

### Step 2 — Inspect the visual reference

Classify its origin and intended use according to Section 2.

Identify:

- Main body.
- Number of positions.
- Ports and their identifiers.
- Actuators.
- Springs and return mechanisms.
- Flow paths.
- Blocking marks.
- Arrows and directional features.
- Additional graphical elements.

For a circuit, also identify the connections between components.

### Step 3 — Establish normative requirements

Determine the corresponding standardized symbol and its applicable requirements.

Record:

- Standard and edition.
- Clause or figure, when available.
- Required geometric relationships.
- Port arrangement.
- Direction of flow.
- Actuation and return arrangement.
- Required marks and annotations.

When using a normative figure as a geometric reference, record the measures read or derived from it.

Do not present a derived estimate as a dimension explicitly stated by the standard.

### Step 4 — Establish geometry

Define the symbol's geometry using the project's coordinate system.

Prefer normalized coordinates where the schema requires them.

Record:

- Bounding box.
- Relative proportions.
- Primitive coordinates.
- Port positions.
- Line endpoints.
- Arc centers and radii.
- Orientation.
- Required clearances.
- Relationships between graphical elements.

Keep observed measurements, derived coordinates and inferred values distinguishable.

Do not use arbitrary coordinates merely to make the first rendering look plausible.

### Step 5 — Inspect the existing catalog

Before creating a definition, inspect:

- `assets/catalog/`
- `assets/catalog/README.md`
- Related component definitions.
- Existing symbol review records.
- Existing geometry fixtures.

Reuse the closest valid definition as a starting point when appropriate.

Do not introduce a new behavior variant or a new code path just to accommodate a new symbol. Catalog definitions are data-driven.

A new behavior variant requires the corresponding architectural decision.

### Step 6 — Author the TOML definition

Use the schema and field names documented by the current catalog.

Do not assume an older schema version is still current.

The definition may include:

- Component metadata.
- Ports.
- Port roles.
- Port positions.
- Actuator and spring declarations.
- Kinematics.
- Behavior.
- Behavior positions.
- Symbol primitives.
- Line classes.
- Annotations.

Use the smallest definition that accurately expresses the component.

### Step 7 — Respect renderer responsibilities

Understand which graphical elements are authored in TOML and which are generated by the renderer.

For flat symbols:

- Stubs are declared as TOML symbol primitives.
- Connection circles are drawn by the renderer.
- Port numerals are drawn by the renderer when applicable.
- Port metadata and positions determine connection behavior.

Do not assume that the renderer automatically generates every visible line.

For valves and other component families, inspect the actual drawing implementation before assigning responsibility for a graphical element.

Never duplicate a renderer-generated feature as a TOML primitive.

Never omit a primitive that the current renderer expects the definition to provide.

### Step 8 — Validate the definition

Run the catalog tests and the relevant validation tools.

At minimum:

```bash
cargo test -p punchi-fluid-catalog
```

Check:

- TOML syntax.
- Schema version.
- Required fields.
- Unknown keys.
- Unique identifiers.
- Port roles.
- Port positions.
- Primitive structure.
- Geometry bounds.
- Behavior declarations.
- Connections.
- Conductance and physical values where applicable.

A malformed definition must be corrected rather than hidden by changing validation rules.

### Step 9 — Render and inspect

Use the project's current symbol preview tooling.

The review must include:

- Palette size.
- Canvas size.
- Light theme.
- Dark theme.
- Export resolution.
- Port visibility.
- Numeral readability.
- Geometric alignment.
- Line continuity.
- Clearances.
- Correct rendering of actuators and springs.

Where available, generate the symbol sheet using:

```bash
cargo run -p punchi-fluid-export --bin symbol-sheet -- <id>
```

Check the actual command options before using additional arguments.

Do not approve a symbol solely because its TOML parses.

### Step 10 — Review geometric conformance

Compare the authored definition against the applicable requirements.

When a normative standard figure was used as a geometric reference, check the reconstructed form against its documented measures and relationships.

When an owner-provided sketch or photograph was used, check that the implementation reflects the owner's intended component while remaining consistent with the applicable standard.

Record discrepancies explicitly.

Do not adjust geometry merely to make it resemble an unrelated software rendering.

### Step 11 — Record the review

Create or update:

`assets/symbols/review/<component-id>.md`

Use:

`assets/symbols/review/TEMPLATE.md`

Record:

- Component identifier.
- Functional description.
- Reference provenance.
- Standard and edition.
- Normative clause or figure.
- Measures and geometric relationships.
- Derived coordinates where relevant.
- Automated validation results.
- Visual review results.
- Unresolved questions.
- Reviewer and sign-off.

The repository must not store unauthorized third-party material.

The review should preserve enough information to understand how the geometry was constructed without depending on an external image remaining available.

### Step 12 — Update geometry fixtures

The geometry fixture records the drawing calls produced by the renderer.

When a symbol changes, update its fixture using the repository's documented procedure.

For example:

```bash
UPDATE_SYMBOL_FIXTURES=1 cargo test -p punchi-fluid-draw --test symbol_geometry
```

Read the fixture diff.

Confirm that every coordinate, line width and graphical element changed for an intentional reason.

Commit the TOML definition, fixture and review record together.

### Step 13 — Prepare the pull request

Include:

- Reason for the change.
- Functional description.
- Normative source.
- Reference provenance.
- Geometric measures.
- TOML changes.
- Lint results.
- Fixture changes.
- Light and dark captures.
- Review record.
- Remaining limitations.

One symbol per change is preferred during corrections to the existing catalog.

The owner performs the final approval.

## 5. Valve reconstruction checklist

For directional valves, verify:

- Correct number of positions and ways.
- One equal square per position.
- Correct ordering of positions.
- Correct rest position.
- Correct actuator side.
- Correct spring side.
- Correct port arrangement.
- Correct port identifiers and numerals.
- Correct flow paths for every position.
- Correct blocking marks.
- Correct arrow direction.
- No path crosses into another position unintentionally.

The functional description must agree with the behavior declaration.

For example, a 3/2 valve must have three ports and two positions. Its connections in each position must match the intended function.

Do not assume that every 3/2 or 5/2 valve has the same rest state or actuation mechanism.

## 6. Actuator and cylinder checklist

For cylinders, verify:

- Single-acting or double-acting configuration.
- Barrel.
- Piston.
- Rod.
- Number and location of working ports.
- Return spring, if applicable.
- Correct mechanical arrangement.
- Correct kinematics declaration.
- Correct relationship between the fixed geometry and moving parts.

The renderer must draw moving elements according to the actual catalog contract and simulation state.

Do not add a spring to a double-acting cylinder unless the requested component and its normative representation require it.

## 7. Flat symbols and other components

For flat symbols such as shuttle valves and two-pressure valves, verify:

- Correct number of inlets and outlets.
- Correct port roles.
- Correct identifiers.
- Correct printed numerals.
- Correct graphical primitives.
- Correct functional behavior.

When two ports share a printed numeral, keep their internal identifiers distinct.

For example, two inlets may have identifiers `1a` and `1b`, while both display the numeral `1`, provided the catalog contract supports this arrangement.

Do not confuse the internal port identifier with its printed numeral.

For symbols with no moving parts, do not introduce unnecessary kinematics.

## 8. Circuit diagram checklist

When reconstructing a complete circuit, verify:

- Component identity.
- Correct connections between ports.
- Correct line classes.
- Correct junctions.
- Correct crossing behavior.
- Correct supply and return arrangement.
- Correct pilot and drain lines.
- Correct component numbering.
- Correct diagram layout.

A symbol's visual proximity to another symbol does not establish a connection.

Connections must be represented through the document's actual connection model.

## 9. Pneumatic and hydraulic distinctions

ISO 1219 provides a common graphical language for fluid-power symbols.

However, pneumatic and hydraulic circuits may differ in their physical arrangement, supply, return, exhaust and operating assumptions.

Do not assume that a pneumatic simulation model automatically represents hydraulic behavior.

Verify the actual solver capabilities before claiming that a hydraulic component can be simulated.

A hydraulic symbol may be implemented graphically while its hydraulic physics remains unsupported.

## 10. Common mistakes

Avoid the following:

- Inventing dimensions without identifying their origin.
- Treating an image as proof of normative compliance.
- Treating every reference image as forbidden.
- Treating every reference image as permission to copy external graphical assets.
- Copying another application's internal implementation.
- Confusing a physical component with its standardized symbol.
- Confusing ports with printed numerals.
- Confusing symbol geometry with renderer-generated connection details.
- Drawing flow arrows manually when the behavior model already determines them.
- Duplicating generated circles or numerals.
- Modifying the renderer to compensate for incorrect catalog geometry.
- Changing validation rules merely to accept an invalid definition.
- Claiming a component is simulated when only its drawing exists.
- Marking an unresolved symbol as conformant.
- Updating a fixture without reviewing its diff.

## 11. Handling uncertainty

Classify uncertain information as one of:

- `verified`: supported by a normative requirement or an established project contract.
- `owner-defined`: explicitly supplied or approved by the project owner.
- `derived`: calculated from documented measures or geometric relationships.
- `inferred`: interpretation that still needs confirmation.
- `unresolved`: insufficient information to make a reliable decision.

When a detail is unresolved and affects the symbol's meaning or conformance, ask the owner before finalizing it.

Do not silently convert an inference into a verified requirement.

If the standard's interpretation is unclear, preserve the uncertainty in the review record.

## 12. Validation and acceptance

A symbol is ready for approval when:

- Its function is understood.
- Its reference provenance is recorded.
- Its normative requirements are identified.
- Its geometry has a documented basis.
- Its TOML definition passes catalog validation.
- Its rendered appearance has been inspected.
- Its geometry fixture is updated and reviewed.
- Its conformance review is complete or explicitly marked pending.
- The owner has reviewed the result.

A symbol with unresolved normative requirements must not be described as fully verified.

## 13. Repository rules

Before changing files, inspect the current repository state.

Follow:

- `AGENTS.md`
- `docs/symbology.md`
- Accepted ADRs.
- `assets/catalog/README.md`
- `assets/symbols/review/TEMPLATE.md`
- `assets/symbols/fixtures/README.md`

Do not create a second implementation of existing catalog, drawing or validation responsibilities.

Do not add third-party assets, proprietary symbol libraries, external application manuals or copied graphical resources to the repository.

A normative reference may guide reconstruction under ADR 0040. Its figures and text must not be copied into the repository.

Follow the branch, review, changelog and pull-request rules in `AGENTS.md`.

## 14. Definition of done

The work is complete when PunchiFluidLab contains an independently authored, valid and reviewed symbol definition that:

1. Represents the component requested by the owner.
2. Matches the intended functional behavior.
3. Follows the applicable normative requirements, or is explicitly identified as custom.
4. Uses the project's own TOML schema and rendering system.
5. Passes automated validation.
6. Has a reviewed geometry fixture.
7. Has a documented conformance review.
8. Has owner approval.
