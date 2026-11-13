# Capabilities: what the program does, in what order

A free simulator of pneumatic and hydraulic circuits, for a student with no licence, no
budget and no connection. This document says what the program must be able to do, in what
order it gets built, how each capability is proven, and what is deliberately left out.

It is a plan written in our own words. It describes behaviour, never content: no symbol,
icon, sound, example circuit, manual page or file format from another program enters this
repository, and no such program is named here.

## The gate

A capability starts only when it is written down in this file: what a student must be able
to do, what they see, what changes in the document or the model, and the test that will
prove it. A step without an entry is not ready; an entry without a purpose is a wish.

The procedure is [.agents/skills/capability-plan](../.agents/skills/capability-plan/SKILL.md).

## What settles what

| Question | Source |
|---|---|
| What does this symbol look like? | ISO 1219-1, in the diagram conventions of ISO 1219-2 and ISO 1219-3 |
| How much gas flows through this restriction? | ISO 6358, including the choked branch |
| What is this component called? | ISO 5598 vocabulary |
| Does this belong in the program at all? | the teaching goal: a student must be able to do it, understand it or check it |
| How good is our answer? | the test named in the entry, at the level the claim needs |

When those sources leave a question open, the entry says so instead of guessing. The plan
never depends on material the community cannot open.

## Two arcs: cover, then improve

1. **Cover.** The capability does what a teaching simulator is expected to do, proven by
   the evidence in its entry. A product that does less than a student needs is not a
   replacement for anything.
2. **Improve.** Then it gets one recorded decision: keep it as it is, with the reason, or
   improve it, naming the rough edge, what we do instead, why it is cheap and honest, and
   the test that holds it.

A difference you cannot explain is a regression wearing a feature's name. Improving is not
redecorating: if the change cannot be stated as what a student could not do before, it is
taste, and taste does not justify a divergence.

The project's own improvements are fixed: a keyboard path for every action, a check that
points at the fault instead of describing it, undo that never loses a drag, and a theme
that follows the desktop.

## Coverage ledger

Every capability the program is expected to have is a row here, and its state is one of
five words, never blank.

| State | Means |
|---|---|
| `missing` | expected and absent; a gap with no work started |
| `partial` | started; the entry says exactly which part is missing |
| `covered` | the behaviour is there, proven by the evidence in its entry |
| `improved` | covered, plus a named improvement with its test |
| `not planned` | expected and deliberately out of scope; the reason is about **us** — licence, scope, content we do not own — never contempt for a user's need |

`not planned` is the only state that needs an argument. A function the category expects is
the default; dropping it is the exception.

| Phase | Expected capability | State | Entry or reason |
|---|---|---|---|
| F1 | Draw a circuit: place, move, delete, undo, connect port to port | `covered` | proven by the ABI tests and `--selftest`; the keep-or-improve decision is still owed |
| F1 | Copy, paste and duplicate a component | `covered` | one-slot clipboard, Ctrl+C/V/D and Shift+drag; with one selection it copies one component |
| F1 | Select, delete and re-route a pipe, T junctions | `covered` | select, delete, the T and dragging the middle segment; the route is stored with the pipe and one gesture is one undo |
| F1 | Select several components and edit them together | `covered` | band, Shift+click and Ctrl+E; move, copy, turn, reflect and delete are each one change |
| F1 | Line up and distribute a selection | `partial` | four edges aligned to the furthest object; equal spacing and the other four alignments are `missing` |
| F1 | Editable properties of a placed component | `partial` | stroke, bore, rod, mass, supply pressure and conductances are edited per instance and applied as one undo; the designation is covered separately; summary, actuation, starting position, mounting angle, leakage and damping are missing |
| F1 | Turn and reflect the selection | `covered` | quarter turns and mirror stored per placement, with `Alt+R` and `Alt+M` |
| F1 | Check the circuit before running, nine faults | `partial` | 7 of 9, each shown where it happens; the remaining two need a pipe whose ends do not fit and the electrical library with its encoder (F6) |
| F1 | Give a component a designation | `covered` | a name per placement, edited in the properties panel, drawn under the symbol and saved with the sheet; 40 characters, trimmed, no control characters |
| F2 | The sheet is the network the solver runs | `covered` | built from the document; a 5/2 valve file drives a double-acting cylinder; junction and throttle are part of the graph |
| F2 | Pipes coloured by pressure while running | `covered` | each hose takes the pressure of its port; at atmosphere it keeps the neutral ink |
| F3 | Pressure gauge: a dial that reads the line it is connected to | `covered` | one file with `behaviour = "gauge"`, a scale and one working port; two tests and a symbol review |
| F3 | Manometer, flow meter, force meter and a strip chart | `partial` | the pressure gauge reads its port, the flow meter reads the gas it carries and the recorder charts a chosen quantity over 60 s; the force meter is `not planned` until a mechanical domain exists |
| F3 | Strip chart | A student has to see how a value changed, not only what it is now: whether the pressure settled, whether two strokes took the same time, what a valve change did. | `chart.recorder`, a file with `behaviour = "recorder"`, a 60 s window and 10 samples per second; placed on the sheet with no ports, its source chosen in the inspector among the quantities the components can report (pressure at a port, flow through a meter, position of a cylinder), saved with the placement in format 5; the session keeps one bounded ring per recorder and the sheet draws the curve auto-scaled, with its current value ([ADR 0015](adr/0015-a-recorder-samples-a-named-quantity.md)) | a moving cursor and exporting the series as data | `a_recorder_keeps_its_window_and_drops_the_oldest`, the ABI test `a_recorder_charts_the_quantity_it_is_bound_to`, the `--selftest` steps, and `assets/symbols/review/chart.recorder.md` |
| F3 | Force meter | A student may eventually need the force a rod delivers, as a load cell would show it. | `not planned`: the model has pressure, volume and position, but no mechanical link between the rod and a load, so any number would be `p·A` wearing the name of a force measurement | a mechanical domain (a rod with a load and a reaction), which needs its own ADR and phase | the ledger says `not planned`, with the reason above, instead of an instrument that teaches the wrong quantity |
| F3 | Run until state change | `partial` | stops when a piston reaches an end or a valve changes position; relays and switches need F6 |
| F4 | The document knows it is modified, and the program asks before losing it | `covered` | the title carries `•` while the sheet holds work that is not in the file; Nuevo, Abrir and closing stop at a confirmation that offers Guardar, Descartar or Cancelar, and a save clears the mark |
| F4 | A safety copy while the work is unsaved | `covered` | written every thirty seconds while the document is modified; on start it is offered and recovered as unsaved work, never loaded on its own |
| F1 | Give a component a designation | A name is how a schematic is read and how a parts list is written: *1A1* tells the reader which cylinder the drawing means, and a second document can refer to it. | `Placement::designation`, one undoable change, trimmed, at most 40 characters, refused rather than truncated; the sheet stores it in format 3 and an older sheet reads with no names; the drawing prints it under the box and the properties panel edits it as a text field, not a number. The check reports two actuators with one name, which is the reference's fault 7. | circuit-level designations are the sheet `title` and are not editable yet; the label-mismatch fault needs the displacement encoder (F6) | `a_designation_is_one_undo_and_an_empty_name_is_no_name`, `a_version_two_sheet_still_reads_and_gains_an_unnamed_component`, `a_designation_that_cannot_be_read_is_refused`, `two_cylinders_with_one_designation_are_reported_at_the_second_one`, the ABI test `a_component_can_be_named_and_the_name_survives_a_save_and_an_undo`, and `--selftest` steps from `designation: the example has no names` to `designation: two cylinders with the same name` |
| F5 | Check a circuit without opening the window | `covered` | `punchi-fluid-cli check <file> [--json] [--strict]` over the sheet format, with a versioned JSON contract ([ADR 0011](adr/0011-the-check-is-a-public-command.md)) and exit codes a pipeline can trust; the reference has no equivalent because its format and its check are both inside its window |
| F4 | Recent documents | `covered` | the last ten documents opened or saved, in `punchi-fluid-file::recent`, bounded and written atomically; Archivo shows them and one click reopens it; an entry whose file is gone is dropped instead of offered |
| F4 | Versioned document format, autosave and recent documents | `covered` | format 3 with atomic write and bounded parsing, the four file commands, a safety copy every thirty seconds, the last ten documents and the unsaved-work warning; a sheet from an older version reads with defaults |
| F4 | A project of several circuits in one document | `covered` | format 4, the selector, add/rename and the `PipeAcrossCircuits` fault; evidence in the decision log |
| F4 | Layers, graphic objects, text and references, images | `missing` | F4 and later |
| F6 | The control circuit | Without it the button on the sheet is a picture: the student wires a rail to a button and a button to a coil, and nothing moves. | a **boolean net**: an electrical supply marks its positive terminal live, liveness travels through wires and **closed contacts**, and a **solenoid valve** sits in its actuated position while its coil terminal is live; a click toggles a contact and the valve follows at once ([ADR 0018](adr/0018-the-control-circuit-is-a-boolean-net.md)) | relays, limit switches, and the voltage/current model the boolean one replaces | `a_contact_energises_the_solenoid_it_is_wired_to`, `a_contact_shows_whether_it_is_open_or_closed`, the nine `--selftest` steps `control: …`, and the solenoid valve's review |
| F6 | The electrical library, first slice | A pneumatic diagram is half a machine: the valve is moved by a solenoid and the solenoid by a button, and a student who draws only the air has drawn half the circuit. | a **24 V supply** and a **push button** as files, with the `electrical` port role, the electrical line class (a wire between two terminals is dotted), the check leaving terminals alone and the solver adding nothing to the gas ([ADR 0017](adr/0017-the-electrical-circuit-is-a-second-domain.md)) | the control model — a live line, a contact joining two lines, a coil moving a valve — and the rest of the electrical library | `a_hose_between_two_terminals_is_drawn_dotted`, `a_pneumatic_hose_keeps_its_own_class`, `an_open_electrical_terminal_is_not_a_pneumatic_fault`, `an_electrical_component_adds_nothing_to_the_gas`, `a_contact_needs_two_electrical_terminals`, and the two symbol reviews |
| F5 | Check a circuit without opening the window | A course with forty student circuits is corrected by hand in a program that only checks inside its own window: forty windows, and nothing a machine can read. The command returns the same faults the window draws, in the same order, with the same words, plus a JSON object that carries its own version and exit codes a job can act on. | `punchi-fluid-cli check`, with `--json` for the contract and `--strict` to turn findings into a failed job; exit `0` checked, `1` unreadable, `2` wrong command line, `3` faults under `--strict`. It reads the sheet format the editor saves, so what a student hands in is what is checked. | it uses the built-in catalog, so a circuit that depends on a user catalog file is checked against the built-in definition; `--catalog` is a later phase | `a_clean_sheet_passes_and_says_so`, `a_fault_is_listed_and_only_fails_the_job_when_it_is_asked_to`, `the_json_carries_its_version_and_one_object_per_fault`, `a_broken_file_is_an_error_in_the_shape_that_was_asked_for`, `a_wrong_command_line_is_refused_instead_of_guessed`, and the gate's own smoke test over a clean and a faulty sheet |
| F5 | Choose the paper a drawing is printed on | `covered` | the ribbon lists the nine named papers, landscape and a **custom size**; the panel sets the four margins and the PNG resolution, and `--paper` reaches the named ones headless. Missing: it does not remember the last choice per user |
| F5 | Print what is drawn | `covered` | `pf_session_export` builds the page from the document in print ink, Archivo has Exportar, and `--export` prints without a window; the caller supplies the date, the drawing is fitted to the frame and the scale is recorded, and the ABI test `an_export_carries_the_caller_date_and_fits_a_large_circuit` proves it |
| F5 | Parts list, title-block legend, printing | `covered` | the printed sheet carries the frame, the title block, a legend of line classes and port numerals and the **parts list**; a full symbol key is the remaining item |
| F5 | DXF and TIFF interchange | `missing` | F5: SVG first, DXF only if asked |
| F6 | Electrical, digital and GRAFCET libraries | `partial` | a 24 V supply, a push button and a solenoid 3/2 valve as files, the `electrical` port role and line class, and the **boolean control model**: a rail energises a line, a closed contact joins two lines and a coil moves its valve ([ADR 0018](adr/0018-the-control-circuit-is-a-boolean-net.md)); the rest of the electrical library, the digital library and GRAFCET are missing |
| F7 | Third-party teaching material | `not planned` | that content is not ours; F7 authors our own circuits |
| F8 | Saved configurations, i18n, packaging | `missing` | F8 |

Rows grow as the log grows. A capability the category expects and this table does not name
is a hole, and holes are found by reading the list of teaching topics, not by waiting for
someone to miss a function.

## Where we stand

| Area | State |
|---|---|
| Model, catalog of definition files, versioned format, CLI | done |
| Drawing in millimetres, `Surface`, PNG/PDF | done |
| Export from the window and without one | done: the date comes from the caller and the drawing is fitted to the page |
| Window: ribbon, palette, sheet, status, theme, transport | done |
| Editing: place, move, delete, undo/redo, connect port to port | done |
| Editing: select/delete and re-route a pipe, T junctions | done |
| Editing: multi-selection with move, copy, turn, reflect, delete, align and even spacing | done |
| Editing: circuit check (7 of the 9 faults, each with a place to point at) | done |
| Projects: several circuits in one document with a selector | done: format 4, add/rename/select, save/open, and the cross-circuit fault |
| Editing: editable properties of a placed component | partial: the numbers the model uses; the remaining fields are in the ledger |
| The sheet is the circuit the solver runs | done: network built from the document |
| Instruments and state-change runs | partial: run until a change, a pressure gauge and a flow meter; force and the strip chart are not started |
| Saving and opening a circuit | done: versioned sheet format, atomic write, save as, open from the command line |
| Parts list, full legend, SVG | not started |
| Teaching material, electrical/digital/GRAFCET libraries | not started |

## Phases

Each phase is independently shippable, ends with a working program, and names the evidence
that proves it. The order is by *what makes the product usable*, not by what is easy.

### F1 — Editing, complete (next)

A circuit can be drawn without a workaround.

- The properties panel edits what the definition file allows: name, summary, conductance,
  spring/actuation on a valve, stroke and bore on a cylinder — and writes it back.
- **Check the circuit** before running, with the nine faults a teaching simulator is
  expected to find: an object outside the sheet, a pipe crossing a component, overlapping
  pipes, overlapping components, a pipe whose ends do not fit, an open pneumatic
  connection, two cylinders with one designation, a label that does not match, a pipe
  through an unrelated circuit. Each fault is a variant of one error enum, has a test, and
  is shown where it happens — not in a dialog the user has to map back onto the drawing.
- Equal spacing between the marked objects, and the four remaining alignments.
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
- Run until state change, with the transport already built.
- Evidence: a gauge within 0.1 % of the status bar, a 60 s chart that drops old samples
  without growing, and no allocation in the loop.

### F4 — Documents and projects

- File format v2: components by catalog id, pipes, view, layers.
- Autosave and crash-safe write; recent documents; a project is a folder of documents.
- Evidence: a migration test from v1, an atomic-write test with a simulated crash, and a
  round trip that returns a byte-identical document.

### F5 — Output

- Parts list from the document, printed with the sheet, and the title-block legend.
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
- Evidence: every example opens, runs and passes the F1 check; nothing in it derived from
  another program's material.

### F8 — 1.0

- i18n complete, AppImage and Flatpak, signed release, user guide, and the reproducibility
  the changelog already promises.

## Decision log

One entry per capability, newest first: why it exists, how it behaves, what we have, the
gap, and the test that proves the gap closed. The behaviour is written as a sentence a user
would say, in our own words. The first four fields are the gate to start; the evidence is
the gate to call it done.

| Phase | Capability | Purpose and behaviour | We have | Gap | Evidence |
|---|---|---|---|---|---|
| F4 | A project of several circuits in one document | A course or a machine needs the power circuit, its control and a variant to travel together; handing in one file per circuit loses the relationship and one undo cannot cover a move between them. A document holds several circuits, the window shows one at a time with a selector, and a pipe that joins two circuits is reported as the ninth fault. | [ADR 0013](adr/0013-a-project-is-one-document.md) and format 4: a circuit list, project-level placements and pipes with a circuit id, one active sheet and one undo history. The ABI adds, renames and selects circuits, and the window has the selector and the rename panel. | the geometric crossing of the reference (circuits are pages) and a parts list by circuit | `a_version_three_sheet_reads_as_one_circuit`, `a_project_keeps_two_circuits_and_a_pipe_between_them`, `a_component_or_pipe_with_an_unknown_circuit_is_refused`, the ABI test `circuits_can_be_added_renamed_and_survive_a_save_and_open`, `a_pipe_between_two_circuits_is_the_ninth_fault`, and the `--selftest` steps from `circuit: the project starts with one` to `circuit: the selected one is active` |
| F4 | Recent documents | A student opens the same three circuits all day; walking the file dialog every time is the cost that makes a program tiring. The last ten documents sit in Archivo, newest first, and one of them opens with a click. | `punchi-fluid-file::recent` holds at most ten paths, refuses a path that is empty, longer than 4096 bytes or carrying a NUL, deduplicates by moving a repeat to the front and writes atomically; the sheet file and the list are both untrusted input with the same treatment. The core remembers on every save and open, and `openRecent` drops an entry whose file has gone, so the menu is the files that can still be opened. | a project as a folder is still missing, and the list is per user, not per project | `the_newest_document_is_first_and_a_repeat_moves_up`, `the_list_stops_at_the_limit`, `a_list_round_trips_and_never_writes_a_temporary_file`, `an_entry_whose_file_is_gone_is_dropped_on_read`, `a_path_too_long_or_with_a_nul_is_refused_instead_of_stored`, the ABI test `the_documents_that_were_saved_or_opened_are_remembered_and_can_be_reopened`, and ten `--selftest` steps from `recent: the list is emptied` to `recent: forgetting leaves the list empty` |
| F4 | A modified document that is copied and that asks before it is lost | Work that closes without a question is work the program takes away: the student edits a circuit, closes the window, and the session is gone. The document says it has changed, the title says so too, and the program asks once before discarding it; besides the file the user saves, a copy is written so a crash costs seconds instead of the session. | the document counts its own changes and the ABI compares that number with the one from the last save or open; `•` in the title; a confirmation for Nuevo, Abrir and closing that names the work and offers Guardar, Descartar or Cancelar; a safety copy every thirty seconds, **offered** on start and recovered as unsaved work without a path, so the next save asks where it goes. | recent documents and a project folder are still missing; the mark does not fall when an undo returns the sheet to its saved shape, which `Document::revision` states and which is deliberate — the circuit was edited. | `the_revision_counts_edits_undo_and_redo`, the ABI test `unsaved_work_is_known_and_survives_a_crash_in_a_safety_copy`, and twelve `--selftest` steps from `unsaved: saving leaves the document clean` to `unsaved: starting over begins clean` |
| F1 | Line a selection up on the component furthest out | Aligning several objects drags them until they reach the one already at that margin, so a row of symbols ends up flush without the user measuring anything. | four buttons — left, right, top, bottom — that move each marked placement so its box edge meets the furthest one; the whole alignment is one change, so one undo puts the circuit back; the box used is the turned one, so an alignment is not thrown off by a symbol on its side | eight alignments, and equal spacing between objects | `aligning_moves_everything_onto_the_object_already_at_that_edge`, the ABI test `lining_up_the_selection_moves_it_onto_the_component_furthest_out`, and the `--selftest` steps `align: the left edges coincide` and `align: one undo restores it` |
| F1 | Select several components and edit them together | A student moves, copies, turns or deletes a block of the schematic as one thing, and one undo takes the gesture back. A click selects one; a band selects what it covers; the editing commands apply to everything marked. | the selection is a set in the session; a click replaces it, Shift+click toggles one, the band and Ctrl+E mark several, and the last one marked is the one the properties form opens on. Move, copy, turn, reflect and delete are each one change. Shift+drag copies, told apart from Shift+click by whether the pointer moved | the pipe is not part of a multi-selection, and a copy does not carry the pipes between components | `moving_a_selection_is_one_drag_and_one_undo`, `removing_and_copying_a_selection_are_one_change_each`, `turning_a_selection_keeps_the_mirror_of_each_component`, the ABI test `a_selection_of_several_components_moves_copies_and_goes_away_as_one_change`, and the `--selftest` selection steps |
| F1 | Turn and reflect the selection | Symbols are drawn in one orientation and have to be turned or mirrored to read as a real schematic, without redrawing the geometry. | a placement stores its quarter turns and its mirror; `Alt+R` and `Alt+M` are the keyboard path, and the ribbon carries both buttons. The symbol is turned by mapping every drawing call about the centre of its box, so a direction is not a second copy of the geometry, and the port numerals stay upright. Pipes follow the ports. A turn is one entry in the history | groups do not exist yet, so a reflection is about the symbol's own axis; text stays upright on purpose | `a_quarter_turn_moves_the_right_edge_to_the_bottom`, `four_quarter_turns_are_the_symbol_as_it_was_drawn`, `an_arc_turns_with_the_symbol_and_is_reversed_by_the_mirror`, `a_turn_is_one_undo_entry_and_four_of_them_are_the_symbol_as_it_was`, the ABI test `a_turned_component_puts_its_ports_where_the_symbol_is_drawn`, and the `--selftest` turn and reflection steps |
| F1 | Copy, paste and duplicate a component | Repeating a symbol is the most common edit in a schematic; it must keep the per-instance values and cost one undo. | Ctrl+C copies the selected placement, Ctrl+V pastes it ten millimetres away, Ctrl+D duplicates it, and Shift+drag copies the component under the pointer and drags the copy; the copy keeps the catalog id, the box and the values, gets a new identity and is one undo | copying several components at once with the pipes between them; needs multi-selection on pipes | `a_duplicate_keeps_the_values_gets_a_new_identity_and_one_undo_removes_it`, the ABI test `copy_paste_and_duplicate_are_one_change_each`, and the paste, duplicate and Shift+drag `--selftest` steps |
| F1 | Editable properties of a placed component | Two cylinders of the same catalog entry are rarely the same cylinder: the student edits the stroke or the bore of one instance and the model follows. | a panel on the sheet, not a dialog: per-instance overrides of supply pressure, valve/throttle conductance, cylinder stroke, bore, rod and moving mass; Apply is one undo entry, the model is rebuilt from the document, and a rod that is not smaller than the bore is refused | summary, actuation/spring, starting piston position, mounting angle, internal leakage and damping | `editing_a_placement_is_one_undoable_change`, the ABI tests `a_cylinder_edit_changes_the_model_and_is_one_undo` and `a_supply_pressure_edit_reaches_the_gas`, and the `--selftest` step that edits the stroke, sees the model change and undoes it |
| F1 | Pipe classes | A pilot line and a drain line are read by their dashes before anyone reads a label, and a sheet drawn with every hose solid hides which line commands and which evacuates. | **Editar ▸ Conducto** gives the selected hose the working, pilot or drain class; the pattern comes from the class, the pipe keeps it in format 6, and one undo returns the previous one | the electrical class, which waits for F6 and is not offered yet | `a_pipe_takes_another_line_class_and_keeps_it_in_the_file`, `a_pipe_keeps_its_class`, `a_version_five_project_reads_its_pipes_as_working_lines`, and the six `--selftest` steps `pipe class: …` |
| F1 | Blind plugs and silencers | A line that is capped to put it out of service has to say so, or the check reports a fault the student cannot clear; an exhaust with a silencer has to breathe through it, or the drawing lies about the cylinder it serves. | `plug.blind` caps one port and adds no physics, and the check accepts it; `silencer.pneumatic` vents one exhaust through its declared conductance, so a narrow one slows the return ([ADR 0016](adr/0016-a-termination-is-a-component.md)) | the noise itself, which the model does not have and the review says so | `a_blind_plug_traps_the_air_and_so_holds_the_rod`, `a_silencer_slows_the_exhaust_it_is_fitted_to`, `a_capped_port_is_not_an_open_connection`, the catalog tests, and the two symbol reviews |
| F1 | Even spacing between marked objects | Two symbols dragged apart set the drawing's rhythm, and the rest of the row has to follow without measuring by eye; a reader notices an uneven gap long before a wrong one is named. | **Repartir ▸ Horizontal / Vertical** spreads three or more symbols so the gaps between their **boxes** are equal, keeping the two outermost in place; the whole spread is one undo | snapping a spread to the grid, and spacing measured between centres as an option | `even_spacing_equalises_the_gaps_and_keeps_the_ends`, `even_spacing_is_one_undo`, `even_spacing_refuses_what_it_cannot_do`, the ABI test `even_spacing_keeps_the_ends_and_is_one_undo`, and the four `--selftest` steps `spacing: …` |
| F1 | Re-route a hose by dragging a segment | A route that crosses a symbol has to be corrected without deleting the hose and losing the connection. | pressing a hose selects it; dragging a segment moves that segment and its bends, and both ends stay on their ports; the route is stored with the pipe and one drag is one undo | nodes along a diagonal are not constrained to 45° | `re_routing_a_hose_keeps_both_ends_on_their_ports`, and the `--selftest` steps `editing: dragging the segment re-routes the pipe` and `editing: one undo restores the route` |
| F1 | T junctions | Two hoses that meet need a junction, and making the user place one by hand is the step that ruins a first schematic. | dropping a hose on another hose splits the second and inserts a junction at the drop point; one undo removes the junction and restores the single hose | a junction has three ports; a four-way node is not offered | `a_hose_dropped_on_another_hose_makes_a_junction`, and the `--selftest` steps `T junction: there is a pipe to drop on`, `T junction: the junction appears where it was dropped`, `T junction: the pipe splits in two and the third arrives` and `T junction: one undo undoes the whole junction` |
| F1 | Check the circuit before running | Most faults are visible before anything moves: an object off the sheet, two symbols on top of each other, a hose through a component, an open port. Reporting them where they are saves the student the guess. | 7 of 9 faults, each drawn where it happens rather than listed in a dialog: outside the sheet, overlapping components, a hose crossing a component, overlapping hoses, a pneumatic port with nothing attached, two cylinders with one designation, and a pipe that joins two circuits | the two that need a pipe whose ends do not fit and the displacement encoder | `crates/punchi-fluid-check` one test per fault, plus the capture of a sheet with two faults marked |
| F2 | The sheet is the network the solver runs | The circuit the student draws is the one that is simulated, with no separate model to keep in step. | the solver builds volumes and restrictions from the document; a 5/2 valve that exists only as a file drives a double-acting cylinder; the junction and the throttle are part of the graph | components whose behaviour is still fixed in the demo model | `the_gas_that_entered_is_the_gas_the_network_holds`, `a_five_port_valve_file_drives_a_double_acting_cylinder`, and the ABI circuit test |
| F2 | Hoses coloured by pressure while running | Pressure has to be readable on the sheet, not only in the status bar. | each hose takes the pressure of its port and is drawn in the matching ink; at atmosphere it keeps the neutral ink | a legend of the ink scale | `hoses_take_the_pressure_of_their_port`, and the capture of a running sheet |
| F5 | The export panel | A sheet for a workshop is not always A4 with an 18 mm frame: a course wants its own margins, a plotter a custom size, a slide a smaller raster, and a quick sketch no parts list. | **Archivo ▸ Hoja ▸ Márgenes, tamaño y lista…** sets a named or custom paper, four margins, the PNG resolution and the parts list; the settings travel to the core in one structure and are refused when they leave no room for the drawing | remembering the settings between sessions, and a preview | `the_export_panel_settings_reach_the_page` (custom size, 5 mm margins, no list, and the two refusals), and the three `--selftest` steps `export panel: …` |
| F5 | The sheet as a vector drawing | A raster page cannot be zoomed into a symbol, measured or placed in another document; a course or a workshop asks for the drawing, not a photograph of it. | **Exportar** offers SVG beside PDF and PNG; the writer is hand-made, deterministic and measures in millimetres, and it draws the same page the PDF draws: frame, block, parts list and the circuit | editing the SVG after export, and an SVG symbol library | `the_sheet_becomes_a_vector_document`, `the_same_page_is_the_same_text_on_both_devices`, `a_title_with_markup_is_escaped_instead_of_breaking_the_document`, the ABI test `an_export_can_be_a_vector_drawing`, and the four `--selftest` steps `export: …` |
| F5 | Parts list on the printed sheet | A technician cannot build from a drawing that says "3/2 valve" and not how many, with which designation, under which name; the list is the half of a sheet that turns symbols into a bill of materials. | the printed page carries a **parts list** in a column above the title block: one row per definition with its quantity, its catalog name and the designations of the group in placement order; the column takes its width from the drawing, which is fitted to what remains | grouping by a user's own category, and a toggle to leave it off | `identical_components_are_one_row_of_two`, `the_parts_list_is_drawn_with_its_counts`, `the_parts_column_takes_room_from_the_drawing`, and the ABI export test that finds `lista de piezas` in the PDF |
| F3 | Pressure gauge | A student needs to read a value at a point in the circuit, as they would with a real instrument. | `gauge.pressure`, a file with `behaviour = "gauge"`, a scale in the file and one working port; the network gives the port a node and adds no physics; the dial is the file's symbol and the reading is drawn from the port pressure | a moving needle; the number is drawn, not the needle angle | `a_gauge_reads_its_port_and_adds_nothing_to_the_physics`, the ABI test `a_pressure_gauge_reads_the_line_it_is_connected_to`, and `assets/symbols/review/gauge.pressure.md` |
| F3 | Flow meter | A student has to read how much air passes a point, which the pressure at that point does not say. | `flow.meter`, a file with `behaviour = "flow_meter"`, a conductance and a nominal scale in normal litres per minute; two working ports inline; the solver models it as an ISO 6358 restriction and keeps the gas that went through it, and the reading is drawn beside the symbol in the normal litre of ISO 6358 (0 °C, 1 013.25 hPa), positive from port 1 to port 2 ([ADR 0014](adr/0014-a-flow-meter-is-a-restriction-that-reports.md)) | a total volume and a moving needle; the number is the evidence today | `a_flow_meter_reads_the_flow_it_carries`, the ABI test `a_flow_meter_reads_the_line_it_is_connected_to`, and `assets/symbols/review/flow.meter.md` |
| F3 | Run until state change | Watching a stroke finish at 60 fps teaches less than stopping where something changed. | a **Hasta cambio** command: it enters simulation, advances until the first piston reaches an end or any valve changes position, leaves the transport paused there, and gives up after a bounded number of steps | relays and switches, which need F6 | the `--selftest` steps `transport: run until change stops`, `transport: it stops at the stop` and `transport: it stays paused` |
| F4 | Save and open a circuit | Work that cannot be saved is a demo, not a tool. | a versioned sheet format (v3) with atomic write and bounded parsing; Nuevo, Abrir, Guardar, Guardar como with Ctrl+N/O/S/Shift+S; `--save` and `--open` from the command line; autosave, recent documents and the unsaved-work warning are covered by their own rows | a project as a folder, and the sheet title is not editable | the format tests, the ABI round trip, and the `--selftest` steps `document: it saves`, `document: it opens`, `document: the same components return` and `document: the same pipes return` |

## Rules that do not bend

1. **Cover first, improve second, and say which.** A capability is not done at `covered`
   without its keep-or-improve decision, and an improvement that is only taste is not an
   improvement.
2. **Behaviour is ours to describe; content is never taken.** A symbol is drawn by hand
   against ISO 1219-1 and reviewed; an example circuit is authored here.
3. **A capability is done when it is proven**, in the sense of
   [.agents/skills/rust-testing-and-verification](../.agents/skills/rust-testing-and-verification/SKILL.md).
4. **Nothing is faked to look finished.** A pipe that does not carry air says so until it
   does; a panel that cannot edit yet says where it will.
5. **Improve where it is cheap and honest**: a keyboard path for every action, a check that
   points at the fault, undo that never loses a drag, and a theme that follows the desktop.
