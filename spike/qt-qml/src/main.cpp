// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The spike's window: a Qt Quick front end over the Rust core.
//!
//! `--grab <file>` renders one frame offscreen and exits, so the drawing can be checked
//! in CI or in a headless session the same way the export tests check a page.

#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTimer>

#include <cstdio>
#include <cstring>

#include "punchi_fluid.h"
#include "session.h"

namespace {

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
    // Two frames: the first paints, the second captures what the model stepped into.
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

    PfAbi abi {};
    if (pf_abi(&abi) != PF_OK || abi.version != PF_ABI_VERSION) {
        std::fprintf(stderr, "the C++ header and the Rust core disagree on the ABI\n");
        return 2;
    }
    std::fprintf(stderr, "core %s, PfCall %u bytes, PfFrame %u bytes\n", pf_version(),
                 abi.call_size, abi.frame_size);

    QString grab;
    const QStringList arguments = app.arguments();
    const bool toggle = arguments.contains(QStringLiteral("--toggle"));
    const int grabIndex = arguments.indexOf(QStringLiteral("--grab"));
    if (grabIndex >= 0 && grabIndex + 1 < arguments.size()) {
        grab = arguments.at(grabIndex + 1);
    }

    QQmlApplicationEngine engine;
    engine.loadFromModule("PunchiFluid", "Main");
    if (engine.rootObjects().isEmpty()) {
        return 1;
    }
    if (toggle) {
        // The same path a click takes, without needing a pointer: the screenshot then
        // proves the actuated square, not just the rest one.
        for (QObject *root : engine.rootObjects()) {
            for (FluidSession *session : root->findChildren<FluidSession *>()) {
                session->toggleValve();
            }
        }
    }
    if (!grab.isEmpty()) {
        if (grabAndExit(engine, grab) != 0) {
            return 1;
        }
        // The grab is scheduled on a timer, so the event loop has to run for it to fire.
        return app.exec();
    }
    return app.exec();
}
