# Changelog

Versión declarada: `0.1.0`, **sin publicar**. El trabajo pendiente se acumula bajo
`## Unreleased` en subsecciones fechadas; la versión cambia solo al preparar una
publicación.

## Unreleased

### Puente hacia una interfaz Qt Quick — 2026-11-06

La costura que ADR 0003 dejó abierta se recorrió hasta el final en una rama: el modelo,
el catálogo y el dibujo siguen en Rust, y una ventana Qt Quick los maneja por un ABI C.
Es una prueba, no una sustitución: `main` conserva la interfaz `egui` y la decisión
—[ADR 0005](docs/adr/0005-qt-quick-over-a-c-abi.md), en estado *propuesto*— la toma el
propietario.

- **Crate nuevo `punchi-fluid-ffi`**: una sesión opaca, telemetría en SI, `PfInk` para el
  color, y el dibujo cruzando como **búfer de comandos** —un `PfCall` por primitiva— en
  vez de como devoluciones de llamada, para que la ventana, el PNG y el PDF consuman la
  misma lista que ya compara `Recording`. Los patrones de trazo se expanden en Rust con
  la misma función que usan los dos backends, así que no hay tres implementaciones de
  guion.
- **`unsafe` está permitido en ese crate y en ningún otro.** El espacio de trabajo lo
  prohíbe con `forbid`, que un crate no puede relajar; `punchi-fluid-ffi` no hereda esos
  lints, los repite con `deny` y añade `undocumented_unsafe_blocks` para que cada puntero
  tenga su justificación escrita.
- **Prueba de la costura**: siete pruebas nuevas que crean una sesión desde el catálogo
  incorporado, la avanzan 400 pasos hasta mover el pistón y presurizar el cilindro,
  comprueban que `pf_abi()` coincide con los tipos de Rust, pasean cada primitiva del
  cuadro para verificar que ninguna se sale del área y comprueban que un puntero nulo se
  rechaza en vez de reventar.
- **Prueba de la ventana**: `spike/qt-qml/` compila con CMake y Qt 6.11, enlaza el núcleo
  como biblioteca estática (7 MB, sin ruta de búsqueda) y `--grab` renderiza una captura
  fuera de pantalla para revisarla sin sesión gráfica.
- **El color sigue viniendo de fuera**: la ventana lee `QPalette` —que el plugin de
  plataforma ya rellenó desde `kdeglobals` y el portal— y la pasa como `PfInk`; en Rust
  no hay un solo literal de color.

#### Verificación

- `cargo test -p punchi-fluid-ffi`: 7 pruebas en verde; `cargo clippy --all-targets -D
  warnings` y `cargo fmt --check` limpios.
- `QT_QPA_PLATFORM=offscreen ... --grab`: 1100×720 escritos, en reposo y accionada. La
  válvula cambia de cuadro activo, la lectura pasa de 1,01 a 5,79 bar y la carrera de 0 a
  100 mm.
- **No se ejecutó `./scripts-dev/check.sh`**: el enlace de los binarios de prueba en
  perfil de desarrollo agota la memoria de esta máquina.

#### Limitaciones conocidas

- El clic real, extremo a extremo, no está probado: la prueba de impacto es unitaria en
  Rust y ambas partes usan la misma función de encuadre. La captura accionada se produce
  por el mismo camino que el clic, no por un evento sintético.
- `QQuickPaintedItem` rasteriza el cuadro completo en CPU. Con 200 componentes y 4 ms de
  presupuesto (M4) habrá que medirlo; si no llega, el paso siguiente es `QSGGeometry` y
  solo cambia la implementación de `Surface`.
- El cajón de utilidades de Qt Quick Controls aún no se ha usado: la barra de la prueba
  son primitivas de Qt Quick.

### Recorrido del vástago del cilindro — 2026-11-06

El pistón recorre ahora la longitud interna que declara el archivo, no una fracción
elegida por el dibujo.

- **La carrera sale de la definición.** `draw_cylinder` reparte la extensión sobre
  `stroke_mm / (stroke_mm + dead_length_mm)` de la cámara interior, así que al 100 % el
  pistón deja contra la pared del lado del vástago solo la longitud muerta declarada, que
  es el espacio donde se comprime el muelle de retorno. Antes recorría el 34 % de la
  cámara: con la lectura en 100 % el pistón seguía a mitad del barril y el muelle apenas
  se comprimía.
- **El barril se acortó en las dos definiciones** (`0.08..0.50` del recuadro) para que el
  vástago siga viéndose fuera del barril en reposo y su punta llegue al borde del
  recuadro a plena carrera, sin alargarse ni acortarse nunca.
- **Cilindro de doble efecto**: el puerto 1 pasa a la línea de eje en la culata —allí no
  hay vástago— y el 2 queda arriba, junto a la pared del lado del vástago, porque el
  vástago ocupa el eje en ese extremo.
- Dos pruebas nuevas: la carrera recorrida y el resto de longitud muerta al 100 %, y el
  muelle que se acorta con la carrera, que nunca sale del barril y que pierde espiras en
  vez de amontonarlas. Las pruebas del pistón dentro del barril y del vástago de longitud
  constante siguen en pie.
- Imágenes de revisión de los cinco símbolos regeneradas con el mismo código de dibujo de
  la hoja (`PUNCHI_FLUID_WRITE_SAMPLE=1 cargo test -p punchi-fluid-export --test generate`).

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **169 pruebas**. Dos nuevas en
  `punchi-fluid-draw`: la carrera recorrida con el resto de longitud muerta al 100 %, y el
  muelle que se acorta, que nunca se dibuja fuera del barril y que pierde espiras en vez de
  amontonarlas.
- Inspección de las tiras de carrera a 0/25/50/75/100 % y de la hoja A4: el pistón llega
  al fondo de la cámara, el muelle se comprime hasta la longitud muerta y la punta del
  vástago coincide con el borde del recuadro. Imágenes de revisión regeneradas.

#### Limitaciones conocidas

- `assets/symbols/review/catalog-*.png` son capturas de `PUNCHI_FLUID_PREVIEW=1`; siguen
  siendo las anteriores a esta corrección y hay que retomarlas con la ventana abierta.

### Habilidades del repositorio para Rust, GitHub y simbología — 2026-11-06

Se creó el conjunto inicial de `.agents/skills/`: nueve procedimientos en inglés que
complementan `AGENTS.md` sin repetirlo, con `license: GPL-3.0-or-later`, descripción con
cláusula `Use when`, procedimiento, trampas y validación.

- **Rust**: `rust-idioms` (propiedad, errores con `thiserror`, unidades tipadas, lints y
  MSRV 1.85), `rust-workspace-architecture` (crates acíclicos y los tres puntos de
  extensión), `rust-testing-and-verification` (nivel de prueba según la afirmación,
  invariantes, referencias y determinismo), `rust-performance-and-memory` (presupuestos,
  rutas sin asignación, cachés acotadas) y `rust-dependencies-and-supply-chain`
  (licencias, `cargo deny`, lockfile y MSRV).
- **GitHub**: `github-collaboration` (Conventional Commits, DCO, plantilla de PR,
  issues, releases y ajustes del repositorio) y `github-actions-security` (inyección por
  `${{ }}`, permisos mínimos, actions fijadas por SHA, cachés y procedencia). El segundo
  deja anotadas las deudas actuales de los flujos: el `contents: write` del workflow de
  release aplica también a los jobs de construcción, y varias actions siguen en
  etiquetas móviles.
- **Dominio**: `fluid-power-symbology` fija el procedimiento de conformidad ISO 1219-1,
  -2 y -3 para símbolos neumáticos e hidráulicos —la regla de no calcar, el vocabulario,
  las listas de válvula, actuador y esquema, las diferencias entre neumática e
  hidráulica, el lint, la revisión y las preguntas abiertas— y `solver-validation`
  convierte las reglas de física en pruebas con referencia y tolerancia (ISO 6358,
  invariantes, integrador y modelo hidráulico pendiente).

El catálogo [`.agents/skills/README.md`](.agents/skills/README.md) registra cada skill,
propone un orden de lectura y deja constancia de las fuentes consultadas (Rust API
Guidelines, Google Rust Style Guide, Effective Rust, Rust Design Patterns, Microsoft
Pragmatic Rust Guidelines, `cargo-deny`, RustSec, Conventional Commits, DCO,
Keep a Changelog, SemVer, guía de endurecimiento de GitHub Actions y OpenSSF
Scorecard). No se copió texto ni se incorporó ningún recurso de terceros; las
colecciones públicas `BeautifulRust` y `github/awesome-copilot` quedan como lectura
adyacente, no como dependencia.

#### Verificación

- Frontmatter comprobado por script: `name` igual al directorio, `license` correcta y
  descripción de 186–225 caracteres con `Use when` (el límite es 300).
- Todos los enlaces relativos de los nueve `SKILL.md` resuelven en el árbol.
- El cambio no toca código Rust. `./scripts-dev/check.sh` se detiene en
  `cargo fmt --all --check` por un archivo ajeno a este cambio,
  `crates/punchi-fluid-export/tests/scratch_strokes.rs`, que ya está sin formatear en el
  árbol; no se reformateó para no mezclar un cambio mecánico ajeno con este. El resto de
  la puerta no se ejecutó porque no hay nada que compilar distinto.

#### Limitaciones conocidas

- `schematic-editor` y `release-and-packaging` siguen pendientes; el catálogo lo dice.
- La edición vigente de la norma de identificación de puertos (ISO 11727 para neumática,
  ISO 9461 para hidráulica) queda por confirmar en el catálogo ISO antes de citarla en
  una revisión de conformidad.

### Animación y cilindro de doble efecto — 2026-11-06

La animación del cilindro se reescribió para que siga la simbología y la definición.

- **El cilindro se dibujaba dos veces**: `draw_component` lo animaba al 40 % y el
  esquema volvía a dibujar el pistón con la extensión real, así que el vástago aparecía
  en dos posiciones. Ahora hay una sola llamada y la extensión viaja como parámetro.
- **El movimiento sale del archivo**: el barril es el rectángulo más ancho que declara la
  definición y el pistón recorre su interior; antes eran constantes escondidas en el
  código. Una prueba comprueba que el pistón no se sale del barril en los dos extremos.
- **El muelle solo se dibuja si el archivo lo declara** y se comprime con la carrera:
  misma cuerda, menos paso y algo más de amplitud al juntarse las espiras.
- **Cilindro de doble efecto nuevo** (`cylinder.double_acting_32_200`): dos puertos de
  trabajo arriba, uno por extremo, sin muelle, vástago por el extremo del vástago. Los
  puertos van arriba porque el vástago sale por ese extremo en la línea de eje y un
  puerto ahí se dibujaría encima.
- **Imágenes de revisión regeneradas** para los cinco símbolos con el mismo código de
  dibujo que la hoja, a 200 dpi y en claro y oscuro, mediante
  `PUNCHI_FLUID_WRITE_SAMPLE=1 cargo test -p punchi-fluid-export --test generate`.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **148 pruebas**. Dos nuevas: el pistón
  no sale del barril en los dos extremos de la carrera, y solo el cilindro de simple
  efecto dibuja muelle (tres líneas frente a dos).
- Inspección de las cinco imágenes: válvula con sus dos estados espejados, cilindro de
  simple efecto con muelle y vástago, cilindro de doble efecto con dos puertos y sin
  muelle.

#### Limitaciones conocidas

- **El cilindro de doble efecto no se simula todavía**: dos líneas de trabajo exigen el
  solver de red de M4. La definición y el símbolo están hechos; el circuito que lo
  acciona, no.

### Corrección de la simbología de las válvulas — 2026-11-06

Revisión pedida por el propietario: la válvula no se leía como el símbolo de la norma.

- **Cada posición se dibuja dentro de su propio cuadro.** Antes el camino de una
  posición cruzaba el límite y se dibujaba sobre el cuadro vecino, así que el símbolo no
  se podía leer estado por estado. Ahora el camino se calcula con el desplazamiento del
  puerto *dentro de un cuadro*, que es el mismo en los dos porque los cuadros son
  iguales: las dos posiciones salen especulares, como en el símbolo real.
- **Los puertos se dibujan en el cuadro de reposo**, un solo juego de números con sus
  rabillos sobre el cuerpo. Así cada puerto tiene un desplazamiento propio e inequívoco
  (antes 1 y 3 caían en el mismo desplazamiento local y el camino de reposo terminaba
  sobre el puerto equivocado).
- **La marca de bloqueo va sobre el rabillo del puerto**, justo fuera del cuerpo, en vez
  de dentro de un cuadro: ahí no se puede confundir con una línea de otra posición.
- Prueba nueva: cada camino de cada posición queda dentro de su cuadro, comprobado punto
  por punto; y la comprobación de que el spool cambia el dibujo sigue en pie.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **146 pruebas**.
- Inspección a 300 dpi del símbolo corregido: dos cuadros iguales, 1 y 3 abajo, 2 arriba,
  reposo con flecha 2→3 y marca de bloqueo en 1, accionada con flecha 1→2 en el otro
  cuadro, muelle a la izquierda y palanca a la derecha.
- **Pendiente**: las capturas de la revisión del símbolo son anteriores a esta
  corrección y hay que regenerarlas; por eso la revisión sigue `pending-300dpi`.

### La barra, medida por su contenido — 2026-11-06

Cuarta revisión. Lo que se vio como «bugeada» tenía cuatro causas concretas, todas en el
código de la barra:

- **Los rótulos se dibujaban con un ancho fijo.** Cada botón reservaba
  `4,2 × grid_unit` para el texto y luego lo escribía igual, así que «Guardar como» se
  salía del botón y se montaba sobre el icono siguiente. Ahora cada botón es un
  `egui::Button` con imagen y texto, que **se mide a sí mismo**: el ancho sale del
  contenido y no de una constante.
- **Cada botón llevaba marco y borde.** Se veía una fila de cajitas dentro de la ventana.
  El estilo plano —sin relleno, sin borde, resaltado sólo bajo el puntero— ahora vive en
  `shell::flat_controls` y lo comparten el menú y la barra, así que no pueden divergir.
- **Los separadores de grupo habían desaparecido** al limpiar los rellenos: `ui.separator()`
  toma su trazo del estado no interactivo, que se había puesto a cero. Se repone con el
  color de contorno del tema.
- **Las texturas se pedían al contexto mientras se subía una textura**: un bloqueo mutuo
  que dejaba la ventana congelada. La caché pasó a ser propiedad de la barra y se pasa por
  parámetro.

Se retiró además el juego de iconos dibujados a mano (doscientas líneas): la fuente es el
tema del escritorio y, si no hay icono, el botón muestra sólo su rótulo, como cualquier
aplicación nativa.

Y dos correcciones de legibilidad, que es de lo que se quejó el propietario:

- **La franja se pinta con el token del tema**, no queda sin pintar. Es lo que explica la
  ventana «mitad clara, mitad oscura» de la captura: un panel sin relleno no se repinta, así
  que al cambiar el esquema del escritorio el lienzo se actualizaba y la barra se quedaba
  con los píxeles del tema anterior, en negro. Ahora la franja se pinta siempre y un cambio
  de tema pide un repintado completo de la ventana.
- **El texto deshabilitado se repara a 4,5:1**, no a 3:1. Un rótulo atenuado sigue siendo
  un rótulo: en la barra oscura se leía mal, y la prueba de contraste lo exige ahora para
  los dos esquemas.
- **El icono sigue al esquema**: sin tema declarado en `kdeglobals`, una ventana oscura
  usa `breeze-dark` y una clara `breeze`. Antes se pedía siempre `breeze` —trazos
  oscuros— y sobre la barra oscura los iconos desaparecían. El tema se vuelve a resolver
  cuando cambia el esquema, y la caché de texturas se limpia con él.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **169 pruebas**.
- Inspección de la ventana en claro y en oscuro: barra plana de una fila, iconos del tema a
  color, rótulos completos sin solaparse, separadores entre grupos, `0.25x` y `Simulación`
  a la derecha.

### Los iconos son los del escritorio — 2026-11-06

Tercera revisión de la barra, pedida contra la herramienta nativa de referencia.

- **Los pictogramas ya no los dibuja el proyecto**: cada acción nombra un icono
  freedesktop (`document-new`, `edit-undo`, `media-playback-start`…) y el crate nuevo
  **`punchi-fluid-icons`** lo busca en el tema de iconos del escritorio y lo rasteriza.
  Es lo que hace que la barra se vea como el resto de la sesión y **cambie con el tema
  sin recompilar**, y también lo que la mantiene dentro de la regla del proyecto: no se
  copia ni se redistribuye ningún recurso ajeno; los ficheros son del escritorio que los
  instaló. Ver [ADR 0004](docs/adr/0004-icons-from-the-desktop.md).
- **Búsqueda freedesktop**: `$XDG_DATA_HOME/icons` y `$XDG_DATA_DIRS/icons`, el tema que
  nombra `kdeglobals`, la cadena `Inherits` de su `index.theme` y `hicolor` al final; la
  carpeta se elige por cercanía al tamaño pedido y nunca se toma la variante `symbolic`,
  porque una barra quiere el icono a color.
- **Rasterizado con `resvg` sin características por omisión**: un icono de Breeze son
  trazos, así que texto, fuentes del sistema e imágenes rasterizadas quedan fuera del
  grafo. Licencias del grafo nuevo comprobadas contra la lista de `deny.toml` con
  `cargo metadata`: todo MIT/Apache-2.0/Zlib.
- **Nuestro dibujo de líneas sigue ahí como respaldo**: una máquina sin tema, un tema sin
  ese icono o un nombre sin estándar no dejan un hueco en la barra.
- **Mano al pasar sobre una acción**: `CursorIcon::PointingHand` en los botones y en los
  menús, con el globo que nombra el comando; una acción no disponible no la muestra.
- **Sin marcos ni rayas de panel**: la barra de menús y la de herramientas son la misma
  superficie que el resto de la ventana, como en la referencia; sólo el contenido se
  mueve. Icono de 1,25 × la unidad de rejilla, alto de botón 1,25 × la fila, ambos del
  tema.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **169 pruebas**. Nuevas: el tema resuelve
  los nueve nombres de la barra en Breeze (`document-new` … `view-refresh`), un SVG plano
  se rasteriza al tamaño pedido, un fichero que no es SVG devuelve `None` en vez de
  fallar, un icono inexistente se reporta como ausente y no como error, y toda acción
  tiene nombre freedesktop o dice que no lo tiene.
- **`cargo deny` no está instalado en esta máquina**: las licencias se comprobaron a mano
  sobre `cargo metadata` (398 paquetes) contra la lista de `deny.toml`; la puerta de CI
  sigue siendo la que manda.
- Inspección de la ventana: iconos de Breeze a color en la barra, menú de texto plano sin
  marcos, `0.25x` y `Simulación` a la derecha, nada más.

### Barra de menús, cinta con iconos y las dos correcciones de símbolo — 2026-11-06

- **Barra de menús en la zona superior**, encima de la cinta: `Archivo`, `Editar`, `Ver`,
  `Simular` y `Ayuda`, y la última entrada abre **Acerca de**, con nombre, versión,
  licencia y el estado de la conformidad de los símbolos. Cada entrada es la misma
  `Action` que ya usan la cinta y el teclado, así que ningún comando existe dos veces.
  El último menú y su última entrada son, como en cualquier escritorio, los de ayuda.
- **Barra de herramientas nativa, como la de Kate**: una sola fila de botones pequeños,
  **icono y rótulo** para los comandos frecuentes (`Nuevo`, `Abrir`, `Guardar`,
  `Guardar como`) e **icono solo con su globo** para el resto, con una raya fina entre
  grupos. Es una **selección**: los menús conservan todos los comandos, así que la barra
  no tiene que crecer hasta convertirse en una cinta. Los iconos son un juego propio,
  dibujado con segmentos sobre la paleta del tema (`punchi-fluid-ui::icons`): ni fuente
  de iconos, ni mapa de bits, ni nada trazado de otro programa, igual que los símbolos
  del esquema. La lista de botones está en una sola tabla y **una prueba la recorre**.
- **Ninguna medida suelta**: el alto de la barra, el lado del icono, el tamaño del
  rótulo, el ancho de un menú y el de la ventana Acerca de son valores del tema
  (`Metrics`), no multiplicadores escritos en el widget.
- **La barra responde al puntero**: al pasar sobre un icono de acción aparece la **mano**
  (`CursorIcon::PointingHand`), igual que en el escritorio, con el globo que nombra el
  comando y su atajo; una acción no disponible no la muestra. Los menús y el botón de
  modo usan el mismo cursor.
- **Los rótulos de la barra siguen el idioma del escritorio**: `Nuevo`, `Abrir`,
  `Guardar`, `Guardar como`, `Deshacer`, `Rehacer` en un escritorio en español.
- **Acerca de no va en la barra**: vive en el menú `Ayuda`, como en la herramienta
  nativa de referencia. La derecha de la barra sólo dice cómo corre el modelo.
- **Los rótulos siguen el idioma del escritorio**: con `LANG=es…` el menú dice
  `Archivo` y `Acerca de`, y con un escritorio inglés dice `File` y `About`. Es un
  puente hasta el pipeline de Fluent que describe [i18n/README.md](i18n/README.md), y
  está dicho así en el código.
- **La fuente de aire comprimido ya no es hidráulica**: el triángulo pasa a **contorno**.
  El relleno no es estilo, es lo que distingue un gas de un líquido en el plano; el
  triángulo macizo es el de la fuente hidráulica.
- **El vástago dejó de encogerse.** Antes se dibujaba desde el pistón hasta el borde de
  la caja, así que al extenderse el cilindro el vástago se acortaba como un telescopio.
  Ahora es una pieza de longitud constante cuya punta acompaña al pistón, y el barril se
  acortó a 0,62 de la caja en las dos definiciones para que el vástago se vea en reposo
  y su punta llegue justo al borde de la caja a plena carrera, sin salirse del dibujo.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **160 pruebas**. Nuevas: cada acción de
  una pestaña aparece en un solo grupo de esa pestaña, la tira de pestañas y el menú
  hablan el mismo idioma, y el vástago conserva su longitud en los tres puntos de la
  carrera con la punta siempre dentro de la caja y fuera del barril en reposo.
- Inspección de la ventana: barra de menús en español, cinta con grupos e iconos, fuente
  de aire con triángulo hueco y cilindro con el vástago rígido.
- Imágenes de revisión regeneradas con el mismo código de dibujo que la hoja.

### El aire dice por dónde va — 2026-11-06

La animación del circuito se completó con lo que muestra el vídeo de referencia: sobre
una línea que lleva aire, una punta de flecha maciza que señala el sentido del caudal y
gira cuando el modelo lo invierte.

- **El modelo expone el caudal con signo.** `PneumaticSystem::working_flow()` es positivo
  cuando la válvula llena el cilindro y negativo cuando lo desairea —el aire cruza la
  válvula al revés—, y `supply_flow()` es el caudal que entra a la válvula por la
  alimentación. Antes el paso interno se calculaba y se descartaba: el dibujo no tenía de
  dónde sacar una dirección.
- **Una flecha por tramo, en el medio del tramo**, no una por línea: un caño que dobla
  sigue diciendo hacia dónde va el aire después del codo. Se omite en tramos demasiado
  cortos para que la punta se lea.
- **Sin caudal no hay flecha.** Un puerto cerrado o un cilindro que ya llegó a la presión
  de alimentación dejan la línea sin flechas, así que la punta es el caudal y no un
  adorno: en reposo la alimentación no marca nada y la línea de trabajo marca el desaireo.
- **Misma tinta que su línea**, tomada del color por presión que ya existía, así que la
  dirección nunca compite con el estado.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **154 pruebas**. Dos nuevas: el caudal de
  la línea de trabajo cambia de signo entre llenar y desairar, y las flechas dibujadas
  apuntan al puerto del cilindro mientras se llena y en contra cuando se desairá —la
  prueba busca la punta como el vértice más lejano y compara distancias al puerto—.
- Inspección del esquema exportado a mitad de carrera: alimentación con tres puntas
  (baja, cruza y sube al puerto 1) y la línea de trabajo con la suya hacia el cilindro.

### La válvula, en la orientación de la norma — 2026-11-06

Segunda revisión pedida por el propietario: el símbolo seguía sin leerse como el de la
norma aunque cada posición ya viviera en su cuadro.

- **Espejado al revés.** El muelle estaba a la izquierda con el cuadro de reposo contra
  él y la palanca a la derecha. En el símbolo de la norma el accionamiento va en un
  extremo y el muelle en el opuesto, y el cuadro que queda contra el muelle es el de
  reposo, el que el resorte mantiene. Ahora la palanca va a la izquierda, el muelle a la
  derecha y las posiciones se cuentan **desde el extremo del muelle**: la posición 0 se
  dibuja en el cuadro pegado al resorte.
- **Rabillos torcidos.** Las posiciones de puerto se leían en el marco del cuerpo y el
  resto del dibujo en el de la caja del símbolo, así que cada rabillo salía inclinado;
  con los puertos ya sobre la mitad derecha, la inclinación se veía. Ahora se leen en la
  caja del símbolo y el rabillo sale recto, como en el símbolo.
- **El cierre de un puerto es una T dentro de su cuadro**, no una barra cruzada fuera
  del cuerpo: el rabillo del puerto entra en el cuadro y termina en una barra. Cada
  posición declara así sus propios cierres, y los dos cuadros se pueden comparar. La T
  de la posición inactiva va en tinta atenuada.
- **El camino de una posición es una recta** de puerto a puerto dentro del cuadro, con
  la punta en el puerto de destino, en vez de una línea quebrada por el centro: vertical
  cuando une dos puertos del mismo lado, inclinada cuando lleva un puerto al escape.
- **Rótulo de la válvula al costado.** Con los puertos hacia abajo, el rótulo bajo el
  cuerpo se escribía encima de la línea de alimentación que sube al puerto 1; ahora va a
  la derecha del cuerpo, alineado a la izquierda, con `haloed_text_anchored` nuevo.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos; **152 pruebas**. Dos nuevas: la barra de
  cierre de cada posición cae dentro del cuadro de esa posición y las posiciones se
  cuentan desde el extremo del muelle; las dos sustituyen a la prueba débil anterior, que
  solo comparaba dos dibujos.
- Imágenes de revisión y hoja del catálogo regeneradas: palanca a la izquierda, muelle a
  la derecha, reposo en el cuadro derecho con la T sobre el puerto 1 y la flecha 2→3,
  accionada en el cuadro izquierdo con la T sobre el puerto 3 y la flecha 1→2.
- Inspección de la ventana: la válvula descansa en «rest», el rótulo queda libre de
  líneas y la alimentación sube al puerto 1 sin cruzar texto.

### Un solo dibujo, exportación PDF y PNG (M2) — 2026-11-06

El esquema se dibuja una vez, en milímetros, y lo consumen la ventana y el papel. La
pantalla y el PDF no pueden divergir porque son la misma secuencia de llamadas.

- **Crate `punchi-fluid-draw`**: trait `Surface` con seis operaciones (línea, polígono,
  rectángulo, círculo, arco, texto), `Recording` que guarda cada llamada para las
  pruebas, y todo el dibujo del símbolo y del circuito movido desde la UI. El trait es
  la decisión del [ADR 0003](docs/adr/0003-one-drawing-two-devices.md).
- **Coordenadas en milímetros** en la frontera; cada dispositivo aplica su escala. Los
  desplazamientos del dibujo son fracciones del tamaño del símbolo, no milímetros de
  papel: por eso la composición es la misma en una ventana y en un A2, y una prueba lo
  comprueba comparando las llamadas de dos áreas proporcionales.
- **Trazos discontinuos resueltos antes del dispositivo**: `dashes()` parte la polilínea
  en segmentos sólidos en milímetros del dibujo, así que pantalla y PDF ponen los huecos
  en el mismo sitio en vez de inventar cada uno su patrón.
- **`EguiSurface`** en `punchi-fluid-ui`: un milímetro es un punto en pantalla. El
  lienzo queda con lo que un archivo no puede tener —posición del puntero, impacto del
  clic, cursor— y las dos pruebas de clic siguen pasando **sin cambios**.
- **Crate `punchi-fluid-export`**: `Page` con A5/A4/A3/A2/Letter/Legal, orientación,
  márgenes y bloque de identificación; marco, leyenda de clases de línea y de numeración
  de puertos; y un **escritor PDF propio**, sin dependencias, determinista: sin fecha de
  creación, sin identificador, sin nombre de aplicación. Texto en Helvetica como cadenas
  hexadecimales `WinAnsi`.
- **El bloque de identificación consulta las revisiones**: `punchi-fluid-catalog::reviews`
  compila los cuatro archivos de revisión en el binario y el export pregunta si puede
  quitar la nota «symbols not yet verified against ISO 1219-1». Mientras alguno diga
  `pending-300dpi`, la nota está.
- **Backend PNG** (`tiny-skia` + `ab_glyph` + `fontdb`): el mismo `Surface` en un mapa
  de bits, de 72 a 600 dpi. Se rechaza antes de asignar memoria si la resolución o el
  tamaño superan el presupuesto (80 Mpx). El texto se compone con la fuente sans del
  sistema; si la máquina no tiene ninguna, el dibujo conserva las formas y omite el
  texto, porque un diagrama sin rótulos sigue siendo un diagrama y un fallo no.
- **Legibilidad medida en el dispositivo, no en el dibujo**: el trait expone
  `device_scale()` y el dibujo omite los números de puerto cuando en el dispositivo
  medirían menos de 12 px. Así una miniatura de paleta no los muestra y una hoja A4 a
  300 dpi sí, con la misma definición.
- **La nota de «sin verificar» pasa dentro del marco**: la banda de margen pertenece al
  papel y una prueba de píxeles lo comprueba.
- **Panel de exportación** (`Ctrl+Shift+E` para PDF, `Ctrl+Shift+P` para PNG): formato,
  resolución, papel, orientación, márgenes, campos del bloque, destino y tamaño
  resultante, sin modal. La escritura es atómica: se escribe un `.part` y se renombra.
- **Halo en las etiquetas**: los rótulos se dibujan sobre un fondo del color de la hoja,
  de modo que una tubería puede pasar por detrás de un nombre sin volverlo ilegible. Sin
  medir el texto y sin una capa distinta en cada dispositivo.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos de clippy.
- **145 pruebas** (antes 93): `draw` 13, `export` 31, `ui` 15, y el resto igual.
- Pruebas de aceptación del export: MediaBox dentro de 1 pt del papel elegido, una sola
  página, sin `/Producer`, `/CreationDate`, `/ID` ni rutas ni usuario, dos renderizados
  idénticos byte a byte, nota presente mientras la revisión esté abierta, rechazo de una
  hoja sin área de contenido, dibujo dentro del rectángulo de contenido y diferencia
  solo de escala entre dos tamaños.
- Pruebas del ráster: A4 a 300 dpi da 2480 × 3508 y A3 horizontal 4961 × 3508 (leído del
  IHDR del PNG), banda de margen sin tinta, dibujo presente dentro del contenido, dos
  renderizados idénticos, y rechazo de 1200 dpi o de un mapa de bits por encima del
  presupuesto **antes** de asignar.
- Inspección visual del PDF con `pdftoppm` (150 y 100 dpi) y del PNG a 300 dpi: hoja,
  marco, leyenda, bloque y símbolos se leen, y los números de puerto aparecen a tamaño
  de impresión.

#### Limitaciones conocidas

- **Falta la imagen de revisión por símbolo**: el PNG a 300 dpi de la hoja existe y se ve
  correcto, pero la revisión pide una imagen por símbolo y el restrictor no está en el
  circuito fijo. Por eso las cuatro revisiones siguen `pending-300dpi` y la nota de «sin
  verificar» sigue en el documento.
- **El texto del PNG depende de la fuente del sistema**: misma versión de fuente, mismos
  bytes; otra máquina puede producir otro archivo.
- Las esquinas redondeadas se dibujan cuadradas en PDF y PNG (0,1 mm de radio no se ve a
  ningún tamaño de impresión razonable).
- El ancho del texto en PDF se estima con una tabla de Helvetica; un rótulo centrado
  puede quedar una fracción de milímetro desplazado respecto de la pantalla.

### Conformidad de símbolos ISO 1219 (M1.5, sin cerrar) y hoja de catálogo — 2026-11-06

Los símbolos dejan de ser dibujos a mano: se declaran y el renderizador los construye.
Donde antes había coordenadas había una promesa que el lint puede comprobar.

- **Esquema 2 del catálogo.** Un archivo antiguo se rechaza con el número de versión en
  el mensaje. Campos nuevos: `body = [x, y, ancho, alto]`, `[spring] side = …`,
  `[actuator] kind, side`, `line_class` obligatorio en toda línea y polilínea, y
  `blocked = ["1"]` por posición.
- **El cuerpo no se dibuja, se deduce.** `ComponentDefinition::body_squares()` parte el
  cuerpo declarado en un cuadro igual por posición, así que dos cuadros no pueden salir
  desiguales ni desalineados. El lint comprueba que el ancho sea exactamente
  `alto × posiciones`.
- **El camino sale de la física.** La flecha de cada posición se enruta desde la tabla
  `connection`, con el sentido desde el primer puerto al segundo y una línea de cota por
  posición para que dos caminos no se pisen. La flecha no puede apuntar al revés porque
  el archivo no la dibuja.
- **Partes declaradas.** Muelle y accionamiento (palanca, botón, pedal, electroimán,
  pilotaje) se dibujan con la geometría de `punchi-fluid-ui::parts`; el lint exige muelle
  cuando `spring_return = true` y accionamiento cuando `actuation != none`.
- **Números de puerto y marcas de bloqueo** desde la tabla de puertos y desde `blocked`,
  no desde el archivo. Por debajo del umbral de legibilidad el número no se dibuja: un
  borrón no es un número.
- **`symbol lint`** en `punchi-fluid-catalog::lint`, ocho reglas con una prueba cada una
  (cuerpo, posición vacía, puerto sin usar, muelle, accionamiento, geometría, clase de
  línea, nada dibujado dentro del cuerpo). Un archivo que falla se omite y se explica.
- **Clases de línea** con patrón propio en el renderizador: trabajo continua, pilotaje
  discontinua corta, drenaje discontinua larga, eléctrica punteada.
- **Cuatro símbolos redibujados** desde cero con la geometría derivada, más el cilindro
  que ahora dibuja pistón, vástago y muelle en la vista previa y en la miniatura.
- **Revisión por símbolo** en `assets/symbols/review/`: un `.md` con estado, fecha,
  procedencia y tabla de comprobaciones, más capturas clara y oscura por símbolo y de la
  hoja completa. `status: pending-300dpi` en los cuatro: falta el render a 300 dpi, que
  necesita la exportación.
- **Prueba de la carpeta de revisión** (`crates/punchi-fluid-catalog/tests/review.rs`):
  falla si un componente de fábrica no tiene revisión, si no declara estado, si no tiene
  fecha o si le falta una de las dos capturas.
- **Vista previa del catálogo**: `PUNCHI_FLUID_PREVIEW=1 ./scripts-dev/run.sh` dibuja
  todas las definiciones grandes, en una rejilla cuadrada, sin barra lateral ni barra de
  estado. Es la hoja de la que salen las capturas de la revisión.

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos de clippy.
- **93 pruebas** (antes 69): `catalog` 23 (7 previas + 16 de lint), `review` 3, `ui` 15,
  y el resto igual. Las dos pruebas de clic siguen pasando sin cambios.
- Capturas de esta sesión: la hoja de catálogo y los cuatro símbolos en claro y oscuro,
  en `assets/symbols/review/`.

#### Limitaciones conocidas

- **A 300 dpi no está comprobado.** Por eso los cuatro estados son `pending-300dpi` y la
  exportación seguirá marcando el documento como no verificado hasta que cambien.
- **El restrictor fijo es el símbolo más débil**: su garganta se dibujó sin la norma a
  mano y así se dice en su revisión.
- **Ninguna revisión tiene segunda firma**: es un pase automático, no una revisión humana.
- La edición vigente de la norma de numeración de puertos sigue sin confirmarse
  (pregunta 2 de §13 de `docs/retomar.md`).
- Puntos de unión en cruces de tres líneas: llegan con las tuberías (M4).

### Armazón de la aplicación (M1) y scripts de desarrollo — 2026-11-06

La ventana deja de ser una barra de botones: tiene cinta, barra lateral de catálogo,
hoja, barra de estado y la paleta del escritorio, con teclado completo.

- **Crate `punchi-fluid-theme`**, hoja de dependencias que no usa ningún otro crate
  nuestro. Lee `kdeglobals` (`[Colors:*]`, `[General] font`, `[KDE]
  AnimationDurationFactor`) y el portal `org.freedesktop.appearance color-scheme` por
  `zbus` (5.13.2, la última compatible con el mínimo declarado 1.85), con lectura y
  suscripción en un hilo aparte: un bus ausente o lento no retrasa el primer fotograma.
  Sin preferences del portal se deduce el esquema del brillo del fondo.
- **Tokens semánticos**: `Surface`, `Text`, `Accent`, `GridMinor`, `WorkLine`, etc. Una
  paleta por esquema, con los colores del escritorio y una tabla de respaldo. Los pares
  texto/fondo se **reparan** hasta 4.5:1 y las líneas hasta 3:1, de modo que un tema de
  usuario ilegible sigue siendo legible; la retícula queda por debajo a propósito.
- **Regla del color, con prueba**: `Color32::` fuera de `punchi-fluid-theme` falla
  `tests/no_colour_literals.rs`. El lienzo, el panel de catálogo y la barra de estado
  piden tokens; las líneas de presión y de suministro salen de la paleta.
- **Cinta** (`ui/src/ribbon.rs`): pestañas Archivo, Editar, Ver y Simular; transporte
  siempre visible; por debajo de 1100 px se colapsa a un menú; cada acción es un
  `Action` con etiqueta, atajo y motivo cuando todavía no existe («llega con M2/M3»),
  así que nada queda deshabilitado sin explicación.
- **Barra lateral** (`ui/src/sidebar.rs`): grupos por categoría, búsqueda por nombre,
  `id` y nombre español, miniatura dibujada con el símbolo de la propia definición,
  arrastre con el `id` como carga y pie con la carpeta y los archivos omitidos.
- **Hoja y barra de estado** (`ui/src/shell.rs`): retícula doble, valores de tiempo,
  presión, posición, velocidad y aire consumido, definición, modo y último mensaje.
  Modos Edición y Simulación visibles; en simulación un clic acciona la válvula.
- **Escritorio** (`app.rs`): `accesskit` activado, tema aplicado una vez y recargado
  cuando cambia el portal o con `Ctrl+Shift+T`, ventana mínima 900×600, panel de ayuda
  con todos los atajos (`F1`). `PUNCHI_FLUID_THEME=light|dark` fija la paleta para
  capturas y demostraciones.
- **Scripts de desarrollo**: `scripts-dev/run.sh` (compila solo si hace falta y ejecuta
  el binario; `--demo`, `--light`, `--dark`, `--release`, `--no-build`) y
  `scripts-dev/test.sh` (compila las pruebas una vez y las ejecuta directamente;
  acepta filtros, `-p` y `--no-build`).

#### Verificación

- `./scripts-dev/check.sh` en verde, 0 avisos de clippy.
- **69 pruebas**: `core` 7, `catalog` 7, `file` 3, `solver` 18, `theme` 17,
  `contrast` 4, `no_colour_literals` 1, `ui` 10, `click` 2. Las dos pruebas de clic
  siguen pasando sin cambios.
- Capturas del 2026-11-06 en esquema claro y oscuro con los scripts
  (`run.sh --dark`, `run.sh --light`), ventana de 1280×800.
- El cambio de tema del portal está cableado y probado en unidad; el cambio en vivo
  desde Preferencias de Plasma **no se ha medido** en esta sesión (no se tocó la
  configuración del propietario), solo el camino forzado por variable de entorno.

#### Limitaciones conocidas

- Los iconos simbólicos de Breeze todavía no se cargan: los botones de acción usan
  texto. La familia tipográfica del escritorio se lee pero no se carga; el tamaño sí.
- Los textos de la interfaz siguen en literales ingleses hasta el pase a Fluent (M6).
- El modo inicial es Simulación, porque el editor no existe; cambia a Edición en M3.

### Nota de continuidad para retomar el proyecto — 2026-11-06

`docs/retomar.md`: un documento único, en español, para abrir una ventana nueva sin
contexto y continuar sin preguntar nada.

- **§0 en cinco minutos**: los dos comandos que devuelven al proyecto y qué documento
  leer según lo que se quiera hacer.
- **§2 estado real** con evidencia por área y el total de pruebas desglosado
  (`core` 7, `catalog` 7, `file` 3, `solver` 18, `ui` 2 = 37), más el recordatorio de
  que no hay repositorio git.
- **§3 entorno verificado** (Fedora 44, Plasma 6.7.5, Wayland, `kdeglobals` con sus
  valores, portal de color, ausencia de herramientas de inyección de clic).
- **§4 mapa del repositorio** con el recuento de líneas de cada fichero clave y qué
  documento contiene qué, para no duplicar información.
- **§5 bitácora paso a paso** de lo construido, con las decisiones y **cómo se comprobó
  cada una**, incluidas las trampas encontradas (constante `R` equivocada en la
  verificación, círculo invisible por conversión de coordenadas, etiquetas solapadas,
  orden de cajas de la válvula).
- **§6 verificación reproducible**: comandos y salida esperada, incluida la prueba del
  catálogo de usuario.
- **§7 el modelo físico con sus números** (constantes, ecuaciones, carrera ≈ 0.13 s,
  presión final ≈ 2.5 bar) para detectar regresiones sin deducirlo del código.
- **§8 el catálogo**, **§9 la simbología con la tabla de carencias frente a ISO 1219**,
  **§10 la interfaz**, **§11 el orden de trabajo**, **§12 el registro de decisiones**,
  **§13 las siete preguntas abiertas con valor por defecto**, **§14 las reglas**,
  **§15 lo que no está hecho** y **§16 el texto exacto para pedir la continuación**.
- Índices actualizados: `docs/README.md`, ambos README y `AGENTS.md` señalan
  `docs/retomar.md` como punto de entrada cuando se retoma sin contexto.

#### Limitaciones conocidas

- Es documentación: no cambia el código. Sigue sin haber repositorio git ni exportación.

### Orden de trabajo para continuar — 2026-11-06

Se documenta en `docs/` cómo retomar el proyecto y qué se construye a continuación, para
que la siguiente sesión no dependa de la memoria de nadie.

#### `docs/next-steps.md`

- **Estado real** del repositorio, área por área, con la evidencia que respalda cada
  fila y marcando sin adornos lo que es solo especificación (armazón, exportación,
  conformidad de símbolos, editor).
- **Orden de trabajo razonado**: M1 (armazón y tema) → M2 (un solo dibujo y
  exportaciones) → M1.5 (conformidad de símbolos). El motivo es que la capa `Surface`
  de M2 es la que necesita el renderizador conforme, así que M2 ya incluye números de
  puerto, clases de línea y marcas de bloqueo.
- **Pasos ejecutables**: para cada hito, los ficheros exactos a crear o tocar, la firma
  pública de las piezas nuevas (`Theme`/`Token`, `Surface`/`Stroke`/`LineClass`,
  `Page`/`PaperSize`/`TitleBlock`), las dependencias (con `zbus` en lugar de `dbus`
  para no arrastrar `openssl-sys`, que `deny.toml` prohíbe), las trampas conocidas, los
  criterios de aceptación y las pruebas que los demuestran.
- **Preguntas para el propietario** con su valor por defecto si no hay respuesta: `git
  init`, edición vigente del estándar de identificación de puertos, umbral de colapso
  de la cinta, valores por defecto de exportación, contenido del bloque de
  identificación y comportamiento de la barra lateral.
- **Lista de comprobación de sesión** para no empezar sin la puerta verde ni terminar sin
  evidencia declarada.

#### Índices

- `docs/README.md` explica ahora el orden de lectura y señala `next-steps.md` como punto
  de entrada; README en ambos idiomas y `AGENTS.md` lo enlazan.

#### Limitaciones conocidas

- Sigue sin haber repositorio git: la primera acción pendiente es una decisión del
  propietario, y el documento lo dice en su primera sección.
- Las estimaciones son de pasos, no de fechas: no hay calendario que prometer.

### Diseño de interfaz, simbología normativa y estándares — 2026-11-06

Antes de escribir la cáscara de la aplicación queda escrito qué se va a construir y
con qué criterio se acepta, incluida la exigencia de que los símbolos cumplan la norma
vigente.

#### Simbología

- Nuevo `docs/symbology.md`: contrato normativo con **ISO 1219-1:2012** (símbolos),
  ISO 1219-2:2012 (esquemas) e ISO 1219-3:2016 (módulos de símbolos), verificados
  contra el catálogo de ISO el 2026-11-06. Requisitos propios: un cuadrado igual por
  posición, flechas en el sentido del flujo, marca de bloqueo en puertos cerrados,
  numeración de puertos tomada de la definición, muelle y accionamiento en sus
  extremos, clases de línea para trabajo, pilotaje y drenaje, y puntos de unión solo
  donde concurren tres líneas.
- Garantías del renderizador y lista de revisión por símbolo, con la parte
  automatizable asignada a `symbol lint` (M1.5) y la parte visual a una revisión
  registrada con capturas en claro y oscuro.
- **Estado declarado sin adornos:** los cuatro símbolos de fábrica son aproximaciones
  y no están verificados; hasta que exista la revisión, la exportación marca el
  documento como no verificado. La tabla de carencias conocidas está en el documento.
- `assets/symbols/` deja de ser el almacén de símbolos (ahora son definiciones del
  catálogo) y pasa a guardar el material de revisión. Sin recursos de terceros: nada
  trazado de otro programa ni del texto de la norma.

#### Interfaz

- `docs/ui-ux.md`: cinta superior con cuatro pestañas y transporte siempre visible,
  barra lateral que **es** el catálogo (miniaturas dibujadas con el símbolo de cada
  definición, búsqueda, arrastre a la hoja), hoja con rejilla y dos modos visibles
  (edición y simulación), barra de estado.
- Tema del escritorio: `kdeglobals` (`[Colors:*]`, acento, tipografía) más el portal
  `org.freedesktop.appearance color-scheme`, con cambio en vivo; iconos Breeze
  `-symbolic` teñidos y cacheados; capa semántica de tokens y prohibición de colores
  literales en los widgets.
- Accesibilidad y lenguaje: `accesskit` (feature disponible en eframe 0.37), contraste
  medido, teclado completo, movimiento reducido según `AnimationDurationFactor`, y
  cadenas en Fluent.

#### Plan y estándares

- `docs/plan.md`: hitos M1 (armazón), M1.5 (conformidad de símbolos), M2 (un solo
  dibujo y dos exportaciones), M3 (documento con deshacer), M4 (red y solver real),
  M5 (instrumentos) y M6 (idioma y empaquetado), cada uno con criterios de aceptación
  verificables y el nivel de evidencia que exige. Incluye la prueba de modularidad que
  justifica el proyecto: una válvula 5/2 que funciona **solo como archivo**.
- `docs/engineering.md`: mapa de crates y dirección de dependencias, puntos de
  extensión, presupuesto de memoria con verificación mediante un asignador que cuenta
  bytes y una prueba de cero asignaciones en el bucle de simulación, presupuesto de
  rendimiento con método de medición, modelo de amenazas con controles (archivos no
  confiables, exportación, metadatos, cadena de suministro, límites de píxeles),
  pirámide de validación y prácticas de GitHub.

#### Repositorio

- `SECURITY.md`, `CONTRIBUTING.md` y `CODE_OF_CONDUCT.md` (adopción por referencia del
  Contributor Covenant 2.1, sin copiar su texto).
- `.github/`: `CODEOWNERS`, plantilla de pull request con la lista de comprobación,
  formularios de incidencia (bug y feature, con referencia normativa obligatoria para
  símbolos), `config.yml`, `dependabot.yml` y flujos de trabajo `audit.yml` (cargo-deny
  y cargo-audit) y `release.yml` (etiqueta → binarios, checksums y borrador de
  release). `ci.yml` gana el trabajo de MSRV 1.85.
- `deny.toml` con lista explícita de licencias compatibles con GPL-3.0-or-later,
  `.editorconfig` y `clippy.toml`.
- README en ambos idiomas: interfaz, «la simbología sigue ISO 1219» con su estado real,
  hitos e índice de documentos.

#### Limitaciones conocidas

- Nada de esta entrada está implementado todavía: son especificación y andamiaje. El
  código sigue siendo la rebanada vertical de `0.2`.
- La comprobación de la edición vigente del estándar de identificación de puertos queda
  pendiente en la revisión de M1.5; desde esta máquina solo se pudieron confirmar
  ISO 1219-1/2/3.
- `symbol lint`, el renderizador con números de puerto y clases de línea, y la
  exportación PNG/PDF están planificados, no escritos.

### Rebanada vertical y catálogo de componentes — 2026-11-06

El simulador deja de tener las válvulas escritas en Rust y pasa a leerlas de una
carpeta de archivos; sobre ese catálogo corre el primer circuito completo, con
física, símbolos dibujados y clic en la válvula.

#### Catálogo de componentes (`punchi-fluid-catalog`)

- Nuevo crate: define el formato de las definiciones (TOML, `schema = 1`), las valida
  y las carga desde dos orígenes, con el usuario por encima del de fábrica.
- Una definición declara puertos con rol semántico (`supply`, `working`, `exhaust`,
  `mechanical`), comportamiento (`supply`, `throttle`, `cylinder`, `valve`) y el
  símbolo como primitivas normalizadas (`line`, `polyline`, `polygon`, `rect`,
  `circle`, `arc`).
- Una válvula es una lista de posiciones y, en cada una, los caminos abiertos como
  `(puerto, puerto, conductancia)`; el solver los clasifica por rol, no por número.
- Un archivo inválido se omite y se reporta; nunca impide arrancar. Un archivo con
  `disabled = true` retira una definición de fábrica.
- `assets/catalog/`: `supply.workshop`, `valve.3_2.spring_return`,
  `cylinder.single_acting_20_100` y `throttle.fixed`, compilados con `include_str!` y
  documentados en `assets/catalog/README.md`.
- `PUNCHI_FLUID_CATALOG_DIR` permite apuntar a otra carpeta mientras se edita.

#### Física (`punchi-fluid-solver`)

- `orifice` reescrito según ISO 6358 de verdad: conductancia sónica `C`, relación
  crítica `b`, rama crítica y elipse subcrítica, con el cálculo a mano como prueba.
- Nuevo módulo `pneumatic`: un volumen de gas con la **masa como estado integrado**
  (la presión se deriva con el gas ideal, así que no puede ser negativa y el balance
  de masa es exacto), válvula conmutando entre presurizar y purgar, y pistón con
  muelle, amortiguamiento, rozamiento de Coulomb con adherencia y topes de carrera.
- Subpaso interno de 2 µs con cota de subpasos por llamada: el paso que pide la
  interfaz no puede desestabilizar ni bloquear un fotograma.
- Pruebas: caudal crítico contra el cálculo a mano, tiempo de carrera acotado entre
  los dos límites de flujo, balance de masa a 1e-9, purga con retorno por muelle y
  10⁵ pasos sin NaN ni presiones negativas.

#### Interfaz (`punchi-fluid-ui`, `punchi-fluid-desktop`)

- El lienzo dibuja los símbolos desde las definiciones, resalta la posición activa de
  la válvula, colorea la línea de trabajo según la presión y coloca el pistón, el
  vástago y el muelle según la carrera.
- Clic sobre la válvula para cambiar de posición (con prueba sin ventana que comprueba
  que solo la válvula responde), barra con Play/Pausa, paso de 10 ms, Reset, actuador,
  velocidad (0,05× a 1×) y barra de estado con tiempo, presión, posición, velocidad y
  aire consumido.
- Panel de catálogo: definiciones cargadas con su origen, carpeta en uso, recarga en
  caliente y el motivo de cada archivo omitido.
- `PUNCHI_FLUID_DEMO=1` arranca accionada y en marcha, para capturas y demostraciones.

#### Documentación

- `assets/catalog/README.md`: cómo añadir, modificar y borrar un componente, con la
  referencia de campos y la lista de rechazos del validador.
- ADR `docs/adr/0002-components-are-files.md` con la decisión, sus costes y las
  alternativas descartadas; `AGENTS.md` incorpora las reglas del catálogo.
- README en ambos idiomas: qué existe, cómo se ejecuta, catálogo y hoja de ruta.

#### Limitaciones conocidas

- El circuito sigue siendo uno fijo: el lienzo coloca tres componentes en posiciones
  conocidas y no hay documento que los describa. Falta el editor (v0.3).
- La fachada `Solver` sobre un `Circuit` genérico mantiene el primer paso de
  relleno; el circuito real lo resuelve `pneumatic`, que se conectará a la red en v0.4.
- `punchi-fluid-core` todavía enumera los tipos de componente del primer formato
  (`ComponentKind`): el catálogo es la fuente de verdad del comportamiento y del
  símbolo, y el enum queda como capa de instancia hasta que v0.3 haga que un
  componente colocado referencie un `id` del catálogo.
- Solo hay modelo neumático, sin hidráulica, traducciones ni empaquetado.
- El clic está cubierto por una prueba de egui sin ventana; no se ha podido inyectar
  un clic real en la sesión.

### Esqueleto del proyecto — 2026-11-06

Nace `punchi-fluid-lab` como workspace de Rust con seis crates y la separación
entre dominio, física, formato, interfaz y aplicación. Sin publicar.

#### Dominio y formato

- `punchi-fluid-core`: componentes (`Supply`, `CylinderSingleActing`, `Valve3_2`,
  `Throttle`, `Reservoir`) con sus puertos, `Circuit` con validación de conexiones, y
  unidades tipadas `Pressure`, `Temperature` y `Volume` en SI con constructores que
  rechazan valores imposibles.
- `punchi-fluid-file`: documento JSON con `FORMAT_VERSION`, ida y vuelta de un
  circuito, rechazo explícito de versiones futuras y parseo encapsulado
  (`Document::parse`), para que el CLI no dependa de `serde_json`.

#### Física

- `punchi-fluid-solver`: fachada `Solver` con `load`/`step`/`unload`, validación de
  circuito sin suministro y de paso de tiempo inválido, y un primer paso que reporta
  la presión de suministro en un volumen concentrado.
- Módulo `orifice` con el modelo ISO 6358: rama crítica y subcrítica, entradas
  inválidas devueltas como `None` en lugar de producir NaN.

#### Aplicación

- `punchi-fluid-ui`: lienzo egui con rejilla y marcadores de componente.
- `punchi-fluid-desktop`: ventana con barra de herramientas («Advance 1 ms»), lienzo
  y barra de estado, sobre la API de eframe 0.37 (`App::ui` y `egui::Panel`).
- `punchi-fluid-cli`: `punchi-fluid validate|simulate`.

#### Herramientas y documentación

- `scripts-dev/check.sh`: `cargo fmt --check`, `clippy -D warnings`, pruebas y prueba
  de humo del CLI sobre un circuito temporal.
- Lints del workspace: `unsafe_code` prohibido, `unwrap`/`expect`/`panic` denegados
  fuera de pruebas.
- `AGENTS.md` con las reglas de capas, unidades y física; ADR
  `docs/adr/0001-language-and-layout.md` con la decisión de Rust y su coste.
- README en inglés y español, notas de empaquetado, plan de traducciones y contrato
  del set de símbolos.

#### Limitaciones conocidas

- El solver simula un único volumen y no integra todavía el modelo de orificio en la
  evolución de presiones.
- No hay editor: el lienzo dibuja marcadores en posiciones deterministas.
- Sin modelo hidráulico, sin traducciones y sin empaquetado.
- La ventana no se ha validado visualmente en esta sesión; el verde de
  `check.sh` no lo demuestra.
