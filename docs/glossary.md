# Glossary

The words this project uses in its own sense. Where a term has an ISO definition, the
standard is the authority: ISO 5598 for fluid power, ISO 1219 for the drawing and ISO 6358
for the flow of a compressible gas. The Spanish column is the word the interface shows
today, kept here so a translator does not invent a second vocabulary.

## Fluid power

| Term | Spanish UI | Meaning | Unit or source |
|---|---|---|---|
| absolute pressure | presión absoluta | Pressure measured from vacuum. The model stores absolute pressure. | Pa |
| gauge pressure | presión manométrica | Pressure above the local atmosphere; the number the status bar and the gauge show. | bar |
| atmosphere | atmósfera | The reference pressure for gauge pressure; `101 325 Pa` in the model. | Pa |
| supply | suministro | A source that holds a pressure, for example the 6 bar workshop compressor. | ISO 5598 |
| working line | línea de trabajo | A line that carries air between components during normal operation. | ISO 1219 |
| pilot line | línea de pilotaje | A line that carries a signal, not the working flow. | ISO 1219 |
| exhaust | escape | A line or port that releases air to the atmosphere. | ISO 1219 |
| drain | purga | A line or port that removes condensate or releases a chamber. | ISO 1219 |
| sonic conductance | conductancia sónica | The `C` of ISO 6358: flow per unit of upstream pressure when the flow is choked. | m³/(s·Pa) |
| critical pressure ratio | relación de presión crítica | The `b` of ISO 6358: below this downstream/upstream ratio the flow is choked. | dimensionless |
| choked flow | flujo crítico | Flow that no longer grows when the downstream pressure falls. | ISO 6358 |
| dead volume | volumen muerto | The volume a chamber keeps at zero stroke, expressed in the model as a length. | mm equivalent |
| stroke | carrera | The usable travel of a piston. | mm |
| bore | diámetro interior | The inside diameter of the cylinder barrel. | mm |
| rod | vástago | The piston rod; its diameter reduces the return chamber area of a double-acting cylinder. | mm |
| moving mass | masa móvil | The mass the gas has to accelerate: piston, rod and the load it carries. | kg |
| spring rate | constante del muelle | Stiffness of a return spring. | N/m |
| preload | precarga | The force a spring exerts at zero stroke. | N |
| damping | amortiguamiento | Viscous resistance proportional to velocity. | N·s/m |
| Coulomb friction | rozamiento de Coulomb | Constant resistance that opposes motion and holds a piston at rest. | N |
| valve position | posición | One of the states of a directional valve, drawn as one square. | ISO 1219 |
| actuation | accionamiento | How a valve changes position: lever, push button, spring, solenoid. | ISO 1219 |
| spring return | retorno por muelle | The spring that returns a valve to its default position when the actuator is released. | ISO 1219 |
| latching valve | válvula enclavada | A valve with no spring return: it keeps the position it was put in. | ISO 5598 |
| throttle | estrangulador | A fixed or adjustable restriction. | ISO 5598 |
| junction | unión en T | A three-port point where hoses meet; it adds no volume and no restriction. | ADR 0006 |
| gauge | manómetro | A passive instrument that shows the pressure at its port. | ADR 0008 |
| flow meter | caudalímetro | An instrument in the line that measures the gas going through it, in normal litres per minute. | ADR 0014 |
| normal litre | litro normal | A litre at 20 °C and 1 013.25 hPa, the reference the flow reading uses; the file's scale is in this unit. | ADR 0014 |
| service unit | unidad de mantenimiento | The filter, regulator and lubricator group that prepares the air. | ISO 5598 |
| filter | filtro | Removes particles and condensate from the air. | ISO 5598 |
| lubricator | lubricador | Adds oil to the air for the parts that need it. | ISO 5598 |
| regulator | regulador | Holds a downstream pressure below the supply. | ISO 5598 |
| silencer | silenciador | Reduces the noise of an exhaust; not implemented yet. | ISO 5598 |
| blind plug | tapón ciego | Closes a port; not implemented yet. | ISO 5598 |

## The document and the drawing

| Term | Spanish UI | Meaning | Home |
|---|---|---|---|
| sheet | hoja | The saved document: placements, pipes, orientation and designations. It is one circuit today. | `punchi-fluid-doc` |
| circuit | circuito | The pneumatic network the sheet describes. | `punchi-fluid-solver` |
| project | proyecto | A document with several circuits and one selector; each circuit is a sheet. | [ADR 0013](adr/0013-a-project-is-one-document.md) |
| placement | colocación | One component on the sheet: catalogue id, centre, box, orientation, per-instance edits and designation. | `punchi-fluid-doc` |
| designation | designación | The name a user gives one placement, drawn under its symbol. At most 40 characters, trimmed. | `punchi-fluid-doc` |
| catalog id | identificador de catálogo | The stable `family.variant` name of a definition file, for example `valve.3_2.spring_return`. | `assets/catalog/` |
| definition | definición | A TOML file that describes a component: ports, behaviour, symbol and values. | ADR 0002 |
| pipe | tubo | A connection between two ports, stored with its orthogonal route. "Hose" is used for the physical object; the stored entity is a pipe. | `punchi-fluid-doc` |
| route | recorrido | The waypoints between the two ends of a pipe. | `punchi-fluid-doc` |
| port | puerto | A named connection point on a component, with a role: supply, working or exhaust. | `punchi-fluid-catalog` |
| finding | falta | A design fault the check reports, with the place it happens. | `punchi-fluid-check` |
| line class | clase de línea | The ISO 1219 class of a drawn line: working, pilot, drain or electrical. | `punchi-fluid-draw` |
| print ink | tinta de papel | The black-on-white palette the exported page uses, independent of the desktop theme. | `punchi-fluid-export` |
| `Surface` | superficie | The trait a drawing calls, implemented once by the window and once by the paper. | ADR 0003 |
| frame | marco | The ISO 5457 border of a printed sheet. | ADR 0012 |
| title block | bloque de identificación | The identification block in the bottom-right of the printed page. | ADR 0012 |

## Working on the project

| Term | Meaning | Home |
|---|---|---|
| capability | What a student must be able to do, with its purpose, behaviour and evidence. | `docs/capabilities.md` |
| coverage ledger | The table that gives every capability a state: `missing`, `partial`, `covered`, `improved` or `not planned`. | `docs/capabilities.md` |
| phase | One branch, one pull request and one reviewable delivery. | `docs/next-steps.md` |
| gate | `./scripts-dev/check.sh`: format, lints, tests, docs and the headless window. | `AGENTS.md` |
| evidence level | How a claim is proven: unit test, invariant, reference curve, ABI test, headless UI test, measurement, screenshot, review. | `docs/style.md` |
| ADR | An architecture decision record: why the system is shaped this way. | `docs/adr/README.md` |
| skill | A procedure for a recurring task, under `.agents/skills/`. | `.agents/skills/README.md` |
| continuity note | The owner's Spanish note for resuming with no context. | `docs/retomar.md` |

## Units

| Quantity | Unit in the model | Unit in the interface |
|---|---|---|
| pressure | Pa, absolute | bar, gauge |
| length | mm | mm |
| volume | m³ | — |
| conductance | m³/(s·Pa) | — |
| mass | kg | g for the air consumed |
| force | N | — |
| stiffness | N/m | — |
| damping | N·s/m | — |
| temperature | K | — |
| time | s | s |

A bare `f64` that carries a physical quantity across a crate boundary is a bug; the typed
values live in `punchi-fluid-core::units`. The drawing API speaks millimetres, the model
speaks SI, and the interface converts once, at the edge.
