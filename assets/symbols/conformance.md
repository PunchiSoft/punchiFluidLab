# Conformance matrix

The work list of the symbol pass: one row per symbol of the core, the figure it must match,
the ports it declares and the state of its review. The figure is the **registration number**
the standard prints next to the symbol (`X…`), or the rule that governs a mark (`RF…`); the
numbers below were read in ISO 1219-1:2006 and are re-read in the 2012 basis (JIS B
0125-1:2020) by the phase that touches the symbol.

The review state is one word:

| State | Means |
|---|---|
| `pending` | the symbol has not been reviewed against this contract yet |
| `in review` | its record, fixture and captures are being rewritten |
| `done` | its record is `complete`, with the figure, the measures and the sign-off |

The **signature** is a separate, stronger mark, defined by
[ADR 0047](../docs/adr/0047-the-signed-symbols-are-immutable.md): `signed` means the symbol
has an entry in `assets/symbols/signed.toml` whose hashes match and whose SSH signature
verifies against `assets/symbols/owner_signers`; `pending` means the hashes are pinned but the
owner has not signed it (allowed while `base_open = true`); `—` means the review is not
`complete`, so there is no entry. A `done` row is not automatically signed.

The **core** is the set the owner fixed on 2026-11-22: the air supply, the air treatment, the
valves, the OR/AND, the cylinders and the terminations. The rest of the catalog is outside
this pass and keeps its own record.

The **second reader is Cristian Muñoz, the owner**: he signed off the core's records on
2026-11-24, and that sign-off is what closes a row. The **2026-11-24 correction of the two
service units** changed their drawing (box, chain-line frame, ports and numerals outside,
internal forms), and the **OR valve** was corrected the same day (continuous through line,
ball tangent to its seat and unfilled); those three rows were reviewed again and **signed on
2026-11-25** in the batch. The **filter-regulator** is a new family member of 2026-11-24; it
and the **automatic-drain filter** were signed in the same batch.
The rows still `pending` are the seven symbols the owner left out of that batch for a known
correction: the port closures, the spool valves and the two-pressure valve.

## The core

| Symbol | Family | Figure / rule | Ports | State | Signature | Record |
|---|---|---|---|---|---|---|
| `supply.network` | supply | `RF059` (9.6.4.1), owner's 8 × 8 mm size | 1 supply, no printed numeral | `done` | `signed` | [review](review/supply.network.md) |
| `supply.compressor` | supply | `X11400` (7.2.4) | 2: `1` intake, `2` delivery | `done` | `signed` | [review](review/supply.compressor.md) |
| `supply.compressor_adjustable` | supply | `X11400` + `RF040` | 2: `1` intake, `2` delivery | `done` | `signed` | [review](review/supply.compressor_adjustable.md) |
| `filter.air` | conditioning | `X11980` (7.4.4.1) | 2 | `done` | `signed` | [review](review/filter.air.md) |
| `filter.air_manual_drain` | conditioning | `X12190` (7.4.4.20) | 2 + drain mark | `done` | `signed` | [review](review/filter.air_manual_drain.md) |
| `lubricator.oil` | conditioning | `X12240` (7.4.4.25) | 2 | `done` | `signed` | [review](review/lubricator.oil.md) |
| `preparation.service_unit` | conditioning | `X12160` (7.4.4.17) | 2, outside the chain-line frame | `done` | `signed` | [review](review/preparation.service_unit.md) |
| `preparation.service_unit_simple` | conditioning | `X12160` (7.4.4.17), simplified representation, with the owner's automatic-drain purge arrow | 2, outside the body | `done` | `signed` | [review](review/preparation.service_unit_simple.md) |
| `preparation.filter_regulator_simple` | conditioning | `X12160` (7.4.4.17), simplified filter-regulator, no lubricator | 2, outside the body | `done` | `signed` | [review](review/preparation.filter_regulator_simple.md) |
| `valve.3_2.base` | valve | `X10280` (6.1.2.6 / 7.1.2.8): composition of the body and positions, no single X-number | 3 | `done` | `pending` | [review](review/valve.3_2.base.md) |
| `valve.3_2.spring_return` | valve | `X10270` (6.1.2.5 / 7.1.2.7) body and spring + `688V1` lever | 3 | `done` | `pending` | [review](review/valve.3_2.spring_return.md) |
| `valve.3_2.solenoid` | valve | `X10280` (6.1.2.6 / 7.1.2.8) | 3 + coil | `done` | `pending` | [review](review/valve.3_2.solenoid.md) |
| `valve.5_2.base` | valve | `X10400` (6.1.2.15 / 7.1.2.16): composition of the body and positions, no single X-number | 5 | `done` | `signed` | [review](review/valve.5_2.base.md) |
| `valve.5_2.spring_return` | valve | `X10400` body and ports + `X10460` (7.1.2.22) spring return + `688V1` lever | 5 | `done` | `pending` | [review](review/valve.5_2.spring_return.md) |
| `valve.shuttle` | valve | `X10740` (6.1.5.5 / 7.1.5.5 in ISO 1219-1:2006; the same in JIS B 0125-1:2020) | 3: `1a`, `1b` inlets printed `1`, `2` outlet | `done` | `signed` | [review](review/valve.shuttle.md) |
| `valve.two_pressure` | valve | `X10620` (7.1.3.6 in ISO 1219-1:2006; **7.1.3.5** in JIS B 0125-1:2020) | 3: `1a`, `1b` inlets printed `1`, `2` outlet | `done` | `pending` | [review](review/valve.two_pressure.md) |
| `cylinder.single_acting_20_100` | actuator | `X11440` (7.3.1, ISO 1219-1:2006 and JIS B 0125-1:2020; also 6.3.1 hydraulic) | 1: `1` working, cap end | `done` | `signed` | [review](review/cylinder.single_acting_20_100.md) |
| `cylinder.double_acting_32_200` | actuator | `X11450` (7.3.2, ISO 1219-1:2006 and JIS B 0125-1:2020; also 6.3.2 hydraulic) | 2: `1` and `2`, one per cap | `done` | `signed` | [review](review/cylinder.double_acting_32_200.md) |
| `plug.blind` | accessory | `2172V1` (8.2.9) closed-path/port bar; the `F038V1` (8.2.10) in-line square is a different figure and is recorded | 1: `1` working, the capped port | `done` | `pending` | [review](review/plug.blind.md) |
| `silencer.pneumatic` | accessory | `2033V1` (8.7.37): the standard draws a 4M × 2M box with three relief lines; the contract's cone differs and is recorded | 1: `1` exhaust | `done` | `pending` | [review](review/silencer.pneumatic.md) |

The two cylinders are `done` after the Phase 5 and Phase 7 passes. `X11440` also prints a
spring-chamber connection at the rod end; the catalog declares the working port only, a
recorded `RF052` omission (the model has one working chamber and the schema has no breather
role). The record states the omission instead of drawing a port the model does not have, and
the row is `done` on a complete, signed record that names it.

**The service-unit correction (2026-11-24).** The owner rejected the drawn FRL: the box was
a 20 mm square that compressed the band, the frame was dashed instead of a chain line, the
numerals sat inside the frame and the internal forms were too small. The two files now
declare their boxes (`46 × 25` and `40 × 20`), draw the frame with the new `boundary` class,
put the ports and their numerals outside it, and redraw the filter, regulator, gauge,
lubricator and the compact's purge arrow at the reference's proportion. Their reviews and
three captures were rewritten, the owner reviewed them at 300 dpi and signed them on
2026-11-25; the export still carries its unverified note because the reviews outside this
pass are open.

The four terminations and pressure elements are `done` after the Phase 6 and Phase 7 passes.
Two differences are recorded rather than hidden: the plug is drawn with `2172V1`'s
closed-port bar and not with `F038V1`'s in-line square, and the silencer keeps the contract's
cone (ADR 0016) while `2033V1` draws a box with three relief lines. Both records carry those
as open items, and the second reader signed them on 2026-11-24.

## The conditioning family

The owner enumerated four conditioning forms on 2026-11-24: the filter, the filter-regulator,
the FRL and the FRL compact. The first and the last two are in the core table above; the
filter-regulator is the second, added after the FRL correction, and the automatic-drain
filter is the filter's drain choice, both reviewed against the same contract.

| Symbol | Family | Figure / rule | Ports | State | Signature | Record |
|---|---|---|---|---|---|---|
| `preparation.filter_regulator` | conditioning | `X12150` plus the gauge of `X12160` (composed; stated in the record) | 2, outside the chain-line frame | `done` | `signed` | [review](review/preparation.filter_regulator.md) |
| `filter.air_auto_drain` | conditioning | `X12200`/`F065V1` (automatic-drain mark; the figure was not re-read in this pass) | 2 + drain | `done` | `signed` | [review](review/filter.air_auto_drain.md) |

## Outside this pass

Not reviewed by this contract yet; the owner settles their fate. They keep their current
records.

| Symbol | Why it is out |
|---|---|
| the 16 `electrical.*` | the electrical control follows its own convention, not ISO 1219-1 |
| `gauge.pressure`, `flow.meter`, `chart.recorder` | instruments are not in the core |
| `throttle.fixed` | a restriction, not in the core |
| `junction.tee` | the tee is the connection point of `RF050`, drawn by the renderer |

## The rules a review checks against

| Rule | What it says |
|---|---|
| `RF052` | every port of a symbol is drawn |
| `RF053` | `1` inlet, `2`/`4` outlets, `3`/`5` exhausts, `12`/`14` pilots; the designation's room is above or to the left |
| `RF040` | a machine's adjustability arrow sits on the diameter |
| `RF048` | an adjustable element's arrow goes through the element |
| `RF038` | a pump's drive shaft is on the left, or on the right |
