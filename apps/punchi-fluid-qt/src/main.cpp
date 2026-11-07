// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The application: a Qt Quick window over the Rust core.
//!
//! Four flags make it testable without a display, which is how the drawing is verified:
//!
//! ```text
//! --grab <file>        render one frame offscreen and exit
//! --toggle             actuate the valve before that frame, the same path a click takes
//! --theme light|dark   force a palette instead of the desktop one
//! --check-theme        print the contrast ratios and exit 1 when one fails
//! --check-icons        resolve the ribbon's icons and print what the theme returned
//! --check-sounds       prove the packaged samples decode; exits 1 when one does not
//! --selftest           replay every sheet gesture with real events; exits 1 on failure
//! --probe              print the numbers the status bar shows, then exit 1
//! --open <file>        open a saved sheet before anything else
//! --save <file>        save the sheet and exit 0/1, without a window
//! ```

#include <QGuiApplication>
#include <QDir>
#include <QFile>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTimer>
#include <QVector>

#include <cmath>
#include <cstdio>
#include <functional>
#include <memory>
#include "canvasitem.h"
#include "icons.h"
#include "punchi_fluid.h"
#include "session.h"
#include "sounds.h"
#include "theme.h"

namespace {

/// QML problems seen while `--check-qml` is on. A `TypeError` in a binding is silent at
/// build time and fatal to the feature at run time, so it is counted and fails the run.
int qmlProblems = 0;
bool collecting = false;

void collectQmlMessage(QtMsgType type, const QMessageLogContext &context, const QString &message)
{
    if (collecting && (message.contains(QLatin1String(".qml:"))
                       || message.contains(QLatin1String("TypeError"))
                       || message.contains(QLatin1String("ReferenceError"))
                       || message.contains(QLatin1String("Unable to assign")))) {
        ++qmlProblems;
    }
    fprintf(stderr, "%s\n", qUtf8Printable(message));
    Q_UNUSED(type)
    Q_UNUSED(context)
}

int grabAndExit(QQmlApplicationEngine &engine, const QString &path)
{
    const auto roots = engine.rootObjects();
    if (roots.isEmpty()) {
        return 1;
    }
    auto *window = qobject_cast<QQuickWindow *>(roots.constFirst());
    if (window == nullptr) {
        return 1;
    }
    // Later than the first paint, so the capture shows the model stepped and the theme
    // applied, not an empty sheet.
    QTimer::singleShot(400, window, [window, path] {
        const QImage image = window->grabWindow();
        const bool saved = image.save(path);
        std::fprintf(stderr, "grab %s: %dx%d %s\n", qUtf8Printable(path), image.width(),
                     image.height(), saved ? "written" : "FAILED");
        QCoreApplication::exit(saved ? 0 : 1);
    });
    return 0;
}

} // namespace

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("Punchi Fluid Lab"));
    app.setOrganizationName(QStringLiteral("PunchiSoft"));

    const QStringList arguments = app.arguments();
    const auto value = [&arguments](const QString &flag) {
        const int index = arguments.indexOf(flag);
        return (index >= 0 && index + 1 < arguments.size()) ? arguments.at(index + 1) : QString();
    };
    const QString theme = value(QStringLiteral("--theme"));
    if (theme == QStringLiteral("light") || theme == QStringLiteral("dark")) {
        // Before the engine loads, so the session reads it from the application palette
        // exactly as it reads the desktop's.
        QGuiApplication::setPalette(forcedPalette(theme == QStringLiteral("dark")));
    } else if (!theme.isEmpty()) {
        std::fprintf(stderr, "--theme takes light or dark, not \"%s\"\n", qUtf8Printable(theme));
        return 2;
    }

    const bool checkQml = arguments.contains(QStringLiteral("--check-qml"));
    if (checkQml) {
        collecting = true;
        qInstallMessageHandler(collectQmlMessage);
    }

    if (arguments.contains(QStringLiteral("--check-icons"))) {
        // Proves the icon provider independently of QML: Qt strips console.log in release
        // builds, so a QML-only failure here would be invisible.
        ThemeIconProvider provider;
        const QStringList names {
            QStringLiteral("media-playback-start"),
            QStringLiteral("view-refresh"),
            QStringLiteral("open-menu-symbolic"),
        };
        ensureIconTheme();
        std::fprintf(stderr, "theme=%s paths=%s\n", qUtf8Printable(QIcon::themeName()),
                     qUtf8Printable(QIcon::themeSearchPaths().join(QLatin1Char(':'))));
        for (const QString &name : names) {
            QSize size;
            const QPixmap pixmap =
                provider.requestPixmap(name + QStringLiteral("?tone=foreground"), &size, QSize(24, 24));
            std::fprintf(stderr, "  %-24s %dx%d %s\n", qUtf8Printable(name), pixmap.width(),
                         pixmap.height(), pixmap.isNull() ? "null" : "ok");
        }
        return 0;
    }

    if (arguments.contains(QStringLiteral("--check-sounds"))) {
        // The files, not the mixer: whether the decoder accepts them is reported by the
        // QML side through its exit code, for the same reason the icon check lives here.
        // A missing resource or a renamed `.mp3` reads as silence in the mixer, so the
        // header is inspected where it can be named.
        const SoundSources sources = resolveSoundSources();
        int failures = 0;
        const auto report = [&failures](const char *name, const SoundSource &source) {
            const bool wave = isWaveFile(source.url);
            std::fprintf(stderr, "  %-6s %-9s %s %s\n", name, qUtf8Printable(source.origin),
                         qUtf8Printable(source.url.toString()),
                         wave ? "RIFF/WAVE" : "MISSING or not a WAV");
            failures += wave ? 0 : 1;
        };
        report("valve", sources.valve);
        report("air", sources.air);
        report("cyl", sources.cylinder);
        if (failures > 0) {
            return 1;
        }
    }

    if (arguments.contains(QStringLiteral("--check-theme"))) {
        const ThemeTokens tokens = tokensFrom(QGuiApplication::palette());
        const Contrast contrast = measure(tokens);
        std::fprintf(stderr, "contrast: %s\n", qUtf8Printable(describe(contrast)));
        return contrast.passes() ? 0 : 1;
    }

    PfAbi abi {};
    if (pf_abi(&abi) != PF_OK || abi.version != PF_ABI_VERSION) {
        std::fprintf(stderr, "the C++ header and the Rust core disagree on the ABI\n");
        return 2;
    }
    std::fprintf(stderr, "core %s, PfCall %u bytes, PfFrame %u bytes\n", pf_version(), abi.call_size,
                 abi.frame_size);

    const QString reportPaints = value(QStringLiteral("--report-paints"));
    const QString size = value(QStringLiteral("--size"));

    QQmlApplicationEngine engine;
    // The desktop's own icons, resolved through QIcon::fromTheme and tinted with the ink
    // the theme layer picked. A name the theme lacks draws nothing at all.
    engine.addImageProvider(QStringLiteral("theme"), new ThemeIconProvider);
    engine.loadFromModule("PunchiFluid", "Main");
    if (engine.rootObjects().isEmpty()) {
        return 1;
    }

    if (!size.isEmpty()) {
        const QStringList parts = size.split(QLatin1Char('x'));
        bool widthOk = false;
        bool heightOk = false;
        const int width = parts.value(0).toInt(&widthOk);
        const int height = parts.value(1).toInt(&heightOk);
        if (!widthOk || !heightOk || width < 640 || height < 480) {
            std::fprintf(stderr, "--size takes WxH, at least 640x480, not \"%s\"\n",
                         qUtf8Printable(size));
            return 2;
        }
        for (QObject *root : engine.rootObjects()) {
            auto *window = qobject_cast<QQuickWindow *>(root);
            if (window != nullptr) {
                window->resize(width, height);
            }
        }
    }

    if (arguments.contains(QStringLiteral("--tab")) || arguments.contains(QStringLiteral("--menu"))) {
        const QString tab = value(QStringLiteral("--tab"));
        QObject *ribbon = nullptr;
        for (QObject *root : engine.rootObjects()) {
            ribbon = root->findChild<QObject *>(QStringLiteral("ribbon"));
        }
        if (ribbon == nullptr) {
            std::fprintf(stderr, "no ribbon in the scene\n");
            return 1;
        }
        if (!tab.isEmpty()) {
            bool ok = false;
            const int index = tab.toInt(&ok);
            if (!ok || index < 0) {
                std::fprintf(stderr, "--tab takes a non-negative index\n");
                return 2;
            }
            ribbon->setProperty("current", index);
        }
        if (arguments.contains(QStringLiteral("--menu"))) {
            ribbon->setProperty("menuOpen", true);
        }
    }

    const auto sessions = [&engine]() {
        QList<FluidSession *> found;
        for (QObject *root : engine.rootObjects()) {
            found += root->findChildren<FluidSession *>();
        }
        return found;
    };

    const QString openPath = value(QStringLiteral("--open"));
    if (!openPath.isEmpty()) {
        for (FluidSession *session : sessions()) {
            if (!session->openFrom(openPath)) {
                std::fprintf(stderr, "--open failed: %s\n",
                             qUtf8Printable(session->lastError()));
            }
        }
    }

    const QString savePath = value(QStringLiteral("--save"));
    if (!savePath.isEmpty()) {
        bool saved = true;
        for (FluidSession *session : sessions()) {
            saved = session->saveTo(savePath) && saved;
        }
        std::fprintf(stderr, "%s: %s\n", saved ? "saved" : "save failed",
                     qUtf8Printable(savePath));
        return saved ? 0 : 1;
    }

    if (arguments.contains(QStringLiteral("--toggle"))) {
        for (FluidSession *session : sessions()) {
            session->pressValve();
        }
    }
    if (arguments.contains(QStringLiteral("--edit"))) {
        // The reference has no edit button: stopping the simulation returns to editing,
        // and that is the path the flag takes too.
        for (FluidSession *session : sessions()) {
            session->stopSimulation();
        }
    }
    int drop = 0;
    for (int at = 0; at + 1 < arguments.size(); ++at) {
        if (arguments.at(at) != QStringLiteral("--place")) {
            continue;
        }
        bool ok = false;
        const int catalogIndex = arguments.at(at + 1).toInt(&ok);
        if (!ok) {
            std::fprintf(stderr, "--place takes a catalog index, not \"%s\"\n",
                         qUtf8Printable(arguments.at(at + 1)));
            return 2;
        }
        // Free spots of the default composition, so a capture shows the drop without
        // covering the circuit the model is simulating.
        const QPointF spots[] = {QPointF(36.0, 112.0), QPointF(158.0, 112.0), QPointF(96.0, 22.0)};
        for (FluidSession *session : sessions()) {
            const QPointF spot = spots[drop % 3];
            session->placeComponent(catalogIndex, spot.x(), spot.y());
        }
        ++drop;
    }

    int at = 0;
    while (at + 1 < arguments.size()) {
        if (arguments.at(at) != QStringLiteral("--connect")) {
            ++at;
            continue;
        }
        const QStringList parts = arguments.at(at + 1).split(QLatin1Char(':'));
        bool aOk = false;
        bool bOk = false;
        const int aIndex = parts.value(0).toInt(&aOk);
        const int bIndex = parts.value(2).toInt(&bOk);
        if (parts.size() != 4 || !aOk || !bOk) {
            std::fprintf(stderr, "--connect takes a:port:b:port\n");
            return 2;
        }
        for (FluidSession *session : sessions()) {
            if (!session->connectPorts(aIndex, parts.at(1), bIndex, parts.at(3))) {
                std::fprintf(stderr, "--connect refused %s: %s\n",
                             qUtf8Printable(arguments.at(at + 1)),
                             qUtf8Printable(session->lastError()));
                return 1;
            }
        }
        at += 2;
    }

    const QString selectPipe = value(QStringLiteral("--select-pipe"));
    if (!selectPipe.isEmpty()) {
        bool ok = false;
        const int index = selectPipe.toInt(&ok);
        if (!ok) {
            std::fprintf(stderr, "--select-pipe takes an index\n");
            return 2;
        }
        for (FluidSession *session : sessions()) {
            session->selectPipe(index);
        }
    }
    at = 0;
    while (at + 1 < arguments.size()) {
        if (arguments.at(at) != QStringLiteral("--place-at")) {
            ++at;
            continue;
        }
        const QStringList parts = arguments.at(at + 1).split(QLatin1Char(':'));
        bool cOk = false;
        bool xOk = false;
        bool yOk = false;
        const int catalogIndex = parts.value(0).toInt(&cOk);
        const double x = parts.value(1).toDouble(&xOk);
        const double y = parts.value(2).toDouble(&yOk);
        if (parts.size() != 3 || !cOk || !xOk || !yOk) {
            std::fprintf(stderr, "--place-at takes c:x:y\n");
            return 2;
        }
        for (FluidSession *session : sessions()) {
            session->placeComponent(catalogIndex, x, y);
        }
        at += 2;
    }

    const QString zoom = value(QStringLiteral("--zoom"));
    if (!zoom.isEmpty()) {
        bool ok = false;
        const double factor = zoom.toDouble(&ok);
        if (!ok || factor <= 0.0) {
            std::fprintf(stderr, "--zoom takes a positive factor, not \"%s\"\n",
                         qUtf8Printable(zoom));
            return 2;
        }
        for (FluidSession *session : sessions()) {
            session->setZoom(factor);
        }
    }
    if (arguments.contains(QStringLiteral("--no-grid"))) {
        for (FluidSession *session : sessions()) {
            session->setGrid(false);
        }
    }

    const QString tee = value(QStringLiteral("--tee"));
    if (!tee.isEmpty()) {
        // pipe:x:y:component:port — the splice the drop gesture performs, without a pointer.
        const QStringList parts = tee.split(QLatin1Char(':'));
        bool ok = parts.size() == 5;
        const int pipe = parts.value(0).toInt(&ok);
        const double x = parts.value(1).toDouble(&ok);
        const double y = parts.value(2).toDouble(&ok);
        const int attach = parts.value(3).toInt(&ok);
        if (!ok) {
            std::fprintf(stderr, "--tee takes pipe:x:y:component:port, not \"%s\"\n",
                         qUtf8Printable(tee));
            return 2;
        }
        for (FluidSession *session : sessions()) {
            session->splicePipeAt(pipe, x, y, attach, parts.value(4));
        }
    }

    if (arguments.contains(QStringLiteral("--check"))) {
        for (FluidSession *session : sessions()) {
            session->checkCircuit();
            std::fprintf(stderr, "check: %d falta(s)\n", session->checkCount());
            std::fprintf(stderr, "%s", qUtf8Printable(session->checkReport()));
        }
    }
    if (arguments.contains(QStringLiteral("--delete"))) {
        for (FluidSession *session : sessions()) {
            session->removeSelected();
        }
    }

    const QString undos = value(QStringLiteral("--undo"));
    if (!undos.isEmpty()) {
        bool ok = false;
        const int count = undos.toInt(&ok);
        if (!ok || count < 0) {
            std::fprintf(stderr, "--undo takes a non-negative count\n");
            return 2;
        }
        for (FluidSession *session : sessions()) {
            for (int step = 0; step < count; ++step) {
                session->undo();
            }
        }
    }

    if (arguments.contains(QStringLiteral("--inspect"))) {
        for (FluidSession *session : sessions()) {
            session->inspectValve();
        }
    }
    if (arguments.contains(QStringLiteral("--pause"))) {
        for (FluidSession *session : sessions()) {
            session->pauseSimulation();
        }
    }
    const QString steps = value(QStringLiteral("--step"));
    if (!steps.isEmpty()) {
        bool ok = false;
        const int count = steps.toInt(&ok);
        if (!ok || count < 0) {
            std::fprintf(stderr, "--step takes a non-negative count, not \"%s\"\n",
                         qUtf8Printable(steps));
            return 2;
        }
        for (FluidSession *session : sessions()) {
            for (int i = 0; i < count; ++i) {
                session->stepForward();
            }
        }
    }

    if (arguments.contains(QStringLiteral("--probe"))) {
        // Every value the status bar shows, printed, so a run can be compared against a
        // hand calculation instead of against an impression of the window.
        for (FluidSession *session : sessions()) {
            std::fprintf(stderr,
                         "probe: valve=%d bar=%.2f stroke=%.1f mm speed=%.1f mm/s t=%.2f s "
                         "running=%d editing=%d components=%u pipes=%u findings=%d\n",
                         session->valvePosition(), session->pressureBar(),
                         session->positionMm(), session->velocityMmS(), session->timeS(),
                         session->running() ? 1 : 0, session->editing() ? 1 : 0,
                         pf_session_component_count(session->handle()),
                         pf_session_connection_count(session->handle()), session->checkCount());
        }
        return 1;
    }

    if (!reportPaints.isEmpty()) {
        bool ok = false;
        const double seconds = reportPaints.toDouble(&ok);
        if (!ok || seconds <= 0.0) {
            std::fprintf(stderr, "--report-paints takes a positive number of seconds\n");
            return 2;
        }
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst());
        if (window == nullptr) {
            return 1;
        }
        const auto sheets = window->findChildren<SchematicItem *>();
        if (sheets.isEmpty()) {
            std::fprintf(stderr, "no sheet in the scene\n");
            return 1;
        }
        QTimer::singleShot(static_cast<int>(seconds * 1000.0), window, [sheets] {
            int total = 0;
            for (const SchematicItem *sheet : sheets) {
                total += sheet->paintCount();
            }
            std::fprintf(stderr, "paints in the window: %d\n", total);
            QCoreApplication::exit(0);
        });
        return app.exec();
    }

    if (arguments.contains(QStringLiteral("--selftest"))) {
        // Every gesture the sheet claims to support, driven by real mouse and key events so
        // that a missing `acceptedMouseButtons` or a broken mapping fails here instead of in
        // the user's hands. Steps run 350 ms apart: one frame has to be painted between them.
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst());
        struct SelfTest {
            QQuickWindow *window = nullptr;
            SchematicItem *sheet = nullptr;
            FluidSession *session = nullptr;
            int failures = 0;
            int step = 0;
            int valveIndex = -1;
            double valveX = 0.0;
            uint32_t connections = 0;
            uint32_t pipes = 0;
            uint32_t components = 0;
            QPointF valveScene;
            QPointF paletteScene;
            QPointF fromScene;
            QPointF toScene;
            QPointF routeScene;
            int routePipe = -1;
            float routeOffset = 0.0F;
        };
        auto *state = new SelfTest();
        state->window = window;
        state->sheet = window != nullptr ? window->findChild<SchematicItem *>() : nullptr;
        const auto sessions = window != nullptr ? window->findChildren<FluidSession *>()
                                                : QList<FluidSession *>();
        state->session = sessions.isEmpty() ? nullptr : sessions.constFirst();
        if (state->sheet == nullptr || state->session == nullptr) {
            std::fprintf(stderr, "selftest: no hay lienzo o no hay sesión\n");
            return 1;
        }
        FluidSession *session = state->session;

        const auto check = [state](bool ok, const QString &what) {
            std::fprintf(stderr, "selftest: %-48s %s\n", qPrintable(what), ok ? "ok" : "FALLO");
            if (!ok) {
                ++state->failures;
            }
        };
        const auto send = [state](QEvent::Type type, const QPointF &scene, Qt::MouseButton button,
                                  Qt::MouseButtons buttons) {
            QMouseEvent event(type, scene, state->window->mapToGlobal(scene), button, buttons,
                              Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &event);
        };
        const auto sendWith = [state](QEvent::Type type, const QPointF &scene,
                                      Qt::MouseButton button, Qt::MouseButtons buttons,
                                      Qt::KeyboardModifiers modifiers) {
            QMouseEvent event(type, scene, state->window->mapToGlobal(scene), button, buttons,
                              modifiers);
            QCoreApplication::sendEvent(state->window, &event);
        };
        const auto toScene = [state](const QPointF &sheetMm) {
            return state->sheet->mapToScene(state->sheet->scenePointForSheet(sheetMm));
        };
        const auto valveCentre = [state, session] {
            float bounds[4] = {0.0F, 0.0F, 0.0F, 0.0F};
            pf_session_valve_rect(session->handle(), bounds);
            return QPointF((bounds[0] + bounds[2]) * 0.5, (bounds[1] + bounds[3]) * 0.5);
        };
        const auto componentRect = [session](int index) {
            float bounds[4] = {0.0F, 0.0F, 0.0F, 0.0F};
            pf_session_component_rect(session->handle(), static_cast<uint32_t>(index), bounds);
            return QRectF(QPointF(bounds[0], bounds[1]), QPointF(bounds[2], bounds[3]));
        };

        auto steps = std::make_shared<QVector<std::function<void()>>>();

        // The program opens in edit mode, as the reference does. A click there selects and
        // must not actuate anything: the model only moves once the user asks for it.
        steps->push_back([state, check, send, valveCentre, toScene] {
            check(state->session->editing() && !state->session->running(),
                  QStringLiteral("arranque: abre en edición, sin simular"));
            state->valveScene = toScene(valveCentre());
            send(QEvent::MouseButtonPress, state->valveScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonRelease, state->valveScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            std::fprintf(stderr, "selftest: en edición, válvula %d y %.2f bar\n",
                         state->session->valvePosition(), state->session->pressureBar());
            check(state->session->valvePosition() == 0,
                  QStringLiteral("edición: pulsar la válvula no la acciona"));
            check(state->session->pressureBar() < 1.5,
                  QStringLiteral("edición: la presión no sube sin simular"));
            check(state->session->running() == false,
                  QStringLiteral("edición: pulsar la válvula no arranca la simulación"));
            state->session->startSimulation();
            check(!state->session->editing() && state->session->running(),
                  QStringLiteral("Iniciar entra en simulación"));
        });

        // Simulation: the click is a pushbutton. It opens while held and closes on release.
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonPress, state->valveScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            std::fprintf(stderr, "selftest: válvula %d, %.2f bar, %.1f mm, clic en (%.0f, %.0f)\n",
                         state->session->valvePosition(), state->session->pressureBar(),
                         state->session->positionMm(), state->valveScene.x(),
                         state->valveScene.y());
            check(state->session->valvePosition() == 1,
                  QStringLiteral("simulación: mantener el clic abre la válvula"));
            check(state->session->pressureBar() > 2.0,
                  QStringLiteral("simulación: la cámara recibe presión"));
            check(state->session->positionMm() > 5.0,
                  QStringLiteral("simulación: el vástago avanza"));
            send(QEvent::MouseButtonRelease, state->valveScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            check(state->session->valvePosition() == 0,
                  QStringLiteral("simulación: soltar cierra la válvula"));
            state->session->stopSimulation();
            check(state->session->editing(), QStringLiteral("transporte: Detener vuelve a editar"));
        });

        // The properties form edits the instance, and the model is rebuilt from it: the
        // reference lets a cylinder in one circuit be larger without touching the catalog.
        steps->push_back([state, check] {
            state->session->inspectComponent(2);
            const QVariantList fields = state->session->inspectorFields();
            check(!fields.isEmpty(),
                  QStringLiteral("ficha: el cilindro trae sus parámetros"));
            QVariantList values;
            for (const QVariant &entry : fields) {
                QVariantMap row = entry.toMap();
                if (row.value(QStringLiteral("key")).toString()
                    == QLatin1String("stroke_mm")) {
                    row.insert(QStringLiteral("value"), 50.0);
                }
                values.append(row);
            }
            state->session->applyInspectorEdits(values);
            PfTelemetry edited {};
            pf_session_telemetry(state->session->handle(), &edited);
            check(std::abs(edited.stroke_m - 0.05) < 1e-6,
                  QStringLiteral("ficha: la carrera editada llega al modelo"));
            state->session->undo();
            PfTelemetry restored {};
            pf_session_telemetry(state->session->handle(), &restored);
            check(std::abs(restored.stroke_m - 0.10) < 1e-6,
                  QStringLiteral("ficha: deshacer devuelve la del archivo"));
            state->session->closeInspector();
        });

        // Editing: a click selects and opens the properties, a drag moves, Delete removes.
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonPress, state->valveScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonRelease, state->valveScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check, send, componentRect] {
            state->valveIndex = state->session->selected();
            check(state->valveIndex >= 0, QStringLiteral("edición: el clic selecciona"));
            check(state->session->inspectorOpen(), QStringLiteral("edición: la ficha se abre"));
            if (state->valveIndex >= 0) {
                state->valveX = componentRect(state->valveIndex).x();
            }
            send(QEvent::MouseButtonPress, state->valveScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseMove, state->valveScene + QPointF(90.0, 0.0), Qt::NoButton,
                 Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseButtonRelease, state->valveScene + QPointF(90.0, 0.0),
                 Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check, send, toScene, componentRect] {
            check(componentRect(state->valveIndex).x() > state->valveX,
                  QStringLiteral("edición: arrastrar mueve"));
            state->connections = pf_session_connection_count(state->session->handle());
            int cylinder = -1;
            char id[128] = {};
            const uint32_t count = pf_session_component_count(state->session->handle());
            for (uint32_t i = 0; i < count; ++i) {
                id[0] = '\0';
                pf_session_component_id(state->session->handle(), i, id, sizeof id);
                std::fprintf(stderr, "selftest: componente %u = %s\n", i, id);
                if (QString::fromUtf8(id).startsWith(QLatin1String("cylinder"))) {
                    cylinder = static_cast<int>(i);
                }
            }
            QPointF from;
            QPointF to;
            const bool hasFrom = state->valveIndex >= 0
                && state->session->portPoint(state->valveIndex, QStringLiteral("3"), &from);
            const bool hasTo = cylinder >= 0
                && state->session->portPoint(cylinder, QStringLiteral("1"), &to);
            std::fprintf(stderr,
                         "selftest: válvula %d puerto 3 %s, cilindro %d puerto 1 %s, %.1f tubo(s)\n",
                         state->valveIndex, hasFrom ? "sí" : "no", cylinder, hasTo ? "sí" : "no",
                         static_cast<double>(state->connections));
            check(hasFrom && hasTo, QStringLiteral("edición: los puertos se localizan"));
            if (hasFrom && hasTo) {
                state->fromScene = toScene(from);
                state->toScene = toScene(to);
                send(QEvent::MouseButtonPress, state->fromScene, Qt::LeftButton, Qt::LeftButton);
            }
        });
        steps->push_back([state, send] {
            send(QEvent::MouseMove, state->toScene, Qt::NoButton, Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseButtonRelease, state->toScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check, send] {
            const uint32_t now = pf_session_connection_count(state->session->handle());
            check(now == state->connections + 1,
                  QStringLiteral("edición: puerto a puerto dibuja un tubo"));
            state->components = pf_session_component_count(state->session->handle());
            send(QEvent::MouseButtonPress, state->valveScene + QPointF(90.0, 0.0), Qt::LeftButton,
                 Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseButtonRelease, state->valveScene + QPointF(90.0, 0.0),
                 Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            QKeyEvent press(QEvent::KeyPress, Qt::Key_Delete, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &press);
            const uint32_t now = pf_session_component_count(state->session->handle());
            check(now + 1 == state->components,
                  QStringLiteral("edición: Supr borra el componente seleccionado"));
            state->session->undo();
        });

        // Copy, paste and duplicate are one document change each, and Shift+drag copies
        // the selection instead of moving it.
        steps->push_back([state, check] {
            state->session->select(state->valveIndex);
            const uint32_t before = pf_session_component_count(state->session->handle());
            state->session->copySelected();
            state->session->pasteOffset();
            check(pf_session_component_count(state->session->handle()) == before + 1,
                  QStringLiteral("edición: pegar crea una copia"));
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == before,
                  QStringLiteral("edición: deshacer el pegado quita la copia"));
            const int copy = state->session->duplicateComponent(state->valveIndex, 10.0, 10.0);
            check(copy >= 0
                      && pf_session_component_count(state->session->handle()) == before + 1,
                  QStringLiteral("edición: duplicar crea una copia"));
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == before,
                  QStringLiteral("edición: deshacer la copia la quita"));
            state->components = before;
        });
        steps->push_back([state, sendWith, toScene, componentRect] {
            const QRectF box = componentRect(state->valveIndex);
            state->routeScene = toScene(box.center());
            sendWith(QEvent::MouseButtonPress, state->routeScene, Qt::LeftButton, Qt::LeftButton,
                     Qt::ShiftModifier);
        });
        steps->push_back([state, sendWith] {
            sendWith(QEvent::MouseMove, state->routeScene + QPointF(20.0, 0.0), Qt::NoButton,
                     Qt::LeftButton, Qt::ShiftModifier);
        });
        steps->push_back([state, sendWith, check] {
            sendWith(QEvent::MouseButtonRelease, state->routeScene + QPointF(20.0, 0.0),
                     Qt::LeftButton, Qt::NoButton, Qt::ShiftModifier);
            check(pf_session_component_count(state->session->handle()) == state->components + 1,
                  QStringLiteral("edición: Mayús al arrastrar copia"));
            state->session->undo();
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == state->components,
                  QStringLiteral("edición: dos deshaceres quitan la copia movida"));
        });

        // The sound of a circuit is the piston hitting an end of its stroke, once per
        // arrival. Driven by explicit frames rather than the wall clock, so the claim is a
        // number and not a race against a timer.
        steps->push_back([state, check] {
            // Back to the circuit the sheet started with. The editing steps above added
            // hoses of their own, and the model now runs the document: an exhaust port
            // wired to a cylinder does not vent, so those edits would change the circuit
            // this block is about.
            while (state->session->canUndo()) {
                state->session->undo();
            }
            // From rest, so the block does not inherit where an earlier one left the piston.
            state->session->stopSimulation();
            state->session->restart();
            state->session->startSimulation();
            auto *counter = new int(0);
            QObject::connect(state->session, &FluidSession::cylinderReachedEnd, state->session,
                             [counter] { ++*counter; });
            state->session->pressValve();
            // Off the stop and travelling, not far enough to have arrived: the supply fills
            // this chamber fast enough to cross the whole stroke in about five frames.
            for (int i = 0; i < 2; ++i) {
                state->session->stepForward();
            }
            std::fprintf(stderr, "selftest: a media carrera, %d mm y %d impacto(s)\n",
                         static_cast<int>(state->session->positionMm()), *counter);
            check(*counter == 0,
                  QStringLiteral("sonido: no hay impacto mientras el vástago viaja"));
            check(state->session->positionMm() > 5.0,
                  QStringLiteral("sonido: el vástago se mueve de verdad"));

            // On to the stop. Forty frames is 0.64 s, past the 0.48 s a full stroke takes.
            for (int i = 0; i < 40; ++i) {
                state->session->stepForward();
            }
            std::fprintf(stderr, "selftest: en el tope, %d mm y %d impacto(s)\n",
                         static_cast<int>(state->session->positionMm()), *counter);
            check(*counter == 1, QStringLiteral("sonido: llegar al tope se anuncia una vez"));

            // Staying there, and stepping, must not repeat the impact.
            for (int i = 0; i < 40; ++i) {
                state->session->stepForward();
            }
            check(*counter == 1,
                  QStringLiteral("sonido: parado en el tope no repite el impacto"));

            // Releasing lets the spring bring it back, and that arrival is its own event.
            state->session->releaseValve();
            // The network's chamber is bigger than the fixed model's: its hose and valve
            // galleries hold gas too, so venting takes longer than one second.
            for (int i = 0; i < 200; ++i) {
                state->session->stepForward();
            }
            std::fprintf(stderr, "selftest: de vuelta, %d mm y %d impacto(s)\n",
                         static_cast<int>(state->session->positionMm()), *counter);
            check(*counter == 2,
                  QStringLiteral("sonido: volver al otro tope es un segundo impacto"));
            state->session->stopSimulation();
        });

        // Ctrl and a double click turns the component under the pointer, which is the
        // reference's gesture. The port travels with the symbol and one undo puts both
        // back, because a turn is one change to the document.
        steps->push_back([state, check, sendWith, toScene, componentRect] {
            // The turn is an editing gesture: the sheet has to be out of simulation first.
            state->session->stopSimulation();
            const uint32_t index = static_cast<uint32_t>(state->valveIndex);
            state->routeScene = toScene(componentRect(state->valveIndex).center());
            float before[2] = {0.0F, 0.0F};
            pf_session_port_point(state->session->handle(), index, "2", before);
            // Two clicks in place, which is the gesture: the item turns the component on
            // the second one, exactly as a hand doing Ctrl and a double click does.
            sendWith(QEvent::MouseButtonPress, state->routeScene, Qt::LeftButton, Qt::LeftButton,
                     Qt::ControlModifier);
            sendWith(QEvent::MouseButtonRelease, state->routeScene, Qt::LeftButton, Qt::NoButton,
                     Qt::ControlModifier);
            sendWith(QEvent::MouseButtonPress, state->routeScene, Qt::LeftButton, Qt::LeftButton,
                     Qt::ControlModifier);
            sendWith(QEvent::MouseButtonRelease, state->routeScene, Qt::LeftButton, Qt::NoButton,
                     Qt::ControlModifier);
            uint32_t orientation[2] = {0, 0};
            pf_session_component_orientation(state->session->handle(), index, orientation);
            check(orientation[0] == 3,
                  QStringLiteral("edición: Ctrl y doble clic gira al revés"));
            float after[2] = {0.0F, 0.0F};
            pf_session_port_point(state->session->handle(), index, "2", after);
            check(std::abs(before[0] - after[0]) > 0.5 || std::abs(before[1] - after[1]) > 0.5,
                  QStringLiteral("edición: el puerto gira con el símbolo"));
            state->session->undo();
            orientation[0] = 9;
            pf_session_component_orientation(state->session->handle(), index, orientation);
            check(orientation[0] == 0,
                  QStringLiteral("edición: un deshacer devuelve el símbolo a su sitio"));
        });
        // Alt+M reflects it, and reflecting twice is the symbol as it was drawn.
        steps->push_back([state, check] {
            const int index = state->valveIndex;
            state->session->select(index);
            state->session->mirrorSelected();
            uint32_t orientation[2] = {9, 9};
            pf_session_component_orientation(state->session->handle(),
                                             static_cast<uint32_t>(index), orientation);
            check(orientation[0] == 0 && orientation[1] == 1,
                  QStringLiteral("edición: reflejar no gira, invierte"));
            state->session->mirrorSelected();
            pf_session_component_orientation(state->session->handle(),
                                             static_cast<uint32_t>(index), orientation);
            check(orientation[1] == 0,
                  QStringLiteral("edición: reflejar dos veces deja el símbolo igual"));
            state->session->undo();
            state->session->undo();
        });

        // Run until a state change: the transport stops on the frame the piston reaches
        // the stop, which is what the reference's own run mode does.
        steps->push_back([state, check] {
            state->session->restart();
            state->session->pressValve();
            const bool changed = state->session->runUntilChange();
            check(changed, QStringLiteral("transporte: hasta cambio se detiene"));
            check(state->session->positionMm() > 99.0,
                  QStringLiteral("transporte: se detiene en el tope"));
            check(!state->session->running(),
                  QStringLiteral("transporte: queda en pausa"));
            state->session->releaseValve();
            state->session->stopSimulation();
        });

        // The view: zoom and the grid change what the sheet looks like, never the document.
        steps->push_back([state, check] {
            const bool undoes = state->session->canUndo();
            state->session->zoomIn();
            check(state->session->zoom() > 1.0, QStringLiteral("vista: aumentar acerca la hoja"));
            state->session->zoomOut();
            state->session->zoomOut();
            check(state->session->zoom() < 1.0,
                  QStringLiteral("vista: disminuir aleja la hoja"));
            state->session->zoomReset();
            check(qFuzzyCompare(state->session->zoom(), 1.0),
                  QStringLiteral("vista: tamaño original vuelve al encaje"));
            const bool grid = state->session->grid();
            state->session->toggleGrid();
            check(state->session->grid() != grid,
                  QStringLiteral("vista: la cuadrícula se enciende y se apaga"));
            state->session->toggleGrid();
            check(state->session->canUndo() == undoes,
                  QStringLiteral("vista: el zoom no entra en el historial"));
        });

        // A hose is re-routed by dragging its middle segment. The press selects it and the
        // drag moves it; one undo restores the route, because one gesture is one edit.
        steps->push_back([state, check, send, toScene] {
            for (double x = 10.0; x <= 190.0 && state->routePipe < 0; x += 4.0) {
                for (double y = 10.0; y <= 120.0; y += 4.0) {
                    int portIndex = -1;
                    QString port;
                    if (state->session->componentAt(x, y) >= 0
                        || state->session->portAt(x, y, 3.0, &portIndex, &port)) {
                        continue;
                    }
                    const int hit = state->session->pipeAt(x, y, 2.5);
                    if (hit >= 0) {
                        state->routePipe = hit;
                        state->routeScene = toScene(QPointF(x, y));
                        break;
                    }
                }
            }
            check(state->routePipe >= 0,
                  QStringLiteral("edición: se encuentra un conducto que arrastrar"));
            float offset = 0.0F;
            pf_session_pipe_offset(state->session->handle(),
                                   static_cast<uint32_t>(state->routePipe), &offset);
            state->routeOffset = offset;
            send(QEvent::MouseButtonPress, state->routeScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseMove, state->routeScene + QPointF(18.0, 18.0), Qt::NoButton,
                 Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonRelease, state->routeScene + QPointF(18.0, 18.0),
                 Qt::LeftButton, Qt::NoButton);
            float offset = 0.0F;
            pf_session_pipe_offset(state->session->handle(),
                                   static_cast<uint32_t>(state->routePipe), &offset);
            check(std::abs(offset - state->routeOffset) > 0.5,
                  QStringLiteral("edición: arrastrar el segmento reencamina el tubo"));
            state->session->undo();
            pf_session_pipe_offset(state->session->handle(),
                                   static_cast<uint32_t>(state->routePipe), &offset);
            check(std::abs(offset - state->routeOffset) < 0.01,
                  QStringLiteral("edición: un deshacer devuelve el recorrido"));
        });

        // A hose end dropped on a hose leaves a T, and the whole splice is one undo.
        steps->push_back([state, check, send, toScene, componentRect] {
            char id[128] = {};
            int cylinder = -1;
            const uint32_t count = pf_session_component_count(state->session->handle());
            for (uint32_t i = 0; i < count; ++i) {
                id[0] = '\0';
                pf_session_component_id(state->session->handle(), i, id, sizeof id);
                if (QString::fromUtf8(id).startsWith(QLatin1String("cylinder"))) {
                    cylinder = static_cast<int>(i);
                }
            }
            QPointF start;
            if (cylinder < 0
                || !state->session->portPoint(cylinder, QStringLiteral("1"), &start)) {
                check(false, QStringLiteral("T: el cilindro tiene puerto"));
                return;
            }
            state->pipes = pf_session_connection_count(state->session->handle());
            state->components = pf_session_component_count(state->session->handle());
            // Find a point that really is on a hose by asking the model, rather than assuming
            // where the orthogonal route runs.
            QPointF onPipe;
            bool found = false;
            for (double x = 10.0; x < 190.0 && !found; x += 2.0) {
                for (double y = 10.0; y < 125.0 && !found; y += 2.0) {
                    int portIndex = -1;
                    QString port;
                    // Away from every port, or the release would be read as an ordinary
                    // port-to-port connection instead of a junction on the hose.
                    if (state->session->pipeAt(x, y, 1.0) >= 0
                        && !state->session->portAt(x, y, 4.0, &portIndex, &port)) {
                        onPipe = QPointF(x, y);
                        found = true;
                    }
                }
            }
            check(found, QStringLiteral("T: hay un conducto donde soltar"));
            if (!found) {
                return;
            }
            std::fprintf(stderr, "selftest: tubo bajo (%.0f, %.0f)\n", onPipe.x(), onPipe.y());
            state->toScene = toScene(onPipe);
            send(QEvent::MouseButtonPress, toScene(start), Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseMove, state->toScene, Qt::NoButton, Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseButtonRelease, state->toScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            const uint32_t pipes = pf_session_connection_count(state->session->handle());
            const uint32_t components = pf_session_component_count(state->session->handle());
            std::fprintf(stderr, "selftest: tras la T, %u tubo(s) y %u componente(s)\n", pipes,
                         components);
            check(components == state->components + 1,
                  QStringLiteral("T: la unión aparece en el sitio del soltado"));
            check(pipes == state->pipes + 2,
                  QStringLiteral("T: el tubo se parte en dos y llega el tercero"));
            state->session->undo();
            check(pf_session_connection_count(state->session->handle()) == state->pipes
                      && pf_session_component_count(state->session->handle()) == state->components,
                  QStringLiteral("T: un solo deshacer deshace la unión entera"));
        });

        // Dragging out of the palette, which is how a circuit is actually drawn. The drop is
        // driven through the sidebar's own mouse area, so the mapping from the palette to the
        // sheet is the one the user gets.
        steps->push_back([state, check, send] {
            // The palette rows live inside a ListView, whose delegates are children of its
            // content item rather than of the window, so the tree is walked by hand.
            QQuickItem *row = nullptr;
            std::function<void(QQuickItem *)> seek = [&](QQuickItem *item) {
                // A MouseArea written inline in QML gets a generated class name
                // (QQuickMouseArea_QML_3), so the test asks by inheritance, not by name.
                if (row == nullptr && item->isVisible() && item->inherits("QQuickMouseArea")
                    && item->width() > 200.0 && item->height() > 40.0) {
                    row = item;
                    return;
                }
                for (QQuickItem *child : item->childItems()) {
                    seek(child);
                }
            };
            seek(state->window->contentItem());
            check(row != nullptr, QStringLiteral("paleta: la fila del catálogo se encuentra"));
            if (row == nullptr) {
                return;
            }
            const QPointF centre = row->mapToScene(row->boundingRect().center());
            std::fprintf(stderr, "selftest: fila de la paleta en (%.0f, %.0f)\n", centre.x(),
                         centre.y());
            state->paletteScene = centre;
            state->components = pf_session_component_count(state->session->handle());
            send(QEvent::MouseButtonPress, centre, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send, toScene] {
            const QPointF landing = toScene(QPointF(150.0, 40.0));
            send(QEvent::MouseMove, landing, Qt::NoButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send, toScene] {
            const QPointF landing = toScene(QPointF(150.0, 40.0));
            send(QEvent::MouseButtonRelease, landing, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check, componentRect] {
            const uint32_t now = pf_session_component_count(state->session->handle());
            check(now == state->components + 1,
                  QStringLiteral("paleta: soltar en la hoja coloca el componente"));
            if (now == state->components + 1) {
                const QRectF box = componentRect(static_cast<int>(now) - 1);
                std::fprintf(stderr, "selftest: colocado en (%.0f, %.0f)\n", box.center().x(),
                             box.center().y());
                check(std::abs(box.center().x() - 150.0) < 12.0
                          && std::abs(box.center().y() - 40.0) < 12.0,
                      QStringLiteral("paleta: cae donde se suelta, no en otro sitio"));
            }
        });

        // Saving and opening a sheet: the file has to carry the placements, the pipes,
        // their routes and the per-instance values, not just a list of names.
        steps->push_back([state, check] {
            const QString path = QDir::tempPath() + QStringLiteral("/punchi-selftest.pfluid");
            const uint32_t before = pf_session_component_count(state->session->handle());
            const uint32_t pipes = pf_session_connection_count(state->session->handle());
            check(state->session->saveTo(path), QStringLiteral("documento: se guarda"));
            // Change the document, then open the file: the change must disappear.
            if (state->session->componentCount() > 0) {
                state->session->removeComponent(0);
            }
            check(state->session->openFrom(path), QStringLiteral("documento: se abre"));
            check(pf_session_component_count(state->session->handle()) == before,
                  QStringLiteral("documento: vuelven los mismos componentes"));
            check(pf_session_connection_count(state->session->handle()) == pipes,
                  QStringLiteral("documento: vuelven los mismos tubos"));
            QFile::remove(path);
        });

        QTimer::singleShot(400, window, [state, steps] {
            auto *timer = new QTimer(state->window);
            timer->setInterval(350);
            QObject::connect(timer, &QTimer::timeout, state->window, [state, steps, timer] {
                if (state->step >= steps->size()) {
                    timer->stop();
                    std::fprintf(stderr, "selftest: %d fallo(s)\n", state->failures);
                    QCoreApplication::exit(state->failures == 0 ? 0 : 1);
                    return;
                }
                (*steps)[state->step++]();
            });
            timer->start();
        });
        return app.exec();
    }

    if (checkQml) {
        // Every ribbon command runs once, so a handler that throws fails here instead of in
        // the user's hands. Qt strips console.log in release builds, so the count is printed
        // from C++ and the run fails on it.
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst());
        QTimer::singleShot(600, window, [window] {
            QObject *ribbon = window != nullptr ? window->findChild<QObject *>(QStringLiteral("ribbon"))
                                                : nullptr;
            if (ribbon == nullptr) {
                std::fprintf(stderr, "qml problems: the ribbon is not in the scene\n");
                QCoreApplication::exit(1);
                return;
            }
            QMetaObject::invokeMethod(ribbon, "selfTest");
            collecting = false;
            std::fprintf(stderr, "qml problems: %d\n", qmlProblems);
            QCoreApplication::exit(qmlProblems == 0 ? 0 : 1);
        });
        return app.exec();
    }

    const QString grab = value(QStringLiteral("--grab"));
    if (!grab.isEmpty()) {
        if (grabAndExit(engine, grab) != 0) {
            return 1;
        }
        // The grab is scheduled on a timer, so the event loop has to run for it to fire.
        return app.exec();
    }
    return app.exec();
}
