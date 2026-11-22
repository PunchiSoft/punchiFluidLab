# Changelog

Versión declarada: `0.1.0`, **sin publicar**. El trabajo pendiente se acumula bajo
`## Unreleased` en subsecciones fechadas; la versión cambia solo al preparar una
publicación.

## Unreleased

### La selección es múltiple y se alinea y duplica — 2026-11-22

- **Mayús+clic añade o quita de la selección; Ctrl+A selecciona todo el circuito.** El
  arrastre y Borrar actúan sobre la selección entera, con una sola entrada de deshacer.
- **Duplicar** (la cinta y Ctrl+D) copia la selección desplazada 10 mm como un solo cambio y
  deja las copias seleccionadas; **Alinear** (Izquierda/Derecha/Arriba/Abajo) la alinea con
  el borde más lejano como una sola entrada.
- La cinta enciende Borrar/Girar/Reflejar/Duplicar con una colocación, Alinear con dos, y
  mantiene apagados los mandos que aún no existen (portapapeles, repartir, orientación).
- **Pruebas**: el clic con Mayús alterna la selección y Ctrl+A la toma entera; duplicar deja
  un paso de historial; alinear usa el borde más lejano y un deshacer lo revierte.

### La ventana gana el panel del catálogo — 2026-11-22

- **La tercera región de la ventana**: el panel izquierdo lista el catálogo como paleta de
  azulejos —el símbolo propio, el nombre en español y el id— agrupado por categoría, con el
  número de componentes, la búsqueda por nombre, nombre español e id y las categorías
  plegables. El símbolo lo dibuja el mismo renderizador, con la vista previa del panel (sin
  los puntos de conexión que la hoja añade).
- **Clic en un azulejo añade el componente al centro de la hoja** como una sola operación
  de deshacer, lo deja seleccionado y lo dice en la barra de estado; Ctrl+F lleva a la
  búsqueda. Las definiciones de cierre de puerto (tapón y silenciador) no son entradas de
  la paleta (ADR 0033): se ajustan a un puerto, no se colocan.
- **Pruebas**: la paleta esconde los cierres de puerto; la búsqueda casa nombre, nombre
  español e id; añadir coloca en el centro de la hoja, selecciona y deshace en un paso.

### La ventana borra, gira, refleja y deshace — 2026-11-22

- **Sobre la selección**: Borrar la quita de la hoja, Girar la cuarta parte y Reflejar la
  invierten, cada gesto como **una sola operación de deshacer** (`remove_many`/`turn_many`),
  y Deshacer/Rehacer recorren el historial acotado del documento. Atajos: Supr, Ctrl+Z y
  Ctrl+Y.
- **La cinta enciende lo que existe**: Borrar/Girar/Reflejar solo con selección,
  Deshacer/Rehacer según el historial, y Duplicar/Copiar/Pegar siguen apagados hasta que el
  portapapeles llegue.
- **Pruebas**: borrar deja vacía la hoja y el deshacer la repone; girar y reflejar cambian
  la orientación y el deshacer la devuelve; la cinta sigue a la selección y al historial.

### La ventana selecciona y mueve en la hoja — 2026-11-22

- **Clic en la hoja selecciona la colocación bajo el puntero** (la de arriba manda, solo en
  el circuito activo) y el arrastre con el botón izquierdo la mueve con vista previa en
  vivo; al soltar, el movimiento entra como **una sola operación de deshacer**
  (`Document::move_many`) y la colocación seleccionada se marca con el acento del tema.
- **La primera edición real de la ventana Rust**: el documento se modifica desde el gesto,
  sin ABI ni Qt, y la barra de estado dice qué se seleccionó o movió.
- **Dos pruebas**: el clic selecciona y deselecciona por el hit del documento; un arrastre
  mueve la selección y deja un solo paso de historial.

### La ventana guarda el documento que abrió — 2026-11-22

- **Guardar escribe el documento en el fichero del que se abrió**, con Ctrl+S o el botón de
  la cinta, por `punchi-fluid-session::save`: el documento → la hoja → el mismo texto que
  `load` lee, así que lo guardado se vuelve a abrir. Sin ruta (documento nuevo) lo dice en
  la barra de estado en vez de inventarse un destino; «Guardar como» espera al diálogo.
- **La cinta no finge**: Guardar se enciende solo cuando hay ruta, y Abrir, Nuevo, Guardar
  como y Exportar quedan apagados hasta que su diálogo exista.
- **Dos pruebas**: la sesión guarda y reabre un documento con una colocación; la ventana
  escribe el documento que se le dio, el texto vuelve a ser una hoja válida y la barra de
  estado lo dice.

### El panel muestra el símbolo, el lienzo los puntos de conexión — 2026-11-22

- **La vista previa del catálogo deja de dibujar los círculos de conexión**: el panel
  izquierdo muestra el símbolo tal como es; el lienzo sigue dibujando el punto de conexión
  de cada puerto, que es donde llega la manguera. La distinción está en el renderizador
  (`draw_placed` para la hoja, `draw_placed_preview` para el panel) y la cubre una prueba
  que cuenta los círculos de un mismo símbolo en las dos vistas.
- Sin cambio en el dibujo del lienzo ni en las capturas de revisión de los símbolos.

### La simbología es normativa, no nuestra — 2026-11-22

- **La simbología y las convenciones de circuito son las de la norma**, una sola forma, la
  misma en cualquier aplicación: no hay variante ni transformación de un símbolo nuestra y
  no se cambia en base a una aplicación de referencia, una captura o una imagen. La regla
  entra en la skill de simbología, corrige las skills de referencia (la app padre es
  referencia de comportamiento y flujo, no autoridad de simbología) y queda en AGENTS.md y
  en el contrato: **el panel muestra el símbolo; el lienzo dibuja sus puntos de conexión**.
- Sin cambio de código: es una regla de trabajo, no una edición de símbolos.


### La ventana abre el documento que le dan — 2026-11-22

- **La ventana Rust abre un `.pfluid` desde la línea de comandos** (o arranca con un
  documento nuevo) por el crate de sesión, y **dibuja el circuito** de la hoja activa: las
  colocaciones y los tubos del documento, junto a la hoja, el cajetín y la vista ya
  existentes. Un fichero que no se puede abrir no tumba la ventana: se avisa en la barra de
  estado y se abre un documento nuevo.
- **La sesión estrena su primer uso fuera del ABI**: `punchi-fluid-session::load`; la
  ventana depende ahora de `session`, `catalog` y `file`.
- **Dos pruebas nuevas** del escenario estático: la escena de dos válvulas con un tubo, y
  que el circuito produce llamadas de dibujo. Deuda: los estados vivos —válvulas, lecturas,
  pulsos— llegan con la ruta del fotograma de la sesión.

### La sesión sale del ABI — 2026-11-22

- **Nuevo crate `punchi-fluid-session`**: la conversión fichero↔documento (el formato de
  hoja y el documento vivo) sale de `punchi-fluid-ffi` a un crate propio que el ABI usa y
  la ventana Rust usará para abrir y guardar. Sin cambio de comportamiento: las 92 pruebas
  del ABI siguen pasando y el crate lleva dos pruebas de ida y vuelta (un documento nuevo y
  uno con una colocación).
- **Grafo**: `session → doc, draw, file`; `ffi` depende de la sesión. La ventana podrá
  abrir y guardar sin hablar el ABI, y la extracción del estado de edición sigue en el
  siguiente corte.

### La ventana Rust mueve la hoja — 2026-11-22

- **La hoja se amplía y se desplaza como en la ventana Qt**: Ctrl+rueda amplía sobre el
  puntero, la rueda y el botón central desplazan, las flechas desplazan desde el teclado y
  Ctrl+0 vuelve al encaje, con el rango 0,4–6× y el paso 1,25 de la sesión Qt. El
  desplazamiento está acotado al sobrante de la hoja y se borra al volver al encaje.
- **Los comandos de la pestaña Ver ya actúan**: Aumentar, Disminuir, Tamaño original y
  Restablecer vista mueven la vista y no el documento; la barra de estado lee la
  magnificación en % y en px/mm.
- **Cinco pruebas nuevas de la vista** sobre las 11 anteriores: encaje centrado, ampliación
  sobre el puntero, tope del desplazamiento, vuelta al encaje y rango de los comandos.

### La ventana Rust gana la cinta — 2026-11-22

- **La cinta del prototipo entra en la ventana**: las cinco pestañas, los grupos con botones
  grandes y pequeños, los menús, las galerías de Márgenes y Conducto, el *launcher* de
  Configurar página, el colapso de grupos a su título a 1366 y los atajos del transporte
  (F9/F8/F10/F5), con el estado que deshabilita Pausa y Detener hasta Iniciar. El comando
  elegido se anota en la barra de estado hasta que el corte de la sesión cablee las acciones.
- **Colores y tamaños salen del tema del escritorio**: la cinta no nombra un color; el texto
  usa los tamaños `small`, `body` y `label` del tema.
- **Los 55 iconos SVG van compilados en el binario** con `egui_extras` y su cargador SVG
  (resvg/usvg, MPL-2.0 ya permitida por la allowlist): una ventana instalada no necesita el
  directorio de assets al lado.
- **Cuatro pruebas headless** con `egui_kittest`: las pestañas llegan a sus comandos, el
  transporte sigue su estado, la galería de Márgenes abre sus tarjetas y Alinear colapsa a su
  título. Deuda: la geometría de la cinta sigue siendo la del prototipo para el tamaño mínimo
  fijo, y los comandos reales llegan con la sesión.

### La ventana toma su tema del escritorio — 2026-11-22

- **La ventana Rust lee el tema del escritorio**, no de literales: `kdeglobals` da los
  colores de ventana, vista, botón, selección y enlace; el adaptador deriva los tokens con
  las mismas reglas que la ventana Qt (contraste mínimo 4.5 para el texto y 3.0 para el
  acento y las líneas, dos planos, acento suavizado, radios y tamaños de texto desde la
  unidad de la fuente). Sin `kdeglobals` arranca con la paleta clara de respaldo.
- **La fuente también sale del escritorio**: la familia declarada en `[General] font=`, con
  la sans del sistema como respaldo; no se empaqueta ninguna fuente.
- **El movimiento respeta `[KDE] AnimationDurationFactor`**: 0 desactiva la animación; la
  barra de estado usa el cuerpo de texto y la tinta del tema.
- La entrada BSL-1.0 del portapapeles de Windows sigue siendo la única licencia añadida; no
  entra ninguna de fuente. Deuda de este corte: el portal
  `org.freedesktop.appearance color-scheme` para escritorios que no son KDE y la relectura
  en caliente.

### La ventana Rust: egui y la primera hoja — 2026-11-22

- **El toolkit es egui** ([ADR 0036](docs/adr/0036-the-toolkit-is-egui.md)): la ventana
  usa eframe y egui. Los cinco criterios de ADR 0035 quedan medidos: 200 componentes a
  88–89 fps con 0,67 ms de dibujo y 5,15 ms de teselado, el IME de toolkit, el árbol de
  accesibilidad (con el nombre del `frame` como deuda de eframe), los 55 iconos SVG y la
  página impresa idéntica a la del producto (470 089 bytes, tres escritores). Slint se midió
  y no se elige: 29 fps con texto y 36 sin él en el mismo fotograma.
- **El workspace gana `apps/punchi-fluid`** (paquete `punchi-fluid-window`, binario
  `punchi-fluid`): abre a 1366 × 768, dibuja la hoja de un documento nuevo desde
  `punchi-fluid-export` y muestra el encaje y el tiempo de dibujo en la barra de estado. Sin
  ABI: enlaza `doc`, `draw` y `export` directamente. La cinta, el panel lateral y los
  comandos llegan en los siguientes cortes.
- **La ventana declara Rust 1.89**, porque el árbol de eframe lo exige; los crates del
  núcleo conservan el suelo de 1.85 y el trabajo de versión mínima verifica los dos. El
  toolchain fijado (1.98.1) no cambia.
- **La allowlist de licencias acepta solo BSL-1.0**, la del portapapeles de Windows que
  arrastra `arboard`; es permisiva y compatible con GPL, y esos crates no se compilan en
  Linux. **La ventana no empaqueta ninguna fuente**: egui arranca sin fuentes por defecto y
  la ventana carga la del escritorio al abrirse, como manda la regla de interfaz; no hay
  licencia de fuente que revisar ni fuente de terceros que distribuir.
- **La cinta del spike queda como referencia** para las fases siguientes: cinco pestañas,
  colapso por grupo, galerías, atajos, transporte con estado y pruebas `egui_kittest`.

### Decisión: la ventana pasa a Rust — 2026-11-21

- **La aplicación será un solo binario Rust.** La [ADR 0035](docs/adr/0035-the-window-is-rust.md)
  sustituye a la ADR 0005: la ventana, sus regiones y su entrada son crates Rust,
  `punchi-fluid-ffi` y `apps/punchi-fluid-qt` se retiran y no se publica ABI en C. El dibujo
  es el mismo de siempre —`punchi-fluid-draw` detrás de `Surface`— y el PNG, el PDF y el SVG
  siguen siendo Rust.
- **La interfaz deja de ser adaptable y pasa a ser fija**, al tamaño mínimo declarado en
  `ui-ux.md` y a una escala; las reglas de plegado y los nodos que las pagaron (P-027,
  P-028) quedan obsoletos.
- **La iconografía es un asunto aparte de la simbología.** Los iconos de acción serán
  archivos SVG en `assets/icons`; la simbología ISO 1219, `assets/symbols` y su revisión no
  se tocan.
- **El toolkit se elige con evidencia**, en un spike con cinco criterios —texto en sitio con
  IME, accesibilidad y orden de teclado, un icono a 16 px teñido por el tema, un fotograma de
  200 componentes y una página impresa desde el PDF de Rust— y en una ADR posterior.

### Documento: abrir y guardar no siguen trampas — 2026-11-21

- **Abrir exige un fichero regular y una lectura acotada**: el tamaño que declara el sistema
  ya no se da por bueno. Se comprueba que es un fichero normal antes de abrirlo y otra vez
  por el descriptor que se lee, y la lectura se corta en `MAX_BYTES + 1` bytes, así que un
  FIFO, `/dev/zero` o un fichero que crece mientras se lee se rechazan con `NotRegular` o
  `TooLarge` en vez de agotar la memoria o dejar la ventana esperando.
- **Guardar crea el temporal en exclusiva**: `create_new` sobre `<documento>.pfluid.tmp`,
  desenlazando antes un temporal huérfano y sin seguir un enlace simbólico plantado;
  conserva los permisos del documento que sustituye (un `0600` sigue siendo `0600`),
  fuerza el fichero con `sync_all` antes del `rename` y borra el temporal cuando algo
  falla. Pruebas `a_symlink_at_the_temporary_name_does_not_redirect_the_save`,
  `saving_keeps_the_permissions_of_the_document_it_replaces`,
  `a_failed_rename_leaves_no_temporary_behind`, `a_device_is_refused_instead_of_read` y
  `a_file_over_the_limit_is_refused_before_it_is_read`. Nodo P-038.

### Escape: el chorro sale de la boca del silenciador — 2026-11-21

- **Con un silenciador en el puerto de escape, las puntas del aire salen pasada su caja**,
  no de detrás del símbolo: el cierre es la boca y el aire lo atraviesa. Un puerto tapado
  sigue sin dibujar nada y uno libre sigue saliendo del puerto. Prueba
  `a_silencer_owns_the_mouth_of_the_jet`; el caudal del escape y el chorro entraron en #161.
  Nodo P-037.

### Cinta: el transporte sigue visible al plegarse — 2026-11-21

- **Con la cinta plegada (≤1280 px), la fila de pestañas lleva Iniciar/Pausa, Paso y Detener**
  junto al botón Menú. Hasta ahora el transporte vivía solo dentro del menú, y con un
  circuito en la hoja no había forma evidente de arrancar la simulación: `ui-ux.md` dice que
  el transporte está siempre visible. Play y pausa comparten un control que cambia etiqueta,
  icono y acción según la sesión; el menú conserva el resto (Restablecer, Hasta cambio y las
  velocidades).
- **`--check-layout` lo vigila**: a 1280 comprueba que el grupo está visible y que el menú
  sigue alcanzando Conducto y Diámetro; si el grupo desaparece o se oculta, falla. Captura a
  1200×800 con Menú · Iniciar · Paso · Detener. Nodo P-036.

### Caudal: el aire se ve moverse por los tubos — 2026-11-21

- **La hoja dibuja el aire que el modelo mueve como una corriente de puntas**: varias flechas
  sólidas espaciadas a lo largo del tubo, cada una sobre un trazo corto del color del propio
  tubo dibujado 1,7 veces más grueso que la línea para que se distinga. Mientras una sale por
  el sumidero, otra entra por la fuente, así que el tubo nunca queda vacío con el aire en
  marcha; la retícula es exacta y las marcas no se agrupan. El signo del caudal invierte el
  sentido y un tubo sin caudal no dibuja nada: un final cerrado con presión se queda quieto.
  Una marca única se descartó con la grabación del 2026-11-21, que la cazó saltando del
  sumidero a la fuente y dejando el tubo en blanco entre medias. La decisión está en la
  [ADR 0034](docs/adr/0034-the-sheet-animates-the-models-flow.md).
- **La lectura es el caudal neto del fotograma, no el signo de un subpaso**: un tubo que
  termina en un puerto cerrado oscilaba entre +7,6 y −7,6 l/min en subpasos consecutivos, y
  como el dibujo leía el último, la corriente de flechas cambiaba de sentido a 500 Hz y
  parecía que el aire retrocedía. El solver acumula los subpasos de la llamada y publica la
  media: en reposo la lectura queda en ±0,004 l/min, por debajo del umbral del dibujo, y con
  el aire en marcha conserva su signo y su magnitud. Prueba nueva
  `a_dead_end_line_does_not_report_a_reversing_flow`.
- **El aire se mueve con el caudal**: la velocidad de la corriente sigue `\|flow\|` entre 6 y
  45 mm/s, así que un caudal pequeño va despacio y uno fuerte corre; un tubo largo tarda más
  en cruzarse, como debe. La escala es de legibilidad, no la velocidad física del aire, y el
  orden y el sentido son los del modelo.
- **El reloj es del armazón**: la fase es estado de vista —arrancar la pone a cero, el tick
  de 16 ms la avanza mientras el modelo corre, pausar la congela, un paso suma sus segundos y
  parar o reiniciar la limpian— y nunca entra en el documento, el formato, el deshacer ni la
  marca de modificado.
- **Movimiento reducido y papel congelan una punta**: con `AnimationDurationFactor = 0`, y en
  el PDF, PNG y SVG, cada tubo con aire lleva una sola punta quieta en la mitad del trayecto
  y sin trazo; sin modelo vivo la página no inventa una dirección.
- **ABI 13**: `pf_session_build_frame` recibe `phase_s` y `motion` —y rechaza cualquier valor
  de `motion` que no sea 0 o 1 en vez de adivinar—, `Pipe` lleva `flow_l_min`, `phase_s` y
  `animate`, y la exportación pasa los caudales del último fotograma con `motion = 0`.
- **Evidencia**: pruebas de solver (`a_pipe_reports_the_flow_through_it`,
  `a_zero_flow_pipe_reports_zero_after_a_step`, `a_skipped_pipe_reports_no_flow`), de dibujo
  (`the_stream_never_leaves_the_hose_empty`,
  `the_marks_are_ordered_inside_the_hose_and_do_not_overlap`,
  `the_stream_moves_with_the_phase`, `the_sign_reverses_the_stream`,
  `a_still_hose_draws_nothing`, `reduced_motion_keeps_one_arrowhead_in_the_middle`,
  `a_short_route_draws_nothing`, `the_pace_comes_from_the_hose_and_the_flow_only_hurries_it`,
  `every_mark_draws_a_thick_stroke_with_its_head`,
  `a_drawn_mark_keeps_the_line_class_and_the_numerals`,
  `a_page_without_flow_draws_no_mark`), del ABI
  (`a_printed_page_carries_the_flow_and_freezes_it`,
  `the_frame_accepts_a_phase_and_a_motion_flag_and_refuses_any_other_flag`), los pasos
  `animation:` de `--selftest` y las capturas de la corriente en la hoja.

### Inicio: la página llena la ventana y cuenta lo que ofrece — 2026-11-21

- **La página de Inicio llena la ventana hasta el suelo de 1366×768**: la región restaba el
  alto de la cinta y de la barra de estado aunque estuvieran ocultas, así que la página
  terminaba a unos 600 px y las filas de recientes quedaban cortadas. Ahora resta una barra
  solo mientras es visible, y `--check-layout` comprueba en 1280, 1366, 1440 y 1600 que la
  página mide lo mismo que la ventana y empieza en su esquina. La página sigue siendo un
  `Flickable`: a 1366 las cinco filas se alcanzan desplazándose, no recortadas por una región
  mal medida.
- **Tres plantillas ofrecidas, no cuatro**: las tarjetas de válvula 3/2, válvula 5/2 y
  preparación del aire siguen siendo el render real; la tarjeta eléctrica no se ofrece
  mientras P-026 difiere la conexión, pero `Template::ElectricalControl` y
  `PF_TEMPLATE_COUNT = 4` quedan en la ABI. La prueba de ventana comprueba que la tarjeta 3
  no existe y que las otras tres abren su circuito.
- **Recientes honestos**: cinco filas (el resto, en **Archivo ▸ Recientes**), con nombre,
  carpeta, tiempo **modificado** relativo (absoluto en el tooltip) y **tamaño real** leídos de
  `QFileInfo`, sin abrir ni parsear el archivo. Un documento que ya no está dice **no
  encontrado** y conserva **quitar**, que lo olvida de la lista y nunca borra el archivo. Sin
  recientes, el estado vacío dice el paso siguiente.
- **Volver a la hoja** sustituye a «Volver» y es enfocable con `Enter`/`Espacio`; `Esc`
  abandona la página antes de tocar lo que hay detrás.
- **Documentación**: `docs/ui-ux.md` §2.3 (tres plantillas, las cinco filas y la pista,
  modificado/tamaño, faltante/`quitar`, estado vacío, `Volver a la hoja`, `Esc`), la
  enmienda de la ADR 0031 (cuarta tarjeta oculta hasta P-026, recientes en `recent.json` y no
  en `QSettings`, miniatura y hora de apertura en P-034b), los nodos P-034a/P-034b y la
  corrección de las filas P-012/P-013 ya fusionadas en #145.
- **Evidencia**: capturas a 1366×768 en claro y oscuro con las filas, una fila faltante con
  `recent.json` temporal bajo un `XDG_DATA_HOME` aislado, el detalle de las tres tarjetas y
  el estado vacío en `specs/evidence/20261121-p034a-*.png`. Sin cambio de ABI, diagramas,
  simbología ISO 1219, hoja, zoom, relleno activo ni formato `.pfl`.

### Iconos: las clases de conducto y las velocidades se leen — 2026-11-21

- **Nueve dibujos propios en la familia de acciones** (`actionicons.cpp`): los tres
  conductos llevan el patrón de su clase como lo dibuja la hoja —trabajo sólido, pilotaje
  con la discontinuidad corta 1,6/0,9 y drenaje con la larga 2,6/1,3 de
  `LineClass::pattern`, en anchos de trazo— y la misma flecha de flujo; las tres
  velocidades son un arco de indicador con una, dos y cuatro marcas, sin dígitos
  dibujados (el multiplicador queda en la etiqueta); `check` es el contorno del circuito
  con la marca; `align-ports` son dos bocas sobre un eje; y `tube-diameter` es una sección
  de tubo con la flecha de diámetro. Caja de 24 unidades, trazo de 2, extremos y uniones
  redondeados, una sola tinta del tema, sin SVG ni segundo color; los diagramas, la hoja,
  el zoom, la geometría ISO 1219 y el relleno de la posición activa no cambian.
- **La cinta apunta a los ids nuevos**: Trabajo, Pilotaje y Drenaje dejan `draw-line`;
  0,25×, 1× y 2× dejan `speedometer`; Comprobar deja `dialog-ok-apply`; Puertos deja
  `align-horizontal-center`; y la fila generada del diámetro recibe su icono.
- **`--icon-sheet --scale <n>`** multiplica los tamaños dibujados y la imagen (por defecto
  1) para que la evidencia 2× sea un ráster real; sigue siendo un render manual, no un
  check de la puerta.
- **Inventario y cobertura de la familia** (`docs/reference/icons.md`): una fila por dibujo
  con su comando, metáfora, grupo, tamaño y estado; las reglas de la familia enlazan la
  ADR 0028 sin repetir sus valores; y los duplicados diferidos quedan con su razón
  (`document-properties` ×3, `edit-undo` como restablecer, las dos casas y los dos
  restablecimientos, `go-bottom` en dos posiciones y el `x-office-document` de Vertical).
  El recuento publicado es **55 dibujos**, el conjunto que devuelve `actionicons::names()`,
  no el de comandos.
- **`--check-icons` comprueba también la dirección inversa**: cada comando de la tabla de
  la cinta resuelve a un dibujo propio, y las entradas sin icono son exactamente la lista
  documentada —los grupos personalizados Configurar página y Recientes, las notas Paleta,
  Contraste y Estado, y los enlaces Acerca de y Licencia—; una entrada nueva falla. Se
  retiran los cuatro dibujos huérfanos (`draw-line`, `speedometer`, `dialog-ok-apply`,
  `align-horizontal-center`), que ya no usa ningún comando.
- **La documentación deja de fijar 46 iconos**: `docs/capabilities.md` habla de la familia
  y enlaza el inventario, y la ADR 0028 enlaza el inventario actual.

### Colocación: el símbolo conserva su caja — 2026-11-21

- **Un componente arrastrado desde el catálogo ocupa la misma caja que las plantillas**:
  la caja canónica de `SYMBOL_BOX_MM` (20 mm) para suministro, válvula, estrangulador,
  accesorio y el caso por defecto, y `SYMBOL_BOX_MM × CYLINDER_RATIO` para el actuador
  (44 × 20 mm). Antes el arrastre usaba `default_size` con 34 × 34 para la válvula,
  62 × 26 para el actuador, 26 × 26 para el suministro, 24 × 24 para el accesorio y
  22 × 22 por defecto: el mismo símbolo salía hasta 1,7 veces mayor que el de una
  plantilla y un circuito recién dibujado no se leía como el de la página de inicio. La
  hoja, el zoom y el marco no cambian.
- **La válvula sigue siendo cuadrada**: su cuerpo son dos celdas dentro de una caja
  cuadrada, y una caja rectangular aplastaría las celdas a la mitad de su alto.
- **Prueba**: `a_dropped_valve_and_cylinder_take_the_template_box` en la ABI coloca una
  válvula y un cilindro por `pf_session_place_component` y mide 20 × 20 y 44 × 20 mm;
  `--selftest` conserva sus pasos de arrastre, clic de válvula y rectángulo de componente.
- **Docs**: `docs/ui-ux.md` declara que un componente arrastrado toma la caja de las
  plantillas.
### Skill de lenguaje visual y check de documentación — 2026-11-21

- **Una skill propia, `punchi-visual-language`**, convierte la base visual en procedimiento
  y no en una segunda fuente: enruta por tarea —un control, un token, un icono, la cinta, el
  layout y una revisión— a `docs/ui-ux.md`, el adaptador del tema, ADR 0028,
  `actionicons.cpp`, la skill `qt-adaptive-layout` y los nodos P-028/P-033, con “leer /
  ejecutar / evidencia”, lista de comprobación, trampas y validación. No añade ningún ancho,
  tamaño, contraste, movimiento ni geometría de icono: esos valores viven en su documento.
  Los diagramas de la documentación son P-030 y la simbología ISO 1219 y su registro de
  revisión son un nodo aparte: la skill no los toca.
- **`check-docs.sh` valida la skill y los anclajes**: recorre `.agents/skills/**/*.md`,
  comprueba que cada carpeta tiene su fila en `.agents/skills/README.md` y un `name` igual
  al nombre de la carpeta, y resuelve cada enlace relativo contra el árbol. El `#fragment`
  se valida contra los títulos del documento de destino, y un enlace no puede salir del
  árbol: `../` fuera, una ruta absoluta o un symlink que apunte fuera fallan.
- **`scripts-dev/test-check-docs.sh`** construye un árbol temporal y prueba los fallos que
  la extensión debe detectar —enlace relativo roto, fila ausente, ancla incorrecta— además
  de los escapes de ruta, sin tocar el árbol real; `check.sh` lo ejecuta junto al
  clasificador de documentación.
- **Documentación**: la skill y sus cinco referencias, la fila del catálogo, los nodos
  P-030–P-033 en `docs/work-tree.md` y este registro. Sin cambio de runtime, de dibujo, de
  símbolos ISO 1219 ni de la página exportada.

### Ventana: la cinta se ve a 1366×768 — 2026-11-21

- **1366×768 es el suelo de soporte** (el portátil de entrada y educación del mercado al
  que apunta el proyecto): la cinta sigue **desplegada en una sola fila con sus siete
  grupos**, no se sustituye por el menú reducido. Un grupo que no cabe se convierte en un
  **botón con su nombre y un chevrón**, y sus comandos se abren en un **flyout no modal**;
  el alto de la cinta no cambia (121 px con la unidad por defecto) y la hoja conserva su
  sitio.
- **La prioridad de plegado es un contrato de producto**, de menos a más usado:
  **Repartir, Alinear, Paleta, Historial, Circuito, Componente, Conducto**. Un grupo sin
  comandos —la nota de Paleta— no se pliega, porque no tiene nada que llevar al flyout. El
  conjunto medido es **1366 → Historial, Alinear, Repartir**; **1440 y 1600 → Alinear,
  Repartir**; **1920 → ninguno**. Por debajo de 1280 se mantiene el plegado global de
  P-027; a 125 % de escala un panel de 1366 mide ~1093 lógicos y se registra como el caso
  plegado esperado, sin esconderlo.
- **El flyout conserva todos los comandos y sus etiquetas**: es el mismo contenido del
  grupo expandido, abre con `Enter` o `Space`, se cierra con `Esc` o un clic fuera, toma el
  foco y cada elemento mantiene su nombre accesible. `Alt+D` sigue alcanzando el
  **Diámetro** del grupo Conducto, esté expandido o en su flyout.
- **`--check-layout` gana la matriz de la cinta**: 1280×800 plegado y 1366×768, 1440×900,
  1600×900 y 1920×1080 desplegados, con siete grupos en orden, ningún borde derecho fuera
  del contenido, el inferior dentro del cuerpo, las etiquetas de los expandidos, los
  comandos del flyout de cada plegado, la altura de 121 px y el suelo de la hoja A4. La
  **exención del límite conocido a 1440 se elimina**: cualquier desbordamiento falla.
  También corre con `QT_SCALE_FACTOR` 1,25/1,5/2 y `--font-scale 1.5`.
- **`--selftest` fija el conjunto exacto**: a 1366 la cinta está desplegada, hay siete
  grupos y se pliegan **Historial, Alinear y Repartir**; un `Enter` real sobre el botón de
  Alinear abre su flyout con comandos; a 1920 los siete están expandidos; a 1280 el plegado
  sigue apareciendo. El tamaño de ventana y el plegado recordado se restauran después.
- **La primera ventana conoce la pantalla**: sin tamaño guardado abre a **1440×900** en una
  pantalla de 1080p y, en una menor, al mayor tamaño que cabe en el área disponible menos un
  margen de dos unidades, nunca por debajo de 900×600 y centrada. Así un panel de 1366×768
  abre ~1330×732 (por encima del pliegue de 1280), 1280×800 queda por debajo y a 125 % el
  área lógica se reduce hasta el formulario plegado. Un tamaño recordado se **recorta a la
  pantalla actual** con el mismo margen y mínimo, no solo al mínimo: una ventana guardada en
  un monitor de 1920 vuelve a abrir dentro de un portátil de 1366. `--selftest` prueba las
  parejas 1920×1080 → 1440×900, 1366×768 y 1280×800 con áreas sintéticas, y el recorte de un
  1920×1080 guardado a 1366×768, sin depender de la pantalla de la máquina.
- **Evidencia**: `--check-layout` en 0 fallos en la matriz y en las escalas; los pasos
  `ribbon: 1366 …`, `… Enter …`, `… 1920 …` y `… 1280 …` de `--selftest`; `--check-qml`,
  `--check-theme` y `--check-icons` en verde; las capturas
  `p028-ribbon-1366x768-{light,dark}.png`, `…-1440x900-…`, `…-1920x1080-…` y una de
  1366×768 con el flyout abierto; `./scripts-dev/check.sh`.

### Cierre de puerto: tapón y silenciador — 2026-11-21

- **Un cierre pertenece al puerto, no a la hoja**: un tapón ciego o un silenciador se
  monta en un puerto del componente y el documento lo guarda en un mapa
  `port_closures` (id de puerto → id de catálogo), una entrada por puerto. Un puerto de
  escape acepta **Sin cierre**, **Tapón** o **Silenciador**; uno de suministro o trabajo
  acepta **Sin cierre** o **Tapón**; uno mecánico o eléctrico no acepta ninguno y no abre
  menú. El cierre es una decisión de puerto, no una edición numérica de la definición:
  tiene su propio `Change::Closured` y **un solo deshacer**.
- **Un puerto cerrado queda terminado**: el documento rechaza un conducto que acabe en
  él, el lienzo no empieza uno allí y el lector del archivo marca la hoja como malformada
  (`a pipe ends on a closed port`). El menú deshabilita **Tapón** y **Silenciador**
  mientras haya una manguera y pide quitarla primero: no se borra nada en silencio.
- **El solver solo termina un escape**: un tapón en un escape deja el aire atrapado y su
  presión no decae; un silenciador sustituye el venteo libre por una restricción con su
  conductancia declarada; un escape sin cierre sigue venteando libre. Un tapón en un
  puerto de trabajo o suministro no cambia la red —el puerto ya estaba sellado— y su
  efecto es el rechazo del conducto, el dibujo y la fila de piezas. Una definición de
  cierre que no se puede resolver **no adivina**: mantiene el borde libre y se reporta.
- **El cierre se dibuja en el puerto**, fuera del cuerpo y sobre el borde de salida, con
  el símbolo ISO 1219 propio de la definición (la barra del tapón, el cono del
  silenciador). Su propia numeración `1` **no se dibuja** para que no parezca un
  componente etiquetado; la numeración del puerto anfitrión sigue visible y se mantiene
  libre de la caja del cierre. La huella dibujada crece con el cierre y el cierre no es un
  elemento seleccionable con anillo propio.
- **La lista de piezas nombra el padre y el puerto**: una fila plana tras los componentes
  y antes de los tubos, por ejemplo `Silenciador · 1V1 · escape 3` o
  `Tapón ciego · 1V1 · escape 5`; dos válvulas con la misma etiqueta y el mismo puerto
  comparten fila con cantidad 2. `MAX_ROWS` cuenta componentes, después cierres y después
  tubos, y cada clase tiene su contador de omitidos.
- **Formato 14**: `StoredPlacement.port_closures` es opcional y se omite cuando está
  vacío, así que una hoja de formato 13 se lee con todos los puertos abiertos y una de
  formato 14 sin cierres solo difiere en el número de versión. Los límites: `MAX_PORTS`
  (64) por componente en el catálogo y en el archivo, y un id de puerto o de cierre no
  vacío, ASCII y de 128 bytes como máximo (`MAX_ID`); cada límite tiene su prueba de uno
  por encima.
- **El menú de puerto es no modal**: una **pulsación repetida** sobre un puerto (doble
  clic) o **`Alt+C`** sobre el componente seleccionado abre un `Popup` anclado al puerto,
  con el nombre del puerto (`escape 3`), un selector de los puertos cerrables y las
  opciones **Sin cierre / Tapón / Silenciador**, la actual marcada con una marca y texto,
  y cierre con `Esc` o un clic fuera. La primera pulsación sigue armando el conducto; la
  segunda se atiende **antes** del gesto de tubo, y soltar donde se pulsó ya no intenta
  conectar el puerto consigo mismo. El panel **Propiedades** gana un bloque de solo
  lectura **Cierres de puerto** y se refresca con el cambio, sin una segunda inspección.
- **La paleta deja de ofrecer los dos cierres** como componentes: `plug.blind` y
  `silencer.pneumatic` son definiciones de cierre y el menú las ajusta a un puerto. Siguen
  en el catálogo del ABI —con su índice intacto— para que una hoja antigua que ya coloca
  uno cargue y simule como antes; el filtro vive en `readCatalog()`, no en QML, y el
  recuento y la búsqueda del panel leen la misma lista.
- **ABI 12**: enumeración de puertos y roles, consulta y cambio de cierre, consulta de
  manguera en el puerto y enumeración de las definiciones de cierre con la sesión (para
  incluir un catálogo de usuario). Un rechazo devuelve un `PF_ERR_*` negativo con el
  mensaje en `last_error`; un id vacío quita el cierre; un puerto eléctrico o mecánico lo
  rechaza.
- **La conexión eléctrica queda fuera y diferida como P-026**: un puerto eléctrico no
  ofrece cierre, el menú lo oculta y no se escribe fila de piezas. No se usa, arregla,
  esconde ni prueba aquí.
- **La cadena es P-021b**: el regulador de caudal, el conjunto silenciador + regulador y
  el menú completo siguen en su propio nodo, con el siguiente formato libre y una
  migración explícita del id único a una cadena de un elemento. P-021a no promete el 15.
- **Evidencia**: las pruebas del documento, el formato, el solver, el dibujo, el check, la
  exportación y el ABI; los pasos `close: …` de `--selftest` (26 sondas de cierre, 0
  fallos); `--check-qml`, `--check-theme` y `--check-layout` en verde; la página de 300 dpi
  con las filas `Silenciador · 1V1 · escape 3` y `Tapón ciego · 1V1 · escape 5`;
  las capturas en claro y oscuro del cierre dibujado en el puerto con la numeración
  visible; `./scripts-dev/check.sh`.

### Ventana: el ribbon colapsa a 1280 — 2026-11-21

- La cinta se pliega a **1280 px de ancho o menos**, no a 1240: en esa medida la fila de
  grupos de **Editar** (~1875 px) empieza a cortarse por la derecha, y el grupo **Conducto**
  quedaba fuera de alcance. Por encima de 1280 se mantiene la fila desplegada de siempre.
- **El menú de la cinta gana la fila `Diámetro: …`**: el control de P-024a es un control, no
  un comando, así que la lista aplanada no lo tenía; ahora la misma galería **Métrica /
  Pulgada / Sin especificar** se abre desde el menú y desde `Alt+D`, con el valor en vigor en
  la fila. La barra de estado no cambia.
- **`--check-layout` ve el alcance horizontal**: a 1280 mide la forma plegada y comprueba que
  el menú cabe en el contenido y que Conducto —trabajo, pilotaje, drenaje— y Diámetro son
  alcanzables; a 1440 informa que la fila desplegada mide ~1875 px y **no cabe**, como
  **límite conocido** de este nodo, no como fallo silencioso. `--selftest` gana el paso de
  1280 plegado y 1440 desplegado.
- **El desbordamiento a 1440 queda fuera**: la fila no cabe ni a 1440 ni con los grupos
  actuales; scroll, menú «más» o salto de fila serían otro nodo, que el propietario aún no
  ha abierto. **No hay cambio de documento ni de formato.**
- **Evidencia**: `--check-layout` en 0 fallos a 1280×800 y 1440×900, con `QT_SCALE_FACTOR`
  1,25/1,5/2 y `--font-scale 1.5`; los pasos `ribbon: 1280 …` y `ribbon: 1440 …` de
  `--selftest`; `--check-qml` en 0 y `--check-theme` en verde; capturas a 1280 y 1440 en
  claro y oscuro con el Conducto/Diámetro plegado y la hoja visible; `./scripts-dev/check.sh`.

### Tubo por conexión: 10 mm por defecto — 2026-11-20

- Cada conducto **neumático** guarda su **diámetro exterior nominal** como `Option<u16>` en
  **centésimas de milímetro** (`100..=5000`), con **10 mm por defecto**; una hoja antigua lee
  sus tubos como **sin especificar** y un cable eléctrico no tiene tubo. El valor entero
  conserva `Eq`/`Hash` en el documento y en el archivo, así que el historial compara dos
  conductos de forma exacta.
- **Formato 13**: `StoredPipe` estrena `tube_od_hundredths`, opcional y omitido cuando el
  conducto no lo especifica. Una hoja de formato 12 se lee con todos los tubos sin
  especificar y conserva su ancho histórico de 0,42 mm; la migración es el valor por defecto
  de serde. Un valor fuera de los límites —`0`, `99`, `5001`, `65000`, una cadena o un
  decimal— se rechaza con mensaje, y el límite superior tiene su prueba de uno por encima.
  **P-021 toma el formato 14.**
- **La serie es práctica nominal del proyecto, no una tabla ISO**: métrica de 4 a 50 mm y en
  pulgadas de 1/8" a 1", como entradas separadas (10 mm no es 3/8", 6 mm no es 1/4"); un
  valor dentro de los límites que no está en la lista se muestra como medida (`10,50 mm`).
  La fuente y los límites están en `docs/reference/tube.md`.
- **El trazo sale de una serie de anchos**, no de un mapa continuo: 0,25 mm para 4–5 mm,
  0,35 mm para 6–8 mm y 0,50 mm para 10 mm y más; un tubo sin especificar conserva 0,42 mm.
  El máximo de 0,50 mm queda **por debajo del marco ISO 5457 de 0,7 mm**, para que el borde
  de la hoja siga siendo otra clase de línea, y el patrón de la clase —trabajo llena,
  pilotaje y drenaje discontinuas— no cambia. Los codos del puerto pertenecen al símbolo y no
  engrosan; el anillo de selección conserva `LINE`.
- **La lista de piezas** añade una fila por `(diámetro, clase)` con la **longitud de la ruta
  dibujada** en milímetros enteros, incluidos los tramos de puerto: `Tubo 10 mm · trabajo ·
  3,4 m`, `< 0,1 m` por debajo de un decímetro. Un tubo sin especificar tiene su propia fila
  —el técnico ve que el tubo existe y que su medida es desconocida— y un cable eléctrico no
  tiene ninguna. `MAX_ROWS` cuenta componentes y tubos, los componentes primero, y los
  contadores de omitidos dicen qué se recortó. La página impresa aclara que la longitud es la
  ruta dibujada, sin tolerancia de curvatura, racores ni desperdicio.
- **Conducto ▸ Diámetro** (`Alt+D`) abre la galería **Métrica / Pulgada / Sin especificar**
  con la entrada en vigor marcada por el valor, una marca y un borde, nunca solo por color;
  el cambio sobre el conducto seleccionado es **un solo deshacer**. La barra de estado lee
  `Tubo 10 mm` o `Tubo Sin especificar`, y un cable eléctrico no muestra medida. La serie la
  enumera el ABI (`pf_tube_size_*`), no una lista copiada en QML, y `PF_ABI_VERSION` sube con
  el par `pf_session_pipe_tube` / `pf_session_set_pipe_tube` y
  `pf_session_pipe_is_electrical`.
- **La conexión eléctrica queda fuera y diferida como P-026** por decisión del propietario:
  no se usa, arregla, esconde ni prueba aquí; su clase, su trazo y su dibujo no cambian.
- **La física es P-024b**: conductancia, volumen, caída de presión, velocidad y aviso de
  dimensionado necesitan su referencia y su tolerancia, así que el solver conserva su modelo
  actual de conducto y el dibujo no insinúa una física que no tiene.
- **Evidencia**: las pruebas de la serie y los límites, el ida y vuelta del formato 13 y la
  lectura del 12, el ancho por serie y el cable eléctrico, los patrones de clase, la herencia
  del tubo al dividir, la fila de piezas y la longitud de ruta, un solo deshacer, la
  enumeración del ABI y los pasos `tube: …` de `--selftest`; `--check-theme`,
  `--check-layout` y `--check-qml` en verde; las capturas de 4/10/16 mm en claro y oscuro y
  la página de 300 dpi; `./scripts-dev/check.sh`.

### Presión de alimentación: 4 bar por defecto — 2026-11-20

- Las tres definiciones neumáticas de fábrica declaran **4 bar manométricos**:
  `supply.compressor`, `supply.compressor_adjustable` y `supply.network`. El compresor es la
  fuente por defecto del solver y la que colocan las cuatro plantillas, así que también
  cambia de nombre (`Compresor, 4 bar`, `Red de aire comprimido, 4 bar`, …). La presión es un
  valor del catálogo, no un campo nuevo del documento: no hay cambio de formato.
- **Un valor explícito manda.** Un documento que guardó `pressure_bar = 6.0` en una
  colocación conserva 6 bar al reabrirlo, y el modelo lo usa.
- **Un documento antiguo sin edición cambia a 4 bar.** La presión nunca se guardó si el
  usuario no la editó, así que al abrirlo lee el nuevo valor del catálogo; es el cambio de
  comportamiento deliberado de este hito. Un circuito que necesite 6 bar debe llevar la
  edición por colocación.
- Las plantillas, el ejemplo trabajado, las referencias del manómetro y del cilindro y el
  comentario de conductancia del caudalímetro (unos 60 l/min normales a 4 bar) se actualizan
  con el mismo cálculo y las mismas tolerancias; la escala del manómetro y los límites no
  cambian.
- **Evidencia**: `a_new_supply_is_four_bar`,
  `an_explicit_six_bar_edit_is_kept_when_the_file_is_reopened` y
  `an_old_supply_without_an_edit_reads_the_new_default`, las pruebas de plantillas, ejemplo
  y manómetro, las referencias de presión del solver, el smoke del CLI y `--selftest` en
  verde, y `./scripts-dev/check.sh`.

### Corrección: el ramal de una T — 2026-11-20

- **El ramal de una T sale recto**: los tres puertos de `junction.tee` están en el centro, así
  que `port_edge` empataba en las cuatro distancias y devolvía `Left`; un manómetro por encima
  recibía un codo. `port_edge_towards` (`placed/edge.rs`) resuelve el empate **solo** cuando
  las cuatro distancias coinciden dentro de `EDGE_TIE_MM` y toma el lado que mira al otro
  extremo; un puerto ordinario y una esquina conservan exactamente el resultado anterior.
- **El dibujo y la comprobación comparten el borde**: `draw_pipe`, `pipe_route` y `pipe_ends`
  pasan la pista, así que la inspección mide la misma ruta que la hoja y no inventa un cruce.
  Las dos mitades del tubo partido siguen rectas y el punto de la unión no cambia; el nodo y
  el símbolo son los mismos.
- **`pipe_is_elbow`, `magnet_centre` y `straighten_pipe` conservan el borde sin pista** a
  propósito: el gesto de arrastrar y enderezar mantiene su negativa actual para un ramal de T,
  y cambiarlo es otro nodo.
- **Evidencia**: las pruebas `port_edge_towards_resolves_the_centre_from_the_hint`,
  `an_ordinary_port_keeps_its_edge`, `a_gauge_above_a_tee_gets_a_straight_vertical_stub`,
  `the_two_halves_of_a_spliced_pipe_stay_straight` y
  `the_check_route_matches_the_drawn_route_on_a_tee`; el paso `--selftest` que imprime
  `tee probe branch=2 left=0 right=1 old=-1`; `--selftest`, `--check-layout`, `--check-qml`,
  `--check-theme` y `--check-icons` en verde; las capturas claro/oscuro y la página de 300 dpi.

### Corrección: las zonas de la hoja — 2026-11-20

- **Los ejes estaban cambiados**: la cuadrícula de ISO 5457 §4.4 ponía los números en las
  bandas verticales y las letras en las horizontales, así que una nota como B3 señalaba otro
  campo. Ahora las **letras** `A`, `B`, … bajan por las bandas izquierda y derecha, de arriba
  abajo, y los **números** `1`, `2`, … corren por las bandas superior e inferior, de izquierda
  a derecha, en A3 y mayores. El lienzo y la exportación cambian juntos.
- **`I` y `O` no se usan** en las letras de fila, como exige la norma: la secuencia es `A`…`H`,
  `J`…`N`, `P`…`Z`. Una hoja personalizada que pasa de 24 filas continúa `AA`, `AB`, … sin
  `I` ni `O`; el conteo de campos de A0 (17 en el código, 16 en la tabla de la norma) queda
  como cuestión abierta y no se toca aquí.
- **A4 estrena la forma de dos lados**: decisión del propietario del **2026-11-20**, la hoja
  por defecto también se puede citar por zona. A4 lleva números solo en la banda superior y
  letras solo en la derecha, en las dos orientaciones; las bandas izquierda e inferior quedan
  limpias. A5, Letter, Legal y Oficio siguen sin cuadrícula.
- **El cajetín de A4 no crece**: `is_large_sheet` se separa en `has_reference_zones` y
  `uses_large_block`, así que encender las zonas de A4 no convierte su bloque de 170 × 45 mm
  en la plantilla de 180 × 65 mm de A3.
- Las etiquetas **ceden ante las marcas de centrado** (al menos 1 mm, como mucho medio campo)
  y una banda de menos de 4,5 mm no dibuja ni etiquetas ni reglas, en vez de imprimir un
  glifo fuera del papel.
- **Evidencia**: `cargo test -p punchi-fluid-export` en verde, los pasos `zones:` de
  `--selftest`, `./scripts-dev/check.sh`, las capturas A3/A4 en claro y oscuro, A0 y hoja
  personalizada, y las páginas de 300 dpi de A3 y A4 en las dos orientaciones.

### Símbolos: las designaciones de puerto no tocan el símbolo — 2026-11-20

- El catálogo estrena **`envelope`**, la frontera exterior funcional del símbolo en dos
  esquinas normalizadas. La novena regla del lint exige que esté dentro de `0..=1`, que
  contenga el cuerpo, todos los puertos y el primitivo del marco; un archivo sin envolvente
  sigue siendo válido.
- `draw_numeral` resuelve esa frontera como **envolvente, si no cuerpo, si no la caja de
  20 mm** y coloca el numeral en el lado libre: fuera cuando la hoja deja sitio, dentro
  cuando no lo deja, con `gap = max(1 mm, 6 % del alto)` más media altura de mayúscula.
  La línea de conexión cruza la frontera; la designación no.
- `preparation.service_unit` declara `[0.06, 0.12, 0.94, 0.88]`. El marco deja solo 1,2 mm
  por fuera, así que los numerales 1 y 2 pasan **dentro del marco**, a 1,2 mm de la línea
  discontinua y libres del filtro, el regulador y el reloj. Los demás símbolos conservan su
  aserción para P-018b; el helper de la prueba ya es general.
- La guía de simbología, la plantilla de revisión y el registro de la unidad de
  mantenimiento recogen la regla de holgura; las capturas de 300 dpi y claro/oscuro se
  regeneran.
- **Evidencia**: `cargo test -p punchi-fluid-catalog -p punchi-fluid-draw` en verde, el
  lint de símbolos sobre todo el catálogo, las capturas `preparation.service_unit-300.png`
  y `-{light,dark}.png`, y `./scripts-dev/check.sh`.

### Inicio: una casa fija en la tira y Ctrl+Home — 2026-11-20

- La tira de circuitos estrena un **control de Inicio fijo** a la izquierda, fuera del
  `Flickable`: icono `go-home` de 16 px, tooltip, nombre accesible y una regla de 1 px que
  separa la vuelta del proyecto. La tira se desplaza bajo él y el signo `+` sigue al final de
  la fila. El control **no es un circuito**: no entra en la lista, el archivo ni el historial
  de deshacer.
- **`Ctrl+Home`** abre Inicio desde la hoja y cede la tecla cuando un `TextInput` o un
  `TextEdit` tiene el foco; el comando **Archivo ▸ Inicio** sigue como segunda vía.
- **Volver** existe también en un proyecto nuevo y vacío: `canReturn` cuenta los circuitos,
  no solo la ruta y el modelo, así que abrir Inicio nunca deja al lector sin vuelta.
- Las pestañas de circuito y el `+` son **alcanzables con Tab** y tienen nombre accesible; el
  control de cerrar conserva el suyo. Se retira la pista falsa de `Esc` del control Volver:
  la página se cierra con **Volver** o **Nuevo**.
- El caso de tira de `--check-layout` comprueba la casa dentro de la tira, el `Flickable`
  después de ella y el `+` al final tras desplazarse; como dos circuitos no desbordan el
  ancho mínimo de la hoja, añade circuitos hasta que la fila realmente se desplaza.
- Las capturas de la tira usan `--template 0` además de `--circuits 2`: `--grab` abre Inicio,
  y la plantilla cierra la página para que la tira quede en el marco.
- **Evidencia**: pasos `home:` de `--selftest` con 0 fallos; `--check-layout` correcto a 1280
  y 1440 y a 1,25×, 1,5×, 2× y fuente 1,5×; `--check-qml` sin problemas; y las capturas
  `assets/screenshots/circuit-tabs-{light,dark}.png` y las de Inicio alcanzado desde un
  proyecto.

### Inicio: iconos nítidos, tarjetas derivadas y tema que refresca — 2026-11-20

- Los iconos de Inicio siguen **una sola regla de raster con dos tamaños**: `1,1 × unit` en el
  raíl y `1,8 × unit` en las dos tarjetas de **Empezar**, con `sourceSize` al mismo tamaño y
  `fillMode: Image.PreserveAspectFit`, y el tono siempre tomado del tema. Ya no se amplía el
  fallback de 22 px.
- Las dos tarjetas de **Empezar** reservan una columna de icono fija de `2 × unit`, con el
  icono de `1,8 × unit` centrado, el hueco de `unit / 2` y el texto después; a 1280 y a 1440
  ninguna palabra toca el icono ni el borde, y ambas tarjetas comparten la misma geometría
  interna.
- Las tarjetas de plantilla **derivan la altura de su ancho**: la placa es el ancho de
  contenido sobre la proporción 512:288 y la tarjeta es la placa dividida por 0,65, así que la
  imagen ocupa el 65 % de la tarjeta y la proporción se mantiene en todo breakpoint. A 1280
  la tarjeta mide unos 252 × 200 px y la placa unos 231 × 130 px.
- El dibujo llena el ancho de la placa y conserva su proporción (`Qt::KeepAspectRatio`), con
  la banda transparente del 1,2 % entre el raster 512 × 288 y el área de 180 × 100 mm; el
  detalle queda en una sola línea con elipsis y el título usa `textLabel`.
- La placa es `panel` sobre una tarjeta `surfaceRaised` (`surfaceSunken` al pulsar, borde de
  acento al pasar y al enfocar), así que se lee como papel hundido dentro de la tarjeta.
- Todas las URL `image://theme` llevan `&v=<session.themeRevision>`: un cambio de paleta
  incrementa la revisión y vuelve a pedir los iconos, de modo que nunca queda el tinte
  anterior.
- La caché de iconos tiene un **tope medido**: 73 entradas (conjunto de trabajo 58, grupo
  máximo 21 por tamaño y tinte), con desalojo LRU y la paleta en la clave, para que un cambio
  de tema no devuelva un raster viejo.
- **Evidencia**: `--check-icons` con 0 iconos ausentes a 1×, 1,25×, 1,5× y 2× y las cuatro
  medidas `cache total 58, max group 21`; `--check-layout` correcto a 1280 y 1440, también a
  1,25×, 1,5× y fuente 1,5×; los pasos de tema de `--selftest`; `--report-paints 2` con una
  sola pintura; `--check-qml` sin problemas; las capturas de 1280 y 1440 en claro y oscuro, y
  `start-view-hierarchy-{light,dark}.png`.

### Inicio se lee en planos, con una acción principal — 2026-11-19

- La página es un solo plano con **Empezar**, **Plantillas** y **Recientes** como etiquetas de
  sección y cada elección como tarjeta propia; ninguna caja envuelve ya un grupo entero.
- **Nuevo proyecto** es la tarjeta por defecto, teñida con `accentSoft` y con borde de acento;
  su texto va en tinta, así el hover nunca repinta un relleno bajo una palabra clara.
  **Abrir…** queda como tarjeta secundaria.
- Las tarjetas de plantilla son **foto y etiqueta** y **reparten el ancho de la página**
  (mínimo 10 `unit`, 8 `unit` de alto, ~252 px a 1280): placa hundida con el dibujo, el nombre,
  dos líneas de detalle y un tooltip que dice qué abre.
- El título de la página usa un paso nuevo del tema, `textDisplay` (≈1,5 × `unit`); el raíl
  muestra el atajo (`Ctrl+N`, `Ctrl+O`, `Esc`) y separa sus grupos con una línea.
- Nuevo testigo `onAccent` en el tema, tomado del texto resaltado del escritorio y con la mejor
  de blanco/negro cuando no alcanza 4,5:1; `--check-theme` lo mide e informa como `accent text`.
  Lo usa la **fila activa del raíl**, rellena de acento, con icono y palabra medidos.
- El raíl estrena icono por fila y el cursor de mano sobre las tarjetas de plantilla; cuando la
  lista de recientes supera la página, el aviso a **Archivo ▸ Recientes** comparte la línea de
  la sección en lugar de ocupar una fila aparte.
- **Evidencia**: `--check-theme` con `accent text` ≥ 4,5:1 en claro y oscuro, `--check-qml` sin
  problemas, los pasos `--selftest` de plantillas y recientes sin cambios, y las capturas
  `assets/screenshots/start-view-hierarchy-{light,dark}.png`.

### Menú contextual en la paleta — 2026-11-19

- **Clic derecho** sobre una ficha del catálogo abre un menú con **Añadir a la hoja**, que
  coloca el componente en el centro de la hoja con una sola entrada de deshacer.
- El mismo menú se abre con **`Mayús+F10`** o la tecla Menú sobre la ficha enfocada, y el
  elemento anuncia qué componente va a añadir a la tecnología de asistencia.
- El **botón derecho ya no inicia un arrastre**: el arrastre queda para el izquierdo y el menú
  solo se ofrece en modo edición.
- **Evidencia**: los pasos `--selftest` desde `palette: the right button opens the menu` hasta
  `palette: one undo removes what the menu added`, con 0 fallos.

### Un símbolo mide 20 mm, y su caja es cuadrada — 2026-11-19

- La simbología sobre la hoja tiene un tamaño canónico: **20 mm de lado** para todo componente
  y **44 × 20 mm** para el cilindro, cuyo barril es el único rectángulo. No crece con el papel
  ni se encoge porque el circuito esté lleno; el ejemplo sembrado y las cuatro plantillas usan
  la misma caja.
- Se corrige la **deformación** de las plantillas: dos válvulas se colocaron en cajas de 1,2 : 1
  y 1,5 : 1, y el renderizador estiraba el símbolo. El 3/2 de la captura del propietario salía
  ancho; ahora todas las cajas no-cilindro son cuadradas y hay una prueba que lo exige.
- El cálculo queda escrito en `docs/symbology.md` con su origen
  (`schematic::layout`, `min(0.26·alto, 0.17·ancho, 20 mm)`) y su captura de referencia.
- Nuevas opciones de captura `--example` y `--template N`, y abrir un documento desde la línea
  de comandos ya no deja Inicio delante de la hoja.
- **Evidencia**: `a_symbol_does_not_grow_with_the_paper`,
  `the_four_start_templates_are_bounded_connected_starting_documents` (aspecto y tamaño),
  `dragging_a_component_pulls_its_port_onto_the_other_end` sin cifras fijas, `--selftest` con
  0 fallos y la captura `assets/screenshots/symbol-size-light.png`.

### Arrastrar de la paleta sin perder el gesto — 2026-11-19

- La ficha de la paleta **conserva el puntero** durante todo el arrastre. Antes, el área
  desplazable del catálogo tomaba el gesto en cuanto pasaba su umbral, y una mano que se
  desviaba en diagonal cancelaba el arrastre: la miniatura y el símbolo desaparecían de
  camino. Ahora el recorrido puede cruzar la paleta, los paneles y la hoja sin cancelarse.
- La paleta se sigue desplazando con la rueda o con **su propia barra**, que aparece cuando
  hace falta; ya no se desplaza arrastrando sobre un símbolo.
- **Evidencia**: el paso `--selftest` `palette: a diagonal gesture still places the component`,
  que reproduce el gesto cancelado y **falla sin el cambio**, junto con los pasos `palette: …`
  que ya existían; `--check-qml` y `--check-layout` verdes.

### Recientes en Inicio: tope, ausentes en gris y «quitar» — 2026-11-19

- **Inicio** lista como máximo cinco de los diez proyectos recordados; el resto sigue en
  **Archivo ▸ Recientes** y una línea lo dice cuando la página muestra solo una parte.
- Un proyecto cuyo archivo se movió o se borró **ya no desaparece**: queda en gris como
  `no disponible · ruta`, no se puede abrir y se retira con **quitar**, sin tocar el documento,
  el historial ni la marca de cambios. La lista ya no poda al leerse.
- El ABI gana `pf_session_recent_available` y `pf_session_forget_recent_at`; el límite de la
  página es `PF_RECENT_PAGE_LIMIT`, nunca un número en QML.
- **Evidencia**: `an_entry_whose_file_is_gone_is_kept_until_it_is_forgotten`,
  `pruning_drops_the_entries_whose_file_is_gone`, las pruebas ABI
  `a_missing_document_stays_in_the_list_until_it_is_removed` y
  `the_page_never_asks_for_more_rows_than_the_list_remembers`,
  los pasos `--selftest` desde `recent: the page keeps a missing document` hasta
  `recent: the page limit comes from the ABI`, y capturas claro/oscuro.

### Plantillas con miniatura en Inicio — 2026-11-19

- **Inicio** ofrece cuatro circuitos propios: el ejemplo 3/2 con cilindro simple, uno 5/2 de
  doble efecto, uno de preparación del aire y uno de control eléctrico. Cada tarjeta muestra
  una miniatura dibujada y seleccionarla crea un proyecto limpio; **Nuevo proyecto** sigue vacío.
- Las tarjetas siguen el paso de la interfaz: celda de `10 × 8 unit`, tarjeta cuadrada de
  `8 × 8 unit` y separación de `0.5 unit`. La miniatura reutiliza el renderizador de la hoja.
- La caché tiene cuatro imágenes RGBA de 512 × 288 como máximo (2.25 MiB total) y se libera al
  cambiar el tema o destruir la tarjeta.
- **Evidencia**: 2 pruebas ABI nuevas, 2 pruebas unitarias de plantillas, `--check-qml` sin
  problemas, `--selftest` con 0 fallos (incluida la confirmación ante trabajo sin guardar) y
  capturas claro/oscuro de Inicio revisadas.

### Vidas útiles en Qt y QML — 2026-11-19

- Nueva skill **`qt-lifetimes`**: un dueño por `QObject`, conexiones con objeto de contexto que
  mueren con él, modelos, imágenes y cachés que no crecen, y `grab` solo en captura.
- Incluye las pruebas que lo delatan: reposo sin repintar (`--report-paints`), abrir y cerrar
  cinco veces sin crecimiento, cierre durante una simulación con diálogo abierto, y ASan/LSan
  cuando se sospecha.
- Registrada en `.agents/skills/README.md` y enlazada desde `AGENTS.md` (memoria y rendimiento).
- **Evidencia**: `check-docs` verde; solo documentación.

### Revisión de seguridad por cambio — 2026-11-19

- Nueva skill **`security-review`**: nombra la **superficie** que toca un cambio (documento,
  catálogo, recientes, autoguardado, exportación, portapapeles, diálogos, enlaces, dependencias),
  aplica el control que ya existe en `docs/engineering.md`, exige el **caso hostil** y deja el
  **registro** en la PR.
- Incluye **recomendaciones al programar**: parsear en temporal y permutar, validar antes de
  reservar, no aceptar rutas de un documento, comprobar el destino, fallar cerrado, probar el
  rechazo y no dejar restos.
- Registrada en `.agents/skills/README.md` y enlazada desde `AGENTS.md` (seguridad).
- **Evidencia**: `check-docs` verde; solo documentación.

### Los límites de entrada dejan de depender de la memoria — 2026-11-19

- Nueva skill **`untrusted-input-bounds`**: todo lo que entra de fuera y toda colección o
  caché que crece tiene **un número, un mensaje y una prueba del caso «uno más»**, en el
  crate que posee el dato. Incluye la lista de comprobación (archivo, cadena, colección,
  bucle, asignación, caché, historial, espera), la decisión entre rechazar y truncar, la
  tabla de límites vigentes y los errores que no se cometen.
- Se nombra lo que son: consumo de recursos sin límite (OWASP API4:2023), **CWE-400**, **CWE-770**
  y **CWE-789**.
- Registrada en `.agents/skills/README.md` y enlazada desde `AGENTS.md` (seguridad y cachés).
- **Evidencia**: `check-docs` verde; es documentación, no toca código ni activos.
- **Primera deuda anotada**: la caché de iconos y miniaturas aún no declara tope ni desalojo.

### El programa arranca en Inicio — 2026-11-19

- Una ejecución normal abre en **Inicio**, no en la hoja; las comprobaciones automáticas de la
  ventana (`--selftest`, `--check-layout`, `--check-qml`, `--check-theme`, `--check-icons`)
  mantienen la hoja, que es aquello para lo que fueron escritas.
- **Nuevo** crea el proyecto vacío y cierra la página; **Volver** regresa a la hoja.
- **Inicio ocupa toda la región**: la cinta y la barra de estado no se muestran, el catálogo y
  el dock derecho se ocultan y el raíl toma su sitio; el título de la ventana nombra el
  programa, nunca un documento sin título. Las **plantillas** con miniatura son el trozo
  siguiente.
- **Evidencia**: `--check-qml` 0, `--selftest` 0 fallos, `check.sh` verde.

### La página Inicio — 2026-11-19

- **Inicio** existe como vista: cubre la hoja con su raíl a la izquierda (Nuevo, Abrir,
  Aprender con Acerca de y Licencia), la tarjeta de **Nuevo proyecto** y **Abrir…**, y la
  lista de **recientes** con nombre y carpeta. Se abre desde **Archivo ▸ Inicio** y vuelve a la
  hoja con **Volver**.
- **Nuevo** desde Inicio crea un proyecto vacío y cierra la vista; abrir un reciente hace lo
  mismo. La vista es estado de vista: no entra en el documento.
- Icono propio `go-home` para el comando; `--check-icons` queda en 48 de proyecto.
- **Todavía no es el arranque**: al abrir el programa se sigue viendo la hoja vacía; el
  arranque en Inicio, el raíl en el sitio del catálogo y las plantillas son el siguiente trozo.
- **Evidencia**: `--check-qml` 0 problemas, `--selftest` 0 fallos (la vista no altera los gestos)
  y `--check-icons` sin ausentes.

### Inicio: el programa abre sin circuito — 2026-11-19

- **ADR 0031**: al arrancar se muestra la vista **Inicio** en el área de dibujo —no una
  pestaña ni una cuarta región—, con **Nuevo, Abrir, recientes y cuatro ejemplos** con su
  miniatura, y la cinta reducida a **Proyecto** y **Ayuda**. El **lateral izquierdo** pasa a
  ser el raíl de Inicio (Iniciar, Nuevo, Abrir, Aprender) en el sitio del catálogo; el dock
  derecho se oculta y los comandos que necesitan documento no se muestran en gris.
- **Inicio** se abre también desde **Archivo ▸ Inicio** con un proyecto abierto; **Volver** o
  `Esc` regresa a la hoja.
- **Nuevo proyecto es una hoja vacía** y la hoja invita a empezar: «Arrastra símbolos a la
  hoja para empezar», con el catálogo como camino disponible ahora; las plantillas llegarán
  con la vista Inicio.
- El ejemplo que se sembraba deja de ser el documento de arranque y pasa a ser la **primera
  plantilla**; la fixture se queda en las pruebas y no la ve el usuario.
- Cerrar el último proyecto vuelve a **Inicio**; nada se abre solo, las miniaturas se dibujan
  con nuestro renderizador y todo es local.
- **ADR 0031 aceptada** al fusionarse la PR #127. La F0 deja de ser `missing` y pasa a
  `partial`: la vista Inicio, su raíl, los recientes y las plantillas quedan pendientes.
- **Primer trozo implementado**: la sesión del producto y `Nuevo` parten vacíos; el ABI ofrece
  una carga explícita del ejemplo para la plantilla y las pruebas; la primera pintura y la
  exportación ya no lo siembran de forma implícita. El selftest verifica hoja vacía, mensaje y
  carga explícita de la fixture. En una hoja vacía se ocultan las lecturas físicas (incluida la
  presión negativa ficticia) y el consejo de estado dice cómo empezar.

### Las hojas a 300 dpi de los símbolos — 2026-11-19

- Las **36 fichas** de `assets/symbols/review/` llevan su **hoja a 300 dpi** (`<id>-300.png`,
  60 mm cuadrados en tinta de impresión, 709 px) y la fila de legibilidad respondida. El estado
  sigue `pending-300dpi` hasta que el propietario firme; la exportación mantiene el aviso de no
  verificado.
- **Hallazgo**: `electrical.relay.latching` **falla** a 300 dpi: las etiquetas `A2` y `R1`
  chocan con sus terminales. Queda `blocked` con el defecto escrito y sin firmar.
- `throttle.fixed` sigue **bloqueado** por la forma no confirmada; su hoja no cambia el estado.
- **Evidencia**: 36 hojas nuevas, `cargo test -p punchi-fluid-catalog` verde (38 pruebas) y
  `check-docs` verde.

### El cajetín ya no reserva una banda — 2026-11-18

- El área de dibujo es **el marco entero**: el cajetín protege solo **su propio rectángulo**
  (ancho, alto y posición del layout resuelto). En A4 industrial se recuperan los 4.365 mm² de
  la franja a su izquierda, que antes no admitían nada.
- Un símbolo soltado o movido **solo se empuja si entra en la franja horizontal del cajetín**;
  a su izquierda se baja hasta el marco. La elección horizontal del usuario no se toca.
- **`Sin cajetín` libera el marco**. Si se activa un cajetín encima de un dibujo, la
  comprobación reporta la falta `under_title_block` con el componente señalado.
- El ejemplo inicial se compone en el mayor rectángulo que el cajetín deja libre, así que
  mantiene su forma, y el pegado mueve el grupo solo si alguna caja entra en el cajetín.
- **Evidencia**: seis pruebas nuevas del ABI y de la comprobación, los pasos del `--selftest`
  «el ejemplo queda fuera del cajetín» y «la franja de al lado admite un soltado», y las
  capturas clara y oscura.
- La ADR 0030 queda **`accepted`**; queda anotado que la lista de piezas es solo de la página
  impresa, por lo que el lienzo no la reserva.

### El cajetín protege su caja, no una banda — 2026-11-18

- **ADR 0030**: la reserva del cajetín pasa a ser **su propio rectángulo** —ancho, alto y
  posición, más la lista de piezas—, no su altura a todo lo ancho. En A4 industrial la banda
  reservaba 12.015 mm² para proteger 7.650: el 36 % de la banda, la franja de 97 mm a la
  izquierda del cajetín, es papel que nada tapa y que hoy no se puede dibujar.
- `Sin cajetín` **libera el marco**; si se activa un cajetín encima de un dibujo, la
  comprobación lo marca donde ocurre y el símbolo se puede mover.
- La reserva se recalcula del layout resuelto: no hay campo nuevo ni cambio de formato, y la
  exportación dibuja en el marco con el cajetín por encima.
- **No hay código todavía**: el ADR queda `proposed` hasta que lo aceptes al fusionar, y la
  capacidad F1 queda registrada como `partial` con su evidencia por escribir.
### La cinta deja de repetir la tira — 2026-11-18

- El grupo **Circuito** de Archivo ya no enumera los circuitos: **la tira al pie de la hoja es
  la única lista**. La cinta conserva los comandos **Nuevo**, **Renombrar** y **Cerrar** del
  circuito en pantalla, del mismo tamaño y con el mismo ancho con uno o con 32 circuitos.
- **Teclas de documento**: `Ctrl+N` proyecto nuevo, `Ctrl+T` circuito nuevo, `Ctrl+W` cierra
  el circuito en pantalla (con su confirmación y su undo), `Ctrl+Q` sale y `Ctrl+Shift+Tab`
  recorre hacia atrás; `Ctrl+Shift+W` se retira. Mientras se renombra o se espera una
  confirmación, `Ctrl+T` y `Ctrl+W` no interfieren.
- **Vocabulario**: un archivo es un **proyecto** y una hoja un **circuito**; el grupo, los
  diálogos de abrir y guardar, sus filtros y los avisos de cambios sin guardar dicen proyecto,
  y la tira sigue diciendo circuito.
- **Evidencia**: prueba nueva del `--selftest` (el grupo son tres comandos y no la lista;
  Cerrar cierra y el undo devuelve el circuito), `--check-qml` 0 problemas, `--check-icons` 47
  de proyecto y 0 ausentes, `--selftest` 0 fallos y capturas clara y oscura.

### La interfaz pendiente, escrita — 2026-11-18

- El plan de interfaz gana la lista de lo que queda tras la etapa 7, con cifras que se pueden
  comprobar: 84 espaciados y 23 márgenes literales, 137 tamaños de texto, 53 áreas de ratón
  frente a 4 consejos, la tira de circuitos sin foco ni teclado en la **×** y el **+**, y la
  barra de estado sin agrupar.
- La cinta deja de repetir la tira: el contrato dice que la lista de circuitos vive **solo en
  la tira**, y que Archivo conserva los comandos **Nuevo**, **Renombrar** y **Cerrar** para el
  circuito en pantalla.
- Se anota que hoy **`Ctrl+N` reinicia el documento** y su consejo dice «empieza otro
  circuito», y queda decidido el vocabulario: **un archivo es un proyecto y una hoja es un
  circuito**; el renombrado alcanza el grupo, los diálogos, sus filtros y los consejos.
- Las teclas quedan decididas, siguiendo a los editores con pestañas: **`Ctrl+N`** proyecto
  nuevo, **`Ctrl+T`** circuito nuevo, **`Ctrl+W`** cierra el circuito en vista, **`Ctrl+Q`**
  sale, **`Ctrl+Shift+Tab`** recorre hacia atrás; `Ctrl+Shift+W` se retira.

### El cierre de la pestaña dice lo que cuesta — 2026-11-18

- Pasar el puntero por la **×** de una pestaña muestra «Cerrar circuito»; si el circuito
  tiene componentes, el consejo avisa de que se eliminan con él, que es justo lo que hace el
  clic. Sin atajo inventado: la × no tiene tecla propia.
- **Evidencia**: `--check-qml` 0 problemas y `--selftest` 0 fallos.

### ADR: el selector y la dos presiones deciden por presión — 2026-11-18

- **ADR 0029**: una variante `Behaviour::Pressure` con dos clases, `shuttle` y `two_pressure`.
  El selector conecta la salida a la entrada de mayor presión y cierra la otra; la dos
  presiones solo da salida con las dos presionadas y toma la menor, y descarga cuando una
  falta. Se resuelve como frontera de presión, sin volumen interno inventado ni estado.
- Los dos símbolos serán archivos, con sus puertos `1 · 1 · 2`, su geometría, su fixture y su
  ficha de revisión.
- La ADR queda **`accepted`** al fusionarse, y `capabilities.md` registra la capacidad: comportamiento `Pressure`, validación, reglas del solver, dos archivos, fixtures, hojas y fichas.

### La selección marca el símbolo, no una caja — 2026-11-18

- Al seleccionar un componente, **sus propias líneas toman el acento** en vez de dibujarse un
  rectángulo alrededor. Se parece a la referencia y no añade una segunda geometría que no dice
  nada del símbolo.
- El estado no queda solo en color: el inspector nombra lo seleccionado.
- **Evidencia**: prueba nueva del dibujo (`a_selected_component_is_drawn_in_accent_and_not_boxed`)
  que comprueba que no se añade ninguna llamada y que hay trazo de acento; `--selftest` 0
  fallos y `./scripts-dev/check.sh` verde.

### Ajustar a la hoja, y el + junto a la última pestaña — 2026-11-18

- **Ajustar a la hoja**: el ajuste encuadra el papel entero, márgenes y bordes incluidos, en
  lugar del marco interior. Como el 100 % ya significaba «la hoja ajustada», la fila del 100 %
  desaparece del panel de zoom: era el mismo mando dos veces.
- El **+** de la tira va **pegado a la última pestaña** y se desplaza con ellas, como en un
  navegador, en vez de quedar en el borde de la ventana.
- **Evidencia**: el auto-test mide el **papel** contra el lienzo y su centro contra el centro,
  y comprueba que el ajuste deja el zoom en 1,0; 0 fallos, `--check-qml` 0 y capturas
  retomadas.

### Cerrar circuitos desde la pestaña — 2026-11-17

- **× en todas las pestañas menos la última**: cerrar quita el circuito con sus componentes y
  conductos, y **un deshacer** lo devuelve entero. La última no la lleva: un proyecto sin
  circuitos no es un proyecto.
- **Si tiene trabajo dentro pregunta**, nombrando el circuito, y ofrece Cancelar; si está
  vacío se cierra sin diálogo. El circuito en vista pasa al vecino de la derecha o, si era el
  último, al de la izquierda.
- El **+ se deshabilita** al llegar a 32, con el límite en su ayuda, y **la tira se desplaza**
  en horizontal en vez de crecer a una segunda fila.
- **Evidencia**: catorce pasos nuevos del `--selftest` de la tira (cierre vacío, cierre con
  contenido, cancelar, confirmar, deshacer, última pestaña y tope), 0 fallos; `--check-qml`
  0; 53 capturas retomadas y `./scripts-dev/check.sh` verde.

### Cambiar, añadir y renombrar circuitos en la tira — 2026-11-17

- **Clic** en una pestaña cambia de circuito: vista, sin deshacer ni marca de modificado.
  **`Ctrl+Tab`** recorre las pestañas.
- **`+`** añade un circuito y lo muestra.
- **`F2`** o doble clic renombra **en la propia pestaña**, con `Enter` para guardar y `Esc`
  para cancelar; es edición de documento y un solo deshacer.
- **Evidencia**: pasos nuevos del `--selftest` (`circuits: the tab renames the circuit with
  undo`, `circuits: the plus adds a circuit and shows it`), 0 fallos; `--check-qml` 0;
  capturas retomadas (51) y `./scripts-dev/check.sh` verde.

### Los circuitos, al pie de la hoja — 2026-11-17

- Una tira **dentro de la región del dibujo**, encima de la barra de estado, muestra una
  pestaña por circuito con su icono y su nombre; la activa lleva fondo y regla de acento, no
  solo color. Por ahora es **solo lectura**: dice en cuál se está trabajando.
- El lienzo le cede el alto, así que la hoja no queda tapada.
- Queda para la siguiente fase el `+`, el cambio con `Ctrl+Tab` y el renombrado en la propia
  pestaña con `F2`.
- **Evidencia**: capturas retomadas (51), `--check-qml` 0, `--selftest` 0 y
  `./scripts-dev/check.sh` verde.

### Hoja del símbolo a 300 dpi — 2026-11-15

- `punchi-fluid-export` gana el binario `symbol-sheet <id> <archivo> [--dpi 300]`: dibuja un
  símbolo incorporado en una caja fija de 40 mm sobre un cuadrado de 60 mm, con la tinta de
  impresión, y escribe el PNG. La ficha de revisión ya apunta a la orden real.
- Vive en el export y no en el CLI porque el CLI no depende del export, y añadir esa arista
  pediría un ADR.
- **Evidencia**: hoja de `valve.3_2.spring_return` escrita a 709 × 709 px (60 mm a 300 dpi),
  `./scripts-dev/check.sh` verde y clippy sin avisos.

### La geometría de cada símbolo queda congelada — 2026-11-15

Primera pieza del arnés de símbolos: la que impide que un dibujo se mueva sin que se vea.

- `punchi-fluid-draw/tests/symbol_geometry.rs` dibuja cada definición incorporada en una caja
  fija y compara sus llamadas con `assets/symbols/fixtures/<id>.txt`; **36 fixtures**, 228 KB.
- Un cambio intencionado se regenera con `UPDATE_SYMBOL_FIXTURES=1` y **se revisa el diff**
  antes de commitearlo; falta de fixture es fallo, no aviso.
- La ficha de revisión y el skill enlazan el arnés; el README de los fixtures explica el
  procedimiento y la regla: un fixture aprobado sin leer el diff aprueba cualquier cosa.
- **Evidencia**: la prueba corre en `cargo test --workspace`, dentro de
  `./scripts-dev/check.sh` con **exit 0**.

### Menos margen alrededor de la hoja — 2026-11-15

- El lienzo conserva **medio `unit`** (9 px con la fuente por defecto) en vez de uno entero:
  la banda de área de trabajo alrededor del papel era demasiado gruesa y le quitaba sitio al
  dibujo.
- **Evidencia**: capturas de la ventana retomadas, `--check-qml` 0, `--selftest` 0 y
  `./scripts-dev/check.sh` verde.

### Pendientes de zoom, ajuste y área de trabajo — 2026-11-15

- `interface-plan.md` recoge tres decisiones abiertas del propietario: el ajuste debe encuadrar
  **la hoja** y no el marco; el zoom y el doble clic central se revisan contra las dos
  referencias CAD; y el anclaje del área de trabajo a gran ampliación está por decidir.
- No se toca código: solo queda escrito para no perderlo ni improvisarlo.

### Ficha de revisión por símbolo — 2026-11-15

- `assets/symbols/review/TEMPLATE.md`: función, cláusula, tabla de medidas, capturas a
  300 dpi, lista de comprobación y firma de una persona. Una ficha sin cláusula ni medidas
  no es una revisión.
- El skill de simbología enlaza la ficha desde el paso de corrección.

### La corrección de símbolos empieza con palabras, no con líneas — 2026-11-15

- El skill de simbología añade la **puerta previa**: requisito en nuestras palabras con su
  cláusula, medidas en números y aprobación del propietario, antes de dibujar.
- Una imagen de Commons o de una librería CAD solo recuerda **qué símbolos existen**; nunca
  es la fuente de una proporción.
- La **corrección** queda ordenada: inventario, un símbolo por cambio, lint, capturas a
  300 dpi en claro y oscuro, lista de la familia y firma; la duda termina en «no verificado».

### Parada obligatoria antes de los símbolos — 2026-11-15

Decisión del propietario: la simbología **neumática** va primero y **corregir lo que ya está**
antes de ampliar bibliotecas; la geometría debe ser exacta porque está bajo norma.

- `next-steps.md` añade la parada con su procedimiento: detenerse y preguntar, inventariar,
  acordar la skill de corrección y trabajar **un símbolo por cambio** con capturas, lint y
  revisión.
- `symbology.md` lo deja escrito en el contrato: la siguiente pasada es de corrección, no de
  extensión, y un símbolo no se corrige comparándolo con otro programa.

### Los radios salen del tema — 2026-11-15

- Cuatro tokens de radio en el tema (`radius`, `radiusControl`, `radiusSmall`, `radiusTip`) y
  dieciocho archivos QML sin su propio número. Con la paleta por defecto la ventana es
  **idéntica píxel a píxel** a la publicada.
- De paso, el panel de zoom de la tira recibe su sesión y dibuja tarjeta de verdad en vez de
  depender de lo que hubiera detrás; sus dos capturas se retoman.
- **Evidencia**: diff de `window-light.png` vacío, `--check-qml` 0 problemas, `--selftest` 0
  y `./scripts-dev/check.sh` verde.

### El texto y las alturas de la cinta salen del tema — 2026-11-15

Primera parte de la etapa 7: la cinta deja de tener tamaños escritos a mano.

- `--font-scale 1.5` crece las pestañas, las etiquetas, los iconos y las alturas de la cinta
  y nada se recorta; con la fuente por defecto el resultado es **idéntico al anterior**, que
  es la prueba de que los tokens reproducen lo que había.
- La fila de la ventana adaptable pasa a **`covered`**: era su último hueco.
- Quedan para 7b y 7c el foco y las ayudas de todos los controles, los radios y márgenes
  fuera de la cinta, las lecturas de la barra de estado y el movimiento reducido.
- **Evidencia**: captura con fuente 1,5× en `assets/screenshots/font-large-light.png`,
  `--check-qml` 0, `--selftest` 0 y `./scripts-dev/check.sh` verde.

### Ventana de zoom y glifo propio de Ajustar — 2026-11-15

Con esto se cierra la tira de vista.

- **Ventana**: el panel arma el gesto, un arrastre dibuja el rectángulo sobre la hoja y el
  zoom lo llena con su centro en el centro del lienzo; un clic suelto o `Esc` no mueven la
  vista. Es estado de vista y guarda un paso para «Vista anterior».
- **Glifo de Ajustar**: cuatro esquinas alrededor del espacio, para no compartir la lupa del
  100 %. La familia pasa a **48 dibujos**.
- La capacidad de navegar el lienzo pasa a **`covered`**.
- **Evidencia**: `--selftest` con `view: the strip arms the zoom window`, `view: the zoom
  window magnifies the rectangle`, `view: the rectangle lands centred` y `view: Esc cancels
  the zoom window`, 0 fallos; `--check-qml` 0; `./scripts-dev/check.sh` verde y capturas
  retomadas.

### Mover con la mano desde la tira — 2026-11-15

- **Mover**: un modo visible con puntero de mano; el botón queda marcado, se arrastra con el
  botón izquierdo y sale con `Esc` o el mismo botón. Entrar en simulación lo deja, porque ahí
  un clic debe accionar la válvula. Es estado de vista: no toca el documento.
- El glifo de la mano se dibuja aquí y entra en la familia de iconos.
- **Evidencia**: `--selftest` con `view: the strip turns the hand on`, `view: the hand drags
  the sheet without editing` y `view: Esc leaves the hand mode`, 0 fallos; `--check-qml` 0;
  `./scripts-dev/check.sh` verde y capturas retomadas.

### Cajetín en pantalla y un solo Iniciar/Pausa — 2026-11-15

La tira suma los dos controles que faltaban de la lista acordada.

- **Mostrar cajetín**: on/off de vista. El cajetín deja de dibujarse, la banda que reserva
  sigue reservada y la impresión no cambia; sin deshacer y sin marca de modificado. El botón
  se resalta cuando el cajetín está oculto, que es el estado excepcional.
- **Iniciar/Pausa**: un solo botón que arranca el modelo y lo **pausa donde está** en vez de
  detenerlo y perder la marcha; la sesión ya tenía `startSimulation` y `pauseSimulation`, así
  que no hay estado nuevo.
- La rejilla conserva su interruptor en **Ver** y el borde se sigue dibujando siempre.
- **Evidencia**: `--selftest` con los pasos `view: hiding the block changes nothing in the
  document`, `view: the strip starts the model` y `view: the same button pauses where it
  stands`, 0 fallos; `--check-qml` 0; `./scripts-dev/check.sh` verde y capturas retomadas.

### La tira de vista flota sobre el borde derecho — 2026-11-15

La segunda mitad de la 6c pone la superficie de las operaciones que ya existían.

- La **tira** flota sobre el borde derecho del área de dibujo y no le quita columna al
  papel: **Ajustar al marco**, un **panel de zoom** (−, 100 %, +, vista anterior) y el
  control que la pliega a un solo botón.
- Tamaños e iconos salen de `unit`; el panel y la tira usan la tarjeta, el borde y la sombra
  del tema; cada botón tiene nombre accesible, ayuda con el atajo y se alcanza con el
  teclado. El estado plegado se recuerda como vista y no entra en el documento.
- En ventana **compacta** la tira nace plegada y su botón la abre como las gavetas de los
  paneles.
- **Evidencia**: `--selftest` con los pasos `view: the strip floats over the drawing area`,
  `view: the strip fits the frame without editing`, `view: the strip steps back to the
  previous view` y `view: the strip folds to one button`, 0 fallos; `--check-qml` 0;
  `./scripts-dev/check.sh` verde y capturas de la tira con su panel en claro y oscuro.

### Ajustar al marco y volver a la vista anterior — 2026-11-15

Primera mitad de la tira de vista: la matemática primero, la superficie después.

- **Ajustar al marco**: el zoom mayor que hace que el marco interior (no el papel) llene el
  lienzo, con el desplazamiento que lo centra. También funciona por debajo de 1:1 sin perder
  el centrado, que es lo que antes descartaba el límite de desplazamiento.
- **Doble clic del botón central** ajusta al marco; la pulsación simple sigue desplazando, y
  el intervalo sale del escritorio.
- **Un paso de historial**: la vista anterior se guarda antes de un ajuste o de un zoom en el
  puntero, y volver atrás la restaura. `Ctrl+0` sigue siendo el papel a 1:1.
- **Evidencia**: pasos nuevos del `--selftest` que miden el marco contra el lienzo y su centro
  contra el centro del lienzo (`fit zoom 1,11, frame 560x399 in 560x603`), el paso de vuelta
  atrás, los pasos de zoom y desplazamiento todavía verdes, `--check-qml` 0 problemas y
  `./scripts-dev/check.sh` verde.

### La cinta se pliega y la ventana recuerda dónde estaba — 2026-11-15

Segunda mitad de la etapa adaptable: la cinta minimizada, la geometría recordada de la
ventana, el mínimo de 900 × 600 y las pasadas de escala y de fuente grande.

- **Cinta minimizada**: un control al final de la fila de pestañas la pliega; la pestaña
  activa abre sus grupos como panel **sobre el área de dibujo**, no sobre los encabezados de
  los paneles, y un comando o `Esc` lo cierra. El estado se recuerda como vista, nunca en el
  documento.
- **La ventana recuerda tamaño y posición**, y solo restaura la posición si sigue cayendo en
  una pantalla; maximizada conserva el tamaño anterior. El mínimo pasa a **900 × 600**.
- `--check-layout` comprueba además la cinta plegada (la altura baja, el panel no la
  devuelve y la hoja no cambia), la geometría restaurada y el mínimo; y corre a **1,25×,
  1,5× y 2×** de densidad y con **fuente 1,5×**, con los umbrales siguiendo a la unidad en
  vez de fallar.
- Queda para la etapa 7 que el texto y las alturas de la cinta sean tokens: hoy la cinta no
  crece con la fuente del escritorio.
- **Evidencia**: `--check-layout` con 0 fallos en las tres escalas y con fuente grande,
  `--check-qml` 0 problemas, `--selftest` 0 fallos, `./scripts-dev/check.sh` verde y capturas
  de la cinta plegada en claro y oscuro.

### La ventana elige su modo por el espacio que tiene — 2026-11-15

A 900 px la cinta se plegaba pero el catálogo y el panel derecho conservaban sus 324 y 360
px: a la hoja le quedaban 180 y el dibujo era inservible. El armazón decide ahora su modo
con mínimos de contenido, no con una lista de anchos.

- **Tres modos**: ancha desde 1200 px con la fuente por defecto, media desde 1036 y
  compacta por debajo. El mínimo de la hoja es **480 px** y no cede en ningún modo.
- En **media** los dos paneles se estrechan hasta sus mínimos; en **compacta** se pliegan a
  sus rieles de 24 px, la hoja ocupa todo el ancho y un panel se abre como **cajón** con el
  borde y la sombra de la tarjeta, plegado de inicio para no abrir tapando el dibujo.
- **`--check-layout`** recorre 900, 1000, 1050, 1100, 1200, 1280 y 1920 px y comprueba el
  modo, el suelo de la hoja, que en compacta la hoja ocupa la ventana y que los paneles no
  se pisan; después abre el cajón y comprueba que no le quita columna. Los umbrales salen de
  los mínimos vivos, así que una fuente mayor los mueve en vez de romper la prueba.
- **Defecto encontrado de paso**: en un perfil nuevo el catálogo abría en su mínimo de
  220 px y no en `18 × unit`, porque el valor por defecto se recortaba antes de que el tema
  fijara la unidad. Ahora el enlace sobrevive cuando no hay preferencia guardada.
- Queda para **6a-ii** la cinta minimizada, la geometría recordada de la ventana y la pasada
  con fuente grande y escalas 1,25× y 1,5×.
- **Evidencia**: `--check-layout` con 0 fallos, `--check-qml` 0 problemas, `--selftest` 0
  fallos, `./scripts-dev/check.sh` verde, una pasada a 2× y capturas de compacta plegada,
  compacta con cajón, media y ancha.

### Las marcas de centrado siguen la norma — 2026-11-15

Las cuatro marcas del borde llevaban una flecha que la norma no pide y cruzaban todo el
margen de archivo. ISO 5457 §4.3 las fija de otra manera y ahora se dibujan así.

- Cuatro líneas continuas de **0,7 mm** en los extremos de los ejes de simetría, desde el
  marco y **10 mm más allá**; sin flecha ni triángulo.
- En el borde de archivo de 20 mm la marca termina **dentro del margen**, no en el borde del
  papel, porque la norma mide desde el marco.
- La misma regla en pantalla y en la exportación: el lienzo y `punchi-fluid-export` dibujan
  las cuatro líneas, y una prueba mide sus extremos y comprueba que no queda ningún
  triángulo relleno en la hoja.
- `docs/reference/title-block.md` lo registra como **referencia**: el requisito, la cláusula
  y la advertencia de que es un registro de diseño y no una certificación.
- La tira de vista queda con **un solo «mostrar»**, el cajetín: la rejilla conserva su
  toggle en **Ver** y el borde se dibuja siempre.
- **Evidencia**: `punchi-fluid-export` con 39 pruebas verdes, `--check-qml` 0 problemas,
  `--selftest` 0 fallos, capturas de las cuatro marcas ampliadas en claro y oscuro, y
  `./scripts-dev/check.sh` verde.

### La pestaña activa se asienta en el cuerpo de la cinta — 2026-11-15

La pestaña activa dejaba aire sobre el cuerpo y el subrayado no llegaba a los bordes, así
que parecía recortada de la cinta en vez de ser la misma superficie.

- La pestaña activa ocupa la fila completa, baja hasta el cuerpo con las esquinas inferiores
  rectas, pierde el borde inferior y el subrayado de acento va a todo el ancho, en la
  costura.
- Las pestañas inactivas siguen centradas a la misma altura, para que todos los textos
  compartan la línea, y el anillo de foco se mantiene.
- El borde del papel se dibuja **una sola vez**, con `lineStrong`; la segunda línea que
  duplicaba el contorno del área de trabajo desaparece.
- **Evidencia**: `--check-qml` con 0 problemas, `--selftest` con 0 fallos, capturas de la
  cinta en claro y oscuro y `./scripts-dev/check.sh` verde.

### Guía adaptable y tira de vista — 2026-11-15

Se documenta cómo se construye la ventana adaptable y se especifica la tira de vista que
acompaña al lienzo, para no perder las decisiones ni lo que falta.

- Nuevo skill **`qt-adaptive-layout`**: umbrales medidos sobre el espacio disponible,
  `Layout` frente a anclas, mínimos por panel y la matriz de tamaños y escalas.
- `ui-ux.md` añade la regla «Layout o anclas, nunca ambas», el criterio de umbral, el mínimo
  de la hoja por encima del ancho preferido y la especificación de la **tira de vista**
  (§3.1): flotante sobre el borde derecho, sin minimapa, ajuste al marco con doble clic
  central, historial de vista, tres toggles y un solo play/pausa.
- `interface-plan.md` registra las decisiones y la **etapa 6c**, con lo que existe hoy, la
  pregunta abierta sobre el cajetín y el marco, y lo que queda fuera (minimapa, rueda de
  navegación, cubo, cámara).
- `capabilities.md` documenta la navegación del lienzo como capacidad propia y suma las
  pasadas a 125 % y 150 % a la evidencia de la ventana adaptable.
- La guía externa aportada por el owner se usó como referencia: no se copia su texto ni su
  código.

### La hoja es una tarjeta sobre el área de trabajo — 2026-11-15

El dibujo deja de flotar sobre el mismo plano que la ventana: el área de trabajo se hunde,
la hoja se eleva con su borde y su sombra, y los editores flotantes comparten una sola
tarjeta.

- El área de trabajo usa `surfaceSunken`; el lienzo conserva un margen de `unit`; la hoja se
  rellena con `panel`, se bordea con `lineStrong` y lleva la sombra del tema desplazada una
  vez. El fondo del dibujo sigue el papel, para que los símbolos que borran contra la hoja
  coincidan.
- **Un `Card`** para Configurar página, Exportar, Comprobar, Renombrar, la recuperación, la
  confirmación, Acerca de y Licencia: mismo radio, borde, sombra y jerarquía.
- `--check-theme` mide además el borde de la hoja contra el área de trabajo: **3,39:1** en
  claro y **4,83:1** en oscuro.
- **Evidencia**: `--check-theme`, `--check-qml` con 0 problemas, `--selftest` con 0 fallos,
  `./scripts-dev/check.sh` verde, capturas a 1:1 y con zoom en claro y oscuro, y la tarjeta
  de página y la de Acerca de revisadas.

### El catálogo es una paleta de baldosas — 2026-11-15

La barra lateral termina su forma de paleta: dos columnas de baldosas con el símbolo, el
nombre en dos líneas y el `id`, y un ancho que se puede arrastrar y se recuerda.

- El ancho arranca en **324 px** (`18 × unit`), se arrastra entre 220 y 480 px desde el
  borde interior y se guarda como estado de vista.
- Cada baldosa es enfocable; **Enter** o **Espacio** la coloca en el centro de la hoja, y
  el recorrido de teclado pasa por todas. La selección lleva la barra de acento y la palabra
  del estado, no solo el color.
- La búsqueda y los grupos de la etapa 4a siguen; las baldosas se ordenan dentro de su
  categoría y el pie conserva la carpeta y los omitidos.
- **Evidencia**: los pasos `catalog: a tile is in the scene`, `catalog: Return accepts a
  focused tile`, `catalog: Return places the focused tile`, `catalog: the width is the one
  chosen` y `catalog: the width is remembered` de `--selftest`, `--check-qml` con 0
  problemas, `./scripts-dev/check.sh` verde y las capturas a 324 y 240 px.

### El catálogo se agrupa, se busca y dice qué omitió — 2026-11-15

La barra lateral deja de ser una lista única: muestra el número de componentes, un buscador,
las categorías del propio catálogo plegables y un pie con la carpeta en uso y los archivos
que el cargador tuvo que omitir.

- **Buscar** filtra por nombre, `id` y unas pocas palabras cercanas (cilindro, compresor,
  válvula…), sin distinguir mayúsculas ni tildes. `Ctrl+F` lleva el foco al campo; `Enter`
  coloca el primer resultado en el centro de la hoja.
- **Grupos** por categoría, en el orden del catálogo, con la etiqueta traducida en la
  sesión; el estado plegado se recuerda por grupo en `QSettings`.
- **Pie** con la carpeta que lee el núcleo y el número de archivos omitidos; el detalle de
  cada omisión queda para el panel de catálogo.
- La ventana ahora **carga el catálogo de usuario** (antes pasaba `null` al núcleo), con
  `PUNCHI_FLUID_CATALOG_DIR` y `XDG_DATA_HOME` como el núcleo; el ABI gana
  `pf_session_catalog_problems` y su prueba con un archivo roto.
- Los auto-tests y las capturas usan solo el catálogo integrado, para que un archivo del
  usuario no mueva los índices que las pruebas asumen.
- **Evidencia**: los pasos `catalog: …` de `--selftest`, el test ABI
  `a_broken_user_catalog_file_is_counted_and_the_built_ins_still_load`,
  `--check-qml` con 0 problemas, `./scripts-dev/check.sh` verde y las capturas del catálogo
  en claro y oscuro.

### La cinta completa su familia de iconos — 2026-11-15

Los 46 nombres de icono de la cinta se dibujan ya en el proyecto: ningún comando depende
del tema del escritorio y todos comparten rejilla, trazo y tinta.

- Se suman alinear, repartir, conducto, paneles, transporte, tema, ir arriba/abajo, agenda,
  pausa, avance, salto, detener, actualizar, velocímetro y menú.
- `--check-icons` informa **46 propios, 0 del tema y 0 ausentes**; el tema queda solo como
  respaldo para un comando que no tuviera dibujo.
- La hoja `--icon-sheet` reordena la revisión en 12 columnas por tamaño y muestra los 46 a
  16, 24 y 32 px en claro y oscuro.
- **Evidencia**: `--check-icons`, la hoja de iconos, la pasada completa de capturas de las
  cinco pestañas, el menú plegado, la ventana, el zoom, el panel de página, el cajetín y las
  ayudas, `--check-qml` con 0 problemas, `--selftest` con 0 fallos y `./scripts-dev/check.sh`
  verde.

### La cinta dibuja los iconos de documento, edición y zoom — 2026-11-15

La familia propia crece de ocho a **24 iconos**: los comandos de documento, edición,
transformación y zoom ya no cambian de trazo con el tema del escritorio.

- Se suman Propiedades, Circuito, Nuevo circuito, Renombrar, Girar, Recientes, Comprobar,
  Copiar, Pegar, Duplicar, Reflejar, Todo, Borrar, Aumentar, Disminuir y Tamaño original.
- `--check-icons` informa **24 propios, 22 del tema y 0 ausentes**; el resto sigue cayendo
  al tema hasta 3c-ii.
- La hoja `--icon-sheet` muestra los 24 a 16, 24 y 32 px en claro y oscuro.
- **Evidencia**: `--check-icons`, la hoja de iconos, las capturas de Archivo, Editar, Ver,
  Simular, ventana, zoom y panel de página, `--check-qml` con 0 problemas, `--selftest` con
  0 fallos y `./scripts-dev/check.sh` verde.

### La cinta empieza a dibujar sus propios iconos — 2026-11-15

Los primeros ocho comandos de la cinta dejan de depender del tema del escritorio: sus
glifos se dibujan aquí como geometría, con una sola tinta y el trazo de la referencia.

- `actionicons` dibuja Nuevo, Abrir, Guardar, Guardar como, Exportar, Deshacer, Rehacer e
  Iniciar en una rejilla de 24, trazo de 2 y extremos redondeados; el resto sigue cayendo
  al icono simbólico del tema y, si no hay, a la etiqueta.
- El proveedor resuelve primero la familia propia, después el tema; `--check-icons` informa
  **8 propios, 38 del tema y 0 ausentes**, y falla solo si un comando no resuelve ni por una
  vía ni por la otra.
- Los glifos se dibujan al tamaño que pide Qt, que ya incluye la densidad de la pantalla, y
  la caché distingue esa densidad para no reutilizar un raster de otra pantalla.
- `--icon-sheet` imprime la familia a 16, 24 y 32 px en claro y oscuro, para la revisión
  antes de completar el resto.
- **Evidencia**: `--check-icons`, la hoja de iconos en claro y oscuro, las capturas de
  Archivo, Editar y Simular con los iconos nuevos, `--check-qml` con 0 problemas,
  `--selftest` con 0 fallos y `./scripts-dev/check.sh` verde.

### La cinta tiene dos planos y estados — 2026-11-15

La cinta deja de ser una superficie plana: la fila de pestañas se eleva sobre el cuerpo, la
pestaña activa se une al plano de la hoja con un subrayado de acento, y cada comando
responde al puntero y al teclado.

- La fila de pestañas usa `surfaceRaised`; el cuerpo vuelve al fondo de la ventana, con una
  `line` que cierra la cinta y separadores más finos entre grupos.
- Hover con relleno elevado, pulsado con el plano hundido, foco con anillo de acento y
  selección con `accentSoft`; la pestaña activa lleva su subrayado.
- Las galerías de **Configurar página** y los desplegables de **Archivo** usan los mismos
  bordes, planos y estados; el menú plegado gana una sombra dibujada, sin desenfoque por
  cuadro, y se desplaza cuando no cabe en la ventana.
- Sin un color nuevo: todo sale del tema y `--check-theme` sigue midiendo los pares.
- **Evidencia**: `--check-theme`, `--check-qml` con 0 problemas, `./scripts-dev/check.sh`
  verde y las capturas de las cinco pestañas en claro y oscuro, incluida la de un comando
  deshabilitado.

### Los paneles recuerdan su estado y se ocultan al salir si se pide — 2026-11-14

Los paneles dejan de empezar siempre igual: recuerdan si estaban abiertos, su ancho y si el
auto-hide está encendido. **Ver ▸ Paneles** reúne Catálogo, Propiedades, **Ocultar al
salir** y **Restablecer vista**.

- **Ocultar al salir** es opt-in: con el puntero sobre el raíl durante 350 ms el panel
  vuelve; 800 ms después de salir se pliega. No se oculta con el teclado dentro, con un
  arrastre del catálogo en curso ni con un diálogo abierto.
- La visibilidad, el ancho del catálogo y la marca de auto-hide viven en `QSettings`, no en
  el documento. **Restablecer vista** devuelve los paneles, el zoom y el desplazamiento a
  su estado inicial.
- Las capturas y los auto-tests usan un archivo de ajustes propio, así que no pisan la
  vista que el usuario eligió.
- **Evidencia**: los pasos `panels: the catalog state is remembered`, `panels: auto-hide is
  opt-in`, `panels: the auto-hide flag is remembered`, `panels: leaving the rail hides it`,
  `panels: the edge opens it after the dwell`, `panels: a manual hide stays closed under
  the pointer`, `panels: reset view opens the panels`, `panels: reset view turns auto-hide
  off` y `panels: reset view returns to 1:1` de `--selftest`, `--check-qml` con 0
  problemas, `./scripts-dev/check.sh` verde y las capturas de **Paneles** en claro y
  oscuro.

### Las propiedades y el cajetín comparten el panel derecho — 2026-11-14

El inspector flotante y la columna fija del cajetín se unifican en un solo **panel
derecho**: la hoja conserva su columna, las dos secciones se apilan y se desplazan juntas,
y el panel se pliega a un raíl de 24 px como el catálogo.

- El panel aparece cuando se abre una propiedad o **Editar datos…**; **Ver ▸ Propiedades**
  y `Ctrl+J` lo pliegan y lo devuelven. Si estaba oculto, abrir una propiedad lo trae de
  vuelta.
- Cuando no tiene contenido, cede todo el ancho a la hoja. **Cerrar** cierra una sección y
  deja la otra; el panel solo desaparece cuando no queda ninguna.
- Es estado de vista: no entra en el historial, no marca el documento y no se guarda en el
  archivo.
- **Evidencia**: los pasos `panels: the right dock starts without content`, `panels: the
  inspector fills the right dock`, `panels: the right dock shows the properties`, `panels:
  Ctrl+J folds the right dock`, `panels: folding the right dock is not a document change`,
  `panels: Ctrl+J brings the right dock back` y `panels: the right dock yields when it has
  no content` de `--selftest`, `--check-qml` con 0 problemas, `./scripts-dev/check.sh`
  verde y las capturas del panel con propiedades, cajetín, ambos y plegado.

### El catálogo se pliega a un raíl — 2026-11-14

La barra del catálogo deja de tener un ancho fijo: se oculta y vuelve con el control de su
cabecera, con `Ctrl+B` y desde **Ver ▸ Catálogo**, y la hoja recupera esos 224 px.

- `DockPanel` envuelve el contenido con cabecera, título y borde; al plegarse queda un
  raíl de 24 px con el mismo control, que conserva el foco de teclado. El ancho se anima
  con `motionMs` y con movimiento reducido cambia sin animación.
- El glifo es propio: dos paneles, el que se pliega relleno con el acento suave. Es
  *chrome*, no un símbolo de componente.
- Es estado de vista: no entra en el historial, no marca el documento y no se guarda en el
  archivo. La barra de estado y el resto de la ventana no cambian.
- **Evidencia**: los pasos `panels: the catalog starts open`, `panels: Ctrl+B folds it to
  the rail`, `panels: hiding is not a document change` y `panels: Ctrl+B brings it back`
  de `--selftest`, `--check-qml` con 0 problemas, las capturas en claro y oscuro con el
  catálogo abierto y plegado, y una decena de fotogramas de pintado durante el plegado.

### La ventana gana planos, líneas y movimiento — 2026-11-14

La primera etapa de la interfaz no cambia todavía lo que se ve: prepara el tema para que la
cinta tenga profundidad, los paneles puedan ocultarse y el catálogo se lea como una paleta.
Los planos y las líneas que antes eran literales en cada widget salen ahora del tema.

- `surfaceRaised` y `surfaceSunken` dan dos planos alrededor de la ventana; `line` y
  `lineStrong` separan sin inventar un color; `accentSoft` es el relleno de una fila o una
  pestaña activa; `shadow` es la sombra de la hoja.
- `unit` sigue la fuente del escritorio (18 px en Breeze) y `motionMs` lee
  `[KDE] AnimationDurationFactor`: 180 ms normal y 0 con movimiento reducido.
- `--check-theme` mide también el texto y las líneas sobre el plano elevado y el texto
  sobre el acento suave, en claro y en oscuro: 4,5:1 para el texto y 3:1 para las líneas.
  Los ocho pares pasan.
- Sin cambio de formato ni de disposición: el gris secundario queda algo más firme para
  leerse también sobre el plano elevado, y la cinta y los paneles usan estos tokens en las
  etapas siguientes.
- **Evidencia**: `./scripts-dev/check.sh` verde, `--check-theme` en claro y oscuro,
  `--check-qml` 0 problemas, `--selftest` 0 fallos y `motion 0 ms` con
  `AnimationDurationFactor=0`.

### La cuadrícula arranca en 5 mm y se apaga con un ojo — 2026-11-14

La cuadrícula nace ahora en **5 mm**, el paso que el owner ve mejor para el dibujo, y el
panel la muestra con un **ojo**: abierto cuando la guía se ve, tachado cuando está oculta.

- El paso por defecto pasa de 10 a **5 mm** en el núcleo y en la ventana; el imán que
  coloca los componentes sigue el mismo paso (medio paso más un milímetro: **3,5 mm**).
- El control de **Cuadrícula** del panel tiene un icono de ojo visible (`view-visible` /
  `view-hidden`) y el texto Visible/Oculta, además del desplegable 5/10/20/25 mm.
- El paso sigue siendo estado de vista: no se guarda, no se imprime y no entra en el
  historial.
- **Evidencia**: `the_grid_step_is_a_view_setting_and_the_snap_follows_it` actualizado al
  valor por defecto, los pasos `view: the grid step …` de `--selftest`, `--check-qml` sin
  problemas y las capturas del panel con el ojo y de la cuadrícula a 5 y 20 mm.

### La simulación se puede cancelar y el puntero deja de parpadear — 2026-11-14

**Hasta cambio** era un bucle síncrono de hasta 250 cuadros en el hilo de la interfaz: durante
ese rato la ventana no atendía ni el botón **Detener** ni `F5`, y el escritorio mostraba el
cursor de espera. Ahora avanza un cuadro por vuelta del bucle de eventos y se cancela al
instante con **Detener** o **Pausa**.

- `runUntilChange` ya no bloquea: arranca un `QTimer` que avanza un cuadro, comprueba el
  cambio o el límite de cuatro segundos y termina solo. El estado sigue siendo «en pausa»
  en el cuadro donde cambió.
- **Detener** y **Pausa** cancelan el avance; **Iniciar** también lo cancela y arranca el
  modo normal. La barra de estado muestra «hasta cambio» mientras dura.
- El cursor sobre la válvula ya no pide un repintado en cada transición: el parpadeo del
  puntero era el cursor de espera de la ventana bloqueada.
- La pestaña **Simular** ya no se reconstruye en cada cuadro: su nota de estado incluía
  `session.timeS`, de modo que la tabla entera se reevaluaba sesenta veces por segundo y
  recreaba los botones y su cursor de mano. La nota muestra solo el modo; el tiempo sigue
  en la barra de estado, que se actualiza sin reconstruir la cinta.
- **Evidencia**: los pasos `transport: the fast-forward reports it is running` y
  `transport: Detener cancels the fast-forward` de `--selftest`, la puerta completa y
  `--check-qml` sin problemas.

### La configuración de página sigue el orden de la referencia — 2026-11-14

La primera corrección había dejado el tamaño visible, pero sin el desplegable de perfiles de
márgenes y con iconos demasiado pequeños para leerse como hojas. Este ajuste reproduce el
patrón visible en la referencia dentro del grupo **Configurar página** de Archivo.

- **Orden único en Archivo**: **Márgenes → Orientación → Tamaño → lanzador**. No se crea
  una pestaña Disposición.
- **Márgenes** abre directamente desde la cinta la galería de perfiles: Normal (ISO 5457 /
  NCh 13.Of93), Estrecho, Amplio y Márgenes personalizados…, con miniatura de hoja y
  valores. Normal es el único perfil normado; los otros se marcan no ISO.
- **Orientación** es un desplegable Vertical/Apaisado con una miniatura de hoja, no dos
  baldosas.
- **Tamaño** es el desplegable de papeles con miniatura, proporciones, nombre canónico y
  común; el lanzador de hoja al final abre el panel detallado.
- El panel conserva la galería de márgenes, los campos numéricos, el tamaño personalizado,
  la cuadrícula y la vista previa. Recientes sigue como desplegable a la derecha.
- **Evidencia**: `--check-qml` sin problemas, `--selftest` sin fallos, capturas de Archivo
  y del desplegable de perfiles en claro y oscuro.

### La cuadrícula se ajusta al dibujo — 2026-11-14

La cuadrícula era una guía fija de 10 mm. Ahora se elige su paso en la misma cinta que
configura la página, y el imán que coloca los componentes usa ese mismo paso: las líneas
nunca mienten sobre dónde va a caer una pieza.

- **Disposición ▸ Cuadrícula**: Visible/Oculta y paso de **5, 10 (por defecto), 20 o
  25 mm**, con una línea más marcada cada cinco pasos. El paso es estado de vista: no se
  guarda en el archivo, no se imprime y no entra en el historial.
- **El imán sigue la cuadrícula**: el radio de atracción de los puertos pasa de 6 mm fijos
  a medio paso más un milímetro, así que un paso fino no atrae de más y uno grueso no se
  queda corto.
- **Las líneas se alinean con el origen del papel**, no con el borde del marco: un margen
  que no es múltiplo del paso ya no desplaza la guía respecto del punto donde cae la pieza.
- **Se retira Cuadrícula de Ver** para no repetir el mando; `Ctrl+G` sigue alternando la
  visibilidad y el paso vive en Disposición.
- **Evidencia**: `the_grid_step_is_a_view_setting_and_the_snap_follows_it`, los pasos
  `view: the grid step changes` y `view: the grid step is a view setting` de
  `--selftest`, y las capturas con 5, 10 y 20 mm.

### La hoja se configura como una página, con nombres de verdad — 2026-11-14

Elegir papel era una fila de nueve botones y una lista de códigos. Ahora la hoja se
configura en un pack de página reconocible, sin tocar el formato del archivo: el papel,
la orientación y los márgenes ya eran del documento.

- **Pestaña Disposición**: **Página** con el combo **Tamaño** (miniatura proporcionada,
  nombre normalizado y nombre habitual entre paréntesis) y **Orientación** (Vertical,
  Apaisado); **Márgenes** con la galería **Normal (ISO 5457)**, **Estrecho**, **Amplio** y
  **Personalizado**, cada uno con su vista previa y sus cuatro valores. La opción en vigor
  se marca con borde y visto, no solo con color.
- **Bloque Configurar página en Archivo** y botón en Disposición: abren el panel detallado
  con tamaño, orientación, tipos de margen, cuatro márgenes numéricos y una vista previa
  que sigue cada cambio. No hay Aceptar: cada cambio se aplica al instante y es un undo.
- **La galería de perfiles también vive en el panel**, como en un procesador de texto:
  miniatura con la banda de margen sombreada, nombre, cuatro valores y **Márgenes
  personalizados…** al final; el perfil Normal cita **NCh 13.Of93** además de ISO 5457.
- **Tamaño personalizado en el documento**, no solo en una impresión: se guarda con la
  hoja (formato 7) y se reabre. Los límites de 5 a 200 cm por lado salen del núcleo.
- **Nombres habituales sin tocar el canónico**: `Letter (Carta)`, `Legal (Oficio)` y
  `Oficio chileno (Folio)` en los controles de página; la barra de estado y el archivo
  siguen diciendo `Letter`, `Legal` y `Oficio`.
- **Exportar queda aparte**: el panel conserva la resolución y la lista de piezas; el
  papel, la orientación y los márgenes son los de la hoja del documento
  (`PF_EXPORT_DOCUMENT_PAPER`). La impresión puntual con otro papel sigue disponible en el
  ABI.
- **Evidencia**: `every_paper_reports_the_size_the_drawing_list_needs`,
  `a_custom_paper_is_stored_and_read_back_as_the_document_sheet`,
  `a_custom_print_paper_reaches_the_page`, los pasos `page setup: …` de `--selftest`,
  `--check-qml`, `--check-theme`, `--check-icons` y las capturas de la pestaña, el combo,
  la galería y el panel.

### El español sale de los identificadores y el plan de traducciones queda escrito — 2026-11-14

El español es el idioma de la interfaz y de los catálogos, no el de los identificadores.
Dos nombres del modelo de cajetín lo incumplían y el plan de traducciones describía una
herramienta que no encaja con la ventana.

- **El modelo de cajetín se llama en inglés**: `school` y `technical` sustituyen a
  `escolar` y `tecnico` en el formato 12, en el ABI (`PF_BLOCK_MODEL_SCHOOL` y
  `PF_BLOCK_MODEL_TECHNICAL`), en los identificadores de Rust y C++ y en el nombre de las
  capturas. Un fichero del formato 11 se lee con el modelo que nombraba y al guardarlo
  escribe el nombre inglés; el test de migración lo prueba.
- **El plan de traducciones dice la verdad**: [i18n/README.md](i18n/README.md) describe
  dónde vive hoy el texto, qué falta y el proceso de extracción. La decisión está en
  [ADR 0027](docs/adr/0027-translation-catalogs-are-qts.md): los catálogos son los de Qt
  (`.ts` y `.qm`), el núcleo devuelve claves estables y no frases, y el exportador recibe
  su juego de etiquetas. Fluent no se adopta porque los textos viven en QML y C++ y no
  existe puente.
- **La referencia del formato al día**: [sheet-format.md](docs/reference/sheet-format.md)
  documenta las versiones 7 a 12 y el estado actual del campo `format`.
- **Evidencia**: `a_format_eleven_sheet_reads_its_retired_model_names`, la puerta
  completa, `check-docs` y las capturas renombradas en `assets/screenshots/`.

### Acerca de y Licencia, en la cinta y con el texto completo — 2026-11-14

La fila de pestañas no decía cómo se llama el programa ni con qué licencia viaja. Ahora
lo dice, sin añadir una pestaña más.

- **Dos enlaces de texto a la derecha de las pestañas**: **Acerca de** y **Licencia**,
  con el peso de Archivo, Editar, Ver, Cajetín y Simular, y visibles también cuando la
  cinta se pliega en el botón Menú. No son pestañas ni mandos del circuito.
- **Acerca de** reúne el nombre, la versión del núcleo, «versión en desarrollo, sin
  publicar», una línea de qué es, el repositorio, el enlace para informar de un problema,
  el copyright con la GPL-3.0-or-later y la autoría de PunchiSoft. Los dos enlaces abren
  el navegador del escritorio **solo al pulsarlos**: la aplicación no hace red ni lanza
  nada por su cuenta. Es la única excepción a la regla de no lanzar procesos, y está
  acotada a dos URL fijas.
- **Licencia** muestra el resumen y el **texto completo de la GPL**, compilado en el
  binario con `qt_add_resources` desde `LICENSE`: una copia instalada sin el repositorio
  sigue llevando la licencia que la cubre. La versión de Acerca de sale de `pf_version()`,
  no de una constante escrita en QML.
- **Cierre**: Cerrar, `Esc` o un clic fuera. Abrir cualquiera de los dos no cambia el
  documento ni entra en el historial.
- **Evidencia**: `--check-qml` sin problemas, los pasos `help: …` en `--selftest` (la
  versión mostrada es la del núcleo y la licencia es la GPL completa) y las capturas
  `about-light.png`, `about-dark.png` y `license-light.png`.

### El cajetín y la hoja siguen las guías chilenas — 2026-11-14

La hoja se identifica como un esquema de control y sigue las dos guías técnicas chilenas
que aportó el owner: el cajetín de **NCh 14.Of93 (ISO 7200)**, la hoja de **NCh 13.Of93
(ISO 5457)**, la escritura de **NCh 15 (ISO 3098)** y la escala de **NCh 1471 (ISO 5455)**.
El programa abre ahora en **A4 horizontal**.

- **El cajetín tiene cinco filas**: Empresa / propietario, Proyecto / instalación y
  Equipo / sistema; Título; Nº de plano, Revisión, Hoja, Fecha y Escala; Dibujado, Revisado
  y Aprobado; Normas aplicadas. El industrial mide **180 × 65 mm en A3 y mayores** y
  **170 × 45 mm en A4 y A5**, porque la guía propone el tamaño grande para una plantilla A3
  y pide ajustarlo al espacio; en A4 deja los 10 mm a la izquierda que el owner quería. Los
  modelos **Escolar** (170 × 35 / 180 × 40) y **Técnico** (170 × 40 / 180 × 50) son más
  cortos.
- **Tres modelos de contenido y Sin cajetín**, en un **combo box** con Técnico, Escolar,
  Industrial y Sin cajetín: el modelo elige los campos por omisión, **Personalizar** oculta
  o muestra cada uno sin perder el valor y **Restablecer modelo** vuelve al conjunto del
  modelo. Doce campos editables por circuito —empresa, proyecto, equipo, título, número,
  revisión, fecha, dibujado, revisado, aprobado, escala y normas— más la hoja `n/N`
  derivada. La **escala empieza en `S/E`** («sin escala») y sigue siendo editable; no hay
  material, unidad ni proyección. Los campos, el modelo y la visibilidad viven en el
  **formato 11**, y el **ABI sube a 10** con el modelo, la visibilidad, las posiciones y la
  nota; un archivo anterior se abre como industrial con todos los campos.
- **Alturas de texto de ISO 3098**: 5 mm para título y empresa, 3,5 mm para los valores y
  2,5 mm para los rótulos, cada una dentro de su celda; si la fila escalada no cabe, el
  valor baja un paso antes de cruzar la regla inferior. El cajetín es opcional y se coloca
  con dos combos, contenido y posición; las miniaturas de la posición reflejan el modelo y
  la orientación de la hoja.
- **Zonas de referencia desde A3**: los campos de 50 mm con números en los lados y letras
  arriba y abajo, a 3,5 mm y con reglas de 0,35 mm, siempre dentro de los márgenes; A4
  queda limpia. Las dibujan el lienzo y la exportación.
- **El lienzo reserva la banda del cajetín**: la sesión viva descuenta la altura resuelta
  (45 mm industrial en A4, 65 mm en A3, menos en los modelos cortos) antes de sembrar el
  ejemplo, encajar el dibujo y medir el espacio del chequeo;
  una colocación o un movimiento que caiga bajo el cajetín se sube hasta el borde de la
  banda, así que ningún símbolo queda oculto. Las zonas de referencia siguen en los
  márgenes y no reducen el área de dibujo.
- **Posiciones y nota**: el cajetín puede ir en la **esquina superior derecha**, la
  **inferior derecha** o **abajo centrado de lado a lado**; el combo de posición ofrece las
  tres en vertical y las dos esquinas en horizontal, con miniaturas de las filas reales.
  **Sin cajetín** sigue en el combo de contenido y deshabilita la posición. La **Nota** es
  un texto multilínea con el marcador «escriba aquí su nota.», se guarda con el cajetín y
  todavía **no se imprime**; la colocación en la esquina opuesta es el próximo cambio.
- **La hoja abre en A4 horizontal** (ISO 5457:1999/Amd 1:2010 permite ambas); `A4 normal`
  vuelve a vertical y la orientación se guarda en el archivo. Los márgenes siguen siendo
  20 mm en el borde de archivo y 10 mm en los otros tres, con marco de 0,7 mm y las cuatro
  marcas de centrado.
- **El editor** es un panel **acoplado a la derecha**, no un modal: la hoja sigue visible y
  se encoge para dejarle sitio. Se abre desde la nueva pestaña **Cajetín** (contenido,
  posición y datos) con **Editar datos…** (el botón dice **Añadir cajetín** cuando no hay
  ninguno); una edición es una entrada de deshacer y marca el documento como modificado.
- **Las pruebas y capturas no suenan**: `--grab`, `--selftest`, `--check-qml`,
  `--check-sounds`, `--export` y `--mute` arrancan en silencio, para que ejecutar la app
  durante el desarrollo no dispare el modelo una y otra vez en el equipo de quien prueba.
- **Pendiente y anotado**: una imagen de logo, las longitudes exactas de ISO 7200 (cada
  campo se limita a 120 caracteres y se recorta con `...`), reservar la banda del cajetín en
  el espacio de dibujo del lienzo y las marcas de recorte opcionales.
- **Evidencia**: la migración y los límites del formato, la ida y vuelta por la ABI con
  deshacer y los valores por omisión, las pruebas de geometría 170 × 45 / 180 × 65 y de
  cada modelo, la de que cada texto queda dentro de su celda, las de zonas de referencia (A3
  sí, A4 no), `the_seeded_example_stays_clear_of_the_block_on_a4_and_a3`,
  `every_position_is_geometrically_sound_in_both_orientations`,
  `the_remark_round_trips_and_allows_line_breaks`, los pasos `--selftest` de modelos,
  visibilidad, posiciones y nota, y las capturas `window-light.png` /
  `window-dark.png`, `title-block-light.png` / `title-block-dark.png`,
  `title-block-escolar-light.png`, `title-block-zoom-light.png`,
  `cajetin-tab-light.png`, `cajetin-portrait-light.png`, `cajetin-landscape-light.png` y
  `a3-reference-zones-light.png`.

### La simulación es una cinta y sus mandos son baldosas — 2026-11-14

La simulación vivía fija a la derecha de las pestañas, fuera del alcance de Archivo,
Editar y Ver. Ahora es la pestaña **Simular**, elegible como las demás.

- **Reúne lo que estaba suelto**: Iniciar (F9), Pausa (F8), Paso (F10), Hasta cambio,
  Detener (F5), Restablecer (Ctrl+R), la velocidad 0,25× / 1× / 2× y el estado —edición,
  en marcha o en pausa, con `t`—. Los atajos no cambian: correr el modelo sigue sin exigir
  un puntero ni un cambio de pestaña.
- **Los mandos son baldosas grandes**, icono sobre etiqueta, igual que Nuevo, Abrir y
  Guardar: son lo primero que mira quien simula. La velocidad en vigor queda marcada en su
  baldosa y el estado es una palabra, nunca solo color. Por debajo de 1100 px el botón
  **Menú** lista los mismos mandos.
- **Evidencia**: `--check-qml` sin problemas, `--selftest` sin fallos, y las capturas
  `simular-light.png` / `simular-dark.png` con la pestaña abierta.

### Las marcas de centrado cruzan el margen y la escala queda documentada — 2026-11-14

La marca de la izquierda quedaba corta: el margen de archivo es de 20 mm y la línea solo
recorría 10, así que parecía un hueco frente a las otras tres. Ahora la línea cruza todo el
borde hasta el papel en los cuatro lados, con la flecha en el borde, y el marco del lienzo
usa el grosor de 0,7 mm de ISO 5457.

- **Las cuatro marcas son simétricas en el papel** y recorren el margen completo; la forma
  sigue siendo libre, como permite ISO 5457.
- **La escala no es un campo del cajetín**: ISO 5455 fija las escalas 1:1, 1:2, 1:5, 1:10…
  para un dibujo que representa un objeto, pero un esquema neumático **no está a escala**
  (IEC 61082-1, ISO 1219-2). Si un circuito no cabe en A4, la respuesta es una hoja mayor
  —A3/A2— o símbolos menores, no una escala en el bloque. El ajuste de la exportación al
  papel es una comodidad de reproducción y no se escribe.
- **Evidencia**: `the_sheet_draws_frame_block_and_legend` (cuatro flechas),
  `the_margin_band_holds_no_ink` (la marca es la única tinta del margen) y la captura; la
  puerta completa queda en 350 pruebas, `--check-qml` 0 y `--selftest` 0.

### El flujo de entrega pide confirmación y comprueba el merge — 2026-11-14

El flujo de pull request ya tenía rama desde `main`, puerta verde y merge del owner; le
faltaban las paradas que evitan trabajo perdido en paralelo.

- **Una rama activa**: mientras haya trabajo sin fusionar, otro cambio no abre una rama
  paralela sin autorización explícita del owner. Se termina, se fusiona o se aparta.
- **Confirmación antes del commit**: el agente pregunta si el cambio coincide con lo pedido
  y con lo esperado, y espera la respuesta; una orden de commit no sustituye esa revisión.
- **Verificación del merge**: tras abrir o actualizar el PR se lee su estado. Si no está
  fusionado, se avisa y se pide al owner que lo fusione antes de continuar. Después del
  merge se vuelve a `main`, se actualiza y se pregunta por la limpieza local y remota.
- **Documentación**: [AGENTS.md](AGENTS.md) y
  [.agents/skills/pull-request-workflow](.agents/skills/pull-request-workflow/SKILL.md)
  recogen el flujo y su diagrama Mermaid.

### El marco de ISO 5457 y la norma del cajetín — 2026-11-14

El marco ya se veía, pero le faltaban las cuatro marcas de centrado y una referencia escrita
para el cajetín. Las dos cosas están ahora.

- **Marco de 0,7 mm** con 20 mm en el borde de archivo y 10 mm en los otros tres, como ISO
  5457; las **cuatro marcas de centrado** van en los ejes de simetría, con una línea hacia
  fuera y una pequeña flecha, en el lienzo y en el PDF/PNG/SVG.
- **Referencia del cajetín** en [docs/reference/title-block.md](docs/reference/title-block.md):
  qué fija ISO 5457 (marco, bordes, posición, marcas), qué fija ISO 7200 (campos, nombres,
  contenido y longitud) y por qué un esquema neumático usa el cajetín de **diagramas
  eléctricos/de control** (IEC 61082-1), no el de una pieza mecánica: **sin escala, sin
  material, sin unidad y sin símbolo de proyección**. La tabla de campos, el ancho de 180 mm,
  las dos bandas y lo que queda abierto están en la página.
- **La norma no impone una cuadrícula única**: fija los campos y el sitio; la disposición es
  de la empresa. El documento lo dice para no presentar una plantilla como si fuera la norma.
- **El bloque visible deja de imprimir la escala**: el ajuste de una página grande al papel
  es una comodidad de impresión, no una escala del esquema.
- **Evidencia**: `the_sheet_draws_frame_block_and_legend` (cuatro flechas),
  `the_margin_band_holds_no_ink` (las marcas son la única tinta del margen) y la captura del
  lienzo; la puerta completa queda en 350 pruebas, `--check-qml` 0 problemas y `--selftest`
  0 fallos.

**Siguiente**: el cajetín de 180 mm para diagramas de control, con los campos de la norma y
sin escala ni material, guardado por circuito en el formato 8 y editable por bloques; el
editor es la rama que sigue.

### La manguera rodea también los componentes que encuentra — 2026-11-14

La ruta automática ya rodeaba los dos símbolos que une; ahora también aparta los que están
en medio.

- **Búsqueda incremental**: se calcula la ruta, se añaden las cajas que atraviesa y se
  recalcula, como mucho **3 pasadas y 6 cajas extra**. Un circuito normal no paga nada más;
  el grafo solo crece cuando hay un componente en el camino.
- **El caso difícil no se disfraza**: si el plano está tan apretado que no hay pasada, se
  conserva la forma simple y la comprobación informa el cruce; no se inventa un dibujo.
- **El desplazamiento manual sigue siendo tuyo**: una ruta arrastrada conserva su forma, y si
  cruza un cuerpo se informa.
- **Una manguera se dibujaba dos veces por fotograma**: el bucle duplicado de `draw_placed`
  se elimina, así que la ruta se calcula una vez y no dos.
- **Evidencia**: `a_third_component_on_the_way_is_also_avoided`,
  `two_components_on_the_way_are_both_avoided`,
  `a_component_between_the_ports_is_routed_around`, y las pruebas anteriores de la ruta y de
  la comprobación; la puerta completa queda en 350 pruebas, `--check-qml` 0 problemas y
  `--selftest` 0 fallos.

### La manguera rodea el símbolo y la comprobación mide lo que se dibuja — 2026-11-14

Una manguera que sale de un puerto lateral y entra por el superior de un compresor dibujaba
una línea por encima del cuerpo: el trazado tomaba el codo a la altura de la salida. Ahora la
ruta sale por el eje del puerto, rodea las dos cajas del símbolo con 2 mm de holgura y entra
por el eje del otro puerto.

- **Ruta que reserva el espacio**: un pequeño grafo ortogonal alrededor de las dos cajas elige
  el camino más corto con menos codos, con la mayor holgura que quepa —2 mm, y 1, 0,2 o 0
  cuando los símbolos están casi pegados—; si no hay paso, conserva la forma simple y la
  comprobación lo delata.
- **Un desplazamiento no puede deshacer la mejora**: en una pareja perpendicular el offset no
  tiene segmento que mover y se ignora; en una paralela se conserva la forma que el usuario
  arrastró, y si cruza un cuerpo se informa como falta.
- **La comprobación ve la misma ruta**: usa los bordes de los puertos y el desplazamiento
  arrastrado, no un segundo cálculo. «Un tubo cruza un componente» incluye ahora los propios
  extremos de la manguera; solo se perdona el tramo corto del puerto al borde del símbolo.
- **Evidencia**: `a_hose_goes_round_the_symbols_it_joins`,
  `the_compressor_hose_goes_over_the_compressor`,
  `a_perpendicular_hose_ignores_an_offset_and_still_goes_round`,
  `a_tight_gap_still_gets_a_route_around`, `aligned_boxes_keep_a_straight_hose`,
  `a_dragged_offset_still_takes_its_own_shape`,
  `a_dragged_pipe_through_a_third_component_is_reported` y
  `a_dragged_hose_through_its_own_end_is_reported`; la puerta completa queda en 347 pruebas,
  `--check-qml` 0 problemas y `--selftest` 0 fallos.
- **Siguiente**: una manguera que atraviesa un tercer componente sigue cruzada por la ruta
  automática; la comprobación la informa, y considerar todas las cajas es el paso que queda.

### El imán de puertos: la manguera se endereza al arrastrar — 2026-11-13

Dos puertos en el mismo eje solo forman una línea recta si la otra coordenada coincide, y la
rejilla de 10 mm no llega a todas las diferencias: la válvula y el cilindro del circuito de
arranque están a 8,02 mm. Arrastrar el componente ahora lo resuelve.

- **Imán al arrastrar**: si la manguera sale y entra por el mismo eje, el puerto que queda a
  menos de 6 mm se pega al eje del otro extremo, fuera de la rejilla; el movimiento sigue
  siendo un solo deshacer.
- **Teclado**: `Alt+A` y **Alinear ▸ Puertos** mueven un extremo de la manguera seleccionada
  hasta dejar los dos puertos alineados. Una manguera con codo entre ejes perpendiculares se
  rechaza en vez de adivinar.
- **Evidencia**: `dragging_a_component_pulls_its_port_onto_the_other_end`,
  `a_hose_between_parallel_ports_straightens_and_a_bent_one_is_refused`, los pasos
  `magnet: …` de `--selftest` y la puerta completa (340 pruebas, `--check-qml` 0 problemas y
  `--selftest` 0 fallos).
- **Pendiente**: una manguera puede cruzar el símbolo del extremo cuando los puertos están en
  ejes perpendiculares —el compresor del ejemplo—; el enrutado tiene que reservar ese espacio y
  la comprobación medir la misma ruta que se dibuja. Queda anotado como hueco en
  `docs/capabilities.md`.

### La hoja es el papel que se imprime — 2026-11-13

El lienzo deja de ser un rectángulo lógico de 200 × 130 mm y pasa a ser la hoja: A4 vertical
con el marco de ISO 5457 por defecto, y A0–A5, Letter, Legal y Oficio desde la cinta. Lo que
se dibuja, lo que se comprueba y lo que se imprime son la misma hoja.

- **El papel vive en el documento**: `Sheet` en `punchi-fluid-core`, guardado con el formato
  7, con la orientación y los cuatro márgenes. Elegir papel, girarlo o mover un margen es un
  cambio con deshacer, y el archivo lo conserva ([ADR 0026](docs/adr/0026-the-sheet-is-the-paper.md)).
- **El lienzo dibuja la hoja**: encaja el papel en la ventana, dibuja su borde y el marco, y
  la cuadrícula se queda dentro del espacio de dibujo.
- **La comprobación mide el marco**: «fuera de la hoja» es ahora fuera del espacio que deja
  el marco, el mismo rectángulo que se ve y que se imprime.
- **La exportación usa la hoja del documento**: el tamaño personalizado del panel queda como
  excepción de una impresión, con la resolución y la lista de piezas.
- **Los archivos anteriores no se mueven**: un formato 6 se lee como A4 sin marco, que es el
  área en la que se dibujó; nada pasa a estar «fuera» por abrirlo con esta versión.
- **El ejemplo se compone para A4 vertical**: el cuadrado de cada símbolo ocupa 0,17 del
  ancho, no 0,20, para que las cuatro cajas entren y se puedan repartir con huecos positivos.
- **Evidencia**: `a_new_document_is_a_framed_a4_and_one_undo_restores_it`,
  `a_version_six_sheet_reads_as_an_unframed_a4`,
  `the_sheet_keeps_its_paper_orientation_and_margins_in_the_file`,
  `a_sheet_whose_frame_leaves_no_drawing_space_is_refused`, las pruebas del ABI
  `the_sheet_is_a4_and_a_paper_change_is_one_undo`,
  `the_check_measures_the_document_sheet_and_not_a_fixed_area` y
  `the_sheet_survives_a_save_and_an_open`, y los nueve pasos `sheet: …` de `--selftest`; la
  puerta completa queda en 338 pruebas, `--check-qml` 0 problemas y `--selftest` 0 fallos.

**Siguiente**: el tamaño personalizado como hoja de trabajo, una hoja por circuito, el cuadro
de rotulación en el lienzo y la escala elegida por el usuario. El marco de ISO 5457 todavía no
dibuja las marcas de centrado ni la cuadrícula de zonas.

### El zoom responde a la rueda y la hoja se desplaza — 2026-11-13

Ctrl y la rueda acercan la hoja allí donde está el puntero, que es el gesto que la mano
prueba sin que nadie lo explique; la rueda sola desplaza, el botón central arrastra y las
flechas hacen lo mismo desde el teclado.

- **Anclado al puntero**: el punto de la hoja que está bajo el cursor se queda quieto
  mientras cambia la escala, entre 0,4× y 6× y en los pasos de un cuarto de los botones.
- **Desplazamiento acotado**: el borde de la hoja se detiene en el marco, así que una
  sacudida de la rueda no deja al lector mirando el fondo.
- **Teclado**: flechas para desplazar; `Ctrl+=`, `Ctrl++`, `Ctrl+-`, `>` y `<` para la
  escala; `Ctrl+0` para volver a 1:1 y soltar el desplazamiento.
- **El estado de vista no es documento**: ni el zoom ni el desplazamiento entran en el
  historial ni marcan la hoja como modificada.
- **La barra de estado dice la ampliación** en tanto por ciento.
- **Evidencia**: los nueve pasos `view: …` de `--selftest`, que anclan el punto bajo el
  puntero, desplazan con rueda, flechas y botón central, y comprueban que nada entra en el
  historial; la puerta completa (326 pruebas, `--check-qml` 0 problemas).

**Siguiente**: la hoja dejará de ser un área lógica de 200 × 130 mm y pasará a ser el papel
elegido (ISO 216: A0–A5, Letter, Legal, Oficio y tamaño propio, A4 por defecto), con el
marco de ISO 5457, para que lo que se ve sea lo que se imprime.

### El relé enclavado: un pulso que se convierte en memoria — 2026-11-13

Un arranque que sigue funcionando cuando el dedo suelta el pulsador, y un paro que se queda
abierto: las dos cosas piden memoria. El relé enclavado es el primer relé cuyo estado
sobrevive al gesto.

- **Dos bobinas y un contacto**: `reset = ["R1", "R2"]` opcional sobre el mismo `relay`; la
  de set cierra, la de reset abre, y con ninguna viva el contacto conserva el estado
  ([ADR 0025](docs/adr/0025-a-latching-relay-holds-its-last-pulse.md)).
- **Seis terminales en tres parejas**: la validación acepta cuatro o seis y rechaza una
  pareja compartida o ausente.
- **El enclavamiento vive en el mismo punto fijo**: cada pasada lee las bobinas, actualiza el
  estado y vuelve a llenar las aristas; la topología cacheada mantiene el paso sin memoria
  dinámica.
- **El estado es del ensayo**, no del documento.
- **Evidencia**: `a_latching_relay_holds_until_the_reset_coil` (set, suelta, mantiene, reset,
  suelta, mantiene abierto), la revisión del símbolo y el lint.

### La familia manual: el panel de mando — 2026-11-13

Un panel no es un solo pulsador: la seta debe abrir al golpearla y quedarse abierta, el
selector debe quedarse donde se gira, y un pulsador cerrado en reposo es la mitad que el otro
pulsador no cubre.

- **Cuatro archivos, un comportamiento**: interruptor mantenido, selector, seta con
  `normally_closed = true` y pulsador NC. Todos son `contact`; el clic mantiene el estado y
  el reposo lo declara el archivo.
- **La seta es un contacto NC enclavado**: abre al pulsarla y no vuelve hasta el clic
  siguiente, que es lo que un paro de emergencia debe hacer.
- **La comprobación y el dibujo leen el reposo NC**: un contacto cerrado en reposo conduce
  desde el primer fotograma, sin que nadie lo toque.
- **Evidencia**: `a_normally_closed_contact_opens_when_clicked`, las cuatro revisiones de
  símbolo y el lint del catálogo.

### Presostato y vacuostato: la línea también decide — 2026-11-13

Una secuencia no solo mira dónde está el vástago: mira si el depósito se ha cargado, si la
pinza ha agarrado, si la ventosa ha hecho vacío. El presostato es el contacto que responde
desde la línea en la que está enroscado.

- **Dos terminales y un puerto de trabajo**: `threshold_bar` en bar de presión relativa,
  `above = true` cierra al llegar al umbral y `above = false` cierra por debajo, que es el
  vacuostato con un umbral negativo ([ADR 0024](docs/adr/0024-a-pressure-switch-reads-its-own-port.md)).
- **Sin binding**: lee su propio puerto con `Network::port_pressure`; no hay fuente que
  elegir ni identidad que guardar, porque el puerto es la línea.
- **El estado es del ensayo**: la comprobación de diseño lee el reposo; cada `advance` lee la
  presión y vuelve a asentar la red.
- **La barra la dibuja el renderizador**; el círculo sensor, el vástago y el enlace a trazos
  están en el archivo.
- **Evidencia**: `a_pressure_switch_follows_its_line_up_and_down` (sube con la válvula
  accionada y baja al descargar), las dos revisiones de símbolo y el lint.

### El final de carrera: la secuencia sabe dónde está el vástago — 2026-11-13

Una secuencia no solo necesita manos, bobinas y tiempo: necesita saber dónde está la máquina.
El vástago está fuera, así que el paso siguiente puede empezar. El final de carrera es la
primera pieza eléctrica cuyo estado viene del modelo del gas.

- **Dos terminales y un rodillo**: `end = "extended"` o `"retracted"`, con `normally_closed`
  para el contacto que cierra con el rodillo libre ([ADR 0023](docs/adr/0023-a-limit-switch-follows-a-cylinder-end.md)).
- **El cilindro se elige, no se adivina**: el inspector escribe la misma fuente que un
  registrador, con la cantidad `position`, así que no hay campo nuevo ni migración; el
  selector lista solo cilindros.
- **Cierra en el extremo, no cerca**: extensión `>= 0.98` o `<= 0.02`, los mismos márgenes
  con los que la telemetría llama «fuera» o «recogido» al vástago.
- **El estado es del ensayo**: la comprobación de diseño lee el reposo, porque no ha habido
  carrera; cada `advance` lee la extensión y vuelve a asentar la red.
- **Evidencia**: `a_limit_switch_closes_at_the_end_of_its_cylinder` (raíl, final de carrera,
  lámpara, válvula y 500 pasos), las dos revisiones de símbolo y el lint.

### El zumbador: una alarma que suena mientras la línea está viva — 2026-11-13

Una luz dice «mira»; un zumbador dice «actúa». El zumbador es la primera pieza cuya salida se
oye y la primera que dura: la ventana tiene que saber cuándo empieza y cuándo para.

- **Un terminal y la misma red booleana**: `type = "buzzer"`, cargado por el mismo
  asentamiento que la lámpara; no añade aristas y el circuito se comporta igual con él o sin
  él ([ADR 0022](docs/adr/0022-an-audible-indicator-owns-its-sound.md)).
- **La hoja lo dice con palabras**: `sonando` o `en silencio`, a través de `Reading::Audible`,
  además del cuerpo y las ondas del archivo.
- **El ABI responde una sola pregunta**: `pf_session_buzzer_active` devuelve 1 si suena algún
  zumbador y 0 si ninguno; la ventana consulta una vez por refresco, no una por pieza.
- **El sonido vive en la ventana**: la sesión Qt expone `buzzerActive`, emite
  `buzzerChanged` solo al cambiar y el QML enlaza un `SoundEffect` en bucle; el silencio del
  usuario lo detiene y, al quitar el silencio, vuelve solo si sigue sonando.
- **La muestra se genera**: `scripts-dev/make-sounds.py` escribe `buzzer.wav` desde una
  semilla fija, con fundamental y armónicos impares y un número entero de ciclos por segundo
  para que el bucle no chasquee; `--check-sounds` prueba que el binario la decodifica.
- **Evidencia**: `a_buzzer_sounds_while_its_line_is_live`, `--check-sounds`, la revisión del
  símbolo y el lint del catálogo.

### La lámpara indicadora: ver la línea sin mover nada — 2026-11-13

Una línea que solo alimenta una lámpara no tiene parte móvil, así que el estudiante no podía
ver si estaba viva. La lámpara es el resultado visible más pequeño del circuito de mando y la
primera pieza que un lector comprueba de un vistazo.

- **Un terminal y nada más**: `type = "lamp"`, con la validación que rechaza un segundo
  puerto o un rol neumático ([ADR 0021](docs/adr/0021-an-indicator-reads-the-net.md)).
- **Encendida significa que su terminal está vivo**, con el mismo asentamiento que los relés.
  La lámpara no añade aristas: observa la red, no la conduce, y el circuito se comporta igual
  con ella o sin ella.
- **El estado no es solo color**: una lámpara encendida gana ocho rayos alrededor del círculo
  que declara su archivo y la palabra «encendida»; apagada no tiene ninguno y dice «apagada».
- **La interfaz puede preguntar**: `pf_session_indicator_active` devuelve 1, 0 o error, para
  que la barra de estado y el futuro zumbador lean el mismo estado que la hoja.
- **El zumbador va aparte**: su sonido continuo necesita el camino de audio y sus propias
  pruebas; queda en la cola como capacidad propia y no se declara hecho a medias.
- **Evidencia**: `a_lamp_lights_while_its_line_is_live`,
  `a_lamp_shows_its_state_as_rays_not_only_colour`, la revisión del símbolo y el lint.

### Temporizadores TON y TOF, contados por el paso — 2026-11-13

Un pulsador que actúa en el acto no puede arrancar una secuencia después de un tiempo de
llenado ni mantener un motor tras el botón de paro. Los dos temporizadores de relé resuelven
esas dos preguntas sin dormir el modelo ni leer el reloj de la máquina.

- **El temporizador es un relé con una regla de tiempo**: `kind = "ton"` (cierra tras el
  retardo con la bobina viva) o `"tof"` (abre tras el retardo con la bobina muerta), con
  `delay_s` entre 0 y 3600 s ([ADR 0020](docs/adr/0020-timers-count-steps.md)).
- **El paso da el tiempo**: cada `advance(dt)` asienta la red, actualiza los temporizadores
  con `dt` y vuelve a asentar con los contactos nuevos. Un TON con la bobina muerta se
  reinicia en el acto; un TOF con la bobina viva cierra en el acto.
- **Sin asignar memoria por paso**: la topología de control se cachea al editar y el paso
  solo rellena listas con capacidad reservada; el tiempo transcurrido está acotado por el
  retardo.
- **El estado es del ensayo**: no se guarda; un circuito reabierto empieza con los
  temporizadores en reposo.
- **Evidencia**: `a_timer_declares_a_bounded_delay_and_an_instant_relay_does_not`,
  `a_ton_contact_closes_after_its_delay` y `a_tof_contact_opens_after_its_delay`, con pasos
  fijos de 10 ms, además de las dos revisiones de símbolo.

### El relé: una bobina que cierra su contacto — 2026-11-13

Un circuito de mando que solo puede conmutar una mano no puede encadenar dos líneas. El relé
es la pieza que deja que una señal mueva otra, y la primera realimentación de la red: su
contacto puede alimentar la bobina de otro relé, o la suya propia.

- **Un archivo, dos parejas**: `coil = ["A1", "A2"]` y `contact = ["11", "14"]` nombradas por
  el propio relé, porque los cuatro terminales comparten el rol eléctrico
  ([ADR 0019](docs/adr/0019-a-relay-is-a-coil-and-a-contact.md)).
- **La bobina se energiza cuando exactamente uno de sus terminales está vivo**, que es la
  sombra booleana de una diferencia de potencial; el contacto es una arista mientras la
  bobina está energizada (o mientras descansa, si el archivo lo declara cerrado).
- **La red se asienta en pasadas acotadas** (`2 · relés + 2`): un relé puede alimentar a otro
  o a sí mismo, y el circuito termina igualmente. La comprobación eléctrica usa el mismo
  asentamiento, así que un cortocircuito a través de un contacto de relé también se ve.
- **El estado es del ensayo**, no del documento: no se guarda y un relé no se pulsa con el
  ratón; su contacto se mueve porque su bobina está viva.
- **La barra la dibuja el renderizador**, como la del pulsador, y el enlace mecánico va a
  trazos.
- **Evidencia**: `a_relay_names_two_coil_and_two_contact_terminals`,
  `a_relay_closes_its_contact_while_its_coil_is_live` (raíl, pulsador, relé y válvula de
  solenoide), la revisión del símbolo y el lint del catálogo.

### La novena comprobación: el raíl en cortocircuito — 2026-11-13

La referencia cierra sus nueve comprobaciones con una etiqueta que no coincide, porque allí un
contacto se ata a su bobina por nombre. Aquí el contacto y la bobina van unidos por un cable,
así que esa falta no existe; la equivalente bajo nuestras reglas es eléctrica y se puede
señalar antes de ejecutar.

- **El positivo del raíl unido a su retorno** —por cables y contactos cerrados— se señala como
  `short_circuit`, en la fuente y antes de ejecutar. La red booleana lo decide sin simular:
  terminales como nodos, cables y contactos cerrados como aristas.
- **Un contacto abierto es un hueco**: el pulsador en reposo no convierte en cortocircuito lo
  que el ensayo todavía no ha cerrado.
- **La comprobación queda completa**: 9 de 9 en nuestro modelo, con la falta de etiquetas
  registrada como no aplicable y su razón escrita.
- **Evidencia**: `a_rail_joined_to_its_return_through_a_closed_contact_is_a_short`,
  `a_live_rail_with_no_return_is_not_a_short`,
  `an_open_contact_in_the_path_is_a_gap_not_a_short`, y el contrato del CLI con el noveno
  nombre estable y su mensaje en español.

### El pulsador mueve la válvula — 2026-11-13

Con la biblioteca eléctrica sola, el pulsador era un dibujo: se cableaba el raíl al pulsador
y el pulsador a la bobina, y no pasaba nada. Ahora el circuito de mando funciona.

- **La red de control es booleana** ([ADR 0018](docs/adr/0018-the-control-circuit-is-a-boolean-net.md)):
  la fuente marca vivo su positivo, la línea se propaga por cables y **contactos cerrados**, y
  una **válvula de solenoide** queda accionada mientras su bobina está viva. Sin tensión, sin
  corriente y sin cortocircuito: un modelo que el estudiante puede comprobar a mano.
- **El contacto se pulsa con un clic** y la válvula se mueve en el acto; el estado es del
  ensayo, no del documento, así que un circuito reabierto empieza con el pulsador en reposo.
- **La barra del contacto la dibuja el renderizador**: arriba cuando está abierto, sobre los
  terminales cuando está cerrado, con la misma regla que mueve un pistón.
- **Válvula 3/2 de solenoide** como archivo nuevo: el mismo cuerpo 3/2, las espiras al lado
  opuesto al muelle y el terminal de bobina donde llega el circuito de mando.
- **Evidencia**: `a_contact_energises_the_solenoid_it_is_wired_to` (raíl, pulsador y bobina,
  con la válvula siguiendo en ambos sentidos), la prueba del dibujo abierto/cerrado, nueve
  pasos `--selftest` `control: …` y la revisión del símbolo.

### La biblioteca eléctrica empieza — 2026-11-13

Un esquema neumático es medio mecanismo: la válvula la mueve un solenoide y el solenoide un
pulsador. Sin el circuito de mando, el estudiante ha dibujado la mitad de la máquina.

- **Fuente de 24 V y pulsador** como archivos, con sus símbolos dibujados a mano y sus
  revisiones. El terminal es un **rol de puerto**, `electrical`, no un tipo de componente:
  bobinas, contactos y raíles lo declaran y el catálogo no necesita un formato nuevo.
- **El cable es punteado**: una manguera entre dos terminales se dibuja con la clase
  `electrical` sin que nadie elija una clase, y el cuerpo del símbolo queda sólido.
- **La comprobación deja los terminales en paz**: un terminal sin cable no es aire que
  escapa; la falta de conexión mal encajada llegará con el resto de la biblioteca.
- **El solver no cambia**: el circuito de mando se dibuja y se comprueba, y el aire se simula
  exactamente igual que sin él. El modelo de control —línea viva, contacto que cierra y
  bobina que mueve la válvula— es la rebanada siguiente, con su ADR.
- **Evidencia**: el cable punteado, el terminal que no es falta, la prueba de que el aire no
  cambia y la validación de los dos terminales del contacto; más las dos revisiones.

### La hoja se configura antes de imprimirse — 2026-11-12

La exportación salía siempre en A4 con 18 mm de marco. Un taller pide otra hoja, un trazador
un tamaño propio, una diapositiva menos resolución, y un esquema rápido no necesita lista de
piezas.

- **Archivo ▸ Hoja ▸ Márgenes, tamaño y lista…** abre el panel: papel con nombre o
  **personalizado en milímetros**, cuatro márgenes, resolución del PNG y la **lista de
  piezas** sí o no.
- Las opciones viajan al núcleo en **una estructura** (`PfExportOptions`), no en una lista de
  argumentos que crece con cada campo; el ABI sube a la versión 5.
- Una hoja cuyos márgenes no dejan sitio al dibujo, o un tamaño fuera de los límites, se
  **rechazan con un mensaje** en vez de imprimir una página vacía.
- **Evidencia**: `the_export_panel_settings_reach_the_page` y los tres pasos `--selftest`.

### Las mangueras doblan una vez cuando pueden — 2026-11-12

El trazador automático siempre dibujaba una Z, aunque los dos puertos no la pidieran: una
manguera que salía de un costado y entraba por abajo daba dos quiebres y se veía como una
escalera.

- **El trazado lee la dirección de cada puerto**: un puerto lateral y uno inferior se unen
  con **un solo doblez**; dos puertos en lados paralelos conservan la Z, que es el camino
  ortogonal más corto entre ellos y el que el usuario puede arrastrar.
- Una manguera con un solo doblez **no tiene tramo central** que mover, y el arrastre lo dice
  en vez de no hacer nada.
- **Evidencia**: las pruebas de trazado del dibujo, las de ABI de ruta y guardado, y los pasos
  `--selftest` que encuentran una manguera arrastrable.

### La hoja de arranque, con círculos de conexión y fuentes que apuntan a su tubo — 2026-11-12

Dos correcciones de simbología que se veían en la hoja:

- **Cada puerto lleva su círculo de conexión**, el pequeño círculo abierto que dice al
  principiante dónde llega la manguera. Lo dibuja el renderizador para todos los símbolos y
  lo rellena con el color del papel, así que el último milímetro del tubo pasa por detrás y
  la conexión se lee como un terminal, no como una línea que termina.
- **Los triángulos de las fuentes apuntan al tubo**: la red y el compresor tenían el vértice
  arriba y el puerto abajo, de modo que la fuente parecía entregar hacia arriba, lejos de su
  propia conexión. Ahora el vértice está en el vástago y el punto es donde se conecta.
- **La plantilla de arranque es la composición del esquema**: la fuente abajo a la
  izquierda, la unidad de mantenimiento a su derecha, la válvula al centro-derecha y el
  cilindro arriba a la derecha, con el aire leyéndose de izquierda a derecha por abajo y
  subiendo al cilindro.
  Los rótulos flotantes del ejemplo desaparecen de la hoja editable: el documento tiene
  designaciones, el inspector y la barra de estado, y aquellos rótulos se cruzaban con las
  mangueras de la nueva composición.
- Las 18 revisiones de símbolos se regeneraron con las capturas claro/oscuro, y la de las
  dos fuentes registra el vértice y el círculo.
- **Evidencia**: `every_port_carries_its_connection_circle`,
  `a_source_triangle_points_at_its_connection`, la revisión visual de las dos fuentes y el
  gate completo.

### La hoja también sale en SVG — 2026-11-12

Un PDF se imprime y un PNG se pega en una diapositiva, pero ninguno se puede ampliar hasta un
símbolo, medir ni editar. La exportación ofrece ahora **SVG**.

- **El mismo dibujo, vectorial**: marco, bloque de identificación, lista de piezas y circuito,
  medidos en milímetros (`viewBox` del tamaño del papel), con el texto como texto.
- **Escritor propio y determinista**: sin dependencias, sin identificador ni fecha de
  generación; dos exportaciones de la misma hoja son el mismo archivo.
- **La misma página que el PDF**: una prueba compara los textos de los dos dispositivos, así
  que ninguno puede quedarse con una hoja distinta.
- El título se escapa (`&`, `<`, `>`), porque un nombre de archivo con un ampersand no puede
  romper el documento.
- **Evidencia**: cuatro pruebas del documento vectorial, la paridad PDF/SVG, la prueba ABI
  `an_export_can_be_a_vector_drawing` y cuatro pasos `--selftest`.

### La hoja impresa lleva su lista de piezas — 2026-11-12

Un plano del que no se puede construir no sirve: los símbolos dicen qué tipo de válvula es,
pero no cuántas hay, con qué designación ni con qué nombre.

- **La hoja exportada incluye una lista de piezas** en una columna encima del bloque de
  identificación: una fila por definición con su cantidad, el nombre del catálogo y las
  designaciones del grupo en orden de colocación.
- **La lista se agrupa por definición**: dos cilindros iguales son una fila de cantidad dos,
  que es como se lee una lista y lo que hace visible un cambio de cantidad.
- **La columna quita espacio al dibujo**, que ya se ajustaba a lo que queda: una lista larga
  encoge el esquema, nunca lo tapa.
- **Evidencia**: la prueba de agrupación, la de la hoja que dibuja cabecera, cantidad,
  nombres y designaciones, la de la columna que recorta el área de dibujo, y la prueba ABI
  que encuentra «lista de piezas» en el PDF exportado.

### Clases de conducto — 2026-11-12

Un esquema se lee por las líneas antes que por las etiquetas: un tubo de pilotaje y uno de
drenaje se distinguen por su trazo, y hasta ahora todos los tubos salían llenos.

- **Editar ▸ Conducto ▸ Trabajo / Pilotaje / Drenaje** cambia la clase del tubo
  seleccionado. El dibujo toma el trazo de la clase —lleno, discontinuo, discontinuo largo—
  y el tubo la guarda en el **formato 6**; un tubo de trabajo no se escribe, así que una hoja
  anterior se ve igual que siempre.
- **Un solo deshacer** devuelve la clase anterior, como el resto de las órdenes de edición.
- La clase eléctrica **no se ofrece todavía**: llega con la biblioteca eléctrica (F6), y el
  formato no la escribe hasta entonces.
- **Evidencia**: la prueba ABI que cambia la clase, guarda, abre y deshace; las pruebas de
  formato con la migración v5 y el trazo; y seis pasos `--selftest`.

### Tapón ciego y silenciador — 2026-11-12

Un esquema tiene que poder decir dos cosas que una manguera no dice: que un puerto está
**cerrado a propósito** y que un escape **respira por un silenciador**.

- **Tapón ciego** (`behaviour = "plug"`): un puerto, sin física propia. Cierra la línea y la
  comprobación deja de llamar «abierta» a esa conexión, que era un fallo que el estudiante
  no podía resolver.
- **Silenciador** (`behaviour = "silencer"`): un puerto de escape y una conductancia al
  ambiente. Ahora el escape pasa por esa restricción, así que un silenciador estrecho frena
  de verdad el retorno del cilindro. El ruido **no** se modela, y el archivo y su revisión
  lo dicen.
- **Los dos son archivos** ([ADR 0016](docs/adr/0016-a-termination-is-a-component.md)), con
  símbolo dibujado a mano y revisión: el tapón reutiliza la barra de puerto cerrado de las
  válvulas; el silenciador es un cono con dos líneas de alivio.
- **Evidencia**: la prueba del escape restringido, la del cilindro que queda retenido con el
  escape tapado, la de la comprobación que acepta el puerto tapado, las pruebas de catálogo
  y cuatro pasos `--selftest`.

### Repartir los objetos con huecos iguales — 2026-11-12

Un esquema dibujado a mano se reconoce por su ritmo: dos símbolos separados marcan la
distancia y el resto de la fila tiene que seguirla. Hasta ahora había que medir a ojo.

- **Editar ▸ Repartir ▸ Horizontal / Vertical** reparte tres o más símbolos marcados con
  **huecos iguales entre sus cajas** —no entre sus centros, que es lo que hace que un
  cilindro ancho y una válvula estrecha parezcan separados— y deja quietos los dos de los
  extremos, así que la hoja conserva su extensión.
- **Un solo deshacer** devuelve todos los centros, como en alinear y girar.
- Se niega cuando no hay nada que igualar (menos de tres) o cuando los símbolos no caben
  entre sus propios extremos, en vez de amontonarlos.
- **Evidencia**: pruebas de `doc` para los huecos, el deshacer y los rechazos; prueba ABI que
  mide las cajas y deshace; cuatro pasos `--selftest`.

### Un registrador de 60 s en la hoja — 2026-11-12

El manómetro y el caudalímetro dicen cuánto vale una magnitud ahora; ninguno dice cómo
llegó hasta ahí. El **registrador** es un componente que dibuja una magnitud elegida
durante el último minuto, con su valor actual y su curva, y viaja con el documento.

- **Un comportamiento propio** (`recorder`), sin puertos: observa, no se conecta. El archivo
  declara la ventana y la frecuencia de muestreo, y el catálogo acota su producto para que
  ningún archivo pida un búfer sin límite ([ADR 0015](docs/adr/0015-a-recorder-samples-a-named-quantity.md)).
- **La fuente se elige en el inspector** entre lo que el circuito puede informar —presión en
  un puerto, caudal de un caudalímetro, posición de un cilindro— y se guarda con la
  colocación en el **formato 5**, con migración por pasos para que un proyecto v4 conserve
  sus circuitos.
- **Las muestras no se guardan**: son del motor de simulación, no del documento. Al abrir un
  archivo el registrador empieza vacío, y una hoja abierta no crece con cada segundo.
- **La curva es del modelo**: el símbolo dibuja el papel y sus graduaciones, nunca una traza
  que parecería una medida que no existe. El marco sin fuente lo dice, en vez de pintar un
  cero.
- **El dinamómetro queda `not planned`**, con la razón escrita: medir fuerza exige un enlace
  mecánico que el modelo no tiene; una cifra `p·A` llamada fuerza enseñaría lo que no es.
- **Evidencia**: la prueba del anillo acotado, la prueba ABI que enlaza, dibuja y deshace,
  los pasos `--selftest` y la revisión del símbolo con capturas claro/oscuro.

### Un caudalímetro en la línea — 2026-11-12

Un manómetro dice la presión en un punto, no cuánto aire pasa por él. El nuevo
**caudalímetro** se coloca en serie y mide el gas que lo atraviesa, que es la segunda
magnitud que un estudiante necesita leer en un esquema neumático.

- **Un comportamiento propio** (`flow_meter`), no un manómetro con otra escala: el aparato
  tiene dos puertos, uno de entrada y otro de salida, y es la restricción que mide
  ([ADR 0014](docs/adr/0014-a-flow-meter-is-a-restriction-that-reports.md)).
- **La física es del archivo**: `conductance` declara la restricción ISO 6358 que añade y
  `nominal_l_min` la escala; un caudalímetro mayor es otro archivo, no otra rama de código.
- **La lectura sale del modelo**: la cifra se dibuja junto al símbolo en litros normales
  por minuto (0 °C y 1 013,25 hPa, el litro normal de ISO 6358), con signo según el sentido
del aire, y el inspector
  muestra la escala y la conductancia.
- **Evidencia**: la prueba de solver contra el cálculo a mano, la prueba ABI
  `a_flow_meter_reads_the_line_it_is_connected_to`, los pasos `--selftest` que lo colocan,
  lo conectan y leen su cifra, y la revisión del símbolo con capturas claro/oscuro.

Quedan pendientes el dinamómetro y el registrador de 60 s (P2.2).

### Un proyecto son varios circuitos en un documento — 2026-11-12

La referencia agrupa circuitos en un proyecto y comprueba que un conducto no circule por un
circuito al que no está conectado. El propietario eligió la forma del contenedor: **un solo
documento con varios circuitos y un selector**, no un archivo de proyecto que referencia
archivos de circuito.

- **ADR 0013** fija la forma: formato 4 con una lista de circuitos, colocaciones y tubos a
  nivel de proyecto con su circuito, una hoja activa y un solo historial de deshacer.
- **La novena falta se adapta**: como cada circuito es una hoja, el error equivalente es un
  tubo cuyos extremos están en circuitos distintos. El editor no lo crea y la comprobación
  lo señala como `PipeAcrossCircuits`.
- **La migración** lee un archivo v1–v3 como un proyecto de un circuito y guarda v4, con
  una prueba por versión anterior y una por el tubo que une dos circuitos.
- **La ventana** gana el grupo **Circuito** en Archivo: el selector marca el circuito en
  pantalla, **Nuevo circuito** añade una hoja vacía y la abre, y **Renombrar** cambia su
  nombre. El ABI añade, renombra, selecciona y lista circuitos; cambiar de circuito es un
  cambio de vista, no una entrada de deshacer.
- La exportación imprime el circuito activo y lo nombra en el bloque de identificación.
- La referencia se usa solo para el comportamiento; no entra su manual, símbolos, ejemplos
  ni formato.

Evidencia: las pruebas de migración v1–v3 de `punchi-fluid-file`, la prueba ABI
`circuits_can_be_added_renamed_and_survive_a_save_and_open`, la falta
`a_pipe_between_two_circuits_is_the_ninth_fault` y los pasos de `--selftest` que añaden,
renombran, guardan, abren y cambian de circuito.

### La exportación imprime la fecha real y encaja en la hoja — 2026-11-12

El plano salía con la fecha fija del 7 de noviembre, centrado a tamaño natural: un circuito
más grande que el área libre se recortaba, y el botón anunciaba `Ctrl+E`, que marca todo.
Tres defectos del mismo mando.

- **La fecha la pone quien llama**: `pf_session_export` recibe `date`, la ventana pasa la
  fecha local en ISO 8601 y `--export` usa el mismo camino. El ABI no lee el reloj, así que
  una exportación es reproducible dentro del día. La firma cambia, así que **el ABI sube a
  la versión 3**.
- **El dibujo se escala para encajar**: los límites incluyen los símbolos, las
  designaciones, las lecturas y los recorridos de los tubos; `Scaled` aplica el mismo factor
  a geometría, trazos y texto, y el bloque de título escribe la escala (`1:1`, `1:2`, …).
  Un dibujo pequeño conserva su tamaño y se centra.
- **`Ctrl+Shift+E` abre el diálogo**, y la pista del botón deja de mentir.
- **La nota de símbolos sin verificar vuelve a salir en la exportación de la ventana**: el
  ABI pasa por `block_with_review_state` en lugar de construir un bloque sin nota.

Probado con `an_export_carries_the_caller_date_and_fits_a_large_circuit` (fecha y escala en
el PDF por el ABI), las pruebas de `Scaled` y `bounds_placed`, la prueba de WinAnsi y la
puerta completa.

### El código deja de hablar español — 2026-11-12

La regla ya estaba en `CONTRIBUTING.md` —código, comentarios, pruebas y mensajes de commit
en inglés—, pero los mensajes de `--selftest` y varios diagnósticos seguían en español, y
los commits de las últimas fases se escribieron en español siguiendo el historial en vez de
la regla. Se corrige lo que se puede corregir sin reescribir `main`.

- **`--selftest` en inglés**: las 63 etiquetas de los pasos y sus mensajes de diagnóstico
  (`editing:`, `simulation:`, `unsaved:`, `designation:`, `recent:`, `T junction:`, …) y el
  `FALLO` de la puerta pasan a inglés. El comportamiento no cambia; el texto que lee quien
  ejecuta la herramienta, sí.
- **La prueba de WinAnsi** deja de usar una palabra española: comprueba `á` y `·` con un
  fixture neutro.
- **La regla es canónica** en `AGENTS.md` (nueva sección *Language*), `CONTRIBUTING.md`,
  `docs/style.md` y las skills de colaboración y pull request: inglés para código,
  identificadores, comentarios, logs, pruebas, scripts, asuntos y cuerpos de commit, títulos
  y cuerpos de PR y documentación. Quedan en español la interfaz (hasta F8), `CHANGELOG.md`,
  `docs/retomar.md` y `README.es.md`.
- **Lo que no se toca**: los commits ya mezclados (#27–#31) conservan su asunto en español
  en el historial; no se reescribe `main`. Desde ahora, asunto y cuerpo en inglés.

Validado con la puerta completa, `--selftest` y la prueba de codificación del PDF.

### El CI no repite Rust en un cambio de documentación — 2026-11-12

Un PR que solo tocaba `docs/` compilaba, pasaba clippy y ejecutaba las 244 pruebas. La
referencia de PunchiDock Quickshell resolvía esto con `paths-ignore` y un `docs.yml` aparte;
allí funcionaba porque `main` no estaba protegido. Aquí sí lo está, y un workflow saltado
por rutas deja sus checks en `Pending` y bloquea el merge, así que el patrón se adapta:

- **`changes`** clasifica el diff con `scripts-dev/changed-paths.sh`; el workflow se ejecuta
  siempre. La lista blanca es documentación, archivos comunitarios, capturas y guías de
  autoría; todo lo demás es código. El catálogo TOML, los sonidos y las revisiones de
  símbolos son código porque se compilan o los observan pruebas.
- **`check` siempre reporta**: un cambio de documentación corre `check-docs.sh` (índice,
  enlaces, ADR, crates y etiquetas); un cambio de código corre `check.sh` completo.
- **`window` y `minimum supported Rust version`** se saltan en un cambio de documentación.
  Un job que se ejecuta y se salta reporta éxito, que es lo que acepta la protección de
  `main`; un workflow saltado por rutas no reporta y bloquea.
- **`scripts-dev/test-changed-paths.sh`** prueba el clasificador dentro de `check.sh`; una
  respuesta `docs` equivocada saltaría pruebas que sí pueden observar el cambio.
- `docs/engineering.md`, `CONTRIBUTING.md` y `AGENTS.md` explican la regla. El empuje a
  `main` y el release siguen corriendo la puerta completa.

Validado con las pruebas del clasificador, la puerta completa y el YAML del workflow. La
rama de CI es un cambio de código, así que este PR corre todo; el camino de documentación
se confirma en el siguiente PR que solo toque `docs/`, como se hizo en la referencia.

### Guías para usar el programa — 2026-11-12

La referencia explicaba qué es cada cosa; faltaba el camino para hacerla. Un lector nuevo
tenía que deducir de la ventana cómo colocar, conectar, ejecutar y guardar.

- **`docs/guides/first-circuit.md`**: tutorial de diez minutos — abrir, colocar tres
  componentes, conectar puertos, nombrar el cilindro, ejecutar, comprobar y guardar, con los
  gestos y atajos reales.
- **`docs/guides/export-a-drawing.md`**: elegir papel y orientación, escribir PDF o PNG, la
  tabla de índices de `--paper`, el camino sin ventana y las deudas conocidas (fecha fija,
  sin escala para encajar, sin atajo).
- **`docs/guides/troubleshooting.md`**: síntoma, causa y salida — el ejemplo inicial, una
  tubería rechazada, una válvula que no responde, un catálogo ignorado, una hoja que no
  abre, la exportación recortada y las limitaciones de la interfaz.
- El índice enlaza las tres guías y la lista de pendientes desaparece: ya no hay documentos
  prometidos sin escribir.

No cambia el comportamiento: documenta el que existe, incluidas sus limitaciones.

### La referencia que el código ya tenía — 2026-11-12

El formato de hoja, la CLI y el catálogo se podían leer en el código y en el ADR, pero no
tenían una página de referencia. La consecuencia era un README que mandaba `validate` sobre
un `.pfluid`, y `validate` no lee hojas: lee el formato de circuito antiguo.

- **`docs/reference/sheet-format.md`**: el `.pfluid` versión 3 — versión, límites, campos de
  una colocación y de un tubo, ejemplo completo, qué no guarda y cómo se migra.
- **`docs/reference/cli.md`**: qué comando lee qué formato, códigos de salida, las seis
  faltas que la comprobación conoce, el contrato JSON con versión y un ejemplo de error.
  `check` lee la hoja; `validate` y `simulate` siguen en el formato antiguo.
- **`docs/reference/catalog.md`**: el mapa del catálogo — carpetas, campos, roles de puerto,
  comportamientos y lo que el cargador rechaza— con la guía de autoría como referencia
  canónica.
- **El README y README.es corrigen sus ejemplos**: usan `check` sobre la hoja y explican que
  `validate`/`simulate` son el camino antiguo; `run.sh` usa `--toggle` y
  `--theme light|dark` (no `--demo`, `--dark` ni `PUNCHI_FLUID_THEME`, que no existen) y
  aclara que `--toggle` no arranca la simulación. El índice lista las tres páginas y la
  lista de pendientes se acorta.

No cambia el comportamiento ni el formato: documenta el que ya existe.

### Una arquitectura documental, no una carpeta de archivos — 2026-11-12

La documentación tenía buenos documentos sueltos y ningún índice que dijera cuál era
vigente, cuál era especificación y cuál era historia. El README raíz repetía una tabla que
ya estaba en `docs/`, los ADR no tenían índice ni plantilla, y nada comprobaba que un
enlace llevara a un archivo existente.

- **`docs/README.md` es el índice único**: tipo de documento (tutorial, how-to,
  referencia, explicación, proceso, decisión), estado (`current`, `target`, `historical`,
  `internal`), orden de lectura por lector y la lista de lo que falta por escribir.
- **`docs/architecture.md`**: el mapa del sistema — crates y aristas permitidas, el flujo
  de hoja a red, dibujo y exportación, los tres puntos de extensión y las deudas que
  condicionan las próximas fases.
- **`docs/style.md` y `docs/glossary.md`**: cómo se escribe y se enlaza un documento, y el
  vocabulario del proyecto en inglés con la palabra que muestra la interfaz en español.
- **`docs/adr/README.md` y `docs/adr/template.md`**: el índice de las doce decisiones con
  estado y fecha, y la plantilla de la siguiente. Los ADR 0011 y 0012 ganan la cabecera de
  estado y fecha que les faltaba.
- **`check-docs.sh` comprueba la estructura**: todo documento está enlazado desde su
  índice, todo enlace relativo resuelve, los ADR son contiguos y declaran estado y fecha,
  y cada archivo empieza por un solo `H1`.
- **El README ya no duplica el índice**: enlaza a `docs/README.md` y conserva solo los
  archivos de contrato y comunidad. `AGENTS.md` apunta al índice y a la arquitectura.

No cambia el comportamiento ni el dibujo: es la estructura que hace que la documentación
se pueda leer, revisar y comprobar.

### La documentación vuelve a decir lo que el proyecto hace — 2026-11-12

El PR de la exportación actualizó el changelog y las reglas, pero no el libro de
capacidades ni la nota de continuidad: durante cinco días la documentación dijo que no
había editor, que el solver de red no existía y que la ventana no podía exportar. Una
sesión nueva habría empezado a arreglar cosas ya hechas.

- **P1.7 queda marcado como entregado** en `next-steps.md`, y la exportación aparece en el
  estado de `capabilities.md` y de `retomar.md` con sus dos deudas: la fecha del bloque de
  título está fija en 2026-11-07 y el dibujo se centra en vez de escalarse para encajar.
- **Se corrigen las contradicciones**: 6 de 9 faltas de la comprobación, no 5; formato de
  hoja v3, no v2; autoguardado, recientes y aviso de cambios sin guardar ya existían; el
  catálogo tiene 14 definiciones; el workspace tiene 10 crates; el total real es 244
  pruebas de Cargo.
- **El README deja de prometer un panel de exportación que no existe** (márgenes, tamaño
  personalizado, resolución y atajos `Ctrl+Shift+E/P`): ahora describe el diálogo de
  guardado, el grupo Hoja y las deudas. La sección de la ventana habla del transporte y de
  los atajos reales, no de la interfaz retirada.
- **`supply.workshop` deja de aparecer**: la definición se llama `supply.compressor` desde
  el juego de símbolos; se corrige en la hoja limpia del gate, en `next-steps.md` y en
  `retomar.md`, y la hoja de humo vuelve a nombrar un componente que existe.
- **`ui-ux.md` distingue spec de estado**: la barra lateral plana, el estado real de la
  barra de estado y el panel de exportación incompleto quedan declarados; los iconos Breeze
  dejan de figurar como pendientes porque ya se cargan y se tiñen.

Lo que no cambia: los símbolos siguen `pending-300dpi`, la lista de piezas y las
bibliotecas eléctrica/digital/GRAFCET no existen, y el modelo hidráulico no está empezado.

### Exportar el plano, desde la ventana — 2026-11-07

La exportación existía como crate y no como función: el marco, el block de título y los dos
escritores estaban hechos, y la ventana no tenía con qué pedirlos. La promesa de F5 llevaba
tres documentos diciendo que faltaba el mando.

- **El ABI gana la exportación**: `pf_session_export(sesion, ruta, tipo, papel, apaisado, dpi)`,
  más `pf_export_paper_count` y `pf_export_paper_name` para que el panel construya su lista desde
  la misma tabla que el escritor. Un papel fuera de límites o un índice que no existe se
  rechazan con el mensaje en el canal de error, no con una hoja a medias.
- **Lo que se imprime es lo que está en la hoja**: la exportación compone el papel y **la sesión
  dibuja su documento** —las mismas colocaciones, los mismos recorridos, las mismas
  designaciones—. La función que arma la escena es una sola y la usan la pantalla y el papel, así
  que no pueden separarse.
- **La cinta gana «Exportar»** en *Documento* y un grupo *Hoja* con los nueve papeles y la hoja
  apaisada; el papel en uso va marcado. La exportación arranca en el papel que esta máquina
  imprime —Oficio donde el sistema dice Chile, A4 en cualquier otra parte—, que es un punto de
  partida y no una decisión.
- **`--export <ruta> [--paper N] [--landscape] [--dpi N]`** permite probarlo sin ventana, y el
  gate lo usa: escribe un PDF y un PNG en Oficio apaisado y comprueba las cabeceras y el
  `/MediaBox`.

Con esto quedan dos huecos declarados: el tamaño personalizado no tiene panel, y la resolución
del PNG se queda en 300 dpi sin mando propio.


### La exportación dibuja lo que le den — 2026-11-07

El crate de exportación sabía componer una hoja pero sólo sabía dibujar en ella **el circuito
fijo**: la ventana no podía imprimir el plano que el usuario tiene delante, y por eso la
exportación seguía siendo una promesa.

- **`render_pdf_with` y `render_png_with`**: el marco, los márgenes, el block de título y los
  dos escritores siguen siendo del crate; **el dibujo lo pone quien llama**, con una función que
  recibe la superficie y el área útil. Así la sesión puede estampar su documento sin que
  exportación tenga que aprender qué es un documento ni ganar una arista de dependencia nueva.
- **`print_ink()` pasa a ser público**: quien dibuja su propia hoja necesita la misma tinta de
  papel —negro sobre blanco— que el circuito fijo.
- Tres pruebas: la hoja del llamador es un PDF de verdad, conserva el papel que se le dio
  (Oficio, medido en el `/MediaBox`), y dos veces el mismo dibujo da el mismo archivo.

Con esto, la pieza que falta para «Exportar» es el mando: la llamada del ABI y el botón de la
cinta. El trabajo pesado —componer la hoja y dibujar el documento dentro— ya no tiene dueño
ambiguo.


### El juego de papeles, en el código y no solo en el ADR — 2026-11-07

El ADR 0012 prometía ISO A4–A0, Letter, Legal y Oficio, y el código conocía A5, A4, A3, A2,
Letter y Legal. Un plano para imprimir en Oficio no se podía declarar.

- **A1 y A0 entran**, con sus medidas: 594 × 841 y 841 × 1189 mm.
- **Oficio entra**, 216 × 330 mm. No es «un A4 un poco más largo»: es más largo que A4, más
  corto que Legal y **0,1 mm más ancho** que Legal, tan cerca que se confunde con él. El ADR
  decía «más estrecho que Legal» y era falso; queda corregido.
- **Tamaño personalizado**: `PaperSize::custom(ancho, alto)` devuelve `None` fuera de los
  límites (50–2000 mm por lado) o con una medida que no es un número. El rechazo ocurre donde
  el llamador aún puede hacer algo con él, antes de reservar el búfer del ráster.
- **El tamaño elegido sigue escrito en el archivo**: la prueba del `/MediaBox` cubre ahora
  Oficio y A0 apaisado, no solo A4, A3 y Letter.
- Cuatro pruebas nuevas: cada tamaño con nombre es el papel que dice ser; el contenido no se
  sale del papel en ninguno; un tamaño fuera de límites se rechaza antes de dibujar; y Oficio
  se distingue de Legal por lo que de verdad los distingue.


### El juego de símbolos neumáticos, dibujado como la norma lo dibuja — 2026-11-07

La fuente era un círculo con un triángulo y nada más, y no había ni filtro, ni lubricador, ni
unidad de mantenimiento. Ahora el juego entero está en el catálogo, con las proporciones de la
forma y no las que salen de una casilla cualquiera.

- **Fuente de aire comprimido** (`supply.network`): el triángulo hueco sobre un vástago, que es
  la conexión a la red cuando el compresor está dibujado en otra parte.
- **Compresor** (`supply.compressor`): el mismo triángulo dentro del círculo, con la entrega
  saliendo por abajo y terminando en el círculo de conexión.
- **Filtro** (`filter.air`), **filtro con purgado manual** (`filter.air_manual_drain`) y
  **unidad de lubricación** (`lubricator.oil`): el cuadrado girado 45°, con el elemento en línea
  de trazos y el purgador en el filtro con drenaje; el lubricador **no** lleva trazos, porque no
  filtra: esa ausencia es la diferencia.
- **Unidad de mantenimiento** (`preparation.service_unit`) y su **forma simplificada**
  (`preparation.service_unit_simple`): marco de trazos alrededor de las tres funciones en el
  orden en que el aire las encuentra; el regulador como cuadrado con las dos flechas enfrentadas,
  su muelle encima y la **flecha diagonal** que dice que el ajuste es regulable; el dial como el
  manómetro que es, sobre un vástago que toma la línea después del regulador. La simplificada es
  el cuerpo con el elemento de trazos, el dial dentro y el drenaje saliendo por abajo.
- **Ninguna figura se deforma**: todas se dibujan en una casilla cuadrada —la única excepción es
  el cilindro, cuyo barril es un rectángulo y sigue siendo un rectángulo— y cada cuadrado se
  declara con lados iguales. La casilla de la unidad de mantenimiento pasa a ser cuadrada, y las
  miniaturas de revisión usan la misma proporción que la hoja, así que una deformación ya no
  puede esconderse en la evidencia.
- **Un restrictor declarado en un archivo ahora conduce**: el modelo crea la arista entre sus
  dos puertos con la conductancia que el archivo declara. Antes, poner una unidad en serie
  cortaba la línea: el cilindro no recibía aire y la prueba que comparaba «con restrictor»
  contra «sin él» pasaba por el motivo equivocado.
- Las reglas de forma, proporción y punto de conexión quedan en
  [docs/symbology.md](docs/symbology.md), con la razón de cada trazo.


### Los puertos del cilindro, en su tapa — 2026-11-07

Tres formas que estaban dibujadas de memoria y no como la norma las dibuja, y una evidencia
que no probaba nada porque era más vieja que el código.

- **Un puerto por tapa, saliendo hacia abajo de su propia tapa.** El cilindro de doble efecto
  tenía uno en la tapa y el otro en el borde superior del barril, donde se leía como parte del
  contorno y se cruzaba con el vástago; el de simple efecto tenía el suyo en la cara lateral.
  Ahora los dos salen hacia abajo y a la misma profundidad, así el par se lee de un golpe como
  un cilindro mandado por los dos extremos. El de simple efecto conserva su único puerto en la
  tapa, y el extremo del vástago sigue sin ninguno, que es lo que lo hace de simple efecto.
- **El numeral de un puerto va por fuera de su propio rabo.** El 1 y el 3 de la válvula se
  escribían uno al lado del otro entre sus dos rabos y se leían como un puerto con dos nombres.
- **El pistón cruza el diámetro completo**, en los dos cilindros: iba retranqueado un 10 %
  arriba y abajo y se leía como una barra flotando dentro del barril. El pistón es el sello
  entre las dos cámaras y toca las dos paredes en cualquier punto de la carrera.
- **El muelle del cilindro de simple efecto va en tinta llena**, con el mismo grosor que las
  demás líneas funcionales: una línea gris no es una clase de línea en ISO 1219, y es lo
  primero que desaparece en una fotocopia.
- **La palanca es un brazo con empuñadura cruzada**: el brazo sale de la cara del cuerpo en
  ángulo y termina en una barra perpendicular, más fina y más larga. Antes era un vástago que
  giraba en gancho con un borrón en la punta, indistinguible de un fiador o de un pedal.
- **Los accionamientos se dibujan como la tabla de la norma los dibuja**: la palanca lleva su
  círculo de pivote; el pulsador termina en barra de extremos redondeados; el botón de seta en
  cúpula; el pedal en placa inclinada; el accionamiento por pistón es una cúpula apoyada en el
  cuerpo, sin vástago; el rodillo lleva su eje marcado; y los pilotos neumático e hidráulico van
  en caja con triángulo, hueco para aire y relleno para aceite. La tabla queda en
  [docs/symbology.md](docs/symbology.md).
- **Las imágenes de revisión se regeneran**: las que estaban commiteadas eran más viejas que
  el código que decían probar (13,9 KB contra 4,2 KB del mismo símbolo).
- La regla queda en [docs/symbology.md](docs/symbology.md) y tres pruebas la sostienen:
  `each_port_of_a_cylinder_is_drilled_in_an_end_cap` (un puerto por tapa, ninguno en el
  extremo del vástago), `the_piston_meets_both_walls_of_the_bore` y
  `the_return_spring_compresses_with_the_stroke`.

### El papel del plano y la norma chilena, dichos — 2026-11-07

Dos cosas que el proyecto daba por sabidas y no estaban escritas. Un plano se imprime en una
hoja concreta, y esa hoja no es una sola; y la simbología tiene una adopción nacional que un
lector en Chile aplica.

- **El papel es una decisión** ([ADR 0012](docs/adr/0012-the-paper-a-drawing-is-printed-on.md)):
  ISO A4–A0 con marco y márgenes de ISO 5457, Letter y Legal, y **Oficio/Folio 216 × 330 mm**,
  que no es ninguna de las dos y es el papel que hay en la impresora. El dibujo se escala para
  entrar, nunca se recorta, y el tamaño elegido queda escrito en el archivo.
- **Se acabó la promesa falsa**: los dos README decían «papel de A5 a A2 más Letter y Legal»
  cuando el crate conoce A4 y A3 y la ventana no tiene mando de exportación. Ahora dicen lo que
  hay y lo que falta.
- **La adopción chilena aparece**: **NCh-ISO 1219** en la sección de simbología de los dos
  README, y la regla de que un símbolo no se declara conforme hasta que la adopción que le toca
  esté confirmada en su revisión.


### Comprobar un circuito sin abrir la ventana — 2026-11-07

La comprobación existía en la ventana y sólo en la ventana: corregir cuarenta prácticas era
abrir cuarenta ventanas, y del resultado no salía nada que una máquina pudiera leer. La
referencia tampoco puede ofrecerlo, porque su formato y su comprobación viven dentro de su
programa.

- **`punchi-fluid-cli check <archivo> [--json] [--strict]`**: lee el formato de hoja que
  guarda el editor —lo que el alumno entrega— y devuelve las mismas faltas que la ventana
  dibuja, en el mismo orden y con las mismas palabras.
- **Contrato versionado**: con `--json` imprime un solo objeto con `version`, `file`, `count`
  y un elemento por falta, y con la misma forma cuando el archivo no se puede leer. Lo fija
  [ADR 0011](docs/adr/0011-the-check-is-a-public-command.md).
- **Códigos de salida que un trabajo de CI puede usar**: `0` comprobado, `1` ilegible, `2` línea
  de órdenes incorrecta, `3` hay faltas y se pidió `--strict`. Sin `--strict` la lista es un
  dato, no un fallo, para que nadie tenga que capturar un error para enseñar avisos.
- **Probado en dos niveles**: cinco pruebas del contrato (hoja limpia, falta con y sin
  `--strict`, forma del JSON, archivo roto, orden incorrecta) y el smoke test del gate, que
  exige que una hoja limpia pase y una con faltas falle con `3`.

### Por qué existe: el coste de una carrera industrial — 2026-11-07

El README decía para quién es el programa, pero no de dónde viene. Ahora lo dice, y lo dice
encadenando la causa en vez de lamentarla.

- **El precio de entrada**: en Latinoamérica el software que un curso de mecánica o
  electricidad espera que uses cuesta como cuesta para una empresa y suele ir atado a la
  marca de las máquinas del laboratorio; las licencias de estudiante y de docencia escasean,
  así que el curso enseña *alrededor* de la herramienta.
- **La consecuencia, sin culpar a quien la sufre**: el estudiante busca una copia pirata
  antes que reconocer que no puede pagar, y esa copia trae su propia factura —sin
  actualizaciones, con malware en el portátil personal, con credenciales y documentos
  expuestos—. La piratería es un síntoma; la salida honesta es una herramienta que no cuesta
  instalar ni ejecutar.
- **El valor en el taller**: un operario que ha visto llenarse una cámara y cebarse el
  caudal lee un cilindro lento como diagnóstico; un equipo que instala el simulador en cada
  máquina puede avanzar hacia el mantenimiento autónomo sin comprar una licencia por puesto.
- **Declarado como motivo, no como resultado**: el programa es un prototipo y el ledger dice
  lo que falta. Sin cifras inventadas ni promesas de TPM medidas.

### El accionamiento y el muelle de la válvula, a tamaño que se lee — 2026-11-07

La revisión visual marcó los dos extremos de la válvula 3/2. Se dibujaban pegados al borde de
la casilla y al 9-10 % del lado del cuerpo, así que a tamaño de paleta eran un gancho y una
raya: ni se distinguía la palanca con su mango ni el muelle con sus espiras.

- **Tamaño proporcional al cuerpo, no a la línea**: el paso y la amplitud del muelle y el
  alcance del accionamiento suben al 13 % y al 20 % del lado menor del cuerpo, de modo que
  cada espira y el mango se leen en la barra lateral y en la hoja.
- **Fuera del cuerpo**: accionamiento en un extremo y muelle en el otro, ambos arrancando
  claros del borde, que es lo que hace que cada uno pertenezca a su posición.
- **Las marcas de bloqueo siguen donde estaban**: dentro de su casilla, sobre la línea del
  puerto que esa posición cierra, para que las dos posiciones se puedan comparar.

### Los archivos de comunidad, con regla y sin puertas falsas — 2026-11-07

El repositorio tenía la nota de comunidad de GitHub al 100 % y, aun así, tres puertas que no
llevaban a ninguna parte. La nota mide presencia, no utilidad.

- **La regla de sitio, escrita** en [docs/engineering.md](docs/engineering.md): la raíz
  guarda lo que una persona lee —README, licencia, conducta, contribución, seguridad,
  soporte, registro y reglas de los agentes— y `.github/` lo que GitHub lee de una ruta
  fija. GitHub acepta también `.github/` y `docs/`, así que mover el código de conducta no
  mejora nada y rompe los enlaces: se queda donde está, y ahora está dicho.
- **Discussions se activa** y el contacto «Question or idea» deja de dar 404; la wiki vacía
  se desactiva.
- **Las etiquetas de las plantillas existen**: se crea `needs-reproduction` y el formulario
  de propuestas usa `enhancement`. Antes el issue nacía sin etiqueta y el triaje empezaba de
  cero.
- **`SUPPORT.md`**: dónde preguntar, qué promete el proyecto —respuesta a todo, gate y
  prueba como barra— y qué no —plazo de respuesta, fecha de entrega, soporte comercial—.
  Enlazado desde los dos README, `CONTRIBUTING.md` y el selector de issues.
- **`.github/FUNDING.yml`** con la donación que ya existe, y la frase que importa: es
  opcional y nunca necesaria para usar, compilar o redistribuir.
- **`check-docs.sh` lo vigila**: los archivos de comunidad existen, ninguna plantilla cita
  una etiqueta fuera de `.github/labels.txt` y un enlace a Discussions falla si el
  repositorio lo tiene apagado (con `gh` y token; en local avisa).

### El muelle del cilindro deja ver el cilindro — 2026-11-07

La revisión visual del cilindro de simple efecto encontró el defecto en el dibujo, no en el
modelo: el muelle de retorno se dibujaba a lo largo de **toda** la cámara, de modo que en
reposo el símbolo era un rectángulo lleno de alambre y el émbolo quedaba pegado a la tapa sin
leerse.

- **Muelle corto contra la tapa del vástago**: como mucho la mitad de la cámara y nunca más
  del 22 % del ancho de la barra, con un mínimo de seis trazos de línea para que siga leyéndose
  como espira. Lo que queda libre entre émbolo y tapa es exactamente lo que hace visible que
  el émbolo viaja.
- **Sigue comprimiéndose**: mismo alambre, menos paso y algo más de amplitud al acercarse el
  émbolo, con el tope de dos espiras que ya existía. A plena carrera el muelle queda reducido a
  una raya contra la tapa, que es lo que un muelle comprimido parece.
- **Contrato y evidencia**: la regla está escrita en [docs/symbology.md](docs/symbology.md) y
  comprobada con capturas a reflujo total (muelle corto, émbolo visible) y a plena carrera
  (muelle comprimido, vástago fuera).

Queda pendiente, en su propio cambio, la revisión del accionamiento y el muelle de la
válvula 3/2: misma clase de defecto —geometría que no se lee a tamaño de paleta—, otra
geometría.

### Nombrar un componente: la designación — 2026-11-12

Un esquema se lee por los nombres: *1A1* dice qué cilindro es, y sin nombres la lista de
piezas imprime tres cilindros iguales. El manual lo pide en el diálogo de propiedades y lo
usa en su séptima falta de la comprobación: *cilindros con la misma designación*.

- **Un nombre por instancia**: `Placement::designation`, con un solo deshacer, recortado, de
  cuarenta caracteres como mucho y rechazado —no truncado— si no se puede leer. Un nombre de
  espacios es ningún nombre, así que dos cilindros sin nombre no son duplicados.
- **Se dibuja**: la designación aparece bajo la caja del símbolo, como en la referencia, con
  el halo del texto para que se lea sobre la trama.
- **Se edita como texto**: el panel de propiedades gana el campo *Designación*, el único que
  no es un número; el formulario manda el texto tal cual y el ABI lo recorta. Antes, el QML
  convertía cada campo con `parseFloat`, que habría dejado la designación en `NaN`.
- **Se guarda**: formato de hoja **v3**, con un archivo v2 que se lee sin nombres y se
  guarda como v3; prueba de migración incluida.
- **La séptima falta, cubierta**: dos actuadores con la misma designación se señalan en el
  dibujo, ignorando mayúsculas y espacios. De las nueve faltas del manual quedan tres, y
  ninguna depende ya de esto: la octava necesita el encoder de desplazamiento (F6) y la
  novena, un proyecto de circuitos (F4).
- **Probado en cuatro niveles**: una prueba del documento, tres del formato (migración,
  rechazo, campo omitido), una de la comprobación y una del ABI que nombra, guarda, reabre,
  vacía, deshace y rechaza un nombre imposible; más siete pasos del `--selftest`. Capturas
  claro/oscuro con `1A1` y `1V1` dibujados y del panel con su campo.

### Los últimos circuitos, a un clic — 2026-11-12

Abrir el mismo circuito tres veces al día pasa por el diálogo de archivos otras tantas.
`Archivo ▸ Recientes` muestra los últimos diez documentos y los abre con un clic, que es lo
que la referencia ofrece en el mismo sitio.

- **La lista es un archivo, tratado como tal**: `punchi-fluid-file::recent` guarda como mucho
  diez rutas, rechaza una ruta vacía, de más de 4096 bytes o con un NUL, mueve al frente la
  que se repite en vez de duplicarla y escribe con archivo temporal y renombrado. Un archivo
  futuro se rechaza y una clave desconocida también.
- **Solo se ofrecen los que se pueden abrir**: al leer, y al abrir desde la lista, se
  descartan las entradas cuyo archivo ya no está. Una lista de rutas muertas es ruido en el
  menú; el archivo que existe pero no se puede leer se conserva, porque eso puede arreglarlo
  el usuario.
- **El core no elige el directorio**: la ventana le dice dónde vive la lista, con la ruta
  que ya usa para la copia de seguridad. Sin esa ruta la lista no existe, que es lo que
  quieren las pruebas y una sesión desechable.
- **Probado en tres niveles**: cinco pruebas del formato (orden, tope, ida y vuelta con
  archivos reales, poda de lo que falta, rutas imposibles), una del ABI que guarda dos
  archivos, comprueba el orden, borra uno del disco, lo ve desaparecer de la lista y
  reabre el otro en una sesión nueva, y diez pasos del `--selftest`.

### El documento avisa de lo que no está guardado — 2026-11-12

Un simulador que cierra sin preguntar es un simulador que se lleva el trabajo del usuario, y
hasta aquí el programa podía cerrar con un circuito editado sin decir nada.

- **El documento sabe que cambió**: cuenta sus propios cambios —cada edición, cada
  deshacer, cada rehacer— y el ABI lo compara con el número que tenía la última vez que se
  guardó o se abrió. Marcar, seleccionar, hacer zoom o pasar la comprobación **no** cuenta:
  el documento es el mismo.
- **El título lo dice**: mientras hay trabajo sin guardar lleva `•` y el nombre del archivo.
  Es un carácter y no un color, porque el color no se ve en una captura ni lo ve todo el
  mundo.
- **Nuevo, Abrir y cerrar preguntan**: un panel nombra lo que se va a perder y ofrece
  Guardar, Descartar o Cancelar. Solo aparece cuando hay algo que perder, así que el camino
  normal sigue sin modales.
- **Copia de seguridad**: cada treinta segundos, mientras hay cambios sin guardar, se escribe
  una copia en los datos de la aplicación. Al arrancar se **ofrece**, nunca se carga sola,
  porque puede ser trabajo que el usuario ya guardó. Recuperarla la deja sin guardar y sin
  ruta: el siguiente guardado pregunta dónde va en vez de pisar la copia.
- **Probado en tres niveles**: una prueba del documento (el contador sube al editar, al
  deshacer y al rehacer), una del ABI que edita, guarda, copia, recupera en una sesión nueva
  y rechaza un archivo roto, y doce pasos del `--selftest`. La ventana se captura con
  `--confirm new|open|close`, que existe para poder ver el panel sin manos.

### La ventana en el README, y el cierre que el proyecto merecía — 2026-11-07

Un README que describe una interfaz sin mostrarla pide un acto de fe, y este proyecto no
pedía nada al lector que no fuese leer.

- **`assets/screenshots/window-dark.png`** es la ventana real: cinta, catálogo, hoja y barra
  de estado, con la paleta oscura del escritorio y una válvula 3/2 en reposo alimentando un
  cilindro de simple efecto. Su [README](assets/screenshots/README.md) fija el origen —una
  captura de este programa, nunca de otro— y las reglas: se regenera cuando cambia el
  ribete, sin contenido privado y a resolución real.
- **Los dos README abren con esa captura** y un pie que dice qué se ve y qué no: nada en
  marcha, tubos en tinta neutra, `0,00 bar`. La interfaz está en español y el plan de idiomas
  queda enlazado.
- **Distintivos** de CI, licencia y estado. El de estado no es adorno: es el aviso de
  construcción en la primera línea que se ve.
- **«Hecho con cariño, para la comunidad» / «Made with love, for the community»**: el motivo
  del proyecto escrito de una vez —el software libre es la razón por la que un taller sin
  presupuesto aprende con la misma clase de herramienta que una empresa— y el lema de la
  casa: **Linux para todos**.

### El agente de IA, y su apoyo, dichos en el README — 2026-11-07

La ayuda de agentes de IA estaba en el flujo de trabajo —`AGENTS.md`, `.agents/`, commits
cofirmados— pero no en la cara pública del proyecto, que es donde un lector decide si
confía en él.

- **Sección «AI-assisted development» / «Desarrollo asistido por IA»** en los dos README:
  los agentes son una ayuda, no autores; el mantenedor responde de las decisiones técnicas,
  la revisión, la validación y lo publicado.
- **Las reglas viajan con el proyecto**: un agente que trabaje aquí cumple lo mismo que una
  persona —unidades tipadas, sin `unwrap` ni `panic!` fuera de pruebas, sin `unsafe` fuera
  del ABI, símbolos a mano contra ISO 1219-1 y nada tomado de otro programa—, y esas reglas
  las escribió y ajustó PunchiSoft para esta arquitectura, no vienen de una plantilla.
- **Coherencia con el estado real**: un agente no puede presentar como hecho lo que el
  ledger marca `missing` o `partial`, ni inventar un resultado para cerrar una tarea.
- **La licencia cubre igual las dos manos**: leer, citar, reimplementar o redistribuir con
  un agente no exime de los avisos, del código fuente correspondiente ni de la atribución.

### En construcción, y el repositorio lo dice — 2026-11-07

El proyecto no prometía nada sobre su estado ni sobre su ritmo, y las estadísticas de GitHub
lo presentaban como un proyecto de Shell y Python además de Rust.

- **Aviso de construcción en los dos README**: prototipo que funciona, no producto
  terminado; una sola persona, en los días libres que deja su trabajo principal; **sin fecha
  de entrega**. Lo que el ledger marca `missing` o `partial` se dice tal cual.
- **`CONTRIBUTING.md`** explica el ritmo: un issue puede esperar días y una revisión una
  semana; es la cadencia, no un olvido.
- **`.gitattributes`** saca de las estadísticas de lenguaje lo que no es el producto —
  `scripts-dev/**` (Shell y Python), `.github/**` y `assets/**` — marca `docs/` y el
  CHANGELOG como documentación, declara binarios y normaliza los finales de línea a LF.
- `AGENTS.md` repite el estado para que nadie presente como hecho lo que está a medias.

### Identidad propia y una puerta que prueba la ventana — 2026-11-07

El proyecto se presentaba como reemplazo de otro programa: lo nombraba en `AGENTS.md`, en los
dos README, en el plan de capacidades y en su skill, y el procedimiento de planificación
dependía de una copia local de un manual y de una biblioteca ajenos, con su ruta, su
inventario y frases literales, en un repositorio público.

- **ADR 0009**: la autoridad es la norma —ISO 1219-1/2/3, ISO 6358, ISO 5598— y el objetivo
  docente. Ningún producto, fabricante, manual, nombre de archivo ni inventario de terceros
  aparece en un archivo rastreado. La regla de contenido se generaliza: ni símbolos, ni
  capturas, ni circuitos de ejemplo, ni formato de archivo de otro programa.
- **`docs/capabilities.md`** sustituye al plan de paridad: fases F1–F8, ledger con cinco
  estados —`missing`, `partial`, `covered`, `improved`, `not planned`— y un registro de
  decisiones escrito en nuestras palabras, sin citas ni recuentos ajenos.
- **La skill `capability-plan`** sustituye a la de paridad: propósito escrito, evidencia y
  mejora razonada. Ya no enseña a leer material externo; dice que el plan no puede depender
  de algo que la comunidad no puede abrir.
- **`check-docs.sh`** pasa a fallar si vuelve un término prohibido, y comprueba que la lista
  de crates de `Cargo.toml` coincide con lo que declaran `AGENTS.md`, los dos README y la
  skill de arquitectura.
- **La cadena de herramientas está fijada** (`rust-toolchain.toml` con una versión, y CI
  leyéndola): main se puso en rojo porque CI instalaba `stable` y la máquina validaba con
  otra versión.
- **La ventana se prueba en CI**: un job instala Qt y el tema de iconos, construye la
  aplicación y la conduce sin pantalla con `--check-qml`, `--check-theme`, `--check-icons` y
  `--selftest`. El salto silencioso de `check.sh` es fatal con `PUNCHI_REQUIRE_UI=1`.
- **tiny-skia 0.12** entra sin cambios de código: la suite de exportación pasa entera.

### Alinear la selección — 2026-11-12

Con la selección múltiple ya se podía mover un grupo, pero no colocarlo: el manual pide
**Edición ▸ Alinear** y dice cuál es la referencia —*siempre el objeto que se halle en el
punto más lejano de la dirección escogida*—, así que el comando no necesita que el usuario
elija un ancla.

- **Cuatro botones**: izquierda, derecha, arriba y abajo, cada uno con su símbolo del
  escritorio. Mueven cada componente marcado hasta que su borde coincide con el del que ya
  estaba más lejos, y el resto de la posición no se toca.
- **La caja que cuenta es la girada**: un símbolo tumbado ocupa otra caja, así que alinear
  usa la que se dibuja y no la del archivo.
- **Un deshacer**: alinear seis componentes es un solo cambio en el historial.
- **Probado en tres niveles**: una prueba del documento (izquierda y derecha, con el
  movimiento vertical intacto y un solo deshacer), una del ABI que alinea cuatro
  componentes, deshace y comprueba que un borde inventado se rechaza, y dos pasos del
  `--selftest`.


### Marcar varios componentes y editarlos juntos — 2026-11-12

El manual lo pide en §4.2 y lo dice de la forma más clara posible: *las funciones de
edición como arrastrar, desplazar, copiar y borrar se refieren a todos los componentes
seleccionados*. Hasta aquí solo había una selección de uno.

- **La selección es un conjunto**: un clic sustituye la selección; `Mayús` y un clic
  añade o quita el componente que está bajo el puntero —el manual lo llama *bidireccional*—;
  el **rectángulo elástico** marca todo lo que cubre al soltar; y **Ctrl+E** (o *Todo* en
  la cinta) marca el circuito entero.
- **`Mayús` sigue copiando al arrastrar**: la referencia distingue los dos gestos por si el
  puntero se movió; aquí igual, así que una pulsación con `Mayús` que no se mueve cambia la
  pertenencia y una que se mueve copia y arrastra la copia.
- **Arrastrar un grupo lo conserva**: si el componente pulsado ya estaba marcado, el
  arrastre mueve todo el conjunto con el mismo paso; si no, la pulsación selecciona ese y
  arrastra solo ese.
- **Un gesto, un deshacer**, ahora también en grupo: mover, copiar, girar, reflejar y
  borrar una selección son **una** entrada del historial, y el documento sabe agrupar los
  movimientos de un arrastre de varios componentes para que no dejen una entrada por píxel.
- **Borrar avisa de lo que borra**: al borrar se limpia la selección, porque una baja
  renumera los índices y una marca vieja señalaría a otro componente. Ese detalle costaba
  un componente de más en el `--selftest`, que es exactamente para lo que está.
- **Probado en tres niveles**: tres pruebas del documento (arrastre agrupado con un solo
  deshacer, borrado y copia de un conjunto, giro con marcas distintas), una del ABI que
  marca, mueve, copia, deshace, marca con el rectángulo, alterna con `Mayús` y borra, y
  ocho pasos del `--selftest` con eventos de ratón y teclado reales.


### Girar y reflejar un componente — 2026-11-12

El manual lo documenta en dos renglones de `Edición`: *los componentes seleccionados pueden
girarse 90°, 180° o 270°* y *los objetos marcados se reflejan en su propio eje*. Sin eso,
un circuito que baja en vertical o una válvula montada al revés no se pueden dibujar, y la
mitad de los esquemas de la referencia son imposibles de reproducir.

- **El giro es del símbolo, no del código**: `punchi-fluid-draw::orient` envuelve la
  superficie de dibujo en una transformación que mueve cada llamada alrededor del centro de
  la caja. Un cuarto de vuelta o un espejo no añaden geometría ni una rama nueva al
  dibujante; el mismo archivo de catálogo vale en las ocho orientaciones.
- **El texto no gira**: los números de puerto se colocan donde el giro los lleva y se
  dibujan derechos, porque un número tumbado no se lee. Un arco sí gira —sus extremos y su
  sentido— y el espejo invierte el recorrido.
- **Los tubos acompañan**: las puntas de un conducto salen de la misma transformación, así
  que al girar un componente sus tubos se reenganchan a los puertos nuevos; la caja de
  selección y la prueba de impacto usan la caja girada, que cambia de ancho y alto.
- **Los gestos de la referencia más uno**: `Alt+R` gira un cuarto (botón **Girar**), `Alt+M`
  refleja (botón **Reflejar**) y `Ctrl` con doble clic gira al revés, con `Shift` para el
  sentido horario. La referencia no tiene teclado para esto; aquí sí, porque ningún mando
  puede depender del ratón.
- **El giro se guarda y se deshace**: el documento guarda los cuartos de vuelta y el espejo
  de cada colocación, y un giro es **una** entrada del historial. El formato de hoja sube a
  **v2**; un archivo v1 se lee tal cual y se guarda como v2, con una prueba de migración.
- **Probado en cuatro niveles**: nueve pruebas de la transformación (incluida la
  grabación de llamadas, para que pantalla y papel giren igual), tres del documento, una
  del ABI que gira, deshace y reabre el archivo, y cinco pasos del `--selftest` con eventos
  reales, entre ellos el doble clic con `Ctrl`.

### Editar el recorrido de un tubo — 2026-11-06

El manual lo pide en la misma página en que explica la T: *arrastrar los segmentos del
conducto para que el circuito se lea*. Faltaba, y un circuito con todos los tubos en su
codo automático no se puede ordenar.

- **El recorrido ahora es del documento**: cada conducto guarda cuánto se separó su
  segmento medio del codo automático, en décimas de milímetro. Se guarda con el tubo, así
  que mover un componente después no pierde el ajuste.
- **Un gesto, un deshacer**: el mismo `Undo` que ya unía los movimientos de un arrastre
  une ahora los de un reencaminado; soltar el botón deja una sola entrada en el historial.
- **La pulsación que selecciona también empieza a arrastrar**, como en la referencia: un
  clic sin movimiento solo selecciona, y el arrastre mueve el segmento que se ve, con la
  misma prueba de impacto contra la polilínea dibujada y no contra la recta entre puertos.
- **Probado en los tres niveles**: dos pruebas del documento (incluido que deshacer otro
  cambio no toca el recorrido), una del ABI que arrastra, deshace y rehace, y un paso del
  `--selftest` con eventos de ratón reales que busca un tubo en la hoja y comprueba que un
  solo deshacer devuelve el recorrido.

### La hoja es la red que simula — 2026-11-06

Hasta aquí el modelo era un circuito fijo de tres piezas y el documento era un dibujo que
lo acompañaba. Ahora es al revés: el documento **es** el circuito.

- **`punchi-fluid-solver::network`** construye una red de volúmenes y restricciones con las
  colocaciones y las conducciones del documento. Cada puerto es un nodo con un volumen
  pequeño; una válvula es un conjunto de restricciones entre sus propios puertos, activas
  según la posición; un cilindro aporta una cámara y, si declara dos puertos de trabajo,
  dos —con el área del anular a partir del diámetro del vástago—; una T une sus tres
  puertos en un solo nodo; un estrangulador es una restricción. La masa es el estado
  integrado, como antes, y el balance sigue cerrándose a 1e-9.
- **Una válvula 5/2 y un cilindro de doble efecto son archivos**: `valve.5_2.spring_return`
  y `cylinder.double_acting_32_200`, con su revisión de símbolo y sus capturas. La prueba
  del solver y la del ABI montan el circuito desde el documento y ven salir y volver el
  vástago sin que ninguna rama del código mencione esos dos identificadores. La válvula
  reproduce la dirección que documenta el manual: con el accionamiento aplicado conduce
  1→4 y, al cesar, el muelle la devuelve a 1→2.
- **El campo `rod_mm`** del cilindro declara el vástago y es lo que da al anular su área;
  el lint lo acota contra el diámetro del émbolo. Está documentado en la guía del catálogo.
- **El clic acciona la válvula que se pulsa**, no la primera que aparezca: el ABI gana
  `pf_session_press_component_valve`, `release_component_valve` y las consultas de posición
  y muelle por colocación, y el lienzo los usa. El dibujo deja de tener una lista de los
  tres componentes conocidos: cada colocación se pinta con **su** posición de válvula y
  **su** carrera, sacadas del modelo.
- **Cada tubo se colorea con la presión de su puerto**, de la tinta neutra a la presión de
  suministro; la barra de estado sigue dando el número, así que el estado no se cuenta solo
  por color.
- **Lo que el modelo todavía no hace, escrito y no escondido**: un tubo conduce pero no
  almacena volumen; un puerto de trabajo abierto queda cerrado y es la comprobación la que
  lo denuncia; los pilotos neumáticos y eléctricos son F6. Cada limitación tiene su fila
  en [docs/reference-parity.md](docs/reference-parity.md).

### La presión se lee como en un manómetro — 2026-11-06

- El `6 bar` de una fuente es **manométrico**, que es lo que dice la placa de datos:
  internamente se guardan 701 325 Pa absolutos, y el ISO 6358 ve la presión absoluta que
  necesita. Antes se guardaban 600 000 Pa, es decir 5 bar reales.
- La barra de estado y `--probe` restan la atmósfera, así que en reposo marcan **0,00 bar**
  y no 1,01. Es lo que muestra la referencia y lo que espera quien lee un manómetro.
- Comprobado con `--probe`: válvula cerrada, 0,00 bar; con la válvula accionada, 6,00 bar y
  100,0 mm a los 0,48 s (30 pasos de 16 ms). Las pruebas del solver comparan contra la
  presión de suministro que el propio modelo calcula, así que siguen valiendo.

### Las propiedades de una instancia se editan — 2026-11-06

La ficha de un componente enseñaba la definición y decía que editar llegaba en M3. Ahora
edita, y lo hace sobre **la instancia**, como la referencia: cambiar el cilindro de un
circuito no toca la biblioteca.

- **La ficha abre el componente que se pulsa**, no siempre la válvula del ejemplo, y
  muestra sus números efectivos: la definición con lo que esa instancia haya cambiado.
- **Campos editables**: presión de una fuente, conductancia de una válvula o de un
  estrangulador, y carrera, émbolo, vástago y masa móvil de un cilindro. La forma trae
  unidades, porque son las que el usuario piensa.
- **Aplicar es un solo deshacer**: los valores viajan en un `PfEditSet` con máscara, el
  documento registra un cambio y el modelo se reconstruye desde él. Un rod mayor o igual
  que el émbolo se rechaza con un mensaje y no cambia nada.
- **Probado en tres niveles**: la prueba del documento, dos del ABI —la carrera editada
  llega a `stroke_m` y deshacer la devuelve; la presión editada a 3 bar deja la cámara en
  3 bar— y un paso del `--selftest` que abre la ficha del cilindro, cambia la carrera a
  50 mm, comprueba el modelo y deshace.
- **Lo que la ficha todavía no edita** —nombre, resumen, accionamiento y muelle, posición
  inicial del émbolo, ángulo de montaje, fugas y amortiguamiento— está escrito, con su
  fila, en [docs/reference-parity.md](docs/reference-parity.md).

### Guardar y abrir un circuito — 2026-11-06

El trabajo no se podía guardar: el editor vivía en memoria y cerrar la ventana lo perdía.
Ahora el documento tiene archivo, y el archivo es suyo.

- **Formato de hoja versionado** en `punchi-fluid-file::sheet` (ADR 0007): colocaciones
  por id de catálogo con su caja, conducciones por id de colocación y puerto, el recorrido
  de cada tubo y los valores propios de cada instancia. No es el formato heredado del
  CLI: la hoja lleva geometría e instancias que aquel no tiene.
- **Archivo no confiable**: se rechazan claves desconocidas, toda colección tiene tope
  (2000 componentes, 4000 tubos, 8 MiB leídos), todo número se comprueba, una versión
  futura se rechaza y un archivo que no valida **no toca el documento abierto**.
- **Escritura atómica**: se escribe un temporal en la misma carpeta y se renombra; un
  corte no deja un archivo a medias.
- **Cuatro comandos**: Nuevo (Ctrl+N), Abrir (Ctrl+O), Guardar (Ctrl+S) y Guardar como
  (Ctrl+Shift+S), con diálogo de archivos. La referencia usa Ctrl+A y Ctrl+G (apéndice
  A.1); aquí se usan los atajos de la plataforma, que es la mejora registrada.
- **Probado**: seis pruebas del formato (ida y vuelta byte a byte, clave desconocida,
  versión futura, topes, escritura atómica), una del ABI que guarda con una carrera y una
  edición, abre en una sesión nueva y las recupera, el rechazo de un archivo futuro sin
  perder lo abierto, y un paso del `--selftest` que guarda, cambia el documento, abre y
  cuenta. `--save <archivo>` y `--open <archivo>` permiten hacerlo sin ventana.

### Hasta el próximo cambio de estado — 2026-11-06

El manual describe un modo de simulación que corre **hasta que cambia el estado** y vuelve
a la pausa (§3.2): el vástago llega a un tope, se acciona una válvula, un relé o un
interruptor.

- **Comando «Hasta cambio»** en el transporte: entra en simulación, avanza hasta el primer
  tope del vástago o el primer cambio de posición de una válvula, y **se queda en pausa en
  ese cuadro**. Si no ocurre nada en cuatro segundos simulados, se detiene y lo dice con
  el valor de retorno.
- **Los dos disparadores que ya existen** se comprueban por el estado del modelo, no por
  los sonidos ni por la interfaz: la posición del vástago contra la carrera declarada y la
  posición de la válvula antes y después de cada cuadro.
- **Probado** con el `--selftest`: reinicia, acciona la válvula, corre hasta el cambio y
  exige que se haya detenido, que el vástago esté en el tope y que el transporte quede en
  pausa. Relés e interruptores llegarán con la biblioteca eléctrica (F6), y están escritos
  como pendientes en [docs/reference-parity.md](docs/reference-parity.md).

### Copiar, pegar y duplicar un componente — 2026-11-06

El manual lo agrupa en §4.2: la selección va al portapapeles con Ctrl+C y vuelve con
Ctrl+V, y arrastrar con **Mayús** copia en vez de mover, con el puntero cambiado a la
marca de copia.

- **Ctrl+C / Ctrl+V**: un portapapeles de una colocación —la selección es una— que
  conserva el id de catálogo, la caja y los valores propios de la instancia. Pegar crea
  una identidad nueva, así que los tubos del original se quedan donde están.
- **Ctrl+D** duplica y desplaza diez milímetros; **Mayús+arrastrar** copia el componente
  bajo el puntero y arrastra la copia, dejando el original quieto.
- **Cada gesto es un solo deshacer**; el duplicado reutiliza la misma entrada que un
  «colocar» y mover la copia se funde en una entrada, como cualquier arrastre.
- **Probado** en el documento (identidad nueva, valores conservados, un deshacer), en el
  ABI (copiar sin selección se rechaza, pegar, desplazar y duplicar, con deshacer) y en el
  `--selftest`, que además usa un evento de ratón con Mayús para el gesto real.
- **Lo que falta de §4.2** —copiar varias a la vez con sus tubos, alinear y distribuir—
  necesita selección múltiple; está anotado en [docs/reference-parity.md](docs/reference-parity.md).

### Un manómetro que lee su puerto — 2026-11-06

El primer instrumento de F3, y el primero que no es una pieza de circuito sino una
medida.

- **`gauge.pressure`** es un archivo con `behaviour = "gauge"`, una escala declarada
  (0–10 bar) y un solo puerto de trabajo. La decisión está en
  [ADR 0008](docs/adr/0008-a-gauge-is-a-behaviour.md): una variante de comportamiento y no
  una etiqueta que el dibujo pega al lado de cualquier componente.
- **No cambia la física**: la red le da al puerto un nodo como a cualquier otro y no
  añade volumen, restricción ni fuerza; el manómetro lee `port_pressure` y el número se
  dibuja sobre la esfera. Un circuito con manómetro simula igual que sin él (lo exige la
  prueba).
- **La esfera es del archivo y el número del modelo**: la aguja, el cubo y el vástago son
  primitivas con su clase de línea; la lectura es texto, para que una escala de 0–60 bar
  sea otro archivo y no otra rama.
- **Probado** en el solver (lee la misma presión que la línea de trabajo y el vástago
  sigue moviéndose), en el ABI (se coloca, se conecta, se acciona la válvula y el marco
  contiene la lectura dentro de la caja del manómetro) y con su revisión de símbolo y
  capturas claro/oscuro.

### El lienzo vuelve a responder al ratón — 2026-11-06

El diagnóstico que faltaba era el más simple y lo delataba el propio dibujo: el elemento del
lienzo no declaraba ningún botón aceptado, así que Qt nunca le entregaba una pulsación. Todo
lo demás estaba escrito —selección, arrastre, puertos, tubos, borrado— y nada de ello podía
ocurrir; por eso el usuario veía un diagrama que no se dejaba tocar mientras las pruebas de
Rust seguían en verde.

- **La causa**: `QQuickPaintedItem` no acepta botones por omisión en este Qt, y la clase no
  lo declaraba. La corrección es una línea, y el valor está en la prueba que la sostiene.
- **`--selftest` recorre los doce gestos con eventos de ratón y de teclado de verdad**:
  mantener el clic abre la válvula, la cámara recibe presión y el vástago avanza; soltar la
  cierra; Detener vuelve a edición; el clic selecciona y abre la ficha; arrastrar mueve;
  puerto a puerto dibuja un tubo; `Supr` borra. Antes de pulsar espera al primer cuadro,
  porque antes de él la composición no ha corrido y toda coordenada que la ventana informa
  es falsa: la primera versión de la prueba apuntaba a (260, 12) creyendo apuntar a la
  válvula y «demostraba» un fallo distinto del real.
- **La compuerta lo ejecuta**: `check.sh` construye la ventana y corre `--check-qml`,
  `--check-theme`, `--check-icons` y `--selftest` en modo sin pantalla, y se salta ese paso
  —avisando— donde Qt no está configurado. Con la línea retirada, nueve comprobaciones
  fallan y la compuerta se pone roja; con ella, cero.
- **El circuito de partida se dibuja con sus dos conducciones**, que es lo que hace que la
  hoja se lea como un circuito y no como tres piezas sueltas; la comprobación deja de
  denunciar dos conexiones neumáticas abiertas en el ejemplo de trabajo. Las dos pruebas del
  ABI que contaban tubos pasan a contar respecto del sembrado.
- **Cada tubo se dibuja una sola vez**: la pasada del circuito volvía a trazar sus
  conducciones por su cuenta, con otro recorrido y en color de presión, encima de las del
  documento. Ahora el documento es el único dueño de los tubos y la pasada fija solo aporta
  símbolos, pistón animado y rótulos. El rótulo de la alimentación pasa por encima del
  símbolo: debajo quedaba escrito sobre la línea que sube al puerto 1.
- **La paleta se prueba arrastrando de verdad**: pulsar en una fila del catálogo y soltar en
  la hoja coloca el componente exactamente donde se suelta —medido, (150, 40) mm—, que es el
  gesto con el que se dibuja un circuito. Al escribir la prueba apareció un segundo detalle:
  un `MouseArea` declarado en línea se llama `QQuickMouseArea_QML_3`, así que buscar por
  nombre de clase exacto no lo encuentra nunca; se busca por herencia.
- **La comprobación cubre 5 de las 9 faltas y las marca en el dibujo**: se añade la conexión
  neumática abierta, con la coordenada del puerto y no la del componente. Cada falta se señala
  donde ocurre con un anillo discontinuo y un aspa —forma además de color, para que se lea en
  una captura y para quien no distingue el color—; el ABI gana `pf_session_finding_at` y una
  prueba que exige que **toda** falta reportada tenga un sitio al que apuntar, y que una
  posición fuera de la lista se rechace en vez de contestar con basura. Un puerto de escape queda exento, porque la
  propia válvula 3/2 del ejemplo respira por el 3 y una comprobación que denuncia el ejemplo
  es una comprobación que el usuario apaga. Las cuatro restantes no están olvidadas: la
  conexión que no encaja necesita la biblioteca eléctrica (F6) y las otras tres necesitan
  designaciones y agrupación en circuitos (F4); cada una está escrita con su bloqueo en el
  registro y en el módulo.
- **La lista de las nueve faltas queda observada y registrada** en
  [docs/reference-parity.md](docs/reference-parity.md) tal como la publica el manual (§4.10),
  con un hallazgo de comportamiento que decide una discusión antes de tenerla: la referencia
  muestra los fallos **en diálogos uno tras otro** y después deja simular igual. La lista es
  suya; la presentación es nuestra, y la mejora se apoya en una regla escrita.
- **`--probe` imprime los números de la barra de estado** para poder compararlos con un
  cálculo a mano: en reposo 0,00 bar y 0,0 mm —presión manométrica, como un manómetro—; con la
  válvula accionada, 6,00 bar y 100,0 mm a los 0,48 s (30 pasos de 16 ms), sin deriva al seguir
  simulando.
- **El sonido del circuito es el impacto del cilindro, no la válvula.** El manual los separa
  (§4.13): `switch.wav` para interruptores y relés, `valve.wav` para válvulas, `horn.wav` para
  la bocina y **`cylinder.wav` para las posiciones finales de los cilindros**. Faltaba justo
  ese, el único que dice que el circuito hizo algo. Ahora suena cuando el vástago llega a un
  tope **y llega moviéndose**: uno que se arrastra hasta el tope no tiene nada que contar. El
  sample es nuevo, generado con la misma receta reproducible (golpe metálico, retumbo del
  barril y cola de aire), y la compuerta comprueba que el binario lo decodifica.
- **Un rebote suena una sola vez.** El modelo puede rebotar unos milímetros en el tope antes
  de que la cámara cargue, y eso producía dos golpes separados 50 ms — a un oído, un solo
  golpe. Las llegadas dentro de ese margen se funden en una, y el reinicio borra la memoria
  de llegadas: un modelo que vuelve a t=0 no ha llegado a ningún sitio.
- **El rebote queda escrito como comportamiento medido**, no como sorpresa futura: con paso
  de 16 ms el vástago llega al tope antes de que la cámara cargue, el muelle lo devuelve unos
  milímetros y el aire lo vuelve a sacar; con 1/60 s no ocurre. Es el modelo, no un fallo
  numérico, pero la referencia no lo muestra, así que hay una prueba que fija el estado
  estacionario y otra que **falla si el rebote desaparece**, para que el detector de impacto
  se pueda simplificar a propósito y no por accidente. Tunear la física sin una curva de
  referencia habría sido inventar, que es justo lo que no toca.
- **La aplicación abre en modo edición**, como la referencia: nada se mueve y un clic sobre
  la válvula la selecciona, no la acciona. Simular se pide a propósito, con Iniciar (F9) o
  Pausa (F8), y sólo entonces el clic acciona. Antes abría simulando, así que se podía tocar
  la válvula sin haber dado Iniciar nunca — la queja era exacta y el fallo estaba en dos
  valores iniciales. La compuerta ahora lo exige: pulsa la válvula en edición y comprueba que
  la posición no cambia, que la presión no sube y que la simulación no arranca sola; con los
  valores antiguos, esa prueba se pone roja.
- **Vista con zoom y cuadrícula**, como el apartado «Zoom» del manual: Aumentar (`>`),
  Disminuir (`<`), Tamaño original (`Ctrl+0`) y la cuadrícula (`Ctrl+G`). El zoom mantiene el
  centro de la hoja en el centro de la ventana, así que ampliar no echa el circuito fuera de
  la vista, y **no entra en el historial**: es vista, no documento, y una prueba lo exige.
- **El borde de la hoja se dibuja.** Sin él, «un componente se sale de la hoja» era un aviso
  que el usuario no podía ver, y un circuito que se desborda no se distinguía de uno que cabe.
- **La interfaz habla un solo idioma.** Los nombres del catálogo y el de la posición de la
  válvula salen ahora de `name_es` cuando el fichero lo trae —«Aire comprimido, 6 bar»,
  «Válvula 3/2, retorno por muelle», «reposo»— en vez de mezclar español en la ventana con
  inglés en el dibujo. Los ficheros ya traían el dato; nadie lo leía. El campo opcional se
  documenta en la guía del catálogo.
- **Las uniones en T, como en la referencia**: soltar un extremo de tubo sobre un conducto
  inserta la T donde se suelta, parte ese conducto en dos y une el tercer puerto. El gesto
  está probado con eventos reales y, sobre todo, **el empalme entero es un solo paso de
  deshacer** — cinco operaciones que se deshacen con una pulsación, porque un usuario al que
  se le pide pulsar cinco veces por un gesto entiende que el programa no respeta su trabajo.
  Si el empalme no puede terminar, no deja nada a medias: hay una prueba para eso.
- **La T es un componente, no un nodo del documento** ([ADR 0006](docs/adr/0006-a-junction-is-a-component.md)).
  Las dos opciones se sopesaron: un nodo nuevo habría reescrito `Connection`, el historial,
  el encaminado, la prueba de impacto, la comprobación y el ABI, cinco sitios que hoy están
  bien. Un fichero con tres puertos en el mismo punto no reescribe ninguno, y respeta la
  regla de que un componente es siempre un fichero. `Behaviour::Junction` es la excepción que
  la propia regla prevé, y por eso va con ADR.
- **La T no lleva numerales**: sus tres puertos son un punto, así que tres identificadores
  impresos uno sobre otro se leían como un borrón. ISO 1219-1 dibuja una unión como un punto
  relleno y nada más. Su revisión de conformidad queda en
  [assets/symbols/review/junction.tee.md](assets/symbols/review/junction.tee.md), con capturas
  en los dos temas y lo que sigue sin verificar.
- **La comprobación no denuncia los puertos de una unión**: un nodo modelado como componente
  tiene puertos que pueden quedar sin usar, y eso es un artefacto del modelo, no un circuito a
  medio hacer. Una comprobación que grita en falso enseña a ignorarla.
- **`--check-qml` comprueba de verdad**: el recuento de problemas existía y nadie lo leía ni
  llamaba a `selfTest()`, así que el indicador era decorativo. Ahora ejecuta todos los
  comandos de la cinta e informa `qml problems: N` con el código de salida correspondiente.

### Una puerta antes de programar — 2026-11-06

Faltaba el paso entre «decidir qué hace falta» y «programar», y ahora es una puerta: la
skill de planificación fija el procedimiento y [docs/capabilities.md](docs/capabilities.md)
guarda la prueba de que se siguió.

> Sustituido el 2026-11-07 por [ADR 0009](docs/adr/0009-identity-and-standards.md): el
> proyecto se describe por su misión y por las normas, ningún producto de terceros se nombra
> y ningún material ajeno entra en el repositorio.

- **Ninguna capacidad empieza sin su propósito y su prueba escritos**: qué tiene que poder
  hacer un estudiante, cómo se comporta, qué tenemos, qué falta y qué test lo cerrará. La
  lista de fases dice qué construir; el registro dice qué se decidió. Una fila sin propósito
  bloquea la capacidad.
- **Qué se lee y qué no**: la norma (ISO 1219, ISO 6358, ISO 5598) y nuestro objetivo
  docente. Nada de otro programa entra aquí: ni símbolos, ni capturas, ni páginas de manual,
  ni formato de archivo, ni una copia local descrita en el repositorio.
- **Dónde aterriza cada cosa**: una tabla capa por capa —definición, documento,
  comprobación, física, dibujo, exportación, interfaz— con la prueba que la demuestra, para
  que «replicar» no acabe en un `match` sobre el id de una válvula.
- **La referencia es la base, no una lista de deseos.** El trabajo va en dos arcos y en ese
  orden: primero se replica lo que la base hace —una sustitución que hace menos no es una
  sustitución—, y después **se mejora**, que es el motivo del proyecto. Mejorar primero es
  imposible de juzgar: una diferencia que no se puede explicar contra la base es una
  regresión con nombre de función.
- **Mejorar es una decisión registrada, no un gusto**: cada capacidad termina en «se deja
  como la base, porque…» o en el borde romo que tiene, qué hacemos en su lugar, por qué es
  barato y honesto y qué prueba lo sostiene. Quedarse en paridad sin esa decisión es dejarlo
  a medias: el registro la exige.
- **Una tabla de cobertura con toda la base dentro**: cada función del inventario es una fila
  en uno de cinco estados —`none`, `partial`, `base`, `better`, `not adopted`— y nada queda
  en blanco. La función de la base es lo que se hace por defecto; no adoptarla es la
  excepción y lleva argumento sobre nosotros (licencia, alcance, contenido), nunca desprecio
  por la base.

### Plan de paridad con la referencia, por fases — 2026-11-06

Se leyó la referencia —el manual y la estructura de su biblioteca— para enumerar qué hace,
y de ahí sale [docs/reference-parity.md](docs/reference-parity.md): ocho fases, cada una
con un final usable, su criterio de aceptación y **la evidencia que la prueba**.

- **Medido, no supuesto**: biblioteca de la referencia con 210 definiciones (110 neumáticas,
  50 eléctricas, 28 digitales, 10 varias, 6 DDE, 6 GRAFCET) y 533 circuitos didácticos.
- **Qué se replica y qué no**, escrito antes que el código: comportamiento sí; ni un
  símbolo, ni un circuito de ejemplo, ni una frase del manual, ni su formato `.ct`.
- **F1, la siguiente**: cerrar la edición —seleccionar, borrar y reencaminar tubos, uniones
  en T, propiedades editables— y **comprobar el circuito** antes de simular, con las nueve
  faltas que enumera la propia referencia como requisito, cada una con su prueba y señalada
  en el dibujo.
- **F1, primer trozo hecho**: un tubo se puede **seleccionar** —se dibuja en el color de
  acento— y **borrar** con `Supr`, y el borrado se deshace. El impacto se mide contra la
  polilínea trazada, no contra la recta entre puertos, así que responde a lo que se ve.
  Componente y tubo nunca están seleccionados a la vez: `Supr` sabe qué quitar. Una prueba
  nueva del ABI lo recorre entero —encontrar, seleccionar, borrar y deshacer—.
- **F1, segundo trozo hecho**: `punchi-fluid-check`, la comprobación previa. Decide cuatro
  faltas —componente fuera de la hoja, dos componentes solapados, tubo que cruza un
  componente ajeno, y dos tubos superpuestos— y las devuelve **con coordenadas**, para que
  la interfaz señale en vez de obligar a interpretar una frase. Las otras cinco que
  enumera la referencia no están: una conexión abierta, dos cilindros con la misma
  designación, una etiqueta que no coincide y un tubo por un circuito ajeno necesitan
  nombres o la red, y una comprobación que adivina es peor que ninguna. Está escrito así en
  el crate, no solo aquí.
- La comprobación destapó un error en el propio comprobador: dos cajas separadas se
  reportaban como solapadas, porque `Rect::from_corners` normaliza las esquinas y convertía
  el hueco entre ellas en un rectángulo con área. Lo cazó la prueba de «dos componentes
  lado a lado no se solapan».
- Después: **F2** la hoja es el circuito (red de volúmenes, M4), **F3** instrumentos y
  ejecución hasta cambio de estado, **F4** documentos v2, **F5** salida, **F6** bibliotecas
  eléctrica/digital/GRAFCET (cada variante con ADR), **F7** material didáctico propio y
  **F8** 1.0.


### Edición: tender conexiones de puerto a puerto — 2026-11-06

Lo que faltaba para que editar sea editar: en la referencia un circuito se construye
arrastrando de una conexión a otra, y eso es lo que ahora hace la hoja.

- **`punchi-fluid-doc` gana conexiones**: cada extremo es `(id de la componente, puerto)`, y
  las componentes tienen **identidad estable** —un `id` que se asigna una vez y no se
  reutiliza—, así que borrar una no renumera los extremos de las demás y deshacer la
  devuelve con sus tubos en su sitio.
- **Se rechaza lo que no es un tubo**: un puerto consigo mismo, un duplicado en cualquier
  sentido, o una componente que no está en la hoja. Se devuelve `false` con mensaje, nunca
  un éxito silencioso. Un tubo dibujado desde cualquiera de los dos extremos es el mismo
  tubo, porque los extremos se guardan en orden canónico.
- **El gesto**: pulsar sobre un puerto —el puntero se vuelve una cruz—, arrastrar y soltar
  sobre otro. El tubo en curso se dibuja discontinuo en el color de acento; soltar fuera de
  un puerto lo cancela.
- **Trazado ortogonal**: recto cuando los puertos se alinean, si no un codo con un solo
  quiebre, en la tinta de línea.
- **El documento se siembra al crear la sesión**, no en el primer cuadro: una interfaz puede
  colocar y conectar antes de pintar, y un documento vacío hasta el primer repintado le
  rechazaría la primera conexión.

#### Verificación

- `punchi-fluid-doc`: **13 pruebas**, cinco nuevas sobre conexiones —unir y deshacer,
  el mismo tubo desde los dos extremos, rechazo del lazo y de la componente ausente, y
  borrar una componente devolviendo sus tubos con identidades intactas al deshacer—.
- ABI: **13 pruebas**; la nueva une dos puertos, comprueba que el duplicado y el lazo se
  rechazan, y que deshacer quita el tubo.
- Captura: un tubo entre la alimentación y un estrangulador recién soltado, con su codo.
- `clippy -D warnings` y `fmt --check` limpios; `--check-qml` sin problemas.

#### Limitaciones conocidas

- **La física sigue siendo la del circuito fijo**: un tubo dibujado no conecta nada en el
  modelo hasta M4, y la interfaz no lo disimula. Por eso el circuito simulado dibuja además
  sus propias tuberías, que pueden solaparse con una dibujada a mano.
- Todavía no se pueden **seleccionar ni borrar tubos**, ni reencaminarlos arrastrándolos, ni
  se insertan uniones en T al soltar un tubo sobre otro.


### Documento editable: arrastrar componentes del catálogo a la hoja — 2026-11-06

El modo Editar ya edita algo. Arrastrar un componente desde la barra lateral y soltarlo
sobre la hoja lo coloca, y la hoja deja de ser un dibujo fijo: es un documento.

- **Crate nuevo `punchi-fluid-doc`**: colocaciones (id del catálogo y caja en milímetros)
  y un historial de cambios **acotado a 100 entradas**. El documento no sabe de física, ni
  de dibujo, ni de interfaz.
- **Deshacer y rehacer de verdad**: colocar, mover y borrar se deshacen; un arrastre es
  **una sola entrada** de historial, no una por píxel, porque los movimientos consecutivos
  del mismo componente se funden en el que ya está arriba.
- **El circuito de partida no es una edición**: se siembra en el primer cuadro y no se
  puede deshacer. Sembrar por «¿está vacío el documento?» era el error evidente y estaba
  mal: un componente soltado antes del primer cuadro lo habría suprimido.
- **Arrastre desde la paleta**: pulsar un componente, mover y soltar sobre la hoja. El
  componente que se lleva se dibuja con **su propio símbolo** mientras viaja, y el destino
  se ajusta a la rejilla de 10 mm. El arrastre está hecho a mano —un estado y un fantasma—
  en vez de con el drag-and-drop de Qt, para que el puntero y cualquier cosa que maneje la
  ventana recorran **el mismo camino** y haya un único punto de entrada verificable.
- **Selección y teclado**: un clic selecciona y arrastra; `Supr` borra; `Ctrl+Z` y
  `Ctrl+Shift+Z` recorren el historial; `Esc` cancela el arrastre y cierra el panel.
- **La hoja se dibuja desde el documento**: las tres componentes simuladas se dibujan en
  sus cajas del documento y las tuberías las siguen, así que mover una las mueve. Las
  componentes añadidas se dibujan con su símbolo y **nada más**: no tienen física hasta que
  existan conexiones (M4), y la interfaz no lo disimula.
- **La pestaña Editar deja de estar vacía**: Deshacer, Rehacer y Borrar, disponibles solo
  cuando significan algo.
- **Se retira el botón «Editar» y la pestaña Simular.** El modo no es un interruptor
  propio: lo cambia el transporte, como en la referencia. **Detener (F5)** vuelve a
  edición; **Iniciar (F9)** entra y arranca; **Pausa (F8)** se queda en simulación —y desde
  edición entra sin arrancar, para revisar ajustes antes de mover el modelo—; **Paso
  (F10)** avanza un cuadro y queda en pausa; **Restablecer (Ctrl+R)** vuelve a `t = 0`. El
  transporte lleva además la velocidad, que es donde se usa. Quedan tres pestañas y ningún
  comando repetido, y los atajos son los del manual de la referencia.
- **Se retira «Animar»**: no existe en la referencia y duplicaba al transporte, que es
  quien manda sobre iniciar, pausar, paso y reiniciar. Estar en modo Simular *es* la
  animación; la pestaña lleva solo lo que el transporte no tiene —el modo y la velocidad—,
  y el modo usa un icono propio para no confundirse con el ▶ del transporte. La regla queda
  escrita en [ui-ux.md](docs/ui-ux.md): **ningún comando aparece dos veces**.
- **Todos los comandos pasan por un único despachador**, transporte incluido, y
  `--check-qml` ejecuta cada uno y falla si QML informa de algo. Hacía falta: un *setter* de
  `Q_PROPERTY` no es un método invocable desde QML, y `session.setRunning(false)` en un
  `onTriggered` lanzaba un `TypeError` que ni el compilador ni el cargador veían hasta
  pulsar el botón. Ahora se asigna la propiedad, y el gate lo comprueba: con el error
  reintroducido a propósito, `--check-qml` sale 1; sin él, sale 0.

#### Verificación

- `punchi-fluid-doc`: 10 pruebas — colocar y deshacer, rehacer, una edición nueva descarta
  la línea de rehacer, deshacer un movimiento, el arrastre como una sola entrada, quitar y
  recuperar en su sitio, el historial nunca supera su límite, el impacto prefiere el de
  arriba, y sembrar no es editable.
- ABI: una prueba nueva coloca un componente del catálogo, comprueba que nace seleccionado
  y ajustado a la rejilla, que responde a un clic dentro de su caja, lo mueve, deshace,
  rehace y lo borra.
- Capturas fuera de pantalla: dos componentes soltados junto al circuito, y la misma escena
  con `--undo 1` donde solo queda uno — el historial funciona de punta a punta.
- `./scripts-dev/test.sh`: **146 pruebas** en verde; `clippy -D warnings` y `fmt --check`
  limpios.

#### Limitaciones conocidas

- Soltar y arrastrar con puntero real no está probado con un evento sintético: la captura
  usa el mismo `placeComponent` que el gesto, y el ABI lo prueba por su cuenta.
- Las componentes añadidas no se simulan ni se conectan; eso es M4.
- El documento vive en memoria: guardar y abrir llegan con el formato v2.


### Sonido en las interacciones — 2026-11-06

La sugerencia llegó con la pregunta correcta: en un taller real el aire se oye antes de
verse. Entran dos eventos y nada más —el accionamiento de la válvula y la puesta en
marcha— porque el resto del ruido de una instalación depende de un modelo que todavía no
existe: caudal por ramal, régimen crítico en cada restricción y el escape de cada
componente.

- **La válvula suena cuando la corredera se mueve**, no cuando se pulsa: `FluidSession`
  emite `valveActuated()` solo si `pf_session_press_valve` o `pf_session_release_valve`
  cambiaron la posición. Una válvula de muelle devuelve la corredera al soltar, así que un
  gesto suena dos veces, que es exactamente lo que hace una de verdad.
- **La marcha arranca el aire**: `transportStarted()` dispara una cama de ruido que cae
  conforme el volumen se carga, y `transportStopped()` la corta donde estaba. Al abrir la
  ventana no suena nada: el modelo ya está en marcha y un simulador que saluda con un
  siseo es una sorpresa, no una confirmación.
- **`SoundFeedback.qml`** es lo único que sabe de audio: escucha a la sesión y decide si
  suena o no. La ventana arranca con sonido y se silencia con el botón de la barra de
  estado o con `Ctrl+M`, y la etiqueta dice el estado —`Sonido`/`Silencio`—, nunca un
  color. En silencio todo se comporta y se lee igual: el sonido nunca es la única señal.
- **Las muestras las genera el repositorio**: `scripts-dev/make-sounds.py` sintetiza
  `assets/sounds/valve.wav` y `air.wav` con semilla fija a partir de ruido filtrado, así
  que no entra material de terceros ni hay licencia que rastrear. El carácter se midió
  contra grabaciones reales de aire comprimido: centroide espectral de 8,4 kHz en las dos,
  sin continua y con pico entre −4 y −6 dBFS.
- **Se pueden sustituir sin compilar**: un WAV con el mismo nombre en
  `$XDG_DATA_HOME/punchi-fluid-lab/sounds/` gana sobre el que va dentro del binario, la
  misma regla de prioridad que el catálogo. Es el camino previsto para una grabación de
  taller; el binario sigue siendo un archivo solo.
- **`--check-sounds`** prueba lo que a QML no se le puede preguntar: comprueba que las dos
  muestras están en el binario y empiezan con RIFF/WAVE, deja que `SoundEffect` las cargue
  y sale con 0 o 1. Un `.mp3` renombrado falla aquí en vez de quedarse en silencio, que es
  como fallan los formatos que el mezclador no entiende.
- El sonido necesita el módulo **Qt Multimedia** en tiempo de ejecución: se importa desde
  QML, no se enlaza, así que compilar no exige el paquete de desarrollo.

### Modo edición y modo simulación, con el clic sobre la válvula — 2026-11-06

El botón **Accionar** no existía en la aplicación de referencia y no tenía sentido: allí se
interactúa con el símbolo. Se retira, y en su lugar entra lo que de verdad distingue a un
simulador de esquemas: dos modos, y el mismo clic significando cosas distintas en cada uno.

- **Modo Editar**: el clic sobre la válvula **abre sus propiedades**; el modelo queda
  congelado y el transporte no está disponible, con la pista explicando por qué. El panel
  muestra lo que dice el archivo —nombre, `id`, resumen, categoría, puertos, posiciones y
  tipo de retorno— leído por `pf_session_describe_valve`, no un formulario inventado por la
  interfaz.
- **Modo Simular**: el clic **acciona** la válvula. La válvula del catálogo es de muelle, así
  que se **mantiene pulsada**: el gesto se compone de `pf_session_press_valve` al bajar y
  `pf_session_release_valve` al soltar, y el muelle devuelve la corredera. Una válvula sin
  muelle se enclava; la interfaz pregunta `pf_session_valve_spring_return` en vez de
  suponerlo.
- **La válvula se anuncia**: el puntero cambia sobre ella y un anillo discontinuo, en el
  color de acento, marca el área de impacto. El anillo se dibuja con `pf_session_valve_rect`,
  el rectángulo real del núcleo, no con coordenadas adivinadas en C++.
- El grupo **Modo** del ribbon muestra cuál está en vigor, y el estado del transporte dice
  `edición` en lugar de `en pausa`.
- `Espacio` mantiene la válvula igual que el puntero, y solo está armado en modo simulación.

#### Verificación

- Once pruebas del ABI: la nueva comprueba el gesto completo —con la válvula en posición 0,
  pulsar la lleva a la 1 y soltar la devuelve a la 0— sobre el modelo real.
- Capturas fuera de pantalla: modo edición con el panel de propiedades abierto, y válvula
  mantenida con `t 0,19 s` y el cilindro al 100 % y 2,6 bar.
- `./scripts-dev/test.sh`: 135 pruebas en verde, `clippy -D warnings` y `fmt --check` limpios.


### Transporte de simulación: iniciar, pausar, paso a paso y reiniciar — 2026-11-06

El botón de reproducir ya no era un conmutador suelto: el transporte tiene ahora los cuatro
comandos de la aplicación de referencia, siempre visibles a la derecha de las pestañas.

- **`▶ Reproducir` · `⏸ Pausa` · `⏭ Paso` · `↺ Reiniciar`**. Iniciar y pausar solo están
  disponibles en el estado en que significan algo, y la pista dice por qué cuando no
  ("ya está en marcha", "ya está en pausa"): un botón gris tiene que explicarse.
- **Un paso es exactamente un cuadro**: `FluidSession::stepForward()` avanza
  `frameSeconds × velocidad`, el mismo número que usa el bucle de animación, así que el
  modo paso a paso y la reproducción continua no pueden divergir. `frameSeconds` es una
  constante de la sesión y el temporizador de QML la lee, en vez de repetir `0,016`.
- **Un paso siempre deja el transporte en pausa**, como el botón de paso de un simulador;
  un paso dado mientras corre haría competir los dos modos. Además fuerza el repintado
  aunque el modelo estuviera en reposo, porque el usuario lo ha pedido.
- **El estado deja de ser solo color**: el transporte escribe `en marcha · t 0,19 s`, y la
  barra de estado mantiene las lecturas.
- **`F10` da un paso**; `F5` reproduce o pausa; `Ctrl+R` reinicia; `Espacio` acciona la
  válvula. La pista de cada botón nombra su atajo.
- En la pestaña Simular, el grupo **Paso a paso** ofrece `Paso` y `Animar` en tamaño
  grande, y el icono de la válvula pasa a ser el de invertir, que es lo que hace.

#### Verificación

- Capturas fuera de pantalla: en pausa con `t 0,16 s` tras diez pasos sin accionar, y
  accionada con `t 0,19 s` tras doce pasos, con el cilindro al 100 % y 2,6 bar — el modelo
  avanza de verdad, un cuadro por pulsación.
- `--report-paints 4` en reposo: **1 repintado**; el paso no reactiva el bucle.
- `./scripts-dev/test.sh --no-build`: 134 pruebas en verde.


### Cinta con grupos, iconos del escritorio y transporte — 2026-11-06

La cinta pasa de ser una fila de botones a la forma de un ribbon de escritorio: pestañas
**Archivo · Editar · Ver · Simular**, grupos con su nombre debajo, botones grandes con el
icono encima y pequeños con el icono al lado, y el transporte (reproducir/pausa y
reiniciar) fijo a la derecha de las pestañas, así que ejecutar el modelo nunca exige
cambiar de pestaña.

- **Los iconos son los del escritorio.** `QIcon::fromTheme` los busca en Breeze —el mismo
  tema que usa el resto de la sesión— y solo tiñe los simbólicos, que son monocromos:
  teñir un icono a color lo convertía en una silueta negra. Un nombre que el tema no
  tiene deja el botón con su etiqueta sola, como hace una aplicación nativa.
- **`QIcon::themeSearchPaths()` está vacío fuera de una sesión de escritorio** (solo
  contenía `:/icons`), así que ninguna búsqueda funcionaba. Se añaden los directorios XDG
  que el tema de plataforma habría puesto; una sesión que ya los tiene no se toca.
- **Una pestaña sin comandos lo dice en una línea** en vez de mostrar botones grises.
  Archivo y Editar esperan al editor; Ver muestra el contraste medido del tema.
- **Por debajo de 1100 px la cinta se pliega**: quedan las pestañas y el transporte, y un
  botón **Menú** abre los comandos de la pestaña. La lista de comandos es una sola tabla y
  tanto los grupos como el menú se generan de ella.
- **Transporte y velocidad**: `FluidSession` gana `running` y `speed`; pausar detiene el
  bucle de la hoja sin tocar el modelo. Atajos: `Espacio` acciona la válvula, `F5`
  reproduce o pausa, `Ctrl+R` reinicia. Todos los botones son alcanzables con el teclado y
  exponen rol, nombre y descripción a la accesibilidad.
- **Tres banderas nuevas de diagnóstico**: `--check-icons` prueba el proveedor de iconos
  fuera de QML, `--size WxH` abre a un tamaño concreto y `--tab`/`--menu` dejan una
  pestaña o el menú plegado listos para una captura, sin puntero.

#### Verificación

- Capturas fuera de pantalla en claro y oscuro, con la cinta desplegada, la pestaña Ver y
  el menú plegado a 1000×700.
- `--check-icons`: los tres nombres resuelven a 24×24; `--check-theme`: 15,55:1 / 3,03:1 /
  4,84:1, sale 0.
- `--report-paints 4`: **1 repintado** con la ventana en reposo, así que la cinta nueva no
  reactivó el bucle.
- `./scripts-dev/test.sh --no-build`: 134 pruebas en verde.

#### Limitaciones conocidas

- El clic sobre los botones no está probado con un evento sintético; cada uno llama al
  mismo método del `FluidSession` que ya se prueba por el ABI.
- Archivo y Editar están vacías a propósito: guardar, abrir y exportar son M3 y M6.


### Armazón Qt Quick, perfil de memoria y una fuga de dibujo — 2026-11-06

[ADR 0005](docs/adr/0005-qt-quick-over-a-c-abi.md) pasa a **aceptado**: el modelo, el
catálogo, el formato y el dibujo siguen en Rust; la ventana es Qt Quick y habla con el
núcleo por un ABI C. `punchi-fluid-ui`, `punchi-fluid-desktop`, la mitad egui de
`punchi-fluid-theme` y `punchi-fluid-icons` se retiran del espacio de trabajo.

- **`apps/punchi-fluid-qt`**: ribbon de tres pestañas, barra lateral que refleja el
  catálogo con el símbolo propio de cada definición, hoja y barra de estado. Un binario
  de 7 MB con el núcleo enlazado estáticamente.
- **El catálogo cruza la frontera**: `pf_session_catalog` lista las definiciones en orden
  estable y `pf_session_preview` dibuja una sola, con el mismo código de dibujo que la
  hoja, para que la miniatura no pueda diferir del símbolo.
- **El tema se mide, no se supone**: `--check-theme` imprime los tres contrastes y falla
  si uno no llega (texto 4,5:1; secundario y acento 3:1). Un color del escritorio que no
  alcanza su criterio se desplaza lo justo para alcanzarlo, así que un acento invisible
  sobre el fondo deja de ser posible.
- **Se corrigió una fuga de memoria del dibujo.** `dashes()` podía girar para siempre: la
  fase del guion caía sobre un límite, el paso se redondeaba a cero en `f32` y el bucle
  añadía un punto por vuelta hasta que el núcleo mataba el proceso a **13,2 GB
  residentes**. Ahora el avance tiene un suelo y el bucle un presupuesto derivado de la
  longitud, de modo que un defecto futuro cuesta un dibujo malo y no la sesión.
- **La ventana deja de repintar cuando no hay nada que ver.** `FluidSession::settled`
  detiene el bucle tras medio segundo de quietud: una ventana en reposo pasa de **238
  repintados en cuatro segundos a 1**, y deja de mantener despierto al compositor de un
  panel de 144 Hz.
- **Perfil de compilación para la máquina real**: `.cargo/config.toml` con dos trabajos de
  `cargo` y solo tablas de líneas en `dev`; `scripts-dev/limits.sh` deriva de la memoria
  del equipo un tope por binario de prueba y `check.sh` ejecuta las pruebas por
  `test.sh`, con el tope aplicado.
- **Habilidad nueva** `machine-resource-budget`: la máquina medida, el presupuesto, cómo
  leer una muerte por OOM en `journalctl` y cómo acotar el culpable hasta una prueba.

#### Verificación

- `./scripts-dev/test.sh`: **134 pruebas en verde**, cada binario bajo un tope de 3,8 GB;
  ninguna muerte por memoria.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --check` limpios.
- `--report-paints 4` en reposo: **1 repintado**; accionada: 29 y se detiene.
- `--check-theme` en ambos esquemas: texto 11,54:1 / 15,55:1, secundario 3,52:1 / 3,03:1,
  acento 3,05:1 / 4,84:1. Ambos devuelven 0.
- Capturas fuera de pantalla en claro y oscuro, en reposo y accionada.

#### Limitaciones conocidas

- El clic real, extremo a extremo, sigue sin prueba automática: la de impacto es unitaria
  en Rust y ambas partes usan la misma función de encuadre.
- `QQuickPaintedItem` rasteriza el cuadro completo en CPU; con 200 componentes habrá que
  medirlo contra el presupuesto de 4 ms.
- La hoja dibuja un circuito fijo: colocar componentes desde la barra lateral es M3.

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
- `assets/catalog/`: `supply.compressor`, `valve.3_2.spring_return`,
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
