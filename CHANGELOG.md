# Changelog

Versión declarada: `0.1.0`, **sin publicar**. El trabajo pendiente se acumula bajo
`## Unreleased` en subsecciones fechadas; la versión cambia solo al preparar una
publicación.

## Unreleased

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

### La referencia se estudia antes de replicarla — 2026-11-06

Faltaba el paso entre «leer la referencia» y «programar», y ahora es una puerta:
[.agents/skills/reference-parity](.agents/skills/reference-parity/SKILL.md) fija el
procedimiento y [docs/reference-parity.md](docs/reference-parity.md) guarda la prueba de
que se siguió.

- **Ninguna capacidad empieza sin su observación escrita**: qué hace la referencia, visto
  dónde y cuándo, qué tenemos, qué falta y qué prueba lo cerrará. La lista de fases dice
  qué construir; el registro de estudio dice qué se observó. Una fila sin observación
  bloquea la capacidad, y el registro ya tiene la primera.
- **Qué se lee y qué no**: el manual `doc/fl_sim_p.pdf` para el significado, las 210
  definiciones de `sym/` para el inventario de funciones y los 533 circuitos de `dida/`
  para el comportamiento. Nada se copia: ni un `.ct` se parsea, ni una página del manual
  se pega, ni una captura de la referencia entra en `assets/`; su carpeta es de solo
  lectura y queda fuera del repositorio.
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
