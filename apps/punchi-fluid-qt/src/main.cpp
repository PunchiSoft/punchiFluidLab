// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The application: a Qt Quick window over the Rust core.
//!
//! Four flags make it testable without a display, which is how the drawing is verified:
//!
//! ```text
//! --grab <file>        render one frame offscreen and exit
//! --toggle             actuate the valve before that frame, the same path a click takes
//! --confirm <what>     open the unsaved-work confirmation (`new`, `open` or `close`)
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

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <limits>
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

    // The self test turns sheet millimetres into window pixels to drive its gestures, so it
    // needs a viewport where the whole sheet is on screen and the canvas does not scroll
    // under it: a window whose aspect does not match the sheet's moves the drawing between
    // the press and the release, and the gesture then lands somewhere the test did not mean.
    // `--size` still overrides this.
    const QString pinned = arguments.contains(QStringLiteral("--selftest")) && size.isEmpty()
                               ? QStringLiteral("1280x800")
                               : size;

    if (!pinned.isEmpty()) {
        const QStringList parts = pinned.split(QLatin1Char('x'));
        bool widthOk = false;
        bool heightOk = false;
        const int width = parts.value(0).toInt(&widthOk);
        const int height = parts.value(1).toInt(&heightOk);
        if (!widthOk || !heightOk || width < 640 || height < 480) {
            std::fprintf(stderr, "--size takes WxH, at least 640x480, not \"%s\"\n",
                         qUtf8Printable(pinned));
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

    const QString exportPath = value(QStringLiteral("--export"));
    if (!exportPath.isEmpty()) {
        const QString paperArg = value(QStringLiteral("--paper"));
        const QString dpiArg = value(QStringLiteral("--dpi"));
        bool exported = true;
        for (FluidSession *session : sessions()) {
            int paper = session->defaultPaper();
            if (!paperArg.isEmpty()) {
                bool ok = false;
                paper = paperArg.toInt(&ok);
                if (!ok || paper < 0 || paper >= session->paperNames().size()) {
                    std::fprintf(stderr, "--paper takes an index the export offers\n");
                    return 2;
                }
            }
            const int kind = exportPath.endsWith(QLatin1String(".svg"), Qt::CaseInsensitive)
                ? 2
                : (exportPath.endsWith(QLatin1String(".pdf"), Qt::CaseInsensitive) ? 1 : 0);
            const qreal dpi = dpiArg.isEmpty() ? 300.0 : dpiArg.toDouble();
            exported = session->exportTo(exportPath, kind, paper,
                                         arguments.contains(QStringLiteral("--landscape")), dpi)
                && exported;
        }
        std::fprintf(stderr, "%s: %s\n", exported ? "exported" : "export failed",
                     qUtf8Printable(exportPath));
        return exported ? 0 : 1;
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
    // The confirmation is the one panel that cannot be reached by a flag-less capture, and
    // it is where the user decides whether work is lost: it is driven here on purpose.
    for (int at = 0; at + 1 < arguments.size(); ++at) {
        if (arguments.at(at) != QStringLiteral("--confirm")) {
            continue;
        }
        const QString what = arguments.at(at + 1);
        for (QObject *root : engine.rootObjects()) {
            root->setProperty("pendingCommand", what);
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
            /// A file the sheet is saved to, so a step that needs a clean document has one.
            QString unsavedPath;
            /// The document the recent list is proved with.
            QString recentDocument;
        };
        auto *state = new SelfTest();
        state->window = window;
        state->sheet = window != nullptr ? window->findChild<SchematicItem *>() : nullptr;
        const auto sessions = window != nullptr ? window->findChildren<FluidSession *>()
                                                : QList<FluidSession *>();
        state->session = sessions.isEmpty() ? nullptr : sessions.constFirst();
        if (state->sheet == nullptr || state->session == nullptr) {
            std::fprintf(stderr, "selftest: no canvas or no session\n");
            return 1;
        }
        FluidSession *session = state->session;

        const auto check = [state](bool ok, const QString &what) {
            std::fprintf(stderr, "selftest: %-48s %s\n", qPrintable(what), ok ? "ok" : "FAIL");
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
                  QStringLiteral("startup: opens in editing, without simulating"));
            state->valveScene = toScene(valveCentre());
            send(QEvent::MouseButtonPress, state->valveScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonRelease, state->valveScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            std::fprintf(stderr, "selftest: editing, valve %d and %.2f bar\n",
                         state->session->valvePosition(), state->session->pressureBar());
            check(state->session->valvePosition() == 0,
                  QStringLiteral("editing: pressing the valve does not actuate it"));
            check(state->session->pressureBar() < 1.5,
                  QStringLiteral("editing: the pressure does not rise without simulating"));
            check(state->session->running() == false,
                  QStringLiteral("editing: pressing the valve does not start the simulation"));
            state->session->startSimulation();
            check(!state->session->editing() && state->session->running(),
                  QStringLiteral("start enters simulation"));
        });

        // Simulation: the click is a pushbutton. It opens while held and closes on release.
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonPress, state->valveScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            std::fprintf(stderr, "selftest: valve %d, %.2f bar, %.1f mm, click at (%.0f, %.0f)\n",
                         state->session->valvePosition(), state->session->pressureBar(),
                         state->session->positionMm(), state->valveScene.x(),
                         state->valveScene.y());
            check(state->session->valvePosition() == 1,
                  QStringLiteral("simulation: holding the click opens the valve"));
            check(state->session->pressureBar() > 2.0,
                  QStringLiteral("simulation: the chamber receives pressure"));
            check(state->session->positionMm() > 5.0,
                  QStringLiteral("simulation: the rod advances"));
            send(QEvent::MouseButtonRelease, state->valveScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            check(state->session->valvePosition() == 0,
                  QStringLiteral("simulation: releasing closes the valve"));
            state->session->stopSimulation();
            check(state->session->editing(), QStringLiteral("transport: Stop returns to editing"));
        });

        // The properties form edits the instance, and the model is rebuilt from it: the
        // reference lets a cylinder in one circuit be larger without touching the catalog.
        steps->push_back([state, check] {
            state->session->inspectComponent(2);
            const QVariantList fields = state->session->inspectorFields();
            check(!fields.isEmpty(),
                  QStringLiteral("inspector: the cylinder brings its parameters"));
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
                  QStringLiteral("inspector: the edited stroke reaches the model"));
            state->session->undo();
            PfTelemetry restored {};
            pf_session_telemetry(state->session->handle(), &restored);
            check(std::abs(restored.stroke_m - 0.10) < 1e-6,
                  QStringLiteral("inspector: undo restores the one from the file"));
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
            check(state->valveIndex >= 0, QStringLiteral("editing: a click selects"));
            check(state->session->inspectorOpen(), QStringLiteral("editing: the inspector opens"));
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
                  QStringLiteral("editing: dragging moves"));
            state->connections = pf_session_connection_count(state->session->handle());
            int cylinder = -1;
            char id[128] = {};
            const uint32_t count = pf_session_component_count(state->session->handle());
            for (uint32_t i = 0; i < count; ++i) {
                id[0] = '\0';
                pf_session_component_id(state->session->handle(), i, id, sizeof id);
                std::fprintf(stderr, "selftest: component %u = %s\n", i, id);
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
                         "selftest: valve %d port 3 %s, cylinder %d port 1 %s, %.1f pipe(s)\n",
                         state->valveIndex, hasFrom ? "yes" : "no", cylinder, hasTo ? "yes" : "no",
                         static_cast<double>(state->connections));
            check(hasFrom && hasTo, QStringLiteral("editing: the ports are found"));
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
                  QStringLiteral("editing: port to port draws a pipe"));
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
                  QStringLiteral("editing: Delete removes the selected component"));
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
                  QStringLiteral("editing: paste creates a copy"));
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == before,
                  QStringLiteral("editing: undoing the paste removes the copy"));
            const int copy = state->session->duplicateComponent(state->valveIndex, 10.0, 10.0);
            check(copy >= 0
                      && pf_session_component_count(state->session->handle()) == before + 1,
                  QStringLiteral("editing: duplicate creates a copy"));
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == before,
                  QStringLiteral("editing: undoing the copy removes it"));
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
                  QStringLiteral("editing: Shift-drag copies"));
            state->session->undo();
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == state->components,
                  QStringLiteral("editing: two undos remove the moved copy"));
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
            std::fprintf(stderr, "selftest: mid-stroke, %d mm and %d impact(s)\n",
                         static_cast<int>(state->session->positionMm()), *counter);
            check(*counter == 0,
                  QStringLiteral("sound: there is no impact while the rod travels"));
            check(state->session->positionMm() > 5.0,
                  QStringLiteral("sound: the rod really moves"));

            // On to the stop. Forty frames is 0.64 s, past the 0.48 s a full stroke takes.
            for (int i = 0; i < 40; ++i) {
                state->session->stepForward();
            }
            std::fprintf(stderr, "selftest: at the stop, %d mm and %d impact(s)\n",
                         static_cast<int>(state->session->positionMm()), *counter);
            check(*counter == 1, QStringLiteral("sound: reaching the stop is announced once"));

            // Staying there, and stepping, must not repeat the impact.
            for (int i = 0; i < 40; ++i) {
                state->session->stepForward();
            }
            check(*counter == 1,
                  QStringLiteral("sound: stopped at the stop, it does not repeat the impact"));

            // Releasing lets the spring bring it back, and that arrival is its own event.
            state->session->releaseValve();
            // The network's chamber is bigger than the fixed model's: its hose and valve
            // galleries hold gas too, so venting takes longer than one second.
            for (int i = 0; i < 200; ++i) {
                state->session->stepForward();
            }
            std::fprintf(stderr, "selftest: back, %d mm and %d impact(s)\n",
                         static_cast<int>(state->session->positionMm()), *counter);
            check(*counter == 2,
                  QStringLiteral("sound: returning to the other stop is a second impact"));
            state->session->stopSimulation();
        });

        // A rubber band over the whole sheet marks every component, and Delete takes the
        // group away as one change.
        steps->push_back([state, check, send, sendWith, toScene] {
            state->session->stopSimulation();
            state->session->select(-1);
            const QPointF from = toScene(QPointF(2.0, 2.0));
            const QPointF to = toScene(QPointF(198.0, 128.0));
            send(QEvent::MouseButtonPress, from, Qt::LeftButton, Qt::LeftButton);
            sendWith(QEvent::MouseMove, to, Qt::NoButton, Qt::LeftButton, Qt::NoModifier);
            send(QEvent::MouseButtonRelease, to, Qt::LeftButton, Qt::NoButton);
            const int marked = state->session->selectionCount();
            std::fprintf(stderr, "selftest: the band marked %d component(s)\n", marked);
            check(marked == state->session->componentCount(),
                  QStringLiteral("selection: the band marks everything it covers"));
        });
        steps->push_back([state, check] {
            const uint32_t before = pf_session_component_count(state->session->handle());
            state->session->removeSelection();
            check(pf_session_component_count(state->session->handle()) == 0
                      && before > 0,
                  QStringLiteral("selection: Delete removes the marked set at once"));
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == before,
                  QStringLiteral("selection: one undo brings it all back"));
        });
        // Mayús and a click changes membership: the reference's bidirectional marking.
        steps->push_back([state, check, sendWith, toScene, componentRect] {
            state->session->select(-1);
            const QPointF centre = toScene(componentRect(1).center());
            sendWith(QEvent::MouseButtonPress, centre, Qt::LeftButton, Qt::LeftButton,
                     Qt::ShiftModifier);
            sendWith(QEvent::MouseButtonRelease, centre, Qt::LeftButton, Qt::NoButton,
                     Qt::ShiftModifier);
            check(state->session->selectionCount() == 1
                      && state->session->isSelected(1),
                  QStringLiteral("selection: Shift-click marks without opening the inspector"));
            sendWith(QEvent::MouseButtonPress, centre, Qt::LeftButton, Qt::LeftButton,
                     Qt::ShiftModifier);
            sendWith(QEvent::MouseButtonRelease, centre, Qt::LeftButton, Qt::NoButton,
                     Qt::ShiftModifier);
            check(state->session->selectionCount() == 0,
                  QStringLiteral("selection: Shift-click again unmarks it"));
        });
        // Ctrl+E marks everything, and one drag moves the group without breaking it.
        steps->push_back([state, check, send, toScene, componentRect] {
            QKeyEvent key(QEvent::KeyPress, Qt::Key_E, Qt::ControlModifier, QStringLiteral("e"));
            QCoreApplication::sendEvent(state->window, &key);
            const int marked = state->session->selectionCount();
            check(marked > 1, QStringLiteral("selection: Ctrl+E marks everything"));
            const QRectF box = componentRect(0);
            state->routeScene = toScene(box.center());
            send(QEvent::MouseButtonPress, state->routeScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, send] {
            send(QEvent::MouseMove, state->routeScene + QPointF(90.0, 0.0), Qt::NoButton,
                 Qt::LeftButton);
        });
        steps->push_back([state, check, send, componentRect] {
            send(QEvent::MouseButtonRelease, state->routeScene + QPointF(90.0, 0.0),
                 Qt::LeftButton, Qt::NoButton);
            check(state->session->selectionCount() > 1,
                  QStringLiteral("selection: dragging a group does not undo it"));
            const QRectF box = componentRect(1);
            check(box.left() > 90.0,
                  QStringLiteral("selection: the group moved as a whole"));
            state->session->undo();
        });

        // Lining a selection up moves it onto the component already furthest out, and one
        // undo puts it back: the alignment is one change.
        steps->push_back([state, check, componentRect] {
            state->session->selectAll();
            float b[4] = {0.0F, 0.0F, 0.0F, 0.0F};
            pf_session_component_rect(state->session->handle(), 1, b);
            const float was = b[0];
            state->session->alignSelection(0);
            float a[4] = {0.0F, 0.0F, 0.0F, 0.0F};
            pf_session_component_rect(state->session->handle(), 0, a);
            pf_session_component_rect(state->session->handle(), 1, b);
            check(std::abs(a[0] - b[0]) < 0.01,
                  QStringLiteral("align: the left edges coincide"));
            state->session->undo();
            pf_session_component_rect(state->session->handle(), 1, b);
            check(std::abs(b[0] - was) < 0.01,
                  QStringLiteral("align: one undo restores it"));
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
                  QStringLiteral("editing: Ctrl and a double click turn it the other way"));
            float after[2] = {0.0F, 0.0F};
            pf_session_port_point(state->session->handle(), index, "2", after);
            check(std::abs(before[0] - after[0]) > 0.5 || std::abs(before[1] - after[1]) > 0.5,
                  QStringLiteral("editing: the port turns with the symbol"));
            state->session->undo();
            orientation[0] = 9;
            pf_session_component_orientation(state->session->handle(), index, orientation);
            check(orientation[0] == 0,
                  QStringLiteral("editing: one undo puts the symbol back"));
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
                  QStringLiteral("editing: reflect does not turn, it mirrors"));
            state->session->mirrorSelected();
            pf_session_component_orientation(state->session->handle(),
                                             static_cast<uint32_t>(index), orientation);
            check(orientation[1] == 0,
                  QStringLiteral("editing: reflecting twice leaves the symbol as it was"));
            state->session->undo();
            state->session->undo();
        });

        // Run until a state change: the transport stops on the frame the piston reaches
        // the stop, which is what the reference's own run mode does.
        steps->push_back([state, check] {
            state->session->restart();
            state->session->pressValve();
            const bool changed = state->session->runUntilChange();
            check(changed, QStringLiteral("transport: run until change stops"));
            check(state->session->positionMm() > 99.0,
                  QStringLiteral("transport: it stops at the stop"));
            check(!state->session->running(),
                  QStringLiteral("transport: it stays paused"));
            state->session->releaseValve();
            state->session->stopSimulation();
        });

        // The view: zoom and the grid change what the sheet looks like, never the document.
        steps->push_back([state, check] {
            const bool undoes = state->session->canUndo();
            state->session->zoomIn();
            check(state->session->zoom() > 1.0, QStringLiteral("view: zoom in brings the sheet closer"));
            state->session->zoomOut();
            state->session->zoomOut();
            check(state->session->zoom() < 1.0,
                  QStringLiteral("view: zoom out moves the sheet away"));
            state->session->zoomReset();
            check(qFuzzyCompare(state->session->zoom(), 1.0),
                  QStringLiteral("view: original size returns to the fit"));
            const bool grid = state->session->grid();
            state->session->toggleGrid();
            check(state->session->grid() != grid,
                  QStringLiteral("view: the grid toggles"));
            state->session->toggleGrid();
            check(state->session->canUndo() == undoes,
                  QStringLiteral("view: zoom does not enter the history"));
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
                    if (hit < 0) {
                        continue;
                    }
                    // Only a hose with two jogs has a segment to drag: one bend is
                    // already the shortest orthogonal route. Ask the core by trying the
                    // drag on a scratch edit and taking it back at once.
                    if (pf_session_route_pipe(state->session->handle(),
                                              static_cast<uint32_t>(hit), x, y)
                        == PF_OK) {
                        state->session->undo();
                        state->routePipe = hit;
                        state->routeScene = toScene(QPointF(x, y));
                        break;
                    }
                }
            }
            check(state->routePipe >= 0,
                  QStringLiteral("editing: a pipe to drag is found"));
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
                  QStringLiteral("editing: dragging the segment re-routes the pipe"));
            state->session->undo();
            pf_session_pipe_offset(state->session->handle(),
                                   static_cast<uint32_t>(state->routePipe), &offset);
            check(std::abs(offset - state->routeOffset) < 0.01,
                  QStringLiteral("editing: one undo restores the route"));
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
                check(false, QStringLiteral("T junction: the cylinder has a port"));
                return;
            }
            state->pipes = pf_session_connection_count(state->session->handle());
            state->components = pf_session_component_count(state->session->handle());
            // Find a point that really is on a hose by asking the model, rather than assuming
            // where the orthogonal route runs. Of every point that qualifies, the one furthest
            // from any component is taken: a hose that runs between two bodies an inch apart
            // has a middle too narrow for the drop to be read as a junction, and picking the
            // first point found would make this step depend on where the worked example happens
            // to lay its parts out.
            QPointF onPipe;
            bool found = false;
            qreal bestClearance = -1.0;
            for (double x = 10.0; x < 190.0; x += 2.0) {
                for (double y = 10.0; y < 125.0; y += 2.0) {
                    int portIndex = -1;
                    QString port;
                    // Away from every port, or the release would be read as an ordinary
                    // port-to-port connection instead of a junction on the hose.
                    if (state->session->pipeAt(x, y, 1.0) < 0
                        || state->session->portAt(x, y, 4.0, &portIndex, &port)) {
                        continue;
                    }
                    const QPointF candidate(x, y);
                    qreal clearance = std::numeric_limits<qreal>::max();
                    for (uint32_t index = 0; index < count; ++index) {
                        const QRectF bounds = componentRect(static_cast<int>(index));
                        const qreal dx = std::max({bounds.left() - x, 0.0, x - bounds.right()});
                        const qreal dy = std::max({bounds.top() - y, 0.0, y - bounds.bottom()});
                        clearance = std::min(clearance, dx * dx + dy * dy);
                    }
                    if (clearance > bestClearance) {
                        bestClearance = clearance;
                        onPipe = candidate;
                        found = true;
                    }
                    // Six millimetres of air around the drop is enough for the release to be
                    // read as a junction, and stopping there keeps the sweep short: walking
                    // the whole sheet on every step stalls the event loop this test drives.
                    if (bestClearance >= 36.0) {
                        break;
                    }
                }
                if (bestClearance >= 36.0) {
                    break;
                }
            }
            check(found, QStringLiteral("T junction: there is a pipe to drop on"));
            if (!found) {
                return;
            }
            // Drag from the port nearest the drop rather than from the cylinder's: at a narrow
            // viewport the far end of the sheet can sit outside the canvas, and a press that
            // lands on nothing is a drag that never starts. The gesture is the same one — a hose
            // end released on a hose — and it now begins somewhere the window can see.
            int nearIndex = -1;
            QString nearPort;
            if (state->session->portAt(onPipe.x(), onPipe.y(), 1.0e6F, &nearIndex, &nearPort)) {
                QPointF at;
                if (state->session->portPoint(nearIndex, nearPort, &at)) {
                    start = at;
                }
            }
            std::fprintf(stderr, "selftest: pipe under (%.0f, %.0f)\n", onPipe.x(), onPipe.y());
            state->toScene = toScene(onPipe);
            // Press, drag and release in one step. Three steps of a timer each is three chances
            // for the window to be busy with something else between them, and a drag whose
            // press and release are not read in order is a drag that never happened: this
            // asserts the splice, not the scheduler.
            send(QEvent::MouseButtonPress, toScene(start), Qt::LeftButton, Qt::LeftButton);
            send(QEvent::MouseMove, state->toScene, Qt::NoButton, Qt::LeftButton);
            send(QEvent::MouseButtonRelease, state->toScene, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            const uint32_t pipes = pf_session_connection_count(state->session->handle());
            const uint32_t components = pf_session_component_count(state->session->handle());
            std::fprintf(stderr, "selftest: after the T, %u pipe(s) and %u component(s)\n", pipes,
                         components);
            check(components == state->components + 1,
                  QStringLiteral("T junction: the junction appears where it was dropped"));
            check(pipes == state->pipes + 2,
                  QStringLiteral("T junction: the pipe splits in two and the third arrives"));
            state->session->undo();
            check(pf_session_connection_count(state->session->handle()) == state->pipes
                      && pf_session_component_count(state->session->handle()) == state->components,
                  QStringLiteral("T junction: one undo undoes the whole junction"));
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
            check(row != nullptr, QStringLiteral("palette: the catalog row is found"));
            if (row == nullptr) {
                return;
            }
            const QPointF centre = row->mapToScene(row->boundingRect().center());
            std::fprintf(stderr, "selftest: palette row at (%.0f, %.0f)\n", centre.x(),
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
                  QStringLiteral("palette: dropping on the sheet places the component"));
            if (now == state->components + 1) {
                const QRectF box = componentRect(static_cast<int>(now) - 1);
                std::fprintf(stderr, "selftest: placed at (%.0f, %.0f)\n", box.center().x(),
                             box.center().y());
                check(std::abs(box.center().x() - 150.0) < 12.0
                          && std::abs(box.center().y() - 40.0) < 12.0,
                      QStringLiteral("palette: it lands where it is dropped, not elsewhere"));
            }
        });

        // Unsaved work is known, marked in the title and copied for safety. A crash costs
        // the work since the last copy, and a close asks before losing the rest. The sheet
        // has been edited by the steps above, so the baseline is a save, not an assumption.
        steps->push_back([state, check] {
            state->session->stopSimulation();
            const QString path = QDir::tempPath() + QStringLiteral("/punchi-aviso.pfluid");
            check(state->session->saveTo(path),
                  QStringLiteral("unsaved: saving leaves the document clean"));
            check(!state->session->modified()
                      && !state->session->title().startsWith(QChar(0x2022)),
                  QStringLiteral("unsaved: a saved document has no mark"));
            state->unsavedPath = path;
            if (state->session->componentCount() > 0) {
                state->session->select(0);
                state->session->moveComponent(0, 60.0, 100.0);
            }
            check(state->session->modified(),
                  QStringLiteral("unsaved: moving marks the document"));
            check(state->session->title().startsWith(QChar(0x2022)),
                  QStringLiteral("unsaved: the title warns"));
            check(state->session->hasUnsavedWork(),
                  QStringLiteral("unsaved: closing would ask first"));
            state->session->autosave();
            check(state->session->modified(),
                  QStringLiteral("unsaved: the safety copy is not a save"));
            state->session->undo();
            check(state->session->modified(),
                  QStringLiteral("unsaved: undo does not clear the mark"));
        });
        // Recovering a safety copy brings the work back and leaves it unsaved, because the
        // user never chose where it goes; starting over then discards both.
        steps->push_back([state, check] {
            const int before = state->session->componentCount();
            check(state->session->recoverAutosave(),
                  QStringLiteral("unsaved: the copy is recovered"));
            check(state->session->modified() && state->session->currentPath().isEmpty(),
                  QStringLiteral("unsaved: what was recovered is still unsaved"));
            check(state->session->componentCount() == before,
                  QStringLiteral("unsaved: the copy brings the whole circuit"));
            state->session->discardAutosave();
            check(!state->session->recoveryOffered(),
                  QStringLiteral("unsaved: discarding the copy removes it"));
            state->session->discardAndNewDocument();
            check(!state->session->modified(),
                  QStringLiteral("unsaved: starting over begins clean"));
            QFile::remove(state->unsavedPath);
        });

        // A component can be named, the name is drawn under it and saved with the sheet,
        // and a second cylinder with the same name is what the check reports.
        steps->push_back([state, check] {
            state->session->stopSimulation();
            state->session->select(2);
            check(state->session->designationOf(2).isEmpty(),
                  QStringLiteral("designation: the example has no names"));
            state->session->setDesignation(2, QStringLiteral(" 1A1 "));
            check(state->session->designationOf(2) == QStringLiteral("1A1"),
                  QStringLiteral("designation: it is saved trimmed"));
            state->session->setDesignation(2, QString());
            check(state->session->designationOf(2).isEmpty(),
                  QStringLiteral("designation: clearing it removes it"));
            state->session->undo();
            check(state->session->designationOf(2) == QStringLiteral("1A1"),
                  QStringLiteral("designation: one undo restores it"));
        });
        steps->push_back([state, check] {
            // Two actuators with one name is the fault the check has to report, and it is
            // the reason the designation exists.
            const int copy = state->session->duplicateComponent(2, 10.0, 40.0);
            check(copy >= 0, QStringLiteral("designation: there is a second cylinder"));
            if (copy >= 0) {
                state->session->setDesignation(copy, QStringLiteral("1A1"));
            }
            state->session->select(-1);
            state->session->checkCircuit();
            const QString report = state->session->checkReport();
            const bool named = report.contains(QStringLiteral("misma designación"));
            std::fprintf(stderr, "selftest: the check says: %s\n",
                         qUtf8Printable(report.left(240)));
            check(named, QStringLiteral("designation: two cylinders with the same name"));
            state->session->closeInspector();
            state->session->undo();
            state->session->undo();
        });

        // The documents that were saved are remembered, and one of them comes back from
        // the list without walking the file dialog.
        steps->push_back([state, check] {
            state->session->forgetRecent();
            check(state->session->recent().isEmpty(),
                  QStringLiteral("recent: the list is emptied"));
            state->recentDocument = QDir::tempPath()
                + QStringLiteral("/punchi-reciente.pfluid");
            state->session->saveTo(state->recentDocument);
            const QVariantList list = state->session->recent();
            check(list.size() == 1,
                  QStringLiteral("recent: saving records the file"));
            const QVariantMap entry = list.value(0).toMap();
            check(entry.value(QStringLiteral("path")).toString() == state->recentDocument,
                  QStringLiteral("recent: with its full path"));
            check(entry.value(QStringLiteral("name")).toString()
                      == QStringLiteral("punchi-reciente.pfluid"),
                  QStringLiteral("recent: and with its name for the menu"));
        });
        steps->push_back([state, check] {
            const int before = state->session->componentCount();
            state->session->newDocument();
            check(state->session->openRecent(0),
                  QStringLiteral("recent: it opens from the list"));
            check(before > 0 && state->session->componentCount() == before,
                  QStringLiteral("recent: the whole circuit returns"));
            check(state->session->currentPath() == state->recentDocument,
                  QStringLiteral("recent: the opened file is the one in the list"));
            // A file that is gone is dropped instead of left in the menu, because the list
            // promises documents that can be opened.
            QFile::remove(state->recentDocument);
            check(!state->session->openRecent(0),
                  QStringLiteral("recent: opening one that is gone fails"));
            check(state->session->recent().isEmpty(),
                  QStringLiteral("recent: and it is forgotten from the list"));
            state->session->forgetRecent();
            check(state->session->recent().isEmpty(),
                  QStringLiteral("recent: forgetting leaves the list empty"));
        });

        // Saving and opening a sheet: the file has to carry the placements, the pipes,
        // their routes and the per-instance values, not just a list of names.
        steps->push_back([state, check] {
            const QString path = QDir::tempPath() + QStringLiteral("/punchi-selftest.pfluid");
            const uint32_t before = pf_session_component_count(state->session->handle());
            const uint32_t pipes = pf_session_connection_count(state->session->handle());
            check(state->session->saveTo(path), QStringLiteral("document: it saves"));
            // Change the document, then open the file: the change must disappear.
            if (state->session->componentCount() > 0) {
                state->session->removeComponent(0);
            }
            check(state->session->openFrom(path), QStringLiteral("document: it opens"));
            check(pf_session_component_count(state->session->handle()) == before,
                  QStringLiteral("document: the same components return"));
            check(pf_session_connection_count(state->session->handle()) == pipes,
                  QStringLiteral("document: the same pipes return"));
            QFile::remove(path);
        });

        // A project is one document with several circuits: add one, name it, save the
        // project, open it again and switch between the sheets.
        steps->push_back([state, check] {
            check(state->session->circuits().size() == 1,
                  QStringLiteral("circuit: the project starts with one"));
            const int first = state->session->activeCircuit();
            const int added = state->session->addCircuit();
            check(added >= 0 && state->session->circuits().size() == 2,
                  QStringLiteral("circuit: a new one is added"));
            check(state->session->activeCircuit() == added,
                  QStringLiteral("circuit: the new one is active"));
            check(state->session->renameCircuit(added, QStringLiteral("Control")),
                  QStringLiteral("circuit: it can be renamed"));
            const QVariantList circuits = state->session->circuits();
            check(circuits.value(circuits.size() - 1)
                          .toMap()
                          .value(QStringLiteral("name"))
                          .toString()
                      == QStringLiteral("Control"),
                  QStringLiteral("circuit: the name is kept"));

            const QString path =
                QDir::tempPath() + QStringLiteral("/punchi-circuits.pfluid");
            check(state->session->saveTo(path),
                  QStringLiteral("circuit: the project saves"));
            state->session->newDocument();
            check(state->session->circuits().size() == 1,
                  QStringLiteral("circuit: a new document has one"));
            check(state->session->openFrom(path),
                  QStringLiteral("circuit: the project opens"));
            check(state->session->circuits().size() == 2,
                  QStringLiteral("circuit: both circuits return"));
            check(state->session->activeCircuit() == added,
                  QStringLiteral("circuit: the active one returns"));
            check(state->session->selectCircuit(first),
                  QStringLiteral("circuit: another one can be selected"));
            check(state->session->activeCircuit() == first,
                  QStringLiteral("circuit: the selected one is active"));
            QFile::remove(path);
        });

        // A flow meter is an instrument in the line: it is placed, wired across the working
        // line it measures, and it prints the gas that goes through it in its own unit,
        // from the frame the sheet draws.
        steps->push_back([state, check] {
            int meter = -1;
            const QVariantList entries = state->session->catalog();
            for (const QVariant &entry : entries) {
                const QVariantMap row = entry.toMap();
                if (row.value(QStringLiteral("id")).toString() == QStringLiteral("flow.meter")) {
                    meter = row.value(QStringLiteral("index")).toInt();
                }
            }
            check(meter >= 0, QStringLiteral("flow meter: the catalog offers it"));
            const int index = meter >= 0 ? state->session->placeComponent(meter, 100.0, 60.0) : -1;
            check(index >= 0, QStringLiteral("flow meter: it can be placed"));

            int valve = -1;
            int cylinder = -1;
            const uint32_t count = pf_session_component_count(state->session->handle());
            for (uint32_t i = 0; i < count; ++i) {
                char id[128] = {};
                pf_session_component_id(state->session->handle(), i, id, sizeof id);
                const QString name = QString::fromUtf8(id);
                if (name.startsWith(QLatin1String("valve."))) {
                    valve = static_cast<int>(i);
                }
                if (name.startsWith(QLatin1String("cylinder."))) {
                    cylinder = static_cast<int>(i);
                }
            }
            const bool wired = index >= 0 && valve >= 0 && cylinder >= 0
                && state->session->connectPorts(valve, QStringLiteral("2"), index,
                                                QStringLiteral("1"))
                && state->session->connectPorts(index, QStringLiteral("2"), cylinder,
                                                QStringLiteral("1"));
            check(wired, QStringLiteral("flow meter: it is wired to the working line"));

            if (wired) {
                state->session->pressValve();
                for (int i = 0; i < 5; ++i) {
                    state->session->advance(0.016);
                }
                PfFrame frame {};
                int readings = 0;
                bool positive = false;
                if (pf_session_build_frame(state->session->handle(), 200.0, 130.0, 4.0, &frame)
                    == PF_OK) {
                    for (uint32_t i = 0; i < frame.calls_len; ++i) {
                        const PfCall &call = frame.calls[i];
                        if (call.kind != PF_KIND_TEXT) {
                            continue;
                        }
                        const QString text = QString::fromUtf8(
                            reinterpret_cast<const char *>(frame.text) + call.text_at,
                            static_cast<int>(call.text_len));
                        if (!text.endsWith(QLatin1String(" l/min"))) {
                            continue;
                        }
                        ++readings;
                        bool ok = false;
                        const float value = text.left(text.size() - 6).toFloat(&ok);
                        positive = positive || (ok && value > 0.0f);
                    }
                }
                check(readings > 0 && positive,
                      QStringLiteral("flow meter: it reports normal litres per minute"));
                state->session->releaseValve();
            }
            // The meter and its pipes leave the sheet as they found it; the steps after
            // this one assert on the document's own contents.
            state->session->removeComponent(index);
        });

        // Even spacing: the three symbols of the worked example, spread left to right, keep
        // their extent and end with equal gaps between their boxes.
        steps->push_back([state, check, componentRect] {
            state->session->selectAll();
            const int count = state->session->componentCount();
            check(count >= 3, QStringLiteral("spacing: there are symbols to spread"));
            QVector<QRectF> before;
            for (int index = 0; index < count; ++index) {
                before.append(componentRect(index));
            }
            state->session->spaceSelection(0);
            QVector<QRectF> after;
            for (int index = 0; index < count; ++index) {
                after.append(componentRect(index));
            }
            std::sort(after.begin(), after.end(),
                      [](const QRectF &left, const QRectF &right) {
                          return left.left() < right.left();
                      });
            bool equal = after.size() >= 3;
            const qreal first = equal ? after[1].left() - after[0].right() : 0.0;
            for (int index = 1; equal && index + 1 < after.size(); ++index) {
                const qreal gap = after[index + 1].left() - after[index].right();
                equal = std::abs(gap - first) < 0.05;
            }
            check(equal, QStringLiteral("spacing: the gaps between boxes are equal"));
            const qreal left_before =
                std::min_element(before.begin(), before.end(), [](const QRectF &a, const QRectF &b) {
                    return a.left() < b.left();
                })->left();
            const qreal right_before =
                std::max_element(before.begin(), before.end(), [](const QRectF &a, const QRectF &b) {
                    return a.right() < b.right();
                })->right();
            check(std::abs(after.first().left() - left_before) < 0.05
                      && std::abs(after.last().right() - right_before) < 0.05,
                  QStringLiteral("spacing: the outermost keep their places"));
            state->session->undo();
            bool restored = true;
            for (int index = 0; index < count && index < before.size(); ++index) {
                restored = restored && componentRect(index) == before[index];
            }
            check(restored, QStringLiteral("spacing: one undo puts them back"));
        });

        // The sheet leaves as a vector page too: the same drawing, readable as text and
        // measured in millimetres.
        steps->push_back([state, check] {
            const QString path = QDir::tempPath() + QStringLiteral("/punchi-vector.svg");
            check(state->session->exportTo(path, 2, state->session->defaultPaper(), false, 300.0),
                  QStringLiteral("export: the sheet leaves as SVG"));
            QFile file(path);
            const bool opened = file.open(QIODevice::ReadOnly);
            const QString svg = opened ? QString::fromUtf8(file.readAll()) : QString();
            check(svg.startsWith(QStringLiteral("<?xml")) && svg.contains(QStringLiteral("<svg")),
                  QStringLiteral("export: it is an SVG document"));
            check(svg.contains(QStringLiteral("viewBox=\"0 0 210.0 297.0\"")),
                  QStringLiteral("export: with the paper in millimetres"));
            check(svg.contains(QStringLiteral("lista de piezas")),
                  QStringLiteral("export: and the parts list as text"));
            QFile::remove(path);
        });

        // The control circuit: rail, button and coil. A click on the button closes it, the
        // coil reads the live line and the solenoid valve follows (ADR 0018).
        steps->push_back([state, check] {
            int rail = -1;
            int button = -1;
            int coil = -1;
            const QVariantList entries = state->session->catalog();
            for (const QVariant &entry : entries) {
                const QVariantMap row = entry.toMap();
                const QString id = row.value(QStringLiteral("id")).toString();
                if (id == QStringLiteral("electrical.supply")) {
                    rail = row.value(QStringLiteral("index")).toInt();
                }
                if (id == QStringLiteral("electrical.contact")) {
                    button = row.value(QStringLiteral("index")).toInt();
                }
                if (id == QStringLiteral("valve.3_2.solenoid")) {
                    coil = row.value(QStringLiteral("index")).toInt();
                }
            }
            check(rail >= 0 && button >= 0 && coil >= 0,
                  QStringLiteral("control: the catalog offers the three parts"));
            const int rail_index = state->session->placeComponent(rail, 40.0, 20.0);
            const int button_index = state->session->placeComponent(button, 90.0, 20.0);
            const int coil_index = state->session->placeComponent(coil, 140.0, 20.0);
            check(rail_index >= 0 && button_index >= 0 && coil_index >= 0,
                  QStringLiteral("control: they can be placed"));
            check(state->session->connectPorts(rail_index, QStringLiteral("L"), button_index,
                                               QStringLiteral("1")),
                  QStringLiteral("control: the rail reaches the button"));
            check(state->session->connectPorts(button_index, QStringLiteral("2"), coil_index,
                                               QStringLiteral("Y1")),
                  QStringLiteral("control: the button reaches the coil"));
            check(state->session->valvePositionAt(coil_index) == 0,
                  QStringLiteral("control: the valve rests while the button is open"));
            check(state->session->toggleContactAt(button_index),
                  QStringLiteral("control: a click closes the contact"));
            check(state->session->valvePositionAt(coil_index) == 1,
                  QStringLiteral("control: the solenoid follows the live line"));
            check(state->session->toggleContactAt(button_index),
                  QStringLiteral("control: a second click opens it"));
            check(state->session->valvePositionAt(coil_index) == 0,
                  QStringLiteral("control: the spring returns the spool"));
            state->session->removeComponent(coil_index);
            state->session->removeComponent(button_index);
            state->session->removeComponent(rail_index);
        });

        // The export panel decides what the page carries: the sheet, its margins and
        // whether the parts list is printed.
        steps->push_back([state, check] {
            const QString path = QDir::tempPath() + QStringLiteral("/punchi-panel.svg");
            QVariantMap options;
            options.insert(QStringLiteral("path"), path);
            options.insert(QStringLiteral("kind"), 2);
            options.insert(QStringLiteral("paper"), state->session->defaultPaper());
            options.insert(QStringLiteral("custom_paper"), true);
            options.insert(QStringLiteral("width"), 90);
            options.insert(QStringLiteral("height"), 60);
            options.insert(QStringLiteral("landscape"), false);
            options.insert(QStringLiteral("dpi"), 300);
            options.insert(QStringLiteral("parts_list"), false);
            options.insert(QStringLiteral("margins"),
                           QVariantList { 5, 5, 5, 5 });
            check(state->session->exportWithOptions(options),
                  QStringLiteral("export panel: the settings are accepted"));
            QFile file(path);
            const bool opened = file.open(QIODevice::ReadOnly);
            const QString svg = opened ? QString::fromUtf8(file.readAll()) : QString();
            check(svg.contains(QStringLiteral("viewBox=\"0 0 90.0 60.0\"")),
                  QStringLiteral("export panel: the custom paper reaches the page"));
            check(!svg.contains(QStringLiteral("lista de piezas")),
                  QStringLiteral("export panel: the list can be left off"));
            QFile::remove(path);
        });

        // A pipe's class is a property of the pipe: the selected hose tilts its dashes, the
        // drawing follows, and the file keeps it.
        steps->push_back([state, check] {
            check(state->session->connectionCount() > 0,
                  QStringLiteral("pipe class: the example has hoses"));
            // Select the first hose by its index, then give it a class.
            state->session->selectPipe(0);
            check(state->session->selectedPipe() == 0,
                  QStringLiteral("pipe class: a hose can be selected"));
            state->session->setPipeClass(1);
            check(pf_session_pipe_class(state->session->handle(), 0) == 1,
                  QStringLiteral("pipe class: the hose takes the pilot class"));
            state->session->setPipeClass(2);
            check(pf_session_pipe_class(state->session->handle(), 0) == 2,
                  QStringLiteral("pipe class: and then the drain one"));
            state->session->undo();
            check(pf_session_pipe_class(state->session->handle(), 0) == 1,
                  QStringLiteral("pipe class: one undo returns the previous class"));
            state->session->setPipeClass(0);
            check(pf_session_pipe_class(state->session->handle(), 0) == 0,
                  QStringLiteral("pipe class: a working line is the default again"));
            state->session->selectPipe(-1);
        });

        // The two terminations are files like everything else: they are placed, inspected
        // and removed, and the inspector says what they do.
        steps->push_back([state, check] {
            int plug = -1;
            int silencer = -1;
            const QVariantList entries = state->session->catalog();
            for (const QVariant &entry : entries) {
                const QVariantMap row = entry.toMap();
                const QString id = row.value(QStringLiteral("id")).toString();
                if (id == QStringLiteral("plug.blind")) {
                    plug = row.value(QStringLiteral("index")).toInt();
                }
                if (id == QStringLiteral("silencer.pneumatic")) {
                    silencer = row.value(QStringLiteral("index")).toInt();
                }
            }
            check(plug >= 0 && silencer >= 0,
                  QStringLiteral("terminations: the catalog offers both"));
            const int plug_index = plug >= 0 ? state->session->placeComponent(plug, 40.0, 60.0) : -1;
            const int silencer_index =
                silencer >= 0 ? state->session->placeComponent(silencer, 80.0, 60.0) : -1;
            check(plug_index >= 0 && silencer_index >= 0,
                  QStringLiteral("terminations: they can be placed"));
            state->session->inspectComponent(plug_index);
            check(state->session->inspectorText().contains(QStringLiteral("Cierra el puerto")),
                  QStringLiteral("terminations: the plug says it closes the port"));
            state->session->inspectComponent(silencer_index);
            check(state->session->inspectorText().contains(QStringLiteral("Ventila el escape")),
                  QStringLiteral("terminations: the silencer says it vents the exhaust"));
            state->session->removeComponent(silencer_index);
            state->session->removeComponent(plug_index);
        });

        // A recorder is an empty frame until it is bound: the inspector lists what the
        // circuit can report, and the sheet draws what it sampled.
        steps->push_back([state, check] {
            int recorder = -1;
            const QVariantList entries = state->session->catalog();
            for (const QVariant &entry : entries) {
                const QVariantMap row = entry.toMap();
                if (row.value(QStringLiteral("id")).toString()
                    == QStringLiteral("chart.recorder")) {
                    recorder = row.value(QStringLiteral("index")).toInt();
                }
            }
            check(recorder >= 0, QStringLiteral("recorder: the catalog offers it"));
            const int index =
                recorder >= 0 ? state->session->placeComponent(recorder, 60.0, 60.0) : -1;
            check(index >= 0, QStringLiteral("recorder: it can be placed"));

            state->session->inspectComponent(index);
            const QVariantList sources = state->session->chartCandidates();
            check(!sources.isEmpty(), QStringLiteral("recorder: the circuit offers sources"));
            int position = -1;
            for (const QVariant &entry : sources) {
                const QVariantMap row = entry.toMap();
                // Position is the quantity a cylinder reports and no other part does.
                if (row.value(QStringLiteral("quantity")).toInt() == 2) {
                    position = row.value(QStringLiteral("placement")).toInt();
                }
            }
            check(position >= 0, QStringLiteral("recorder: the cylinder reports its position"));
            check(state->session->setChartSource(position, 2),
                  QStringLiteral("recorder: it can be bound"));
            check(state->session->chartSourcePlacement() == position,
                  QStringLiteral("recorder: the binding is the one chosen"));

            state->session->pressValve();
            for (int i = 0; i < 30; ++i) {
                state->session->advance(0.016);
            }
            PfFrame frame {};
            bool charted = false;
            if (pf_session_build_frame(state->session->handle(), 200.0, 130.0, 4.0, &frame)
                == PF_OK) {
                for (uint32_t i = 0; i < frame.calls_len; ++i) {
                    const PfCall &call = frame.calls[i];
                    if (call.kind != PF_KIND_TEXT) {
                        continue;
                    }
                    const QString text = QString::fromUtf8(
                        reinterpret_cast<const char *>(frame.text) + call.text_at,
                        static_cast<int>(call.text_len));
                    charted = charted || text.endsWith(QLatin1String(" mm"));
                }
            }
            check(charted, QStringLiteral("recorder: it draws the value it charts"));
            state->session->releaseValve();

            while (state->session->canUndo()) {
                state->session->undo();
            }
            state->session->removeComponent(index);
        });

        QTimer::singleShot(400, window, [state, steps] {
            auto *timer = new QTimer(state->window);
            timer->setInterval(350);
            QObject::connect(timer, &QTimer::timeout, state->window, [state, steps, timer] {
                if (state->step >= steps->size()) {
                    timer->stop();
                    std::fprintf(stderr, "selftest: %d failure(s)\n", state->failures);
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
