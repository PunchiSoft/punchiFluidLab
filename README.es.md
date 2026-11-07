# Punchi Fluid Lab

[English](README.md) | **Español**

> 🚧 **En construcción.** Esto es un prototipo que funciona, no un producto terminado.
> Lo construye una sola persona en los días libres que le deja su trabajo principal, así
> que avanza con constancia pero despacio, y **no hay fecha de entrega**: cuando una
> capacidad de [docs/capabilities.md](docs/capabilities.md) dice `missing` o `partial`, es
> exactamente eso, todavía no está. Si algo no existe, se dice; si está a medias, se dice
> qué parte falta.

Simulador libre de circuitos neumáticos e hidráulicos, hecho para que un estudiante sin
licencia, sin presupuesto y sin conexión pueda dibujar el esquema, conectar los puertos,
ejecutar el modelo y ver moverse los actuadores. El modelo, el formato y el dibujo son Rust;
la ventana es Qt Quick.

**Estado: `0.1.0`, sin publicar.** Rebanada vertical que funciona: dibujar y editar un
circuito, conectar puerto a puerto, guardarlo y abrirlo, ejecutar el modelo de gas, leer la
presión en los tubos y exportar la hoja a PNG o PDF.

Sin cuenta, sin clave, sin red. GPL-3.0-or-later: descárgalo, enseña con él, cámbialo,
compártelo.

**Para quién es:** estudiantes de FP e ingeniería, docentes que preparan ejercicios y talleres
que no pueden justificar una licencia comercial. La herramienta con la que se aprende no
debería ser lo que el estudiante no puede pagar.

**Lo que no es:** un visor, un curso ni un servicio en la nube. Sin telemetría, sin
suscripción, sin «edición educativa» recortada.

> **¿Retomas el proyecto?** [docs/retomar.md](docs/retomar.md) es la nota de
> continuidad: estado actual, qué se construyó paso a paso, cómo verificarlo, decisiones
> tomadas y el texto exacto para continuar en una sesión nueva.

## Lo que existe hoy

| Crate | Qué hace ahora |
|---|---|
| `punchi-fluid-core` | Componentes, puertos, conexiones y unidades tipadas (bar, litro, kelvin) |
| `punchi-fluid-catalog` | Definiciones de componente leídas de archivos TOML: esquema, validación y reglas de reemplazo |
| `punchi-fluid-file` | Documento JSON versionado, ida y vuelta y rechazo de versiones nuevas |
| `punchi-fluid-solver` | Integración de masa de gas, cilindro con muelle, amortiguamiento y rozamiento, flujo ISO 6358 |
| `punchi-fluid-doc` | El documento editado: colocaciones, comandos y deshacer acotado |
| `punchi-fluid-draw` | El dibujo único del circuito, en milímetros, tras un trait `Surface` |
| `punchi-fluid-export` | Hojas normalizadas, bloque de identificación, leyenda y escritor PDF determinista |
| `punchi-fluid-ffi` | El ABI C que habla la ventana: una sesión, un cuadro de dibujo, un hit test |
| `punchi-fluid-check` | Comprobaciones de diseño: las faltas que un circuito puede tener antes de ejecutarse |
| `punchi-fluid-cli` | `punchi-fluid-cli validate|simulate <archivo>` para comprobaciones sin ventana |
| `apps/punchi-fluid-qt` | La ventana: cinta, barra lateral de catálogo, hoja, barra de estado y la paleta del escritorio, sobre el ABI C |

## Compilar y ejecutar

```bash
./scripts-dev/run.sh                                    # ventana; compila solo si hace falta
./scripts-dev/run.sh --demo                             # arranca accionada y en marcha
./scripts-dev/run.sh --no-build --dark                  # binario ya compilado, sin compilar
./scripts-dev/run.sh --light                            # fija la paleta para una captura
cargo run --bin punchi-fluid-cli -- validate circuito.pfluid
```

`run.sh` compila solo cuando cambió un fuente y después ejecuta el binario directamente;
`--no-build` no invoca Cargo. `PUNCHI_FLUID_THEME=light|dark` fija la paleta; si no,
la ventana sigue al escritorio.

En la ventana: **Play** ejecuta la simulación, **Slower**/**Faster** cambian la
velocidad (un cilindro de 20 mm con esta válvula completa la carrera en unos 0,13 s de
tiempo simulado), **Actuate valve** o un clic sobre el símbolo de la válvula cambia su
posición, y **Reset** vuelve a empezar. `F1` lista todos los atajos; `Ctrl+F` enfoca
la búsqueda del catálogo.

**Export PDF** (`Ctrl+Shift+E`) o **Export PNG** (`Ctrl+Shift+P`) abre el panel de
exportación: formato, resolución para PNG (72–600 dpi), papel de A5 a A2 más Letter y
Legal, orientación, márgenes, bloque de identificación y destino. La página lleva marco,
bloque de identificación, leyenda de clases de línea y números de puerto y —mientras
alguna revisión de símbolo siga abierta— la nota de que los símbolos aún no están
verificados contra ISO 1219-1. Dos exportaciones del mismo circuito producen los mismos
bytes.

Requisitos: Rust estable (fijado en `rust-toolchain.toml`), Qt 6.5 o superior con Qt Quick,
CMake y Ninja. El núcleo en Rust se compila con `cargo build --release -p punchi-fluid-ffi`;
la ventana con

```bash
cmake -S apps/punchi-fluid-qt -B apps/punchi-fluid-qt/build -G Ninja
cmake --build apps/punchi-fluid-qt/build
apps/punchi-fluid-qt/build/punchi-fluid
```

El núcleo se enlaza como biblioteca estática, así que el resultado es un solo binario sin
ruta de búsqueda. Mantenerlo en Rust es lo que además permite una compilación a
WebAssembly más adelante.

## Los componentes son archivos, no código

Cada válvula, cilindro o suministro es una definición TOML en
[assets/catalog/](assets/catalog/). La aplicación las incorpora al binario y encima
carga `~/.local/share/punchi-fluid-lab/catalog/`, donde un archivo con el mismo `id`
reemplaza al de fábrica:

```toml
id = "valve.3_2.spring_return"
category = "valve"

[behaviour]
type = "valve"
actuation = "toggle"
spring_return = true

[[behaviour.positions]]
name = "rest"
[[behaviour.positions.connection]]
from = "2"
to = "3"
conductance = 1.0e-9
```

Cada posición declara los caminos que abre, clasificados por el **rol** de los
puertos (`supply`, `working`, `exhaust`), así que una válvula 5/2 es un archivo nuevo
y no una rama nueva en el código. El símbolo es una lista de primitivas en el mismo
archivo. Un archivo con un error se omite, se explica en el panel de catálogo y el
resto sigue funcionando. La referencia completa de campos está en
[assets/catalog/README.md](assets/catalog/README.md).

## La interfaz

Tres regiones y nada más: una **cinta** para lo que afecta al documento entero, una
**barra lateral que es el catálogo** (agrupada por categoría, con cada componente
dibujado por su propio símbolo, arrastrable a la hoja) y la **hoja**, donde vive el
circuito y donde ocurre la simulación. Barra de estado abajo. Edición y simulación son
modos visiblemente distintos, para que un clic perdido durante una demostración no
destruya trabajo.

La aplicación toma colores, acento y tamaño de letra del escritorio: lee `kdeglobals`
y sigue el portal `org.freedesktop.appearance color-scheme`, así que se ve como el
resto de Plasma y sigue un cambio de esquema en vivo sin reiniciar. Una capa semántica
de tokens es el único lugar donde existe un color: un `Color32::` en cualquier otro
sitio falla una prueba. Los componentes usan su propio símbolo, nunca un set de iconos
aparte.

**Estado honesto:** las regiones, los tokens, el mapa de teclado y el panel de ayuda
están hechos. Los iconos simbólicos de Breeze todavía no se cargan (los botones usan
texto), la *familia* tipográfica del escritorio se lee pero no se carga (el tamaño sí)
y los textos de la interfaz siguen literales hasta el pase a Fluent de M6.

La especificación completa —regiones, mapa de teclado, requisitos de contraste y lo
que el diseño rechaza— está en [docs/ui-ux.md](docs/ui-ux.md). Los hitos con sus
criterios de aceptación están en [docs/plan.md](docs/plan.md).

## La simbología sigue ISO 1219

Un esquema *casi* conforme es peor que un boceto evidente: se lee como si fuera
correcto. Los símbolos siguen **ISO 1219-1:2012**, con las convenciones de
ISO 1219-2:2012 e ISO 1219-3:2016, y la numeración de puertos sigue la convención de
identificación de puertos de oleohidráulica y neumática.

Lo que eso significa en la práctica, y la revisión que lo exige, está en
[docs/symbology.md](docs/symbology.md): cuadrados iguales por posición de válvula,
flechas en el sentido del flujo, marcas de bloqueo en los puertos cerrados, números de
puerto tomados de la definición, muelle y accionamiento en sus extremos, clases de
línea para trabajo, pilotaje y drenaje, y punto de unión solo donde concurren tres
líneas.

**Estado honesto:** la mitad estructural se impone, no se espera. Una válvula declara su
cuerpo y el renderizador lo divide en cuadros iguales; el camino de flujo y su flecha
salen de la conexión, la marca de bloqueo de la posición, los números de la tabla de
puertos y el muelle y el accionamiento de partes declaradas. Ocho reglas de lint corren
al cargar el catálogo, una prueba por regla.

La mitad visual se revisa por símbolo en
[assets/symbols/review/](assets/symbols/review/) a tamaño de paleta y de lienzo. La
comprobación a 300 dpi necesita la exportación, así que cada revisión dice
`status: pending-300dpi` y la exportación seguirá marcando el documento como no
verificado hasta que dejen de decirlo. Un símbolo reconoce en su propia revisión una
forma más débil: el restrictor fijo se dibujó sin la norma a mano.

## Por qué Rust y por qué estos crates

El producto son tres problemas distintos: la física, el formato de archivo y el
editor de esquemas. Separarlos en crates permite probar el solver sin ventana, migrar
documentos mientras el editor cambia, y confinar la decisión de toolkit —Qt Quick hoy,
QML con `cxx-qt` si la app debe verse nativa en Plasma— a un solo crate.

El razonamiento completo, incluido lo que cuesta, está en
[docs/adr/0001-language-and-layout.md](docs/adr/0001-language-and-layout.md).

## El modelo físico, en un párrafo

La neumática no es CFD. Un circuito es una red de volúmenes unidos por
restricciones: gas ideal en cada volumen, ISO 6358 para el flujo por una restricción
(con la rama crítica por encima de la relación de presiones crítica) y un integrador
rígido, porque la presión cambia en milisegundos. Acertar el modelo importa más que
cualquier decisión de interfaz.

## Validación

```bash
./scripts-dev/check.sh
```

Formato, lints con `-D warnings`, todas las pruebas y una prueba de humo del CLI
sobre un archivo temporal. Un verde demuestra los crates y el formato; la ventana y
la física necesitan ojos y una curva de referencia.

## Hoja de ruta

Los hitos, sus criterios de aceptación y la evidencia que exige cada uno están en
[docs/plan.md](docs/plan.md). En resumen:

- **v0.1** — workspace, dominio, formato, lienzo, CLI, catálogo de definiciones.
- **v0.2** — rebanada vertical: suministro, válvula 3/2 y cilindro de simple efecto con
  integración de masa de gas, dinámica del pistón y válvula accionable. *Hecho, en
  revisión.*
- **M1** — armazón: cinta, barra lateral, barra de estado, tema del escritorio,
  accesibilidad. *Hecho.*
- **M1.5** — conformidad de símbolos con ISO 1219, con lint y revisión por símbolo.
- **M2** — un solo dibujo, dos exportaciones: PDF y PNG desde las mismas llamadas, papel
  normalizado, márgenes, bloque de identificación y leyenda. *Hecho.*
- **v0.3** — documento con deshacer, paleta generada del catálogo, colocar y conectar
  componentes, tuberías ortogonales, guardar y cargar. Un componente colocado
  referenciará un `id` del catálogo, lo que retira `ComponentKind` de la capa de
  documento (hoy todavía enumera los tipos del primer formato).
- **v0.4** — solver de red guiado por el grafo del circuito en vez de una disposición
  fija; válvulas 5/2, reguladores de caudal, manómetros.
- **v1.0** — modelo hidráulico, AppImage y Flatpak, canal de traducciones.

## Documentación del proyecto

| Documento | Para qué |
|---|---|
| [AGENTS.md](AGENTS.md) | reglas canónicas para personas y agentes |
| [CHANGELOG.md](CHANGELOG.md) | cambios por fecha, bajo `Unreleased` |
| [docs/retomar.md](docs/retomar.md) | nota de continuidad: todo lo necesario para retomar sin contexto |
| [docs/next-steps.md](docs/next-steps.md) | estado actual del proyecto y orden de trabajo de lo que viene |
| [docs/ui-ux.md](docs/ui-ux.md) | la interfaz: regiones, temas, accesibilidad |
| [docs/symbology.md](docs/symbology.md) | conformidad ISO 1219 y lista de revisión |
| [docs/plan.md](docs/plan.md) | hitos con criterios de aceptación |
| [docs/engineering.md](docs/engineering.md) | modularización, presupuesto de memoria, seguridad y prácticas de GitHub |
| [CONTRIBUTING.md](CONTRIBUTING.md) | cómo preparar el entorno, probar y enviar cambios |
| [SECURITY.md](SECURITY.md) | cómo reportar una vulnerabilidad en privado |
| [docs/adr/0001-language-and-layout.md](docs/adr/0001-language-and-layout.md) | el stack y la división en crates |
| [packaging/README.md](packaging/README.md) | cómo se empaquetará una versión |
| [i18n/README.md](i18n/README.md) | plan de traducciones |
| [assets/symbols/README.md](assets/symbols/README.md) | set de símbolos y su licencia |

## Licencia

GPL-3.0-or-later. Ver [LICENSE](LICENSE).
