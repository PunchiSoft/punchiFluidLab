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
//! ```

#include <QGuiApplication>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickWindow>
#include <QTimer>

#include <cstdio>

#include "canvasitem.h"
#include "icons.h"
#include "punchi_fluid.h"
#include "session.h"
#include "theme.h"

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

    if (arguments.contains(QStringLiteral("--toggle"))) {
        for (QObject *root : engine.rootObjects()) {
            const auto sessions = root->findChildren<FluidSession *>();
            for (FluidSession *session : sessions) {
                session->toggleValve();
            }
        }
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
