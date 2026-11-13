# Retomar el proyecto desde cero

> **Este archivo existe para que abras una ventana nueva, sin contexto, y continúes sin
> preguntar nada.** Está escrito en español a propósito (el resto de `docs/` está en
> inglés por la regla de `AGENTS.md`): es la nota de continuidad del propietario, no
> documentación del producto.
>
> Léelo entero una vez. Después usa §0 y §16 cada vez que retomes.

---

## 0. En cinco minutos

```bash
cd "/home/cristian/Desarrollo/punchi-fluid-lab (Rust)"
./scripts-dev/check.sh                 # tiene que salir 0: formato, clippy, pruebas y ventana
cmake --build apps/punchi-fluid-qt/build   # la ventana que existe hoy
apps/punchi-fluid-qt/build/punchi-fluid   # (Qt 6.5+, CMake y Ninja)
```

Después, en este orden:

| Quiero… | Leo |
|---|---|
| saber qué falta y en qué orden | §11 y [next-steps.md](next-steps.md) |
| entender lo que ya está hecho | §5 y §7 |
| cambiar una válvula sin programar | §8 |
| saber qué está mal en los símbolos | §9 |
| construir la interfaz | §10 y [ui-ux.md](ui-ux.md) |
| planificar una capacidad | [capabilities.md](capabilities.md) y la skill [capability-plan](../.agents/skills/capability-plan/SKILL.md): ninguna empieza sin su propósito y su prueba escritos |
| las reglas que no se rompen | §14 y [AGENTS.md](../AGENTS.md) |

### 0.1 Estado al 2026-11-12 (lo último, y lo que manda)

Desde que se escribió el resto de este archivo cambió la mitad del programa. Lo
autoritativo es [capabilities.md](capabilities.md) (capacidades, estado y
evidencia) y [next-steps.md](next-steps.md) (orden de trabajo). En corto:

| Área | Estado real |
|---|---|
| Redacción del circuito | colocar, mover, borrar, deshacer/rehacer, conectar puerto a puerto, seleccionar y borrar un tubo, **reencaminar un tubo arrastrando su segmento**, T automática al soltar sobre un tubo, copiar/pegar/duplicar (Ctrl+C/V/D y Mayús+arrastrar), **girar y reflejar** (`Alt+R`, `Alt+M`, `Ctrl` y doble clic), **selección múltiple** (Mayús y clic, rectángulo elástico, Ctrl+E) con arrastrar/copiar/borrar/alinear en grupo y un solo deshacer |
| Propiedades | **por instancia**: presión, conductancia, carrera, émbolo, vástago, masa y **designación** (el nombre, dibujado bajo el símbolo); Aplicar es un solo deshacer. Faltan resumen, accionamiento, posición inicial, ángulo de montaje, fugas y amortiguamiento |
| Simulación | **la hoja es la red**: `punchi-fluid-solver::network` construye volúmenes y restricciones desde el documento; válvula 5/2 y cilindro de doble efecto como archivos; T y estrangulador; el clic acciona la válvula pulsada; tubos coloreados por presión |
| Presión | manométrica de punta a punta: 0,00 bar en reposo, 6,00 bar con la fuente de 6 bar |
| Archivo | Nuevo/Abrir/Guardar/Guardar como (Ctrl+N/O/S/Shift+S), formato de hoja versionado con escritura atómica, `--save` y `--open` sin ventana, **documentos recientes** (los últimos diez, en Archivo); **aviso de cambios sin guardar** (título con `•`, confirmación al cerrar o abrir otro) y **copia de seguridad** cada 30 s que se ofrece al arrancar |
| Proyecto | **un documento con varios circuitos**: formato 6, lista de circuitos, selector en Archivo, añadir/renombrar, hoja activa, guardado/apertura de todos y la fuente de cada registrador; la novena falta es un tubo que une dos circuitos |
| Exportación | **desde la ventana y sin ventana**: Archivo ▸ Exportar escribe PDF o PNG (300 dpi) de la hoja que está en pantalla; el grupo Hoja ofrece A5–A0, Letter, Legal y Oficio, apaisado incluido; `--export` hace lo mismo sin ventana; la fecha es la del día y el dibujo se escala para encajar, con la escala escrita en el bloque. Deuda: no hay panel de márgenes, tamaño personalizado ni resolución |
| Comprobación | 7 de las 9 faltas, cada una señalada en el dibujo; las otras 2 necesitan el tubo cuyos extremos no encajan y la biblioteca eléctrica con su encoder (F6) |
| Instrumentos | manómetro (`behaviour = "gauge"`): lee la presión de su puerto y no toca la física; **caudalímetro** (`behaviour = "flow_meter"`): restricción en la línea que informa del gas que la atraviesa, en litros normales por minuto; **registrador** (`behaviour = "recorder"`): componente sin puertos que dibuja una magnitud elegida durante 60 s; el dinamómetro queda `not planned` hasta que exista un dominio mecánico |
| Símbolos | lint de 8 reglas; **17 símbolos** con revisión visual y capturas claro/oscuro, todos en `pending-300dpi`; la revisión a 300 dpi sigue pendiente |

**Valor de réplica, con honestidad:** la *base de edición y simulación* está muy cerca
(la hoja se dibuja, se conecta, se gira, se simula y se guarda), pero del manual completo
falta bastante: repartir con separación igual, las clases de conducto y los
silenciadores/cierres (F1), el dinamómetro y el registrador (F3), las listas de piezas
(F5), y las bibliotecas eléctrica, digital y GRAFCET (F6). Estimación razonable hoy:
**algo más de la mitad del manual**, con F1 casi cerrada, F2 cerrada, F4 entregada, la
exportación entregada (P1.7) y el caudalímetro (P2.1). El siguiente paso, por valor, es el
**dinamómetro y el registrador**, que cierran los instrumentos.

### 0.2 Cómo se entrega desde el 2026-11-12

**Nada se escribe en `main`.** Cada cambio sale en su rama desde `main` actualizado
(`feat/`, `fix/`, `docs/` o `chore/`) y entra por un pull request con squash y borrado de
la rama. `main` está protegido: sin pull request, sin los trabajos `check`, `window` y
`minimum supported Rust version` y sin historial lineal, el push se rechaza. **El merge
lo hace solo el propietario:** el agente abre el pull request, responde la revisión y se
detiene; no ejecuta `gh pr merge`, no activa el auto-merge y no hace push a `main`, por
más permisos que tenga la sesión. El procedimiento completo, con lo que hay que comprobar
antes de tocar nada, está en
[pull-request-workflow](../.agents/skills/pull-request-workflow/SKILL.md), y el orden de
las fases en [next-steps.md](next-steps.md#phases-one-branch-one-pull-request).

```bash
git fetch origin && git switch main && git merge --ff-only origin/main
git switch -c feat/<descriptor>
./scripts-dev/check.sh                    # en la rama, no en un recuerdo
gh pr create --base main --head feat/<descriptor> --body-file /tmp/pr.md
gh pr checks                              # verde y se entrega la URL al propietario
gh pr merge --squash --delete-branch      # solo el propietario, nunca el agente
```

Cómo comprobarlo sin ventana:

```bash
./scripts-dev/check.sh
apps/punchi-fluid-qt/build/punchi-fluid --probe                 # 0,00 bar y 0,0 mm en reposo
apps/punchi-fluid-qt/build/punchi-fluid --toggle --step 30 --probe  # 6,00 bar y 100,0 mm
apps/punchi-fluid-qt/build/punchi-fluid --save /tmp/c.pfluid
apps/punchi-fluid-qt/build/punchi-fluid --open /tmp/c.pfluid --probe
QT_QPA_PLATFORM=offscreen apps/punchi-fluid-qt/build/punchi-fluid --selftest
```

---

## 1. Qué es este proyecto

`punchi-fluid-lab`: simulador libre de circuitos neumáticos (y más adelante hidráulicos), en
Rust, para que un estudiante sin licencia y sin presupuesto pueda dibujar el esquema,
conectar puertos, ejecutar el modelo y ver moverse los actuadores. **Ningún recurso, símbolo,
manual, circuito de ejemplo ni formato de otro programa entra en este repositorio**.

Dos ideas lo definen y explican casi todas las decisiones:

1. **Los componentes son archivos, no código.** Una válvula nueva es un `.toml` en una
   carpeta; no se recompila, no hay una rama nueva en un `match`.
2. **La simbología cumple la norma vigente.** Un esquema *casi* conforme es peor que un
   boceto evidente, porque se lee como si fuera correcto.

---

## 2. Estado real

`0.1.0` — editor y simulador funcionando, con la exportación en la ventana y la
especificación de lo que sigue.

| Área | Estado | Evidencia |
|---|---|---|
| Dominio, formato de archivo, CLI | **hecho** | pruebas del workspace y prueba de humo del CLI en `check.sh` |
| Catálogo de componentes (TOML, reemplazo, `disabled`) | **hecho** | pruebas de esquema, precedencia y lint; probado en la sesión real con un override y un archivo roto |
| Física de gas, válvula por posiciones, cilindro con muelle y rozamiento | **hecho** | ISO 6358 contra cálculo a mano, balance de masa 1e-9, carrera acotada, 10⁵ pasos sin NaN |
| Lienzo: símbolos del catálogo, clic en la válvula, lecturas en vivo | **hecho** | `--selftest` y capturas de pantalla |
| Armazón (cinta, barra lateral, barra de estado) y tema | **hecho, en Qt Quick** | `--check-theme`, `--check-icons` y `--selftest`; capturas claro/oscuro; barra lateral dibujada con el símbolo de cada definición |
| Editor: documento, deshacer, tuberías, T, reencaminar, copiar, girar, reflejar, selección múltiple, alinear, repartir, clases de conducto, designación | **hecho** | pruebas de `doc`, del ABI y `--selftest`; fila por fila en [capabilities.md](capabilities.md) |
| La hoja es la red que resuelve el solver | **hecho** | pruebas de red, el circuito 5/2 + doble efecto por el ABI, tubos coloreados |
| Proyecto de circuitos | **hecho** | formato 4, selector, añadir/renombrar, guardado y apertura; prueba ABI `circuits_can_be_added_renamed_and_survive_a_save_and_open` y la novena falta `a_pipe_between_two_circuits_is_the_ninth_fault` |
| Caudalímetro | **hecho** | `flow.meter`, restricción ISO 6358 en la línea que informa del caudal en litros normales por minuto; pruebas de solver, ABI y dibujo, pasos `--selftest` y revisión del símbolo |
| Registrador | **hecho** | `chart.recorder`, componente sin puertos con ventana de 60 s y 10 muestras/s; fuente elegida en el inspector, guardada en el formato 5; prueba del anillo y del ABI, pasos `--selftest` y revisión del símbolo |
| Exportación PNG/PDF desde la ventana y sin ventana | **hecho** | pruebas de PDF, ráster, dibujo del llamador y fecha/escala por el ABI; `--export`; el gate escribe un PDF y un PNG en Oficio. Deuda: panel de márgenes, tamaño personalizado y resolución |
| Conformidad ISO 1219 de los símbolos | **mitad estructural hecha, mitad visual revisada, 300 dpi pendiente** | 8 reglas de lint con una prueba cada una, 17 revisiones con capturas claro/oscuro, prueba de la carpeta de revisión |
| Instrumentos y ejecución hasta cambio | **hecho** | manómetro, caudalímetro, registrador de 60 s y «Hasta cambio»; el dinamómetro queda `not planned` con la razón escrita |

**Total de pruebas: 283** — `core` 7, `catalog` 37, `file` 27, `solver` 38, `doc` 38,
`draw` 41, `check` 12, `export` 40, `ffi` 38, `cli` 5.
`./scripts-dev/test.sh` en verde con cada binario bajo su tope de memoria, `clippy -D
warnings` limpio y `cargo fmt --check` limpio.

**El repositorio existe y tiene remoto.** `git init -b main` y el primer commit (la
rebanada vertical tal cual estaba) se hicieron el 2026-11-06; el remoto es
`origin` en GitHub. [ADR 0005](adr/0005-qt-quick-over-a-c-abi.md) está aceptado y jubila
`punchi-fluid-ui`, `desktop`, `theme` e `icons`. Desde el 2026-11-12 nada se escribe en
`main`: cada cambio va en su rama y entra por un pull request que mezcla el propietario.

Los dos números que hay que recordar de la máquina: **dos trabajos de `cargo`** y **cada
binario de prueba bajo un tope de memoria**. `./scripts-dev/test.sh` lo aplica solo; el
porqué y el diagnóstico de una muerte por OOM están en
[.agents/skills/machine-resource-budget](../.agents/skills/machine-resource-budget/SKILL.md).
Y si la GPU aparece al 100 % con **7 W**, es el compositor presentando cuadros en un panel
de 144 Hz, no el simulador calculando.

---

## 3. Entorno verificado (2026-11-06)

| Dato | Valor |
|---|---|
| Sistema | Fedora 44, Plasma 6.7.5, sesión Wayland, `XDG_CURRENT_DESKTOP=KDE` |
| Rust | 1.98.1 (el mínimo declarado es 1.85, hay un job de CI que lo compila) |
| Pantalla | eDP-1 1920×1080, escala 1 |
| Tema KDE | `~/.config/kdeglobals`; `[Colors:Window] BackgroundNormal=36,36,36`; acento `[Colors:Selection] BackgroundNormal=49,91,239`; `LookAndFeelPackage=MacDarkpunchi` |
| Portal de color | `org.freedesktop.appearance color-scheme` → `1` (oscuro). Comprobado con `gdbus` |
| Iconos | `/usr/share/icons/breeze/actions/22/…-symbolic.svg` existen |
| Ventana | Qt 6.5+ con Qt Quick sobre el ABI C; `--selftest` la conduce headless |
| Herramientas ausentes | no hay `ydotool`, `xdotool` ni `wtype`: **no se puede inyectar un clic real**; el clic se prueba con `--selftest` y las pruebas del ABI |

---

## 4. Mapa del repositorio

```
punchi-fluid-lab/
├── Cargo.toml                 workspace: 10 crates, resolver 3, edition 2024, lints estrictos
├── rust-toolchain.toml        toolchain fijado
├── AGENTS.md                  REGLAS canónicas (personas y agentes). Léelo antes de tocar nada
├── README.md / README.es.md   inglés canónico + copia española, 17 secciones equivalentes
├── CHANGELOG.md               en español, bajo `## Unreleased` con subsecciones fechadas
├── SECURITY.md                reporte privado, alcance, garantías
├── CONTRIBUTING.md            entorno, puerta, commits, revisión
├── CODE_OF_CONDUCT.md         Covenant 2.1 por referencia
├── deny.toml, clippy.toml, .editorconfig
├── .github/                   CI (+MSRV), audit (cargo-deny/audit), release, plantillas, CODEOWNERS, dependabot
├── assets/
│   ├── catalog/               14 definiciones de fábrica + README de autoría
│   └── symbols/               material de REVISIÓN de símbolos (no los símbolos)
├── crates/                    ver tabla siguiente
├── apps/punchi-fluid-qt/      LA VENTANA: Qt Quick + CMake, fuera del espacio de Cargo
├── docs/                      este archivo y las especificaciones
├── i18n/, packaging/          planificación (vacías de contenido real)
└── scripts-dev/               check.sh (la puerta), run.sh y test.sh (con --no-build)
```

| Crate | Qué posee | Ficheros clave |
|---|---|---|
| `punchi-fluid-core` | componentes, puertos, conexiones, unidades tipadas | `component.rs`, `network.rs`, `units.rs` |
| `punchi-fluid-catalog` | esquema, validación, carga y lint de las definiciones | `schema.rs`, `catalog.rs`, `lint.rs`, `builtin.rs` |
| `punchi-fluid-file` | formato de hoja versionado, migraciones y recientes | `lib.rs`, `sheet.rs`, `recent.rs` |
| `punchi-fluid-solver` | ISO 6358 + red de volúmenes + cilindro | `orifice.rs`, `network.rs`, `pneumatic.rs`, `lib.rs` |
| `punchi-fluid-doc` | documento editado: colocaciones, comandos y deshacer | `lib.rs` |
| `punchi-fluid-draw` | símbolos ISO 1219 en mm, hoja, `Surface` | `symbol.rs`, `schematic.rs`, `surface.rs`, `placed.rs` |
| `punchi-fluid-check` | faltas de diseño antes de ejecutar | `lib.rs` |
| `punchi-fluid-export` | hoja normalizada, PNG y PDF | `page.rs`, `png.rs`, `pdf.rs`, `sheet.rs` |
| `punchi-fluid-ffi` | ABI C: documento, cuadro de dibujo, hit tests, exportación | `lib.rs`, `include/punchi_fluid.h` |
| `punchi-fluid-cli` | `validate`, `simulate` y `check` sin ventana | `main.rs` |

La ventana vive fuera de `crates/`:

| Aplicación | Qué posee | Ficheros clave |
|---|---|---|
| `apps/punchi-fluid-qt` | ribbon, barra lateral, hoja, barra de estado, tema | `Main.qml`, `RibbonBar.qml`, `Sidebar.qml`, `StatusBar.qml`, `src/{session,replay,theme,canvasitem,previewitem,icons}.*` |

Documentación, para no duplicar nada:

| Documento | Contenido |
|---|---|
| [next-steps.md](next-steps.md) | **orden de trabajo**: pasos, ficheros, firmas, criterios de aceptación |
| [ui-ux.md](ui-ux.md) | interfaz: cinta, barra lateral, hoja, tema KDE, accesibilidad |
| [symbology.md](symbology.md) | contrato ISO 1219, garantías del renderizador, revisión |
| [plan.md](plan.md) | hitos M0–M6 con criterios y evidencia exigida |
| [engineering.md](engineering.md) | modularización, presupuestos de memoria/rendimiento, seguridad, prácticas GitHub |
| [adr/0001](adr/0001-language-and-layout.md) | por qué Rust y por qué estos crates |
| [adr/0002](adr/0002-components-are-files.md) | por qué los componentes son archivos |
| [../assets/catalog/README.md](../assets/catalog/README.md) | cómo se escribe un componente |

---

## 5. Qué se hizo, paso a paso

Bitácora de la sesión, en orden. Cada paso dice **qué se decidió** y **cómo se
comprobó**, porque el "cómo se comprobó" es lo que se pierde al cambiar de ventana.

### 5.1 Dominio, formato y CLI

Se creó el workspace con la separación dominio / física / formato / interfaz /
aplicación. Unidades tipadas (`Pressure`, `Temperature`, `Volume`) que rechazan valores
imposibles en el constructor. `punchi-fluid-file` guarda el documento con
`FORMAT_VERSION` y rechaza versiones futuras; el CLI no depende de `serde_json`.
**Comprobado:** 7 + 3 pruebas, más `validate`/`simulate` sobre un circuito temporal
dentro de `check.sh`.

### 5.2 Los componentes pasan a ser archivos

Se sustituyó la idea de "una válvula = un tipo de Rust" por un catálogo de archivos.
Crate nuevo `punchi-fluid-catalog` con esquema TOML (`schema = 1`), validación y dos
orígenes: fábrica (compilada con `include_str!` desde `assets/catalog/`) y usuario
(`~/.local/share/punchi-fluid-lab/catalog/`, que gana por `id`). Un archivo inválido se
omite y se reporta; `disabled = true` retira uno de fábrica.
**Decisión clave:** una posición de válvula declara los caminos abiertos como
`(puerto, puerto, conductancia)` y el solver los clasifica por **rol** del puerto
(supply/working/exhaust), no por número. Por eso una 5/2 será otro archivo.
**Comprobado:** 7 pruebas (precedencia, `disabled`, clave mal escrita rechazada, archivo
roto no impide arrancar, esquema futuro rechazado). En la sesión real: un override
cambió caudal y nombres (apareció como `[user]`) y un archivo con `sumary` mal escrito
salió en «Skipped files» con línea y columna.

### 5.3 Física real

`orifice.rs` se reescribió según ISO 6358 de verdad: conductancia sónica `C`, relación
crítica `b`, rama crítica y elipse subcrítica (antes había una fórmula aproximada).
`pneumatic.rs` es el circuito: **la masa de gas es el estado integrado** y la presión se
deriva con el gas ideal — así no puede ser negativa y el balance de masa es exacto por
construcción. Pistón con muelle, amortiguamiento, rozamiento de Coulomb con adherencia
y topes. Subpaso interno de 2 µs acotado (máx. 500 000 por llamada): lo que pida la
interfaz no desestabiliza ni bloquea un fotograma.
**Comprobado:** caudal crítico contra el cálculo a mano, tiempo de carrera entre dos
cotas de flujo, balance de masa a 1e-9, purga con retorno por muelle y 10⁵ pasos sin NaN
ni presión negativa. 18 pruebas.

> Trampa encontrada y documentada: en un primer barrido los tiempos no cuadraban hasta
> que se vio que la comprobación estaba usando la constante universal `R` en lugar de la
> específica del aire (`287.058`). El modelo estaba bien; la verificación, no.

### 5.4 Lienzo

`symbol.rs` dibuja las primitivas de la definición (línea, polilínea, polígono,
rectángulo, círculo, arco) y el cilindro se dibuja paramétrico (pistón, vástago y muelle
según la carrera). `canvas.rs` coloca tres componentes en posiciones fijas, conecta los
puertos declarados con tuberías ortogonales, colorea la línea de trabajo según la
presión, resalta la posición activa de la válvula y devuelve `CanvasOutcome::valve_clicked`.
**Comprobado:** dos pruebas headless con `egui::Context::run_ui` (clic en la válvula
actúa; clic en el cilindro o el suministro no) y capturas reales con `spectacle`.

> Trampas encontradas: el círculo salía invisible porque se convertían las coordenadas
> en pares y perdía el radio; las etiquetas se solapaban con el nombre duplicado; la caja
> resaltada no coincidía con la posición porque el orden de los `rect` es el orden de las
> posiciones. Las tres están corregidas y explicadas en el README del catálogo.

### 5.5 Documentación, normas y andamiaje

- README en inglés y español; ADR 0001 (Rust y crates) y 0002 (componentes como archivos).
- `docs/`: interfaz, simbología, plan, ingeniería, orden de trabajo.
- Repositorio: `SECURITY`, `CONTRIBUTING`, `CODE_OF_CONDUCT`, plantillas, `CODEOWNERS`,
  dependabot, workflows `ci`/`audit`/`release`, `deny.toml`.
- **Simbología:** se fijó ISO 1219-1:2012 (+ 1219-2:2012, 1219-3:2016) como contrato y se
  declaró honestamente que los símbolos actuales **no son conformes** (§9).

---

## 6. Cómo verificarlo tú mismo

```bash
# 1. La puerta completa
./scripts-dev/check.sh          # esperado: exit 0, 283 pruebas de Cargo + --selftest, 0 warnings

# 2. Conteo por paquete
for c in core catalog file solver doc draw check export ffi cli; do
  echo -n "$c: "; cargo test -p punchi-fluid-$c 2>/dev/null | grep -oP 'ok\. \K[0-9]+' | paste -sd+ | bc
done
# esperado: core 7, catalog 34, file 20, solver 33, doc 29, draw 39, check 11, export 40, ffi 32, cli 5

# 3. La ventana
apps/punchi-fluid-qt/build/punchi-fluid     # el transporte ejecuta; el clic en la válvula la acciona

# 4. Modo demostración (arranca accionada y en marcha)
apps/punchi-fluid-qt/build/punchi-fluid --toggle   # arranca accionada

# 5. Captura para revisar el dibujo
spectacle -b -n -f -o /tmp/lab.png

# 6. El catálogo del usuario
mkdir -p ~/.local/share/punchi-fluid-lab/catalog
cp assets/catalog/valve.3_2.spring_return.toml ~/.local/share/punchi-fluid-lab/catalog/
# edita la copia y reinicia la ventana (el catálogo se carga al arrancar):
#   - el nombre debe salir con etiqueta [user]
#   - si rompes una clave, el archivo se omite y la ventana sigue arrancando;
#     la razón queda en `Catalog::skipped`, que la ventana todavía no muestra
```

Referencia de lo que se vio la última vez en la ventana: `t = 2.506 s`, `p = 4.99 bar`,
`x = 100.0 mm`, `v = 0.00 m/s`, `air in = 0.233 g`, válvula en `actuated`.

---

## 7. El modelo físico, con sus números

Para no tener que deducirlo leyendo `pneumatic.rs`:

| Magnitud | Valor en el código |
|---|---|
| Constante específica del aire | `287.058 J/(kg·K)` |
| Temperatura | `293.15 K` (estándar) |
| Atmósfera | `101 325 Pa` |
| Suministro de fábrica | `6 bar` manométricos → `700 000 Pa` absolutos |
| Conductancia de la válvula 3/2 | `1.0e-9 m³/(s·Pa)` por camino |
| Relación crítica `b` | `0.3` |
| Cilindro: diámetro / carrera | `20 mm` / `100 mm` |
| Volumen muerto | `5 mm` equivalentes |
| Masa móvil | `0.5 kg` |
| Muelle | `250 N/m`, precarga `5 N` |
| Amortiguamiento / rozamiento | `2 N·s/m` / `1.5 N` |
| Subpaso interno | `2 µs`, máximo `500 000` por llamada |

Ecuaciones: masa de gas como estado (`ṁ = ṁ_entra − ṁ_sale`), presión derivada
`P = m·R_aire·T / V(x)` con `V(x) = A·(x + x_muerto)`; fuerza
`m·ẍ = (P − P_atm)·A − (F_0 + k·x) − c·ẋ − F_roz·sgn(ẋ)` con adherencia a velocidad
cero; caudal ISO 6358 con rama crítica `ṁ = C·p₁·ρ_n·√(T_n/T₁)` y subcrítica
`ṁ = C·p₁·ρ_n·√(T_n/T₁)·√(1 − ((p₂/p₁ − b)/(1 − b))²)`.

Números medidos (para detectar regresiones): con esos parámetros la carrera completa se
completa en **≈ 0.13 s** de tiempo simulado y la presión final ronda **2.5 bar**; por eso
la interfaz arranca a **0.25×** de velocidad.

---

## 8. El catálogo: añadir, cambiar, borrar

Carpeta de fábrica `assets/catalog/`; carpeta del usuario
`~/.local/share/punchi-fluid-lab/catalog/` (o `PUNCHI_FLUID_CATALOG_DIR`).

- **Añadir:** copia el archivo más parecido, cambia `id`, `name` y números, pulsa
  *Reload*. Nada de Rust.
- **Cambiar:** edita el archivo; si tiene el mismo `id` que uno de fábrica, lo reemplaza.
- **Borrar:** borra tu archivo, o crea uno con el mismo `id` y `disabled = true` para
  retirar uno de fábrica.

Identificadores de fábrica: `supply.compressor`, `supply.network`,
`valve.3_2.spring_return`, `valve.5_2.spring_return`,
`cylinder.single_acting_20_100`, `cylinder.double_acting_32_200`,
`throttle.fixed`, `gauge.pressure`, `junction.tee`, los dos filtros, el lubricador y
las dos unidades de mantenimiento. La lista completa es `assets/catalog/`.

Referencia completa de campos (puertos con rol, comportamiento, símbolo, validación):
[assets/catalog/README.md](../assets/catalog/README.md).

---

## 9. La simbología: ISO 1219 (estado real)

Norma aplicable, verificada contra el catálogo de ISO: **ISO 1219-1:2012** (símbolos),
**ISO 1219-2:2012** (esquemas de circuito) e **ISO 1219-3:2016** (módulos de símbolos).
Contrato completo y lista de revisión: [symbology.md](symbology.md).

Lo que exige y **cómo está hoy** en los 17 símbolos de fábrica:

| Requisito | Estado |
|---|---|
| Números de puerto dibujados | **hecho**: salen de la tabla de puertos, el archivo no puede contradecirlos |
| Contenido de cajas como caminos y marcas de bloqueo | **hecho**: el camino sale de la `connection`, la marca de `blocked`, el cuerpo de `body` |
| Muelle y accionamiento como partes declaradas | **hecho**: `[spring]` y `[actuator]`, el lint los exige |
| Clases de línea con su patrón | **hecho**: `line_class` obligatorio en toda línea |
| Proporciones del cilindro y su muelle revisadas | **revisado** a tamaño de paleta y lienzo |
| Recorrido del vástago | **hecho**: el pistón recorre `stroke/(stroke + dead_length)` de la cámara interior y el resto es el muelle comprimido; corregido el 2026-11-06 |
| Legibilidad a 300 dpi | **pendiente** de la revisión a 300 dpi |
| Puntos de unión en cruces de tres líneas | **hecho**: al soltar un tubo sobre otro nace un `junction.tee`, que se dibuja como punto relleno (ADR 0006); un cruce de cuatro no se ofrece |

Las 17 revisiones están en `assets/symbols/review/` con `status: pending-300dpi`:
una prueba falla si un componente de fábrica no tiene revisión, si no dice su estado o
si le falta la captura clara u oscura. Mientras digan `pending-300dpi`, la exportación
lleva la nota «symbols not yet verified against ISO 1219-1».

Mientras no exista la revisión, **la exportación lleva la nota «symbols not yet verified
against ISO 1219-1»** en el bloque de identificación. Es deliberado: un PDF con aspecto
conforme que no lo es sería el peor resultado posible del proyecto.

Pendiente de confirmar (pregunta 2 de §13): la edición vigente del estándar de
identificación de puertos. Desde esta máquina solo se pudieron confirmar 1219-1/2/3.

---

## 10. La interfaz que se va a construir

Resumen; el detalle está en [ui-ux.md](ui-ux.md).

- **Cinta superior** con 3 pestañas (Archivo, Editar, Ver) y el transporte
  `▶ ⏸ ⟲` siempre visible. Colapsa a menú por debajo de 1100 px.
- **Barra lateral que ES el catálogo**: grupos por categoría, búsqueda, arrastre a la
  hoja, y **la miniatura de cada componente es su propio símbolo** (no un set de iconos
  que se desincronice). Pie con la carpeta en uso y los archivos omitidos.
- **Hoja**: rejilla, dos modos visibles (edición / simulación), tuberías ortogonales,
  selección con el color de acento. El hover **nunca** cambia el documento.
- **Barra de estado**: tiempo, presión, posición, velocidad, aire consumido, definiciones,
  modo y último mensaje.
- **Colores del entorno**: `kdeglobals` + portal `color-scheme`, cambio en vivo; acento,
  tipografía e iconos Breeze `-symbolic` teñidos y cacheados. Una capa semántica de
  tokens: **un `Color32::` fuera del crate de tema es un bug**, con test que lo escanea.
- **Accesibilidad**: la de Qt Quick (nombres, teclado, contraste medido) y movimiento
  reducido; `--check-theme` mide el contraste.

---

## 11. Qué se hace a continuación

El orden histórico **M1 → M2 → M1.5 → M3 → M4** se cumplió. Lo que manda hoy es el orden
de entrega de [next-steps.md](next-steps.md): **P1.7, P1.8, P2.1 y P2.2 entregados**,
P3 edición restante, P4 lista de piezas y SVG, P5–P6 bibliotecas
eléctrica/digital/GRAFCET, F7 material didáctico y F8 1.0.

| Hito | Estado |
|---|---|
| **M1** | **hecho** en Qt Quick: cinta, barra lateral, barra de estado, tema del escritorio, iconos Breeze y `--selftest` |
| **M2** | **hecho**: `punchi-fluid-draw`, `punchi-fluid-export`, hojas A5–A0/Letter/Legal/Oficio, bloque de identificación, leyenda, PNG y PDF desde la ventana y sin ella, fecha del día y escala para encajar. Deuda: panel de márgenes, tamaño personalizado y resolución |
| **M1.5** | `symbol lint` (8 reglas) y 17 revisiones visuales; **la revisión a 300 dpi sigue abierta** |
| **M3** | **hecho**: `punchi-fluid-doc`, comandos, deshacer acotado y formato v3 con `id` de catálogo |
| **M4** | **hecho**: conexiones y solver de red; **prueba de modularidad: una 5/2 que funciona solo como archivo** |
| **P1.8** | **hecho**: un documento con varios circuitos, selector, añadir/renombrar y la novena falta (tubo entre circuitos) |
| **P2.1** | **hecho**: caudalímetro en la línea, con su lectura en litros normales por minuto |
| **P2.2** | **hecho**: registrador de 60 s con fuente elegida; dinamómetro `not planned` (necesita dominio mecánico) |
| **P3.1** | **hecho**: repartir con huecos iguales entre cajas, horizontal y vertical, en un solo deshacer |
| **P3.2** | **hecho**: tapón ciego y silenciador como archivos, con el escape restringido de verdad |
| **P3.3** | **hecho**: clases de conducto por tubo (trabajo, pilotaje, drenaje) en el formato 6 |
| **M5** | **hecho**: manómetro, caudalímetro, registrador y «Hasta cambio»; el dinamómetro queda `not planned` |
| **M6** | **no empezado**: i18n con Fluent, AppImage/Flatpak, release firmado |

El paso a paso ejecutable —ficheros exactos, firmas públicas, dependencias, trampas y
criterios de aceptación— está en **[next-steps.md](next-steps.md)**. Los criterios de
aceptación por hito y la evidencia que exige cada uno, en [plan.md](plan.md).

---

## 12. Decisiones tomadas y por qué

Para no volver a discutirlas sin motivo:

| Decisión | Motivo |
|---|---|
| Rust + workspace de crates | física, formato y editor son problemas distintos; el solver se prueba sin ventana y el toolkit queda confinado a un crate ([ADR 0001](adr/0001-language-and-layout.md)) |
| Componentes como archivos TOML | pedido explícito del propietario; una válvula nueva no es un release ([ADR 0002](adr/0002-components-are-files.md)) |
| Masa de gas como estado, presión derivada | la presión no puede ser negativa y el balance es exacto |
| Subpaso interno de 2 µs acotado | estabilidad sin que la interfaz pueda bloquear un fotograma |
| Válvula: `toggle` por clic, no hover | el hover que cambia estado provoca bucles (bug real en el proyecto hermano) |
| Velocidad inicial 0.25× | la carrera real dura 0.13 s; a 1× no se ve |
| Sin `unsafe`, sin `unwrap`/`expect`/`panic` fuera de pruebas | lints del workspace; un fallo recuperable es un error tipado |
| Sin red, sin telemetría, sin autoactualización | superficie de ataque mínima |
| Documentación en inglés, CHANGELOG en español | regla del proyecto; este archivo es la excepción del propietario |
| `printpdf` + `tiny-skia` + `ab_glyph`/`fontdb` elegidos para M2 | vectorial real y rasterizado sin C; `dbus` descartado por arrastrar `openssl-sys`, que `deny.toml` prohíbe (se usará `zbus`) |

---

## 13. Preguntas para ti (con valor por defecto si no contestas)

| # | Pregunta | Hito | Si no hay respuesta |
|---|---|---|---|
| 1 | ¿`git remote add origin` y publicar en GitHub? | **resuelto** | `origin` existe y es GitHub |
| 2 | ¿Edición vigente del estándar de identificación de puertos? | M1.5 | 1/2/4/3/5/12/14 y se dice en la revisión |
| 3 | ¿Cinta colapsa a 1100 o a 1280 px? | M1 | **resuelto**: a 1100 px |
| 4 | ¿Exportación por defecto A4 vertical, 300 dpi, bloque con título/proyecto/revisión/fecha? | M2 | **parcial**: A4–A0, 300 dpi y bloque con título y fecha; la fecha está fija y no hay campos de proyecto ni revisión |
| 5 | ¿El export incluye leyenda de símbolos y lecturas de presión/posición, o solo el diagrama? | M2 | **resuelto**: leyenda sí, lecturas no |
| 6 | ¿Barra lateral de ancho fijo o redimensionable con preferencia guardada? | M1 | **abierto**: el ancho es fijo (248 px), no redimensionable |
| 7 | ¿Solo iconos Breeze simbólicos, o se permite el tema de iconos del usuario? | M1 | **resuelto**: Breeze simbólico, con la etiqueta sola si falta el icono |

---

## 14. Reglas que no se rompen

1. `./scripts-dev/check.sh` en verde **sin silenciar warnings**. Una prueba de
   comportamiento no se borra, desactiva ni debilita para que algo pase.
2. Toda afirmación dice su **nivel de evidencia**: prueba unitaria, prueba headless,
   medición, captura o revisión manual. "Funciona" sin nivel no es una afirmación.
3. **La simbología cumple la norma**, no el gusto. Sin revisión, el documento se marca
   como no verificado.
4. **Los componentes son archivos**: nada de geometría, conductancias ni nombres de
   válvula escritos en Rust.
5. Nada de colores, tamaños ni textos literales en los widgets.
6. Cada caché tiene cota y regla de desalojo; el bucle de simulación no asigna memoria.
7. `CHANGELOG.md` se actualiza en el mismo cambio, bajo `Unreleased`.

Todo lo demás está en [AGENTS.md](../AGENTS.md).

---

## 15. Lo que NO está hecho

- **El repositorio git existe y tiene remoto** (`origin`, GitHub). Lo que no hay es una
  publicación: la versión declarada sigue sin salir.
- **Deudas del armazón**: la familia tipográfica de `kdeglobals` se lee pero no se carga
  (el tamaño sí); el cambio de tema en vivo desde Plasma está cableado, pero no se midió
  tocando la configuración del propietario. Los iconos Breeze sí se cargan y se tiñen.
- **Deuda de la exportación**: no hay panel de márgenes, tamaño personalizado ni resolución
  del PNG. La fecha es la del día, el dibujo se escala para encajar y `Ctrl+Shift+E` abre
  el diálogo. La lista de piezas y la leyenda completa siguen pendientes.
- **La revisión a 300 dpi de los 17 símbolos sigue abierta**: por eso el documento lleva
  la nota «symbols not yet verified against ISO 1219-1».
- **El proyecto existe** desde P1.8: varios circuitos por documento, selector, añadir/renombrar y la novena falta. El título de la hoja no es editable; el proyecto no tiene lista de piezas por circuito.
- **Instrumentos**: manómetro, caudalímetro, registrador de 60 s y «Hasta cambio». El
  dinamómetro está `not planned`: medir fuerza exige un enlace mecánico que el modelo no
  tiene, y una cifra `p·A` con el nombre de fuerza enseñaría lo que no es.
- **Edición incompleta**: separación igual y cuatro alineaciones; propiedades de resumen,
  accionamiento, posición inicial, ángulo, fugas y amortiguamiento; falta del tubo cuyos
  extremos no encajan (la última de las nueve comprobaciones que queda, con la etiqueta del
  encoder).
- **Sin bibliotecas eléctrica, digital ni GRAFCET**, y sin material didáctico propio.
- **Sin hidráulica, sin traducciones (i18n), sin empaquetado (AppImage/Flatpak) y sin
  publicación firmada.**
- **El clic no se ha probado con un clic real**: no hay herramientas de inyección en la
  sesión; la evidencia es `--selftest` y las pruebas del ABI.
- **Los presupuestos de memoria y fotograma no se miden en el gate**: están escritos en
  [engineering.md](engineering.md) y todavía no tienen una prueba que los vigile.
- **El formato v1 y `ComponentKind` siguen en `punchi-fluid-core`/`punchi-fluid-file`**
  por compatibilidad con el CLI y la rebanada original; el editor y la hoja usan el
  formato v3 con `id` de catálogo.

---

## 16. Cómo pedir la continuación en una ventana nueva

Copia esto tal cual en el chat nuevo:

> Trabajamos en `/home/cristian/Desarrollo/punchi-fluid-lab (Rust)`. Lee
> `docs/retomar.md` completo y después `docs/next-steps.md`. Resume en tres líneas el
> estado y continúa con el hito que te indique, respetando `AGENTS.md`. Si algo de lo
> que dice la documentación no coincide con el código, para y dime qué encontraste antes
> de cambiar nada.

Y añade el hito que quieras empezar:

- `Empezamos P3: el resto de la edición de §4.2.`
- `Antes de nada, cierra la revisión a 300 dpi de los símbolos.`
- `Antes de nada, añade el panel de exportación que falta: márgenes, tamaño personalizado y resolución.`
