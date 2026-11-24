# Conformance matrix

The work list of the symbol pass: one row per symbol of the core, the figure it must match,
the ports it declares and the state of its review. The figure is the **registration number**
the standard prints next to the symbol (`X…`), or the rule that governs a mark (`RF…`); the
numbers below were read in ISO 1219-1:2006 and are re-read in the 2012 basis (JIS B
0125-1:2020) by the phase that touches the symbol.

The state is one word:

| State | Means |
|---|---|
| `pending` | the symbol has not been reviewed against this contract yet |
| `in review` | its record, fixture and captures are being rewritten |
| `done` | its record is `complete`, with the figure, the measures and the sign-off |

The **core** is the set the owner fixed on 2026-11-22: the air supply, the air treatment, the
valves, the OR/AND, the cylinders and the terminations. The rest of the catalog is outside
this pass and keeps its own record.

## The core

| Symbol | Family | Figure / rule | Ports | State | Record |
|---|---|---|---|---|---|
| `supply.network` | supply | to read | 1 supply | `pending` | [review](review/supply.network.md) |
| `supply.compressor` | supply | `X11400` (7.2.4) | 2: `1` intake, `2` delivery | `pending` | [review](review/supply.compressor.md) |
| `supply.compressor_adjustable` | supply | `X11400` + `RF040` | 2: `1` intake, `2` delivery | `pending` | [review](review/supply.compressor_adjustable.md) |
| `filter.air` | conditioning | `X11980` (7.4.4.1) | 2 | `pending` | [review](review/filter.air.md) |
| `filter.air_manual_drain` | conditioning | `X12190` (7.4.4.20) | 2 + drain mark | `pending` | [review](review/filter.air_manual_drain.md) |
| `lubricator.oil` | conditioning | `X12240` (7.4.4.25) | 2 | `pending` | [review](review/lubricator.oil.md) |
| `preparation.service_unit` | conditioning | `X12160` (7.4.4.17) | 2 | `pending` | [review](review/preparation.service_unit.md) |
| `preparation.service_unit_simple` | conditioning | `X12160` (7.4.4.17), simplified representation | 2 | `pending` | [review](review/preparation.service_unit_simple.md) |
| `valve.3_2.base` | valve | `X10280` (7.1.2.8) family | 3 | `pending` | [review](review/valve.3_2.base.md) |
| `valve.3_2.spring_return` | valve | `X10280` + spring | 3 | `pending` | [review](review/valve.3_2.spring_return.md) |
| `valve.3_2.solenoid` | valve | `X10290` (7.1.2.9) | 3 + coil | `pending` | [review](review/valve.3_2.solenoid.md) |
| `valve.5_2.base` | valve | to read | 5 | `pending` | [review](review/valve.5_2.base.md) |
| `valve.5_2.spring_return` | valve | to read | 5 | `pending` | [review](review/valve.5_2.spring_return.md) |
| `valve.shuttle` | valve | `X10740` (7.1.5.5) | 3: `1a`, `1b` inlets, `2` outlet | `pending` | [review](review/valve.shuttle.md) |
| `valve.two_pressure` | valve | `X10620` (7.1.3.6) | 3: `1a`, `1b` inlets, `2` outlet | `pending` | [review](review/valve.two_pressure.md) |
| `cylinder.single_acting_20_100` | actuator | `X11440` (7.3.1) | 1 | `pending` | [review](review/cylinder.single_acting_20_100.md) |
| `cylinder.double_acting_32_200` | actuator | `X11450` (7.3.2) | 2 | `pending` | [review](review/cylinder.double_acting_32_200.md) |
| `plug.blind` | accessory | `F038V1` (8.2.10) and the plugged-port rule | 1 | `pending` | [review](review/plug.blind.md) |
| `silencer.pneumatic` | accessory | `2033V1` (8.7.37) | 1 exhaust | `pending` | [review](review/silencer.pneumatic.md) |

## Outside this pass

Not reviewed by this contract yet; the owner settles their fate. They keep their current
records.

| Symbol | Why it is out |
|---|---|
| the 18 `electrical.*` | the electrical control follows its own convention, not ISO 1219-1 |
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
