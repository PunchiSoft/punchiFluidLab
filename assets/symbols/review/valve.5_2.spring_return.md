# valve.5_2.spring_return — conformance review

    status: pending-300dpi
    date: 2026-11-06
    reviewer: automated pass, unreviewed by a second person
    function: five ports, two positions, spring return; feeds one working line and vents the other

## Provenance

Hand-written geometry in `assets/catalog/valve.5_2.spring_return.toml`. The body is two
0.44-squares side by side; the ports, spring and lever are declared, and every internal
path is the connection of one position. No third-party asset and no trace of another
program's symbol.

## Automated checks — `punchi-fluid-catalog` symbol lint

| Rule | Result |
|---|---|
| Two equal squares sharing an edge | pass: `body_squares` |
| Every position has paths or blocking marks | pass: two paths each |
| Every port used and numbered | pass: five ports, numerals from the table |
| `spring_return = true` declares a spring | pass |
| `actuation = momentary` declares an actuator | pass |
| Primitives inside the box, no degenerate line | pass |
| Every line declares its class | pass |
| Legible at palette size | pass: see the images |

## Visual check

| Requirement | Result | Evidence |
|---|---|---|
| Rest position is the square against the spring | pass | the active (shaded) square is the right-hand one, next to the spring |
| Pressing connects 1 to 4, releasing connects 1 to 2 | pass | the two diagonals, each inside its own square |
| The position that does not use 5 shuts it, and 3 is shut in the other | pass | blocking marks inside the square that closes them |
| The two exhaust paths do not share the edge between the squares | pass | 4/5 at the left of each square, 2/3 at the right |
| Legible at palette size (24–30 px) | pass | `catalog-dark.png`, sidebar row |
| Legible at 300 dpi | **not checked** | needs M2 |

## Decisions this symbol carries

- **Port order 4, 2 on top and 5, 1, 3 below**, instead of the equally valid mirrored
  arrangement: it puts each exhaust path at the outer side of its own square, so the two
  squares never draw a line on the shared edge where it would read as one path.
- **Spring on the right**, so position 0 is the right square and the rest position is the
  one against the spring, as the valve checklist requires.
- **Port 5 blocked at rest and port 3 blocked while actuated**: a 5-port valve has an
  exhaust per working line, and the position that does not use one closes it.

## What is still open

- 300 dpi render (M2) and a second reader's signature.
- Exhaust ports are drawn as a stub and a numeral, not with the ISO 1219-1 exhaust
  triangle; the same is true of the built-in 3/2 valve. It is a renderer-wide gap, not a
  property of this file.
- The physical conductances are a datasheet-order value for a small valve, not a
  measurement.

## Images

- `valve.5_2.spring_return-light.png`, `valve.5_2.spring_return-dark.png`
