# Reference parity: what to replicate, in what order

The reference is FluidSIM 4 / Pneumatics. Its manual and library were read to learn **what
the program does**; nothing from them enters this repository. Concretely, never copied:

- no symbol, icon, bitmap or sound from `sym/` (210 definition files) or `dida/` (533
  example circuits);
- no page, sentence or table from the manual;
- no file format, no `.ct` parser, no proprietary constant.

What *is* replicated is capability and behaviour, which is what a user of a program
experiences and what a free replacement must match. Every phase below is written as
observable behaviour and ends with evidence, not with a feature list.

Reference inventory measured on 2026-11-06: library `sym/` = 110 pneumatic, 50 electrical,
28 digital, 10 miscellaneous, 6 DDE, 6 GRAFCET; `dida/` = 533 example circuits; manual
chapters 1–8 covering editing, simulation, teaching material, layers and graphic objects,
text and references, images, parts lists, printing, TIFF/DXF, libraries, projects, saved
configurations and GRAFCET.

## Studying the reference (the gate)

The phases below say **what to build**; this section says **how it is learned** and where
the observation is kept. The procedure is
[.agents/skills/reference-parity](../.agents/skills/reference-parity/SKILL.md); the rule
is one line:

> A capability starts only after its observable behaviour is written in the log below, in
> our own words, with the evidence that will prove it.

The reference is the local copy at `$PUNCHI_REFERENCE_DIR` (default
`$HOME/Descargas/FLUID SIM NEUMATICO`), with its manual `doc/fl_sim_p.pdf`. It is read
only and outside the repository: observe it, never copy from it, and keep extracted text
and screenshots of it out of the tree. A file name in `sym/` says a function exists; only
an observation says what it does.

### Two arcs: the base, then better than the base

The reference is not a wish list to skim: it is **the base product**, and this project is
its replacement. So the work has two arcs, in this order and never mixed:

1. **Replicate the base.** Everything the reference lets a user do, this program does too,
   with the same observable behaviour. That is the floor: a replacement that does less is
   not a replacement, and "we do it differently" is not a reason to skip a function.
2. **Improve the base.** A base copied faithfully is the start, not the end. Every
   replicated capability then gets one recorded decision: keep it as the base has it, with
   the reason, or improve it, with the reason and the test that holds the improvement.

Parity first is not caution, it is the only way an improvement can be judged: a difference
you cannot explain against the base is a regression wearing a feature's name.

The improvement comes from the four rules at the end of this document — a keyboard path for
every action, a check that points at the fault, undo that never loses a drag, a theme that
follows the desktop — and from whatever the base makes painful. It may not be an excuse to
skip a function, may not break ISO 1219 conformance, and may not be invisible: the ledger
below says which capability differs and why.

### Coverage ledger

The whole base, not the part we happen to want: every function named by the inventory is a
row here, and its state is one of five words, never blank.

| State | Means |
|---|---|
| `none` | the base has it and we do not; a gap with no work started |
| `partial` | started; the entry says exactly which part is missing |
| `base` | behaviour matches the base, proven by the evidence in its entry |
| `better` | matches the base and improves it; the improvement is written and tested |
| `not adopted` | the base has it and we deliberately do not; the reason is written |

`not adopted` is the only state that needs an argument, and the argument is about **us**,
never about the base being wrong: a teaching-material library is not adopted because it is
someone else's content; a format is not adopted because it is proprietary.

| Phase | Base capability | State | Entry or reason |
|---|---|---|---|
| F1 | Draw a circuit: place, move, delete, undo, connect port to port | `base` | proven, per "Where we stand"; the keep-or-improve decision is still owed |
| F1 | Copy, paste and duplicate a component | `base` | one-slot clipboard, Ctrl+C/V/D and Shift+drag; multi-selection is not there yet, so copy works on the one selected component |
| F1 | Select, delete and re-route a pipe, T junctions | `base` | select, delete, the T and dragging the middle segment are done; the route is stored with the pipe and one gesture is one undo. Kept as the base: free routing is what a sketch needs |
| F1 | Editable properties of a placed component | `partial` | stroke, bore, rod, mass, supply pressure and valve/throttle conductance are edited per instance, applied as one undo and used by the model; name, summary, actuation/spring, starting piston position, mounting angle, leakage and damping are missing |
| F1 | Turn and reflect the selection | `base` | `Edición ▸ Girar` is `Alt+R`, `Edición ▸ Reflejar` is `Alt+M`, and `Ctrl` and a double click turns anticlockwise with `Shift` for clockwise, as the reference documents; the turn is stored per placement, drawn through one transform and undone in one step |
| F1 | The circuit check, all nine faults | `partial` | 5 of 9, with coordinates; four are blocked on F4 and F6 |
| F2 | The sheet is the network the solver runs | `base` | built from the document; a 5/2 valve file drives a double-acting cylinder, and the junction and throttle are part of the graph |
| F2 | Pipes coloured by pressure while running | `base` | each hose takes the pressure of its port; at atmosphere it keeps the neutral ink |
| F3 | Pressure gauge: a dial that reads the line it is connected to | `base` | one file with `behaviour = "gauge"`, a scale and one working port; the reading is drawn on the dial; two tests and a symbol review |
| F3 | Manometer, flow meter, force meter, strip chart | `partial` | the pressure gauge reads its port on a declared scale; the flow meter, force meter and strip chart are missing |
| F3 | Run until state change | `partial` | stops on a piston reaching an end or a valve changing position; relays and switches do not exist yet (F6) |
| F4 | Document v2, autosave, recent documents | `partial` | a versioned sheet format with atomic write, bounded parsing, Nuevo/Abrir/Guardar/Guardar como and Ctrl+N/O/S/Shift+S; no autosave, no recent list, no warning when closing an unsaved document |
| F4 | A project as a folder of documents | `none` | F4 |
| F4 | Layers, graphic objects, text and references, images | `none` | F4 and later |
| F5 | Parts list, title block legend, printing | `partial` | PNG/PDF export done; list and legend missing |
| F5 | DXF and TIFF interchange | `none` | F5: SVG first, DXF only if asked |
| F6 | Electrical, digital and GRAFCET libraries | `none` | each needs a `behaviour` variant and an ADR |
| F7 | The reference's teaching material | `not adopted` | its circuits are its content; F7 authors our own |
| F8 | Saved configurations, i18n, packaging | `none` | F8 |

Rows grow as the study log grows: a capability the base has and this table does not name is
a hole in the inventory, and a hole is found by reading the manual's chapter list, not by
waiting for someone to miss the function.

The rows marked `base` today come from work done before this ledger existed, and they are
honest about what that costs: the behaviour is proven, and the keep-or-improve decision each
one still owes is the first backlog the ledger makes visible.

### Study log

One entry per capability, newest first, with the five fields the skill defines: observed,
how observed, we have, gap, and the test that proves the gap closed. An entry without an
observation is a wish, and an empty row is not an entry.

| Phase | Capability | Observed, and from where | We have | Gap | Evidence |
|---|---|---|---|---|---|
| F1 | Turn and reflect the selection | §4.2 *Girar y Reflejar*, read 2026-11-12: the selected components turn 90°, 180° or 270° with **Edición ▸ Girar**; `Ctrl` and a double click turns the component under the pointer a quarter **anticlockwise**, and adding `Shift` turns it clockwise; **Edición ▸ Reflejar** reflects the marked objects on their own axis, and a group on the group's central axis. | a placement stores its quarter turns and its mirror; `Alt+R` and `Alt+M` are the keyboard path the reference lacks, and the ribbon carries both buttons. The symbol is turned by mapping every drawing call about the centre of its box, so a direction is not a second copy of the geometry, and the port numerals stay upright and legible. Pipes follow the ports because their ends come from the same mapping. A turn is one entry in the history. | groups do not exist yet, so a reflection is always about the symbol's own axis; the reference also rotates text, and this project keeps glyphs upright on purpose | `a_quarter_turn_moves_the_right_edge_to_the_bottom`, `four_quarter_turns_are_the_symbol_as_it_was_drawn`, `an_arc_turns_with_the_symbol_and_is_reversed_by_the_mirror`, `a_turned_surface_draws_the_same_calls_where_the_turn_puts_them`, `a_turn_is_one_undo_entry_and_four_of_them_are_the_symbol_as_it_was`, `a_turned_symbol_occupies_the_box_it_is_drawn_in`, the ABI test `a_turned_component_puts_its_ports_where_the_symbol_is_drawn`, and `--selftest` steps `edición: Ctrl y doble clic gira al revés`, `el puerto gira con el símbolo`, `un deshacer devuelve el símbolo a su sitio` and the two reflection steps |
| F1 | Copy, paste and duplicate a component | §4.2 *Copiar*, read 2026-11-06: the selected components go to the clipboard with **Ctrl+C** and come back with **Ctrl+V**; dragging with **Shift** held copies them, and the pointer changes to the copy symbol. The reference copies the selection as a whole; with one selection ours copies one component. | Ctrl+C copies the selected placement, Ctrl+V pastes it ten millimetres away, Ctrl+D duplicates it, and Shift+drag copies the component under the pointer and drags the copy; the copy keeps the catalog id, the box and the per-instance values, gets a new identity and is one undo | copying several components at once and the pipes between them, aligning and distributing a selection; these need multi-selection | `a_duplicate_keeps_the_values_gets_a_new_identity_and_one_undo_removes_it`, the ABI test `copy_paste_and_duplicate_are_one_change_each`, and the `--selftest` steps for paste, duplicate and Shift+drag |
| F1 | Editable properties of a placed component | §4.1 *Símbolos configurables* and appendix B *Cilindro de doble efecto*, read 2026-11-06: double-clicking a component (or Edit ▸ Properties) opens a dialog **for that instance**; the cylinder's parameter tab edits maximum stroke, piston position at the start, bore, rod diameter, mounting angle, internal leakage and moving mass, and the calculated areas follow. Appendix B gives the ranges (stroke 1..5000 mm, piston position 0..stroke, bore 1..1000 mm, rod 0..1000 mm). | a panel on the sheet, not a dialog: per-instance overrides of supply pressure, valve/throttle conductance, cylinder stroke, bore, rod and moving mass; Apply is one undo entry, the model is rebuilt from the document and a rod that is not smaller than the bore is refused | name and summary, actuation/spring, starting piston position, mounting angle, internal leakage and damping; the panel edits the numbers the model uses, not a full parameter tab | `editing_a_placement_is_one_undoable_change`, the ABI tests `a_cylinder_edit_changes_the_model_and_is_one_undo` and `a_supply_pressure_edit_reaches_the_gas`, and the `--selftest` step that edits the stroke, sees the model change and undoes it |
| F3 | Pressure gauge | Appendix B, *Manómetro*, read 2026-11-06: *el manómetro indica la presión en su conexión*; it is a measuring instrument with one connection and a scale. | `gauge.pressure`, a file with `behaviour = "gauge"`, a scale in the file and one working port; the network gives the port a node and adds no physics, the dial is the file's symbol and the reading is drawn from `Network::port_pressure` at the dial | a moving needle; the number is drawn, not the needle angle | `a_gauge_reads_its_port_and_adds_nothing_to_the_physics`, the ABI test `a_pressure_gauge_reads_the_line_it_is_connected_to`, and `assets/symbols/review/gauge.pressure.md` |
| F3 | Run until state change | §3.2, read 2026-11-06: the reference has a run mode that leaves the pause when one of four things happens — **the cylinder rod reaches an end stop, a valve is actuated or switched, a relay picks up, or a switch is actuated** — and stops again at the next state change. | a **Hasta cambio** command in the transport: it enters simulation, advances until the first piston reaches an end or any valve changes position, leaves the transport paused there, and gives up after four simulated seconds; the selftest reaches the stop and proves the pause | relays and switches (they arrive with F6); the reference animates the run, we compute it and repaint once | `--selftest` steps `transporte: hasta cambio se detiene`, `se detiene en el tope` and `queda en pausa` |
| F4 | Saving and opening a circuit | Appendix A.1 *Archivo*, read 2026-11-06: the reference has **Nuevo** (Ctrl+N), **Abrir...** (Ctrl+A), **Cerrar** (Ctrl+W, warning when the circuit is unsaved), **Guardar** (Ctrl+G) and **Guardar como...**; the stored circuit is the `.ct` file and the same window keeps the name. | Nuevo, Abrir, Guardar and Guardar como in the Archivo tab and as Ctrl+N/O/S/Shift+S; a JSON sheet with placements by catalog id, pipes by placement id and port, each pipe's route and each instance's values; atomic write (temporary file plus rename), an 8 MiB read ceiling, unknown keys rejected and every number range-checked; a failed open leaves the current document untouched | autosave, a recent-documents list, a warning on closing an unsaved circuit, and a migration from an older sheet version (there is only v1) | `a_sheet_round_trips_byte_identically`, `an_unknown_key_is_rejected_instead_of_ignored`, `a_future_format_is_refused`, `bounds_are_checked_before_the_document_is_trusted`, `a_sheet_is_written_atomically_and_read_back`, the ABI test `a_sheet_saves_and_opens_with_its_edits_and_routes`, and the `--selftest` step that saves, edits, opens and counts |
| F2 | The sheet is the network the solver runs | Appendix B, *Válvula neumática distribuidora de 5/2 vías* and *Cilindro de doble efecto*, read 2026-11-06: the 5/2 valve conducts 1→4 while its pilot is applied and returns by spring to 1→2; the double-acting cylinder is driven by applying air alternately to the two sides of the piston; its editable parameters are stroke 1..5000 mm (100), piston position, bore 1..1000 mm (20) and rod 0..1000 mm (8). §3.1 adds that the simulation colours cylinders and hoses and that the animation keeps the real time proportion. | a network of lumped volumes and ISO 6358 restrictions built from the document's placements and pipes; a 5/2 valve file with a momentary lever drives a double-acting cylinder with two chambers and the annulus area from the rod; a junction is one node; a throttle is a restriction; port pressure is readable | open working ports are sealed instead of venting; a hose conducts but stores no volume; a valve other than the first is actuated by placement, and telemetry reports the last one touched | `a_five_port_valve_file_drives_a_double_acting_cylinder`, `a_chain_is_the_same_circuit_the_fixed_model_simulates`, `a_junction_joins_its_three_ports_into_one_volume`, `a_throttle_between_the_valve_and_the_cylinder_slows_the_fill`, `the_gas_that_entered_is_the_gas_the_network_holds`, and the ABI test `a_five_port_valve_and_a_double_acting_cylinder_are_a_circuit_of_files` |
| F2 | Hoses coloured by pressure while running | §3.1, read 2026-11-06: as soon as the results are computed, *los cilindros y los conductos* appear coloured, and the animation is proportional to real time. | every document hose is drawn from the pressure at its port as a fraction of the supply span; the status bar carries the number too, so colour is never the only signal | none for the base gesture | `--selftest` paints a running circuit, and `hose_pressure` is fed from `Network::port_pressure` (unit test `a_port_reports_the_pressure_a_gauge_would_read`) |
| F1 | Re-route a hose by dragging a segment | §3 of the manual, read 2026-11-06: the same paragraph that documents automatic T junctions tells the reader to *drag the segments of the hose so the circuit reads clearly*, so the route belongs to the document and not to the automatic elbow. | the middle segment of a hose follows the pointer while the button is held; the offset is stored in tenths of a millimetre with the pipe, one gesture is one undo, and the press that selects a pipe also starts the drag | none for the base gesture | `dragging_a_pipe_segment_is_one_undo_entry_and_comes_back`, `a_routed_pipe_survives_the_undo_of_an_unrelated_move`, the ABI test `a_pipe_segment_can_be_dragged_and_undo_puts_it_back`, and the `--selftest` step that drags a hose and proves one undo restores it |
| F1 | T junctions | §3 of the manual, read 2026-11-06. To join a component to a hose that is already drawn a T is needed, and the reference **inserts it automatically when a hose connection is placed directly on an available hose**: the connection grid changes shape as it hovers, and the T appears **where the mouse is released**. The same paragraph then tells the reader to *drag the segments of the hose so the circuit reads clearly*, which is the entry above. | dropping a hose end on a hose inserts a T there, splits the hose in two and joins the third port; the T is also a palette entry, which the base does not offer | none: the drag is the entry above | `a_junction_splits_a_pipe_and_the_whole_splice_undoes_as_one`, `a_splice_that_cannot_finish_leaves_the_pipe_untouched`, `a_junction_alone_is_not_an_open_connection`, and the `--selftest` step that drags a port onto a hose and proves one undo reverts it |
| F1 | Completing the circuit check | Chapter 4.10 *Control de gráficos*, read 2026-11-06. The command is **Ejecutar ▸ Comprobación previa** (F6). It lists nine graphic faults, in this order: an object outside the drawing surface; a pipe through a component; overlapping pipes; overlapping components; overlapping and mismatched connections; **open pneumatic connections**; two cylinders with one designation; labels that do not match; and pipes running through circuits they are not connected to. It then shows them **one after another, each in its own dialog**, and, once they are read, **lets the simulation go ahead anyway**. The same section's worked example carries faults 1 to 3. | 5 of 9, each with its coordinates and **marked on the drawing** with a dashed ring and a cross: outside the sheet, pipe through a component, overlapping pipes, overlapping components, open connection. An exhaust port is exempt, because the reference's own example valve breathes through port 3. | the four that need a feature we do not have yet — mismatched connection (needs the electrical library, F6), two cylinders with one designation, a label that does not match, a pipe through an unrelated circuit (all three need designations and circuit grouping, F4). | one unit test per fault in `punchi-fluid-check`, one ABI test proving every fault has a place to point at, and `--check` printing each finding with its coordinates |

The observation also settles a design question before it is asked: the base reports faults
as a **chain of modal dialogs** and does not stop the run. This project has a standing rule
against a modal on the normal path and against a check the user has to map back onto the
drawing, so the keep-or-improve decision here is recorded rather than discovered — the
fault list is the base's, the presentation is ours, and the rule the improvement rests on is
"a check that points at the fault". The `base` behaviour of *reporting all nine in that
order* is still owed; only the dialogs are not.

## Where we stand

| Area | State |
|---|---|
| Model, catalog of definition files, versioned format, CLI | done |
| Drawing in millimetres, `Surface`, PNG/PDF | done |
| Window: ribbon, palette, sheet, status, theme, transport | done |
| Editing: place, move, delete, undo/redo, connect port to port | done |
| Editing: select/delete and re-route a pipe, T junctions | done |
| Editing: circuit check (5 of the 9 faults, each with a place to point at) | done |
| Editing: editable properties of a placed component | partial: the numbers the model uses; the remaining fields are in the ledger |
| The sheet is the circuit the solver runs | done: network built from the document |
| Instruments and state-change runs | partial: run until a change and a pressure gauge; flow, force and the strip chart are not started |
| Saving and opening a circuit | not started (F4) |
| Teaching material, electrical/digital/GRAFCET libraries | not started |

## Phases

Each phase is independently shippable, ends with a working program, and names the evidence
that proves it. The order is by *what makes the product usable*, not by what is easy.

### F1 — Editing, complete (next)

A circuit can be drawn without a workaround.

- Select and delete a pipe; re-route one by dragging it.
- Drop a pipe on another pipe and get a T junction.
- The properties panel edits what the definition file allows: name, summary, conductance,
  spring/actuation on a valve, stroke and bore on a cylinder — and writes it back.
- **Check the circuit** before running, with the reference's own list of faults as the
  requirement: an object outside the sheet, a pipe crossing a component, overlapping pipes,
  overlapping components, a pipe whose ends do not fit, an open pneumatic connection, two
  cylinders with one designation, a label that does not match, a pipe through an unrelated
  circuit. Each fault is a variant of one error enum, has a test, and is shown where it
  happens — not in a dialog the user has to map back onto the drawing.
- Evidence: one test per fault, a capture of the drawing with two faults marked, and an
  undo/redo test over a pipe edit.

### F2 — The sheet is the circuit

- The solver builds a network of volumes from the connections, not from a fixed three.
- A 5/2 valve dropped in as a **file only** drives a double-acting cylinder, no code change.
- Pipes are coloured by pressure while running; the fixed circuit disappears as a special
  case.
- Evidence: mass balance over the network, the 5/2 test, and the drawing of the same
  document before and after running differing only in ink.

### F3 — Instruments and runs

- Manometer, flow meter, force meter and a strip chart, as catalog entries with readings.
- Run until state change, as the reference does, plus the transport already built.
- Evidence: a gauge within 0.1 % of the status bar, a 60 s chart that drops old samples
  without growing, and no allocation in the loop.

### F4 — Documents and projects

- File format v2: components by catalog id, pipes, view, layers.
- Autosave and crash-safe write; recent documents; a project is a folder of documents.
- Evidence: a migration test from v1, an atomic-write test with a simulated crash, and a
  round trip that returns a byte-identical document.

### F5 — Output

- Parts list from the document, printed with the sheet.
- The title block legend the reference has; export already exists for PNG and PDF.
- Interchange: SVG first (it is a standard and readable), DXF later and only if asked.
- Evidence: a parts list whose counts match the document, and a PDF that opens with the
  legend and no host name.

### F6 — Libraries beyond pneumatics

- Electrical, digital and GRAFCET each need a `behaviour` variant — and each variant needs
  an ADR, because a new variant is the one exception to "a component is a file".
- The catalog stays untrusted input; the loader keeps bounding everything.
- Evidence: an ADR per variant, one worked circuit per library, and the solver's invariants
  holding on a mixed circuit.

### F7 — Teaching material

- Our own example circuits, authored for this project, with the reasoning written down.
- A presentation mode: a sequence of steps over one document.
- Evidence: every example opens, runs and passes the F1 check; no angle, sentence or file
  derived from the reference's material.

### F8 — 1.0

- i18n complete, AppImage and Flatpak, signed release, user guide, and the reproducibility
  the changelog already promises.

## Rules that do not bend while replicating

1. **The base is the floor, and improving it is the goal.** First the same observable
   behaviour, then a recorded decision per capability: keep it as the base has it and say
   why, or do it better and prove it. A replacement that does less is not a replacement; a
   difference that is only taste is not an improvement.
2. **Behaviour, never content.** A symbol is drawn by hand against ISO 1219-1 and reviewed;
   an example circuit is authored here.
3. **A capability is done when it is proven**, in the sense of
   [.agents/skills/rust-testing-and-verification](../.agents/skills/rust-testing-and-verification/SKILL.md).
4. **Nothing is faked to look like the reference.** A pipe that does not carry air says so
   until it does; a panel that cannot edit yet says where it will.
5. **Improve where it is cheap and honest**: a keyboard path for every action, a check that
   points at the fault, undo that never loses a drag, and a theme that follows the desktop.
