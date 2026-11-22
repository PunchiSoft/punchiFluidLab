// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

//! The application: a Qt Quick window over the Rust core.
//!
//! Flags make it testable without a display, which is how the drawing is verified:
//!
//! ```text
//! --grab <file>        render one frame offscreen and exit
//! --toggle             actuate the valve before that frame, the same path a click takes
//! --confirm <what>     open the unsaved-work confirmation (`new`, `open` or `close`)
//! --theme light|dark   force a palette instead of the desktop one
//! --check-theme        print the contrast ratios and exit 1 when one fails
//! --check-layout       resize through the shell's modes; exits 1 when one is wrong
//! --font-scale N       multiply the desktop font, to prove a larger-font run
//! --check-icons        report every ribbon icon and command coverage as project, theme or missing
//! --icon-sheet <file>  render the project icons at 16, 24 and 32 px, for a review
//! --scale <n>          multiply the icon sheet's sizes and image (default 1; manual render)
//! --check-sounds       prove the packaged samples decode; exits 1 when one does not
//! --selftest           replay every sheet gesture with real events; exits 1 on failure
//! --block              open the title-block editor, for a capture of it
//! --block-model N      0 school, 1 technical, 2 industrial, for a capture
//! --about              open the About popup, for a capture of it
//! --license            open the Licence popup, for a capture of it
//! --tab N              open one ribbon tab, for a capture
//! --place ID           place a catalog entry by index or id, for a capture
//! --spot X,Y           place it at that point of the sheet, in millimetres
//! --menu               open the collapsed ribbon menu, for a capture
//! --ribbon-flyout GROUP  open a collapsed group's flyout, for a capture
//! --catalog-closed     start with the catalog folded, for a capture of the rail
//! --catalog-width N    set the catalog width in pixels, for a capture of the tiles
//! --overlay catalog|right  open a compact panel over the sheet, for a capture of it
//! --ribbon-minimized   fold the ribbon and open its tab, for a capture of the panel
//! --view-panel         open the view strip's zoom panel, for a capture of it
//! --circuits N         add circuits before the capture, for a strip with several tabs
//! --right-dock-closed  start with the right dock folded, for a capture of its rail
//! --auto-hide          start with the opt-in auto-hide on, for a capture of Paneles
//! --page-setup         open the page-setup panel, for a capture of it
//! --paper-list         open the paper list on the Disposición tab
//! --margin-list        open the margin gallery on the Disposición tab
//! --grid-step N        set the grid step in millimetres, for a capture
//! --probe              print the numbers the status bar shows, then exit 1
//! --example            load the worked example, for a capture of the seeded sheet
//! --template N         start from one of the four templates, for a capture of it
//! --open <file>        open a saved sheet before anything else
//! --save <file>        save the sheet and exit 0/1, without a window
//! ```

#include <QColor>
#include <QAccessible>
#include <QGuiApplication>
#include <QDir>
#include <QFile>
#include <QFont>
#include <QImage>
#include <QKeyEvent>
#include <QMouseEvent>
#include <QPainter>
#include <QPalette>
#include <QWheelEvent>
#include <QIcon>
#include <QQmlApplicationEngine>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSettings>
#include <QTimer>
#include <QEventLoop>
#include <QVector>

#include <algorithm>
#include <cmath>
#include <cstdio>
#include <limits>
#include <functional>
#include <memory>
#include "actionicons.h"
#include "canvasitem.h"
#include "icons.h"
#include "punchi_fluid.h"
#include "session.h"
#include "sheetframe.h"
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

/// `--check-icons` reverse direction: a command in the ribbon table without a drawing is a
/// coverage hole. The table also holds entries that are not commands — the two custom
/// groups whose controls build their own commands, the three note-only groups and the two
/// help links — and they are the documented allowlist in `docs/reference/icons.md`. Any
/// new iconless entry fails here, so the document and the table cannot drift.
int checkRibbonCoverage(QObject *ribbon)
{
    const QVariantList tabs = ribbon->property("tabs").toList();
    const QVariantList helpLinks = ribbon->property("helpLinks").toList();
    if (tabs.isEmpty() || helpLinks.isEmpty()) {
        std::fprintf(stderr, "--check-icons: the ribbon table is empty\n");
        return 1;
    }

    QStringList iconless;
    int commands = 0;
    int withoutDrawing = 0;
    int badGroup = 0;
    for (const QVariant &tabEntry : tabs) {
        const QVariantMap tab = tabEntry.toMap();
        const QVariantList groups = tab.value(QStringLiteral("groups")).toList();
        for (const QVariant &groupEntry : groups) {
            const QVariantMap group = groupEntry.toMap();
            const QString custom = group.value(QStringLiteral("custom")).toString();
            const bool hasItems = !group.value(QStringLiteral("items")).toList().isEmpty();
            QVariant listed;
            if (!QMetaObject::invokeMethod(ribbon, "groupActions",
                                           Q_RETURN_ARG(QVariant, listed),
                                           Q_ARG(QVariant, groupEntry))) {
                std::fprintf(stderr, "--check-icons: cannot read the group %s\n",
                             qUtf8Printable(group.value(QStringLiteral("title")).toString()));
                ++badGroup;
                continue;
            }
            if (!custom.isEmpty() && !hasItems) {
                iconless.append(QStringLiteral("custom:") + custom);
            } else if (custom.isEmpty() && !hasItems) {
                iconless.append(QStringLiteral("note:")
                                + group.value(QStringLiteral("title")).toString());
            }
            // The generated page-setup and recent commands come from the same function the
            // ribbon draws, so the check follows the code instead of a copy of the table.
            const QVariantList actions = listed.toList();
            for (const QVariant &actionEntry : actions) {
                const QVariantMap action = actionEntry.toMap();
                const QString id = action.value(QStringLiteral("id")).toString();
                const QString icon = action.value(QStringLiteral("icon")).toString();
                ++commands;
                if (icon.isEmpty()) {
                    iconless.append(QStringLiteral("command:") + id);
                    continue;
                }
                if (!actionicons::has(icon)) {
                    ++withoutDrawing;
                    std::fprintf(stderr,
                                 "icons: command %-24s asks for %s, no project drawing\n",
                                 qUtf8Printable(id), qUtf8Printable(icon));
                }
            }
        }
    }
    for (const QVariant &helpEntry : helpLinks) {
        const QVariantMap help = helpEntry.toMap();
        if (help.value(QStringLiteral("icon")).toString().isEmpty()) {
            iconless.append(QStringLiteral("help:")
                            + help.value(QStringLiteral("id")).toString());
        }
    }

    // The documented allowlist: the custom groups have no icon of their own because their
    // controls build the commands, the notes carry text only and the help links are window
    // chrome. A new name here updates docs/reference/icons.md in the same change.
    const QStringList allowed {
        QStringLiteral("custom:page-setup"),
        QStringLiteral("custom:recent"),
        QStringLiteral("help:about"),
        QStringLiteral("help:license"),
    };
    constexpr int allowedNotes = 3;
    int notes = 0;
    int undocumented = 0;
    for (const QString &entry : iconless) {
        if (entry.startsWith(QStringLiteral("note:"))) {
            ++notes;
        } else if (!allowed.contains(entry)) {
            ++undocumented;
            std::fprintf(stderr, "icons: iconless entry \"%s\" is not documented\n",
                         qUtf8Printable(entry));
        }
    }
    int allowedMissing = 0;
    for (const QString &entry : allowed) {
        if (!iconless.contains(entry)) {
            ++allowedMissing;
            std::fprintf(stderr, "icons: documented iconless entry \"%s\" is gone\n",
                         qUtf8Printable(entry));
        }
    }
    if (notes != allowedNotes) {
        std::fprintf(stderr, "icons: %d note-only groups, the document says %d\n", notes,
                     allowedNotes);
        ++undocumented;
    }

    std::fprintf(stderr,
                 "icons: command table %d entries, %d without a drawing, %d bad group(s)\n",
                 commands, withoutDrawing, badGroup);
    std::fprintf(stderr, "icons: iconless entries (allowlist): %s\n",
                 qUtf8Printable(iconless.join(QStringLiteral(", "))));
    return withoutDrawing + badGroup + undocumented + allowedMissing;
}

/// `--check-icons`: every name the window asks for resolves, then the icon cache proves
/// its bound, its eviction and its palette key. The start view is already open, so the
/// cache carries the page's icons and not only the ribbon's.
int runIconCheck(QQmlApplicationEngine &engine, ThemeIconProvider *provider)
{
    QObject *window = engine.rootObjects().isEmpty() ? nullptr
                                                    : engine.rootObjects().constFirst();
    QObject *ribbon = window != nullptr
                          ? window->findChild<QObject *>(QStringLiteral("ribbon"))
                          : nullptr;
    if (ribbon == nullptr) {
        std::fprintf(stderr, "--check-icons: the ribbon is not in the scene\n");
        return 1;
    }
    QVariant listed;
    if (!QMetaObject::invokeMethod(ribbon, "iconNames", Q_RETURN_ARG(QVariant, listed))) {
        std::fprintf(stderr, "--check-icons: the ribbon has no icon list\n");
        return 1;
    }
    ensureIconTheme();
    std::fprintf(stderr, "theme=%s paths=%s\n", qUtf8Printable(QIcon::themeName()),
                 qUtf8Printable(QIcon::themeSearchPaths().join(QLatin1Char(':'))));

    QStringList names;
    const QVariantList listedNames = listed.toList();
    for (const QVariant &entry : listedNames) {
        const QString name = entry.toString();
        if (!names.contains(name)) {
            names.append(name);
        }
    }
    // The start view's own names, so a machine that lacks one of them fails here too.
    const QStringList startNames = {
        QStringLiteral("go-home"), QStringLiteral("x-office-address-book"),
        QStringLiteral("x-office-document"), QStringLiteral("document-new"),
        QStringLiteral("document-open"),
    };
    for (const QString &name : startNames) {
        if (!names.contains(name)) {
            names.append(name);
        }
    }

    int project = 0;
    int theme = 0;
    int missing = 0;
    for (const QString &name : names) {
        const bool own = actionicons::has(name);
        QIcon icon = QIcon::fromTheme(name + QStringLiteral("-symbolic"));
        if (icon.isNull()) {
            icon = QIcon::fromTheme(name);
        }
        const bool themed = !icon.isNull();
        const char *source = own ? "project" : (themed ? "theme" : "missing");
        std::fprintf(stderr, "  %-32s %s\n", qUtf8Printable(name), source);
        project += own ? 1 : 0;
        theme += (!own && themed) ? 1 : 0;
        missing += (!own && !themed) ? 1 : 0;
    }
    std::fprintf(stderr, "icons: %d project, %d theme, %d missing\n", project, theme, missing);
    if (provider == nullptr) {
        std::fprintf(stderr, "icons: no provider to measure\n");
        return 1;
    }
    // The measurement that sets the top: total entries and the largest count for one
    // `(pixels, tone)` pair, the unit of the engineering budget.
    std::fprintf(stderr, "icons: cache total %d, max group %d\n", provider->cacheSize(),
                 provider->maxGroupSize());

    int failures = missing;

    // The reverse direction: a ribbon command with no drawing, or an iconless table entry
    // outside the documented allowlist, fails the same check.
    failures += checkRibbonCoverage(ribbon);

    // The oldest entry leaves when the top is reached: `QCache` is the LRU.
    QSize probeSize;
    provider->requestPixmap(QStringLiteral("lru-probe?tone=muted"), &probeSize, QSize(16, 16));
    const bool inserted = provider->contains(QStringLiteral("lru-probe?tone=muted"), 16);
    for (int index = 0; index <= ThemeIconProvider::kMaxCachedIcons; ++index) {
        const QString id = QStringLiteral("cache-probe-") + QString::number(index)
                           + QStringLiteral("?tone=muted");
        provider->requestPixmap(id, &probeSize, QSize(16, 16));
    }
    const bool bounded = provider->cacheSize() == ThemeIconProvider::kMaxCachedIcons;
    const bool evicted = !provider->contains(QStringLiteral("lru-probe?tone=muted"), 16);
    std::fprintf(stderr, "icons: LRU insert %s, %d entries %s, oldest %s\n",
                 inserted ? "ok" : "FAIL", provider->cacheSize(), bounded ? "ok" : "FAIL",
                 evicted ? "evicted" : "FAIL");
    failures += inserted ? 0 : 1;
    failures += bounded ? 0 : 1;
    failures += evicted ? 0 : 1;

    // A palette change must produce a new key and a new raster, not a stale hit.
    QString accentName;
    for (const QString &name : actionicons::names()) {
        if (actionicons::has(name)) {
            accentName = name;
            break;
        }
    }
    if (accentName.isEmpty()) {
        std::fprintf(stderr, "icons: no project icon for the palette probe\n");
        ++failures;
    } else {
        const QString id = accentName + QStringLiteral("?tone=accent");
        const QSize request(32, 32);
        QSize measured;
        const QPixmap before = provider->requestPixmap(id, &measured, request);
        const QPalette original = QGuiApplication::palette();
        QPalette changed = original;
        changed.setColor(QPalette::Highlight, QColor(190, 40, 40));
        QGuiApplication::setPalette(changed);
        const QPixmap after = provider->requestPixmap(id, &measured, request);
        const bool retinted = before.cacheKey() != after.cacheKey()
                              && before.toImage() != after.toImage();
        QGuiApplication::setPalette(original);
        std::fprintf(stderr, "icons: palette probe %s\n", retinted ? "ok" : "FAIL");
        failures += retinted ? 0 : 1;
    }
    std::fprintf(stderr, "icons: %d failure(s)\n", failures);
    return failures == 0 ? 0 : 1;
}

} // namespace

int main(int argc, char *argv[])
{
    QGuiApplication app(argc, argv);
    app.setApplicationName(QStringLiteral("Punchi Fluid Lab"));
    app.setOrganizationName(QStringLiteral("PunchiSoft"));

    const QStringList arguments = app.arguments();
    // A capture or a self test must not overwrite the panel state and the auto-hide flag
    // the user chose for their own session. Automated runs get a settings file of their
    // own, keyed by the process, so one run cannot inherit the previous one either.
    const bool automatedRun =
        arguments.contains(QStringLiteral("--selftest"))
        || arguments.contains(QStringLiteral("--check-qml"))
        || arguments.contains(QStringLiteral("--check-icons"))
        || arguments.contains(QStringLiteral("--check-layout"))
        || arguments.contains(QStringLiteral("--font-scale"))
        || arguments.contains(QStringLiteral("--grab"))
        || arguments.contains(QStringLiteral("--export"))
        || arguments.contains(QStringLiteral("--probe"))
        || arguments.contains(QStringLiteral("--report-paints"));
    if (automatedRun) {
        QSettings::setDefaultFormat(QSettings::IniFormat);
        QSettings::setPath(QSettings::IniFormat, QSettings::UserScope,
                           QDir::tempPath() + QStringLiteral("/punchi-fluid-lab-settings-")
                               + QString::number(QCoreApplication::applicationPid()));
    }
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
        report("buzz", sources.buzzer);
        if (failures > 0) {
            return 1;
        }
    }

    if (arguments.contains(QStringLiteral("--check-theme"))) {
        const ThemeTokens tokens = tokensFrom(QGuiApplication::palette());
        const Contrast contrast = measure(tokens);
        std::fprintf(stderr, "contrast: %s\n", qUtf8Printable(describe(contrast)));
        std::fprintf(stderr, "spacing: unit %d px, motion %d ms\n", tokens.unit, tokens.motionMs);
        return contrast.passes() ? 0 : 1;
    }

    const QString iconSheet = value(QStringLiteral("--icon-sheet"));
    if (!iconSheet.isEmpty()) {
        // The review capture is a manual render, not a gate check: `--scale 2` renders the
        // icons and the image at twice the size, so the evidence is a real 2x raster and
        // not an upscaled one.
        qreal scale = 1.0;
        const QString scaleText = value(QStringLiteral("--scale"));
        if (!scaleText.isEmpty()) {
            bool scaleOk = false;
            scale = scaleText.toDouble(&scaleOk);
            if (!scaleOk || scale < 0.5 || scale > 4.0) {
                std::fprintf(stderr, "--scale takes a factor from 0.5 to 4, not \"%s\"\n",
                             qUtf8Printable(scaleText));
                return 2;
            }
        }
        const ThemeTokens tokens = tokensFrom(QGuiApplication::palette());
        const QStringList names = actionicons::names();
        const int cell = qRound(56 * scale);
        constexpr int perRow = 12;
        const int rowsPerSize = (names.size() + perRow - 1) / perRow;
        const int columns = std::min(static_cast<int>(names.size()), perRow);
        QImage image(columns * cell, rowsPerSize * 3 * cell, QImage::Format_ARGB32);
        image.fill(tokens.background);
        QPainter painter(&image);
        painter.setRenderHint(QPainter::Antialiasing, true);
        for (int sizeRow = 0; sizeRow < 3; ++sizeRow) {
            const int base = sizeRow == 0 ? 16 : (sizeRow == 1 ? 24 : 32);
            const int pixels = qRound(base * scale);
            for (int index = 0; index < names.size(); ++index) {
                const int row = sizeRow * rowsPerSize + index / perRow;
                const int col = index % perRow;
                const QPixmap icon = actionicons::render(names.at(index), pixels, 1.0, tokens.ink);
                painter.drawPixmap(col * cell + (cell - pixels) / 2,
                                   row * cell + (cell - pixels) / 2, icon);
            }
        }
        painter.end();
        if (!image.save(iconSheet)) {
            std::fprintf(stderr, "--icon-sheet: could not write %s\n",
                         qUtf8Printable(iconSheet));
            return 1;
        }
        std::fprintf(stderr, "icon sheet: %d icons at 16, 24 and 32 px (scale %g) in %s\n",
                     static_cast<int>(names.size()), static_cast<double>(scale),
                     qUtf8Printable(iconSheet));
        return 0;
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

    // The spacing unit follows the desktop font, so a larger font is the case a layout has
    // to survive; scaling it here is how that case is proved without a second desktop.
    const QString fontScale = value(QStringLiteral("--font-scale"));
    if (!fontScale.isEmpty()) {
        bool scaleOk = false;
        const double factor = fontScale.toDouble(&scaleOk);
        if (!scaleOk || factor < 0.75 || factor > 4.0) {
            std::fprintf(stderr, "--font-scale takes a factor from 0.75 to 4, not \"%s\"\n",
                         qUtf8Printable(fontScale));
            return 2;
        }
        QFont font = QGuiApplication::font();
        font.setPointSizeF(font.pointSizeF() * factor);
        QGuiApplication::setFont(font);
    }

    QQmlApplicationEngine engine;
    // The desktop's own icons, resolved through QIcon::fromTheme and tinted with the ink
    // the theme layer picked. A name the theme lacks draws nothing at all. The provider is
    // kept here only while the engine owns it; it is never deleted by this function.
    auto *iconProvider = new ThemeIconProvider;
    engine.addImageProvider(QStringLiteral("theme"), iconProvider);
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

    const bool paperList = arguments.contains(QStringLiteral("--paper-list"));
    const bool marginList = arguments.contains(QStringLiteral("--margin-list"));
    const QString ribbonFlyout = value(QStringLiteral("--ribbon-flyout"));
    if (arguments.contains(QStringLiteral("--tab")) || arguments.contains(QStringLiteral("--menu"))
        || paperList || marginList || !ribbonFlyout.isEmpty()) {
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
        } else if (paperList || marginList) {
            // The page controls and the launcher live in Archivo.
            ribbon->setProperty("current", 0);
        } else if (!ribbonFlyout.isEmpty()) {
            // A collapsed group belongs to the Editar table, the tab P-028 measures.
            ribbon->setProperty("current", 1);
        }
        if (arguments.contains(QStringLiteral("--menu"))) {
            ribbon->setProperty("menuOpen", true);
        }
        if (!ribbonFlyout.isEmpty()) {
            // The ribbon's timer waits for the row to measure and collapse before it opens
            // the group, so the capture shows the resting layout with its flyout.
            ribbon->setProperty("flyoutRequest", ribbonFlyout);
        }
        if (paperList) {
            ribbon->setProperty("paperListRequested", true);
        }
        if (marginList) {
            ribbon->setProperty("marginListRequested", true);
        }
    }

    if (arguments.contains(QStringLiteral("--block"))) {
        for (QObject *root : engine.rootObjects()) {
            if (root->property("titleBlockOpen").isValid()) {
                root->setProperty("titleBlockOpen", true);
            }
        }
    }

    if (arguments.contains(QStringLiteral("--catalog-closed"))) {
        for (QObject *root : engine.rootObjects()) {
            if (root->property("catalogOpen").isValid()) {
                root->setProperty("catalogOpen", false);
            }
        }
    }

    const QString catalogWidth = value(QStringLiteral("--catalog-width"));
    if (!catalogWidth.isEmpty()) {
        bool ok = false;
        const int width = catalogWidth.toInt(&ok);
        if (!ok || width < 220 || width > 480) {
            std::fprintf(stderr, "--catalog-width takes 220 to 480, not \"%s\"\n",
                         qUtf8Printable(catalogWidth));
            return 2;
        }
        for (QObject *root : engine.rootObjects()) {
            if (root->property("catalogWidth").isValid()) {
                root->setProperty("catalogWidth", width);
            }
        }
    }

    // The program opens on Inicio; the automated runs that measure the window or drive the
    // sheet keep it closed so they see the page they were written against (ADR 0031). The
    // icon cache and the start-view geometry are measured with the page open.
    const bool windowTest =
        arguments.contains(QStringLiteral("--selftest"))
        || arguments.contains(QStringLiteral("--check-layout"))
        || arguments.contains(QStringLiteral("--check-qml"))
        || arguments.contains(QStringLiteral("--check-theme"))
        || arguments.contains(QStringLiteral("--check-icons"));
    const bool startViewMeasure = arguments.contains(QStringLiteral("--check-icons"))
                                  || arguments.contains(QStringLiteral("--check-layout"));
    for (QObject *root : engine.rootObjects()) {
        if ((startViewMeasure || !windowTest)
            && root->property("startViewOpen").isValid()) {
            root->setProperty("startViewOpen", true);
        }
    }

    if (arguments.contains(QStringLiteral("--check-icons"))) {
        // The page has to be laid out and its images requested before the cache is
        // measured: one timer tick, then the probes run on the GUI thread.
        QTimer::singleShot(600, &app, [&engine, iconProvider] {
            QCoreApplication::exit(runIconCheck(engine, iconProvider));
        });
        return app.exec();
    }

    if (arguments.contains(QStringLiteral("--page-setup"))) {
        for (QObject *root : engine.rootObjects()) {
            if (root->property("pageSetupOpen").isValid()) {
                root->setProperty("pageSetupOpen", true);
            }
        }
    }

    if (arguments.contains(QStringLiteral("--about"))
        || arguments.contains(QStringLiteral("--license"))) {
        for (QObject *root : engine.rootObjects()) {
            if (root->property("aboutOpen").isValid()) {
                root->setProperty("aboutOpen", arguments.contains(QStringLiteral("--about")));
                root->setProperty("licenseOpen", arguments.contains(QStringLiteral("--license")));
            }
        }
    }

    const auto sessions = [&engine]() {
        QList<FluidSession *> found;
        for (QObject *root : engine.rootObjects()) {
            found += root->findChildren<FluidSession *>();
        }
        return found;
    };

    // For the capture of a content model, before any frame is grabbed.
    const QString blockModel = value(QStringLiteral("--block-model"));
    if (!blockModel.isEmpty()) {
        bool ok = false;
        const int model = blockModel.toInt(&ok);
        if (!ok || model < 0 || model > 2) {
            std::fprintf(stderr, "--block-model takes 0, 1 or 2\n");
            return 2;
        }
        for (FluidSession *session : sessions()) {
            session->setBlockModel(model);
        }
    }

    if (arguments.contains(QStringLiteral("--example"))) {
        for (FluidSession *session : sessions()) {
            if (!session->loadExample()) {
                std::fprintf(stderr, "--example failed: %s\n",
                             qUtf8Printable(session->lastError()));
            }
        }
    }

    const QString templateArg = value(QStringLiteral("--template"));
    if (!templateArg.isEmpty()) {
        bool ok = false;
        const int index = templateArg.toInt(&ok);
        if (!ok || index < 0 || index >= static_cast<int>(PF_TEMPLATE_COUNT)) {
            std::fprintf(stderr, "--template takes 0 to %u\n", PF_TEMPLATE_COUNT - 1);
            return 2;
        }
        for (FluidSession *session : sessions()) {
            session->newDocument();
            if (!session->loadTemplate(index)) {
                std::fprintf(stderr, "--template failed: %s\n",
                             qUtf8Printable(session->lastError()));
            }
        }
    }

    const QString openPath = value(QStringLiteral("--open"));
    if (!openPath.isEmpty()) {
        for (FluidSession *session : sessions()) {
            if (!session->openFrom(openPath)) {
                std::fprintf(stderr, "--open failed: %s\n",
                             qUtf8Printable(session->lastError()));
            }
        }
    }

    // A document asked for on the command line is what the window shows: opening a file,
    // loading the example or a template is the reader choosing the sheet over Inicio.
    if (arguments.contains(QStringLiteral("--example"))
        || arguments.contains(QStringLiteral("--template"))
        || !openPath.isEmpty()) {
        for (QObject *root : engine.rootObjects()) {
            if (root->property("startViewOpen").isValid()) {
                root->setProperty("startViewOpen", false);
            }
        }
    }

    const QString gridStepArg = value(QStringLiteral("--grid-step"));
    if (!gridStepArg.isEmpty()) {
        bool ok = false;
        const double step = gridStepArg.toDouble(&ok);
        if (!ok || step < 1.0 || step > 100.0) {
            std::fprintf(stderr, "--grid-step takes a value between 1 and 100 mm\n");
            return 2;
        }
        for (FluidSession *session : sessions()) {
            session->setGridStepMm(step);
        }
    }

    const QString exportPath = value(QStringLiteral("--export"));
    if (!exportPath.isEmpty()) {
        const QString paperArg = value(QStringLiteral("--paper"));
        const QString dpiArg = value(QStringLiteral("--dpi"));
        bool exported = true;
        for (FluidSession *session : sessions()) {
            int paper = qMax(session->sheetPaper(), 0);
            bool landscape = session->sheetLandscape();
            if (!paperArg.isEmpty()) {
                bool ok = false;
                paper = paperArg.toInt(&ok);
                if (!ok || paper < 0 || paper >= session->paperNames().size()) {
                    std::fprintf(stderr, "--paper takes an index the export offers\n");
                    return 2;
                }
            }
            if (arguments.contains(QStringLiteral("--landscape"))) {
                landscape = true;
            }
            const int kind = exportPath.endsWith(QLatin1String(".svg"), Qt::CaseInsensitive)
                ? 2
                : (exportPath.endsWith(QLatin1String(".pdf"), Qt::CaseInsensitive) ? 1 : 0);
            const qreal dpi = dpiArg.isEmpty() ? 300.0 : dpiArg.toDouble();
            if (paperArg.isEmpty()) {
                // No paper asked for: the document's sheet decides, exactly as on screen.
                QVariantMap options;
                options.insert(QStringLiteral("path"), exportPath);
                options.insert(QStringLiteral("kind"), kind);
                options.insert(QStringLiteral("custom_paper"), false);
                options.insert(QStringLiteral("landscape"), landscape);
                options.insert(QStringLiteral("dpi"), dpi);
                options.insert(QStringLiteral("margins"),
                               QVariantList { session->sheetMarginTop(), session->sheetMarginRight(),
                                              session->sheetMarginBottom(), session->sheetMarginLeft() });
                options.insert(QStringLiteral("parts_list"), true);
                exported = session->exportWithOptions(options) && exported;
            } else {
                exported = session->exportTo(exportPath, kind, paper, landscape, dpi) && exported;
            }
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

    // A capture can name the spot: the strip beside the title block is only visible with a
    // component in it, and the fixed spots below never reach that corner.
    QPointF captureSpot;
    bool hasCaptureSpot = false;
    for (int at = 0; at + 1 < arguments.size(); ++at) {
        if (arguments.at(at) != QStringLiteral("--spot")) {
            continue;
        }
        const QStringList parts = arguments.at(at + 1).split(QLatin1Char(','));
        bool xOk = false;
        bool yOk = false;
        const double x = parts.value(0).toDouble(&xOk);
        const double y = parts.value(1).toDouble(&yOk);
        if (parts.size() != 2 || !xOk || !yOk) {
            std::fprintf(stderr, "--spot takes x,y in millimetres\n");
            return 2;
        }
        captureSpot = QPointF(x, y);
        hasCaptureSpot = true;
    }

    int drop = 0;
    for (int at = 0; at + 1 < arguments.size(); ++at) {
        if (arguments.at(at) != QStringLiteral("--place")) {
            continue;
        }
        bool ok = false;
        int catalogIndex = arguments.at(at + 1).toInt(&ok);
        if (!ok) {
            // An id is easier to write in a capture than a list position.
            for (FluidSession *session : sessions()) {
                for (const QVariant &entry : session->catalog()) {
                    const QVariantMap row = entry.toMap();
                    if (row.value(QStringLiteral("id")).toString() == arguments.at(at + 1)) {
                        catalogIndex = row.value(QStringLiteral("index")).toInt();
                        ok = true;
                    }
                }
            }
        }
        if (!ok) {
            std::fprintf(stderr, "--place takes a catalog index or id, not \"%s\"\n",
                         qUtf8Printable(arguments.at(at + 1)));
            return 2;
        }
        // Free spots of the default composition, so a capture shows the drop without
        // covering the circuit the model is simulating.
        const QPointF spots[] = {QPointF(36.0, 112.0), QPointF(158.0, 112.0), QPointF(96.0, 22.0)};
        for (FluidSession *session : sessions()) {
            const QPointF spot = hasCaptureSpot ? captureSpot : spots[drop % 3];
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
    if (arguments.contains(QStringLiteral("--check-layout"))) {
        // The adaptive shell, proved where it is decided: the window is resized to the sizes
        // the capability names and the mode, the sheet's floor and the panels are read back
        // from the live items. A mode that only exists in a comment fails here.
        auto *window = qobject_cast<QQuickWindow *>(engine.rootObjects().constFirst());
        auto *sheet = window != nullptr ? window->findChild<SchematicItem *>() : nullptr;
        auto *catalog = window != nullptr
                            ? window->findChild<QQuickItem *>(QStringLiteral("catalogDock"))
                            : nullptr;
        auto *dock = window != nullptr
                         ? window->findChild<QQuickItem *>(QStringLiteral("rightDock"))
                         : nullptr;
        if (window == nullptr || sheet == nullptr || catalog == nullptr || dock == nullptr) {
            std::fprintf(stderr, "layout: no window, canvas or dock\n");
            return 1;
        }
        const QList<FluidSession *> layoutSessions = sessions();
        // The dock only exists while it has content, so the check opens the inspector: the
        // squeezed case is the one with both panels asking for room.
        for (FluidSession *session : layoutSessions) {
            session->inspectValve();
        }
        const qreal unit = layoutSessions.isEmpty() ? 18.0 : layoutSessions.constFirst()->unit();
        struct LayoutCase {
            int width;
            int height;
        };
        const LayoutCase cases[] = {
            { 900, 640 }, { 1000, 700 }, { 1050, 700 }, { 1100, 700 },
            { 1200, 760 }, { 1280, 800 }, { 1440, 900 }, { 1920, 1000 },
        };
        // The boundaries are the shell's own content minimums, read from the window, so a
        // desktop with a larger font moves them instead of failing this check.
        const int catalogPref = window->property("catalogWidth").toInt();
        const qreal dockPref = window->property("dockPreference").toReal();
        const qreal floor = window->property("sheetFloor").toReal();
        const auto expectedMode = [&](int width) {
            const qreal room = static_cast<qreal>(width) - 2.0 * unit;
            if (room >= static_cast<qreal>(catalogPref) + dockPref + floor) {
                return QStringLiteral("wide");
            }
            if (room >= 220.0 + 300.0 + floor) {
                return QStringLiteral("medium");
            }
            return QStringLiteral("compact");
        };
        std::fprintf(stderr,
                     "layout: at unit %.0f the wide mode starts at %d and the medium at %d\n",
                     unit, static_cast<int>(static_cast<qreal>(catalogPref) + dockPref + floor
                                            + 2.0 * unit),
                     static_cast<int>(220.0 + 300.0 + floor + 2.0 * unit));
        // The panels animate their width, so every case is read after the motion, not
        // during it: a mode decided on a half-open panel is not what the user sees.
        const auto settle = [] {
            QEventLoop loop;
            QTimer::singleShot(260, &loop, &QEventLoop::quit);
            loop.exec();
        };
        int failures = 0;
        // Inicio: the two way-in cards reserve a 2 x unit icon column and the template
        // cards derive their height from their width, so the plate is 65 % of the card.
        // The page is open here; the shell cases below need the sheet, so it closes after.
        const auto startViewCase = [&](int width, int height) {
            window->resize(width, height);
            settle();
            auto *start = window->findChild<QQuickItem *>(QStringLiteral("startView"));
            bool ok = start != nullptr && start->isVisible();
            QString detail;
            if (ok) {
                // Inicio hides the ribbon and the status bar, and the page covers the whole
                // window: a region measured with the hidden bars still subtracted stops one
                // ribbon above the floor and clips the recent rows.
                ok = std::abs(start->width() - window->width()) <= 1.0
                     && std::abs(start->height() - window->height()) <= 1.0
                     && std::abs(start->x()) <= 1.0 && std::abs(start->y()) <= 1.0;
                if (!ok) {
                    detail = QStringLiteral("page %1x%2 at %3,%4 in a %5x%6 window")
                                 .arg(start->width())
                                 .arg(start->height())
                                 .arg(start->x())
                                 .arg(start->y())
                                 .arg(window->width())
                                 .arg(window->height());
                }
                const qreal railIcon = std::round(unit * 1.1);
                const qreal cardIcon = std::round(unit * 1.8);
                auto *rail = start->findChild<QQuickItem *>(QStringLiteral("railHomeIcon"));
                ok = rail != nullptr && std::abs(rail->width() - railIcon) <= 1.0;
                const QStringList cardNames = { QStringLiteral("New"),
                                                QStringLiteral("Open") };
                for (const QString &name : cardNames) {
                    auto *column = start->findChild<QQuickItem *>(
                        QStringLiteral("startCard") + name + QStringLiteral("IconColumn"));
                    auto *icon = start->findChild<QQuickItem *>(
                        QStringLiteral("startCard") + name + QStringLiteral("Icon"));
                    auto *text = start->findChild<QQuickItem *>(
                        QStringLiteral("startCard") + name + QStringLiteral("Text"));
                    if (column == nullptr || icon == nullptr || text == nullptr
                        || text->parentItem() == nullptr) {
                        ok = false;
                        detail = QStringLiteral("missing card parts");
                        continue;
                    }
                    const qreal textWidth = text->parentItem()->width()
                                            - std::round(unit * 2) - unit / 2.0;
                    bool cardOk = std::abs(column->width() - unit * 2) <= 1.0
                                  && std::abs(icon->width() - cardIcon) <= 1.0
                                  && std::abs(icon->x() + icon->width() * 0.5
                                              - column->width() * 0.5) <= 1.0
                                  && text->x() >= column->x() + column->width() - 0.5
                                  && std::abs(text->x()
                                              - (column->x() + column->width() + unit / 2.0))
                                         <= 1.0
                                  && std::abs(text->width() - textWidth) <= 1.0;
                    for (QQuickItem *child : text->childItems()) {
                        if (child->property("text").isValid()) {
                            cardOk = cardOk
                                     && child->property("contentWidth").toReal()
                                            <= child->width() + 1.0;
                        }
                    }
                    ok = ok && cardOk;
                }
                auto *repeater =
                    start->findChild<QObject *>(QStringLiteral("templateRepeater"));
                const int templateCount =
                    repeater != nullptr ? repeater->property("count").toInt() : 0;
                if (repeater == nullptr
                    || templateCount != start->property("templateCardCount").toInt()) {
                    ok = false;
                    detail = QStringLiteral("template count %1").arg(templateCount);
                }
                // The delegates are reached through the visual tree: a Repeater's items do
                // not answer `findChild` by object name, but its `itemAt` does.
                for (int index = 0; index < templateCount; ++index) {
                    QQuickItem *card = nullptr;
                    const bool got = QMetaObject::invokeMethod(
                        repeater, "itemAt", Q_RETURN_ARG(QQuickItem *, card),
                        Q_ARG(int, index));
                    QQuickItem *content =
                        (got && card != nullptr)
                            ? card->property("contentItem").value<QQuickItem *>()
                            : nullptr;
                    QQuickItem *column =
                        content != nullptr ? content->childItems().value(0) : nullptr;
                    QQuickItem *plate =
                        column != nullptr ? column->childItems().value(0) : nullptr;
                    QQuickItem *title =
                        column != nullptr ? column->childItems().value(1) : nullptr;
                    QQuickItem *note =
                        column != nullptr ? column->childItems().value(2) : nullptr;
                    QQuickItem *preview =
                        plate != nullptr ? plate->childItems().value(0) : nullptr;
                    if (card == nullptr || content == nullptr || column == nullptr
                        || plate == nullptr || title == nullptr || note == nullptr
                        || preview == nullptr) {
                        ok = false;
                        QString missing;
                        if (card == nullptr) missing += QStringLiteral("card ");
                        if (content == nullptr) missing += QStringLiteral("content ");
                        if (column == nullptr) missing += QStringLiteral("column ");
                        if (plate == nullptr) missing += QStringLiteral("plate ");
                        if (title == nullptr) missing += QStringLiteral("title ");
                        if (note == nullptr) missing += QStringLiteral("detail ");
                        if (preview == nullptr) missing += QStringLiteral("preview ");
                        detail = QStringLiteral("template %1 missing %2")
                                     .arg(index)
                                     .arg(missing);
                        continue;
                    }
                    const qreal plateWidth = card->width() - 2.0 * (unit / 3.0)
                                             - 2.0 * (unit / 4.0);
                    const qreal plateHeight = plateWidth / (512.0 / 288.0);
                    const qreal cardHeight = plateHeight / 0.65;
                    const bool templateOk =
                        std::abs(card->property("plateHeight").toReal() - plateHeight) <= 1.0
                        && std::abs(plate->width() - plateWidth) <= 1.0
                        && std::abs(plate->height() - plateHeight) <= 1.0
                        && std::abs(card->height() - cardHeight) <= 1.0
                        && plate->height() / card->height() >= 0.60
                        && plate->height() / card->height() <= 0.70
                        && std::abs(preview->width() - plate->width()) <= 1.0
                        && std::abs(preview->height() - plate->height()) <= 1.0
                        && std::abs(preview->width() / preview->height() - 512.0 / 288.0)
                               <= 0.02
                        && title->property("contentWidth").toReal() <= title->width() + 1.0
                        && note->property("contentWidth").toReal() <= note->width() + 1.0
                        && note->property("maximumLineCount").toInt() == 1;
                    ok = ok && templateOk;
                }
            }
            std::fprintf(stderr, "layout:  start view %4dx%-4d icons, cards and plates  %s%s\n",
                         width, height, ok ? "ok" : "FAIL",
                         detail.isEmpty()
                             ? ""
                             : qUtf8Printable(QStringLiteral(" (") + detail
                                              + QStringLiteral(")")));
            if (!ok) {
                ++failures;
            }
        };
        // The support floor and the widths the capability names. 1366 is the floor; 1280
        // runs with the ribbon folded, but Inicio hides the ribbon and must still hold.
        startViewCase(1280, 800);
        startViewCase(1366, 768);
        startViewCase(1440, 900);
        startViewCase(1600, 900);
        window->setProperty("startViewOpen", false);
        settle();
        // The strip with the fixed home control: the control stays inside the strip, the
        // Flickable starts after it, and the plus remains reachable at the end of the row.
        // The case runs here because the start-page case above closes Inicio first.
        {
            window->resize(1280, 800);
            settle();
            FluidSession *stripSession = layoutSessions.isEmpty() ? nullptr
                                                                 : layoutSessions.constFirst();
            if (stripSession != nullptr) {
                while (static_cast<int>(stripSession->circuits().size()) < 2) {
                    if (stripSession->addCircuit() < 0) {
                        break;
                    }
                }
            }
            settle();
            auto *strip = window->findChild<QQuickItem *>(QStringLiteral("circuitTabs"));
            auto *home = window->findChild<QQuickItem *>(QStringLiteral("homeTab"));
            auto *scroll = window->findChild<QQuickItem *>(QStringLiteral("tabScroll"));
            auto *plus = window->findChild<QQuickItem *>(QStringLiteral("addCircuitButton"));
            bool ok = strip != nullptr && home != nullptr && scroll != nullptr && plus != nullptr
                      && strip->isVisible() && home->isVisible();
            QString detail;
            if (!ok) {
                detail = QStringLiteral("missing home, strip, scroll or plus");
            }
            if (ok) {
                qreal content = scroll->property("contentWidth").toReal();
                ok = home->x() >= 0.0 && home->x() + home->width() <= strip->width()
                     && scroll->x() >= home->x() + home->width();
                if (!ok) {
                    detail = QStringLiteral("home at %1, scroll at %2, strip %3")
                                 .arg(home->x()).arg(scroll->x()).arg(strip->width());
                }
                // Two circuits do not fill the strip at the sheet's floor: the tabs are
                // capped at 200 px and the scroll keeps at least the sheet width beside the
                // panels. Add circuits, within the project limit, until the row really
                // scrolls, so the far end and the plus are proved rather than assumed.
                int guard = 0;
                while (ok && content <= scroll->width()
                       && stripSession != nullptr
                       && static_cast<int>(stripSession->circuits().size())
                              < stripSession->circuitLimit()
                       && guard++ < stripSession->circuitLimit()) {
                    if (stripSession->addCircuit() < 0) {
                        break;
                    }
                    settle();
                    content = scroll->property("contentWidth").toReal();
                }
                ok = ok && content > scroll->width();
                if (!ok) {
                    detail = QStringLiteral("content %1, scroll width %2, strip %3")
                                 .arg(content).arg(scroll->width()).arg(strip->width());
                }
            }
            if (ok) {
                scroll->setProperty("contentX", scroll->property("contentWidth").toReal()
                                                    - scroll->width());
                settle();
                const QPointF plusAt = plus->mapToItem(strip, QPointF(0.0, 0.0));
                ok = plusAt.x() >= 0.0 && plusAt.x() + plus->width() <= strip->width();
                if (!ok) {
                    detail = QStringLiteral("plus at %.0f").arg(plusAt.x());
                }
            }
            if (!ok) {
                ++failures;
            }
            std::fprintf(stderr,
                         "layout:  strip home and scrolling plus                %s%s\n",
                         ok ? "ok" : "FAIL",
                         detail.isEmpty()
                             ? ""
                             : qUtf8Printable(QStringLiteral(" (") + detail
                                              + QStringLiteral(")")));
        }
        for (const LayoutCase &one : cases) {
            window->resize(one.width, one.height);
            settle();

            const QString mode = window->property("layoutMode").toString();
            const QString expected = expectedMode(one.width);
            const qreal sheetWidth = sheet->width();
            const bool modeOk = mode == expected;
            const bool floorOk = sheetWidth >= 480.0;
            // A compact window floats its panels, so the sheet keeps the whole width; a
            // docked one gives them a column and the workspace starts after it.
            const bool floatOk = mode != QLatin1String("compact")
                                 || sheetWidth >= one.width - 2.0 * unit - 1.0;
            const QRectF catalogScene(catalog->mapToScene(QPointF(0.0, 0.0)),
                                      QSizeF(catalog->width(), catalog->height()));
            const QRectF dockScene(dock->mapToScene(QPointF(0.0, 0.0)),
                                   QSizeF(dock->width(), dock->height()));
            const bool overlap = catalog->width() > 0.0 && dock->width() > 0.0
                                 && catalogScene.intersects(dockScene);
            const bool ok = modeOk && floorOk && floatOk && !overlap;
            if (!ok) {
                ++failures;
            }
            std::fprintf(stderr,
                         "layout: %4dx%-4d %-7s sheet %6.1f  mode %s  floor %s  spans %s  "
                         "panels %s  catalog %d  dock %d  unit %.1f\n",
                         one.width, one.height, qUtf8Printable(mode), sheetWidth,
                         modeOk ? "ok" : "FAIL", floorOk ? "ok" : "FAIL",
                         floatOk ? "ok" : "FAIL", overlap ? "overlap" : "clear",
                         static_cast<int>(catalog->width()), static_cast<int>(dock->width()),
                         unit);
        }
        // The compact drawer: a panel asked for must open over the sheet without taking a
        // column from it, which is what keeps the drawing at its floor in a small window.
        window->resize(900, 640);
        settle();
        window->setProperty("catalogOverlay", true);
        settle();
        const bool drawer = catalog->width() > 4.0 * unit
                            && sheet->width() >= 900.0 - 2.0 * unit - 1.0;
        if (!drawer) {
            ++failures;
        }
        std::fprintf(stderr,
                     "layout:  compact drawer catalog %d, sheet %6.1f  %s\n",
                     static_cast<int>(catalog->width()), sheet->width(),
                     drawer ? "ok" : "FAIL");
        window->setProperty("catalogOverlay", false);

        // The ribbon is one row with per-group collapse (P-028). The matrix proves the
        // support floor and the wider resolutions: seven groups in order, nothing past the
        // content edge or the body's bottom, every expanded group still labelled, every
        // collapsed group's flyout opening its commands, and the ribbon height unchanged.
        // Below 1280 the P-027 fold is the fallback and is checked separately.
        auto *ribbon = window->findChild<QQuickItem *>(QStringLiteral("ribbon"));
        auto *groupsRow = window->findChild<QQuickItem *>(QStringLiteral("ribbonGroupsRow"));
        auto *ribbonBody = window->findChild<QQuickItem *>(QStringLiteral("ribbonBody"));
        auto *ribbonMenu = window->findChild<QQuickItem *>(QStringLiteral("ribbonMenu"));
        if (ribbon == nullptr || groupsRow == nullptr || ribbonBody == nullptr
            || ribbonMenu == nullptr) {
            std::fprintf(stderr, "layout: no ribbon, group row, body or folded menu\n");
            return 1;
        }
        ribbon->setProperty("current", 1);
        ribbon->setProperty("minimized", false);
        ribbon->setProperty("bodyOpen", false);
        ribbon->setProperty("menuOpen", false);

        const auto visibleGroups = [groupsRow] {
            QList<QQuickItem *> found;
            for (QQuickItem *group : groupsRow->childItems()) {
                if (group->objectName() == QLatin1String("ribbonGroup") && group->isVisible()) {
                    found.append(group);
                }
            }
            return found;
        };
        const QStringList ribbonOrder = {
            QStringLiteral("Historial"), QStringLiteral("Circuito"),
            QStringLiteral("Componente"), QStringLiteral("Alinear"),
            QStringLiteral("Conducto"), QStringLiteral("Repartir"),
            QStringLiteral("Paleta"),
        };
        const bool nominalMetrics = std::abs(unit - 18.0) < 0.1;
        qreal ribbonBaseline = 0.0;
        qreal sheetFloorAt1366 = 0.0;
        const auto ribbonCase = [&](int width, int height, bool folded) {
            window->resize(width, height);
            settle();
            bool ok = true;
            QString detail;
            const bool collapsed = ribbon->property("collapsed").toBool();
            const bool bodyShown = ribbon->property("bodyShown").toBool();
            if (folded) {
                ok = collapsed && !bodyShown;
                if (!ok) {
                    detail = QStringLiteral("expected the folded form");
                }
            } else {
                ok = !collapsed && bodyShown;
                if (!ok) {
                    detail = QStringLiteral("expected the one-row form");
                }
                const qreal ribbonHeight = ribbon->height();
                if (ribbonBaseline <= 0.0) {
                    ribbonBaseline = ribbonHeight;
                }
                if (std::abs(ribbonHeight - ribbonBaseline) > 0.5) {
                    ok = false;
                    detail = QStringLiteral("height %1 vs baseline %2")
                                 .arg(ribbonHeight).arg(ribbonBaseline);
                }
                if (nominalMetrics && std::abs(ribbonHeight - 121.0) > 0.5) {
                    ok = false;
                    detail = QStringLiteral("height %1, not 121").arg(ribbonHeight);
                }
                const QPointF sheetTop =
                    sheet->mapToItem(window->contentItem(), QPointF(0.0, 0.0));
                const double sheetHeight = sheet->height();
                if (sheetTop.y() < ribbonHeight - 0.5 || sheetHeight < 200.0) {
                    ok = false;
                    detail = QStringLiteral("A4 region at %1 with height %2")
                                 .arg(sheetTop.y()).arg(sheetHeight);
                }
                if (width == 1366) {
                    sheetFloorAt1366 = sheetHeight;
                }
                const QList<QQuickItem *> groups = visibleGroups();
                QStringList titles;
                QStringList collapsedTitles;
                qreal rowRight = 0.0;
                for (QQuickItem *group : groups) {
                    const QString title = group->property("title").toString();
                    titles.append(title);
                    const qreal right =
                        group->mapToItem(ribbon, QPointF(group->width(), 0.0)).x();
                    const qreal bottom =
                        group->mapToItem(ribbon, QPointF(0.0, group->height())).y();
                    rowRight = qMax(rowRight, right);
                    if (right > ribbon->width() + 0.5 || bottom > ribbonBody->height()
                                                            + ribbonBody->y() + 0.5) {
                        ok = false;
                        detail = QStringLiteral("%1 reaches %2 of %3, bottom %4")
                                     .arg(title).arg(right).arg(ribbon->width()).arg(bottom);
                    }
                    const bool groupCollapsed = group->property("collapsed").toBool();
                    const int commands = group->property("commandCount").toInt();
                    const bool note = group->property("hasNote").toBool();
                    if (groupCollapsed) {
                        collapsedTitles.append(title);
                        if (title.isEmpty() || commands < 1) {
                            ok = false;
                            detail = QStringLiteral("%1 collapsed without a command title")
                                         .arg(title);
                            continue;
                        }
                        QMetaObject::invokeMethod(group, "openFlyout");
                        settle();
                        // The Popup builds its content lazily and the commands live inside
                        // a Component, so walking the QObject tree misses them. The group
                        // counts the buttons it actually shows.
                        QVariant flyoutCount;
                        const bool counted = QMetaObject::invokeMethod(
                            group, "flyoutCommandCount",
                            Q_RETURN_ARG(QVariant, flyoutCount));
                        const int flyoutCommands = counted ? flyoutCount.toInt() : 0;
                        if (!group->property("flyoutOpen").toBool() || flyoutCommands < 1) {
                            ok = false;
                            detail = QStringLiteral("%1 flyout open %2, actions %3")
                                         .arg(title)
                                         .arg(group->property("flyoutOpen").toBool() ? 1 : 0)
                                         .arg(flyoutCommands);
                        }
                        QMetaObject::invokeMethod(group, "closeFlyout");
                        settle();
                        if (group->property("flyoutOpen").toBool()) {
                            ok = false;
                            detail = QStringLiteral("%1 flyout did not close").arg(title);
                        }
                    } else if (commands < 1 && !note) {
                        ok = false;
                        detail = QStringLiteral("%1 expanded without a label").arg(title);
                    }
                }
                if (groups.size() != 7 || titles != ribbonOrder) {
                    ok = false;
                    detail = QStringLiteral("%1 groups: %2")
                                 .arg(groups.size()).arg(titles.join(QStringLiteral(", ")));
                }
                if (nominalMetrics && width == 1920 && !collapsedTitles.isEmpty()) {
                    ok = false;
                    detail = QStringLiteral("1920 collapses %1")
                                 .arg(collapsedTitles.join(QStringLiteral(", ")));
                }
                if (nominalMetrics && width == 1366
                    && (!collapsedTitles.contains(QStringLiteral("Repartir"))
                        || !collapsedTitles.contains(QStringLiteral("Alinear")))) {
                    ok = false;
                    detail = QStringLiteral("1366 collapses %1")
                                 .arg(collapsedTitles.join(QStringLiteral(", ")));
                }
                std::fprintf(stderr,
                             "layout:  ribbon %4dx%-4d one row, %d groups, collapsed [%s], "
                             "height %.0f, sheet %.0f  %s%s\n",
                             width, height, static_cast<int>(groups.size()),
                             qUtf8Printable(collapsedTitles.join(QStringLiteral(", "))),
                             ribbonHeight, sheetHeight, ok ? "ok" : "FAIL",
                             detail.isEmpty()
                                 ? ""
                                 : qUtf8Printable(QStringLiteral(" (") + detail
                                                  + QStringLiteral(")")));
            }
            if (folded) {
                std::fprintf(stderr, "layout:  ribbon %4dx%-4d folded to the tab row  %s\n",
                             width, height, ok ? "ok" : "FAIL");
            }
            if (!ok) {
                ++failures;
            }
        };
        ribbonCase(1280, 800, true);
        ribbonCase(1366, 768, false);
        ribbonCase(1440, 900, false);
        ribbonCase(1600, 900, false);
        ribbonCase(1920, 1080, false);

        // The support floor keeps room for the A4 page: the one-row ribbon is 121 px at the
        // default unit, and the sheet region below it must not shrink under the old floor.
        if (nominalMetrics
            && (sheetFloorAt1366 <= 0.0 || sheetFloorAt1366 < 480.0)) {
            std::fprintf(stderr,
                         "layout:  ribbon 1366x768 leaves the sheet %.0f, below 480  FAIL\n",
                         sheetFloorAt1366);
            ++failures;
        }

        // A 1366 panel at 125 % scaling has about 1093 logical pixels, so the fold is the
        // expected form there. It is a recorded case, not hidden: the matrix above runs at
        // the logical sizes the window actually receives.
        bool scaleOk = false;
        const double qtScale = qEnvironmentVariable("QT_SCALE_FACTOR").toDouble(&scaleOk);
        if (scaleOk && qtScale > 1.0) {
            window->resize(qRound(1366.0 / qtScale), qRound(768.0 / qtScale));
            settle();
            const bool folded = ribbon->property("collapsed").toBool()
                                && !ribbon->property("bodyShown").toBool();
            std::fprintf(stderr,
                         "layout:  ribbon 1366 panel at %.2f scale, logical %dx%d, %s  %s\n",
                         qtScale, window->width(), window->height(),
                         folded ? "folded" : "one row", folded ? "ok" : "FAIL");
            if (!folded) {
                ++failures;
            }
            // The recorded scale case leaves the window small; the fold control below is
            // measured at the wide width the matrix ended on. The panels animate, and at 2x
            // one settle can still catch them between widths, so the resize is allowed to
            // finish before the sheet is measured.
            window->resize(1920, 1080);
            settle();
            settle();
        }

        // The fold control at the wide width: it removes height and the open panel must not
        // put it back or take room from the sheet.
        const qreal unfoldedHeight = ribbon->height();
        const qreal wideSheet = sheet->width();
        ribbon->setProperty("minimized", true);
        settle();
        const qreal foldedHeight = ribbon->height();
        ribbon->setProperty("bodyOpen", true);
        settle();
        const bool minimizedOk = foldedHeight < unfoldedHeight
                                 && std::abs(ribbon->height() - foldedHeight) <= 0.5
                                 && std::abs(sheet->width() - wideSheet) <= 0.5;
        if (!minimizedOk) {
            ++failures;
        }
        std::fprintf(stderr,
                     "layout:  ribbon unfolded %.2f (sheet %.2f), folded %.2f, body open %.2f, "
                     "sheet %.2f  %s\n",
                     unfoldedHeight, wideSheet, foldedHeight, ribbon->height(), sheet->width(),
                     minimizedOk ? "ok" : "FAIL");
        ribbon->setProperty("bodyOpen", false);
        ribbon->setProperty("minimized", false);

        // 1280: the folded form is measured, and the Conducto group including the P-024a
        // Diámetro is reachable through the menu instead of clipped past the right edge.
        window->resize(1280, 800);
        settle();
        const bool collapsed = ribbon->property("collapsed").toBool()
                               && !ribbon->property("bodyShown").toBool();
        ribbon->setProperty("menuOpen", true);
        settle();
        QVariant flat;
        const bool listed = QMetaObject::invokeMethod(ribbon, "flatActions",
                                                      Q_RETURN_ARG(QVariant, flat));
        const QVariantList actions = listed ? flat.toList() : QVariantList();
        bool hasWorking = false;
        bool hasPilot = false;
        bool hasDrain = false;
        bool hasDiameter = false;
        for (const QVariant &action : actions) {
            const QString id = action.toMap().value(QStringLiteral("id")).toString();
            hasWorking = hasWorking || id == QLatin1String("pipe-working");
            hasPilot = hasPilot || id == QLatin1String("pipe-pilot");
            hasDrain = hasDrain || id == QLatin1String("pipe-drain");
            hasDiameter = hasDiameter || id == QLatin1String("tube-diameter");
        }
        auto *tubeControl = ribbon->property("tubeControl").value<QObject *>();
        // The transport is not a menu entry: ui-ux says it is always visible, so the folded
        // row must carry play/pause, step and stop without opening anything.
        auto *transport = ribbon->findChild<QObject *>(QStringLiteral("ribbonTransport"));
        const bool transportShown = transport != nullptr
                                    && transport->property("visible").toBool()
                                    && transport->children().size() >= 3;
        const bool menuFits = ribbonMenu->isVisible() && ribbonMenu->x() >= 0.0
                              && ribbonMenu->x() + ribbonMenu->width()
                                     <= ribbon->width() + 0.5;
        const bool foldedOk = collapsed && menuFits && listed && hasWorking && hasPilot
                              && hasDrain && hasDiameter && tubeControl != nullptr
                              && transportShown;
        if (!foldedOk) {
            ++failures;
        }
        std::fprintf(stderr,
                     "layout:  ribbon 1280 folded, menu %.0f at %.0f, Conducto %s, "
                     "Diámetro %s, transport %s  %s\n",
                     ribbonMenu->width(), ribbonMenu->x(),
                     hasWorking && hasPilot && hasDrain ? "reachable" : "missing",
                     hasDiameter && tubeControl != nullptr ? "reachable" : "missing",
                     transportShown ? "visible" : "missing",
                     foldedOk ? "ok" : "FAIL");
        ribbon->setProperty("menuOpen", false);
        settle();

        // The window's own geometry is view state too: a saved size comes back inside the
        // current screen, clamped to the available area minus the unit margin and to the
        // 900x600 minimum, so a window saved on a large monitor still opens on a small
        // one (P-028 §1b). The minimum height is the one the contract states.
        window->resize(1180, 740);
        settle();
        QMetaObject::invokeMethod(window, "saveGeometry");
        window->resize(1000, 650);
        settle();
        QMetaObject::invokeMethod(window, "restoreGeometry");
        settle();
        const QRect area = window->screen() != nullptr
                               ? window->screen()->availableGeometry()
                               : QRect();
        const int areaWidth = area.isValid() ? area.width() : 1180;
        const int areaHeight = area.isValid() ? area.height() : 740;
        const int margin = static_cast<int>(2.0 * unit);
        const int expectedWidth =
            std::max(static_cast<int>(window->minimumWidth()),
                     std::min(1180, areaWidth - margin));
        const int expectedHeight =
            std::max(static_cast<int>(window->minimumHeight()),
                     std::min(740, areaHeight - margin));
        const bool sizeBack = window->width() == expectedWidth
                              && window->height() == expectedHeight;
        const bool tallEnough = window->minimumHeight() >= 600.0;
        if (!sizeBack || !tallEnough) {
            ++failures;
        }
        std::fprintf(stderr,
                     "layout:  geometry %dx%d (saved 1180x740 on %dx%d), minimum height %d"
                     "  %s\n",
                     window->width(), window->height(), areaWidth, areaHeight,
                     window->minimumHeight(), sizeBack && tallEnough ? "ok" : "FAIL");
        std::fprintf(stderr, "layout: %d failure(s)\n", failures);
        return failures == 0 ? 0 : 1;
    }

    if (const QString count = value(QStringLiteral("--circuits")); !count.isEmpty()) {
        // A capture of the tab strip with more than one circuit: the program keeps the
        // circuits the session added, so the frame shows what a project of several looks
        // like. It is a capture hook, not a command of the interface.
        bool ok = false;
        const int wanted = count.toInt(&ok);
        if (ok) {
            for (FluidSession *session : sessions()) {
                while (wanted > 1 && static_cast<int>(pf_session_circuit_count(session->handle()))
                                          < wanted) {
                    if (session->addCircuit() < 0) {
                        break;
                    }
                }
            }
        }
    }
    if (arguments.contains(QStringLiteral("--view-panel"))) {
        // The strip's zoom panel, open, for a capture: a click cannot reach it headlessly.
        for (QObject *root : engine.rootObjects()) {
            if (QObject *strip = root->findChild<QObject *>(QStringLiteral("viewStrip"))) {
                strip->setProperty("panelOpen", true);
            }
        }
    }
    if (arguments.contains(QStringLiteral("--ribbon-minimized"))) {
        // The folded ribbon with the open tab's groups over the sheet, which is the state a
        // capture has to show and a click cannot reach headlessly.
        for (QObject *root : engine.rootObjects()) {
            root->setProperty("ribbonMinimized", true);
            if (QObject *ribbon = root->findChild<QObject *>(QStringLiteral("ribbon"))) {
                ribbon->setProperty("bodyOpen", true);
            }
        }
    }
    if (arguments.contains(QStringLiteral("--overlay"))) {
        // A compact window folds its panels; this opens one over the sheet so a capture can
        // show the drawer, which otherwise needs a click on the rail.
        const QString which = value(QStringLiteral("--overlay"));
        for (QObject *root : engine.rootObjects()) {
            if (which == QLatin1String("catalog")) {
                root->setProperty("catalogOverlay", true);
            } else if (which == QLatin1String("right")) {
                root->setProperty("rightDockOverlay", true);
            }
        }
    }
    if (arguments.contains(QStringLiteral("--right-dock-closed"))) {
        // After the inspector opens, so the fold is what the capture shows.
        for (QObject *root : engine.rootObjects()) {
            if (root->property("rightDockOpen").isValid()) {
                root->setProperty("rightDockOpen", false);
            }
        }
    }
    if (arguments.contains(QStringLiteral("--auto-hide"))) {
        for (QObject *root : engine.rootObjects()) {
            if (root->property("panelAutoHide").isValid()) {
                root->setProperty("panelAutoHide", true);
            }
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
            ThemeIconProvider *icons = nullptr;
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
            QPointF routeTo;
            int routePipe = -1;
            float routeOffset = 0.0F;
            /// The exhaust port the closure probes work on, and the inspector refreshes a
            /// closure change emitted; the panel probe never calls `inspectComponent`.
            QString closurePort;
            int closureRefreshes = 0;
            /// The view gestures: the pointer the wheel is aimed at, the sheet point under
            /// it, and the document state that must not change.
            QPointF viewScene;
            QPointF viewSheet;
            uint32_t viewComponents = 0;
            bool viewCanUndo = false;
            /// The zoom before a fit, so one step back can be checked.
            double viewZoom = 1.0;
            /// The strip's height before it folds, so the fold can be measured.
            qreal stripHeight = 0.0;
            /// A file the sheet is saved to, so a step that needs a clean document has one.
            QString unsavedPath;
            /// The document the recent list is proved with.
            QString recentDocument;
            /// Files written to prove the page's slice of the recent list.
            QVector<QString> recentPages;
            /// Inicio's fixture, the file behind the missing row, and the row the pointer
            /// steps click.
            QString startRecent;
            QString startMissing;
            QQuickItem *startRow = nullptr;
            /// The document state a panel toggle must not change.
            bool panelModified = false;
            bool panelCanUndo = false;
            /// The sheet paints the fold costs, so a runaway animation is visible here.
            int panelPaints = 0;
            /// The palette before the theme step, so the check leaves the desktop as it
            /// found it.
            QPalette paletteBeforeTheme;
            /// The document the home control must leave untouched when it opens Inicio.
            int homeCircuits = 0;
            int homeActive = -1;
            bool homeCanUndo = false;
            /// The remembered ribbon fold, restored after the 1280/1440 width check.
            bool ribbonMinimized = false;
        };
        auto *state = new SelfTest();
        state->window = window;
        state->sheet = window != nullptr ? window->findChild<SchematicItem *>() : nullptr;
        state->icons = iconProvider;
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
        const auto sendWheel = [state](const QPointF &scene, int delta,
                                       Qt::KeyboardModifiers modifiers) {
            QWheelEvent event(scene, state->window->mapToGlobal(scene), QPoint(0, 0),
                              QPoint(0, delta), Qt::NoButton, modifiers,
                              Qt::NoScrollPhase, false);
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
        const auto accessibleName = [](QQuickItem *item) {
            if (item == nullptr) {
                return QString();
            }
            QAccessibleInterface *iface = QAccessible::queryAccessibleInterface(item);
            return iface != nullptr ? iface->text(QAccessible::Name) : QString();
        };
        const auto clickItem = [state, send](QQuickItem *item) {
            if (item == nullptr) {
                return;
            }
            const QPointF centre = item->mapToScene(QPointF(item->width() * 0.5,
                                                            item->height() * 0.5));
            send(QEvent::MouseButtonPress, centre, Qt::LeftButton, Qt::LeftButton);
            send(QEvent::MouseButtonRelease, centre, Qt::LeftButton, Qt::NoButton);
        };

        auto steps = std::make_shared<QVector<std::function<void()>>>();

        // The product opens on an empty sheet: the worked example is a fixture the test asks
        // for, so the gestures below are proved on the circuit they were written around.
        steps->push_back([state, check] {
            check(state->session->componentCount() == 0,
                  QStringLiteral("startup: the window opens with an empty sheet"));
            QObject *hint = state->window->findChild<QObject *>(QStringLiteral("emptySheetHint"));
            check(hint != nullptr && hint->property("visible").toBool(),
                  QStringLiteral("startup: the empty sheet says what to do"));
            QObject *pressure =
                state->window->findChild<QObject *>(QStringLiteral("pressureReading"));
            check(pressure != nullptr && !pressure->property("visible").toBool(),
                  QStringLiteral("startup: model readings stay hidden without a model"));
            const bool loaded = state->session->loadExample();
            std::fprintf(stderr, "selftest: startup load example=%d components=%d error=%s\n",
                         loaded ? 1 : 0, state->session->componentCount(),
                         qUtf8Printable(state->session->lastError()));
            check(loaded,
                  QStringLiteral("startup: the example loads on request (%1)")
                      .arg(state->session->lastError()));
            check(state->session->componentCount() >= 4,
                  QStringLiteral("startup: the example is the fixture"));
        });

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
        // The click opens the properties, which now dock instead of floating: the sheet
        // narrows, so the point recorded for the valve is no longer valid. Close the dock
        // and recompute it before the transport steps use it.
        steps->push_back([state, check] {
            check(state->session->inspectorOpen(),
                  QStringLiteral("editing: the valve click opens the properties"));
            state->session->closeInspector();
        });
        steps->push_back([state, valveCentre, toScene] {
            state->valveScene = toScene(valveCentre());
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

        // The air pulse's clock belongs to the session, not to a timer in the drawing: start
        // clears it, a running model advances it, pause freezes it and stop or restart clears
        // it again. Everything here reads the read-only `phase` property, so the check can
        // not pass by reaching into a private value.
        steps->push_back([state, check] {
            state->session->startSimulation();
            check(state->session->phase() == 0.0,
                  QStringLiteral("animation: start clears the phase"));
            state->session->advance(0.25);
            const double running = state->session->phase();
            std::fprintf(stderr, "selftest: animation motion=%d phase=%.2f\n",
                         state->session->motionActive() ? 1 : 0, running);
            check(running > 0.0 || !state->session->motionActive(),
                  QStringLiteral("animation: running advances the phase"));
            state->session->pauseSimulation();
            const double frozen = state->session->phase();
            state->session->advance(0.25);
            check(state->session->phase() == frozen,
                  QStringLiteral("animation: pause freezes the phase"));
            state->session->stopSimulation();
            check(state->session->phase() == 0.0,
                  QStringLiteral("animation: stop clears the phase"));
            state->session->startSimulation();
            state->session->advance(0.25);
            state->session->restart();
            check(state->session->phase() == 0.0,
                  QStringLiteral("animation: restart clears the phase"));
            // Leave the transport running for the gesture block that follows.
            state->session->startSimulation();
        });

        // Simulation: the click is a pushbutton. It opens while held and closes on release.
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonPress, state->valveScene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            // The transport tick is wall-clock. A loaded runner may have run none between
            // the press and this step, which once failed the pressure check on CI alone, so
            // the model is advanced deterministically — bounded by the stroke's own time —
            // before it is read. The assertions are unchanged.
            for (int tick = 0; tick < 20 && state->session->pressureBar() <= 2.0; ++tick) {
                state->session->advance(0.016);
            }
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
        steps->push_back([state, check, send, componentRect, toScene, valveCentre] {
            state->valveIndex = state->session->selected();
            check(state->valveIndex >= 0, QStringLiteral("editing: a click selects"));
            check(state->session->inspectorOpen(), QStringLiteral("editing: the inspector opens"));
            if (state->valveIndex >= 0) {
                state->valveX = componentRect(state->valveIndex).x();
            }
            // The properties opened in the right dock and the sheet narrowed, so the point
            // recorded before the click is no longer where the valve is drawn.
            state->valveScene = toScene(valveCentre());
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
        steps->push_back([state, sendWith, toScene, valveCentre, check] {
            // Press, drag and release in one step. Three timer steps are three chances for
            // the window to be busy between them, and a drag whose press and move are not
            // read in order is a drag that never happened. The pipe drag uses the same rule.
            // The docked editor, if open, takes the right side of the window and the valve
            // can sit under it. Closed and laid out again, the whole sheet is back.
            state->window->setProperty("titleBlockOpen", false);
            // An earlier click opens the properties inspector on the right; while it is
            // open it covers the valve's point and the press never reaches the sheet.
            state->session->closeInspector();
            QCoreApplication::processEvents();
            PfFrame frame {};
            pf_session_build_frame(state->session->handle(), 4.0F, 0.0F, 0, &frame);
            const QPointF start = toScene(valveCentre());
            const QPointF end = start + QPointF(60.0, 0.0);
            sendWith(QEvent::MouseButtonPress, start, Qt::LeftButton, Qt::LeftButton,
                     Qt::ShiftModifier);
            sendWith(QEvent::MouseMove, end, Qt::NoButton, Qt::LeftButton, Qt::ShiftModifier);
            sendWith(QEvent::MouseButtonRelease, end, Qt::LeftButton, Qt::NoButton,
                     Qt::ShiftModifier);
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

        // A rubber band over the whole drawing space marks every component, and Delete takes
        // the group away as one change. The band follows the sheet's frame, so a paper
        // change moves it with the paper instead of leaving components outside it.
        steps->push_back([state, check, send, sendWith, toScene] {
            state->session->stopSimulation();
            state->session->select(-1);
            const double left = state->session->sheetMarginLeft();
            const double top = state->session->sheetMarginTop();
            const double right =
                state->session->sheetWidthMm() - state->session->sheetMarginRight();
            const double bottom =
                state->session->sheetHeightMm() - state->session->sheetMarginBottom();
            const QPointF from = toScene(QPointF(left + 1.0, top + 1.0));
            const QPointF to = toScene(QPointF(right - 1.0, bottom - 1.0));
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

        // A closure belongs to a port: the sheet opens the port menu on a repeated press
        // over one and Alt+C is its keyboard path. The press sequence is the two clicks a
        // hand makes, replayed, so the real press path is what is proved.
        steps->push_back([state, check] {
            state->session->select(state->valveIndex);
            // The inspector narrows the sheet; it is not this probe's dock, so close it
            // before the port's scene point is measured.
            state->session->closeInspector();
            check(state->session->editing(), QStringLiteral("close: the sheet is editing"));
            state->closurePort.clear();
            const QVariantList ports = state->session->portClosurePorts(state->valveIndex);
            for (const QVariant &entry : ports) {
                const QVariantMap row = entry.toMap();
                if (row.value(QStringLiteral("role")).toInt() == 2) {
                    state->closurePort = row.value(QStringLiteral("id")).toString();
                    break;
                }
            }
            check(!state->closurePort.isEmpty(),
                  QStringLiteral("close: the valve has an exhaust port"));
        });
        steps->push_back([state, check, send, toScene] {
            if (state->closurePort.isEmpty()) {
                return;
            }
            const QPointF scene =
                toScene(state->session->portPosition(state->valveIndex, state->closurePort));
            // Two clicks in place. The first press starts a pipe and its release on the
            // same port is a no-op; the second press is the menu.
            send(QEvent::MouseButtonPress, scene, Qt::LeftButton, Qt::LeftButton);
            send(QEvent::MouseButtonRelease, scene, Qt::LeftButton, Qt::NoButton);
            check(state->session->lastError().isEmpty(),
                  QStringLiteral("close: the first release is not an error"));
            send(QEvent::MouseButtonPress, scene, Qt::LeftButton, Qt::LeftButton);
        });
        steps->push_back([state, check] {
            QObject *menu =
                state->window->findChild<QObject *>(QStringLiteral("portMenu"));
            check(menu != nullptr && menu->property("opened").toBool(),
                  QStringLiteral("close: double-click opens the menu on the port"));
            check(menu != nullptr
                      && menu->property("port").toString() == state->closurePort,
                  QStringLiteral("close: the menu is anchored on that port"));
            check(state->session->lastError().isEmpty(),
                  QStringLiteral("close: a port double click leaves lastError empty"));
            QKeyEvent escape(QEvent::KeyPress, Qt::Key_Escape, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &escape);
        });
        steps->push_back([state, check] {
            QObject *menu =
                state->window->findChild<QObject *>(QStringLiteral("portMenu"));
            check(menu != nullptr && !menu->property("opened").toBool(),
                  QStringLiteral("close: Esc closes"));
            state->session->select(state->valveIndex);
            QKeyEvent key(QEvent::KeyPress, Qt::Key_C, Qt::AltModifier);
            QCoreApplication::sendEvent(state->window, &key);
        });
        steps->push_back([state, check] {
            QObject *menu =
                state->window->findChild<QObject *>(QStringLiteral("portMenu"));
            const QVariantList ports = state->session->portClosurePorts(state->valveIndex);
            const QString first =
                ports.isEmpty()
                    ? QString()
                    : ports.constFirst().toMap().value(QStringLiteral("id")).toString();
            check(menu != nullptr && menu->property("opened").toBool()
                      && menu->property("componentIndex").toInt() == state->valveIndex
                      && menu->property("port").toString() == first,
                  QStringLiteral("close: Alt+C opens it for the selected component"));
            QKeyEvent escape(QEvent::KeyPress, Qt::Key_Escape, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &escape);
        });
        // Each preset the exhaust accepts is applied through the menu itself and undone in
        // one step; the catalog decides the list, the probe only walks it.
        steps->push_back([state, check] {
            QObject *menu =
                state->window->findChild<QObject *>(QStringLiteral("portMenu"));
            check(menu != nullptr, QStringLiteral("close: the menu travels with the sheet"));
            if (menu == nullptr || state->closurePort.isEmpty()) {
                return;
            }
            const QPointF point =
                state->session->portPosition(state->valveIndex, state->closurePort);
            int applied = 0;
            const QVariantList choices =
                state->session->portClosures(state->valveIndex, state->closurePort);
            for (const QVariant &entry : choices) {
                const QVariantMap row = entry.toMap();
                if (!row.value(QStringLiteral("available")).toBool()) {
                    continue;
                }
                const QString id = row.value(QStringLiteral("id")).toString();
                const bool opened = QMetaObject::invokeMethod(
                    menu, "openAt", Q_ARG(QVariant, QVariant(state->valveIndex)),
                    Q_ARG(QVariant, QVariant(state->closurePort)),
                    Q_ARG(QVariant, QVariant::fromValue(point)));
                check(opened && menu->property("opened").toBool(),
                      QStringLiteral("close: the menu reopens on the exhaust"));
                QMetaObject::invokeMethod(menu, "choose", Q_ARG(QVariant, QVariant(id)));
                check(state->session->portClosure(state->valveIndex, state->closurePort) == id,
                      QStringLiteral("close: the preset applies"));
                state->session->undo();
                check(state->session->portClosure(state->valveIndex, state->closurePort)
                          .isEmpty(),
                      QStringLiteral("close: one undo returns"));
                ++applied;
            }
            check(applied >= 2,
                  QStringLiteral("close: an exhaust offers the plug and the silencer"));
        });
        steps->push_back([state, check] {
            state->session->inspectComponent(state->valveIndex);
            check(state->session->inspectorOpen(),
                  QStringLiteral("close: the properties are open on the valve"));
        });
        // The panel is refreshed by the change itself: the closure is set after the panel
        // was opened and no second `inspectComponent` runs before it is read.
        steps->push_back([state, check] {
            if (state->closurePort.isEmpty()) {
                return;
            }
            state->closureRefreshes = 0;
            QObject::connect(state->session, &FluidSession::inspectorChanged,
                             state->session, [state] { ++state->closureRefreshes; });
            state->session->setPortClosure(state->valveIndex, state->closurePort,
                                           QStringLiteral("plug.blind"));
            const QVariantList rows = state->session->inspectorClosures();
            bool listed = rows.size() == 1;
            if (listed) {
                const QVariantMap row = rows.constFirst().toMap();
                listed = row.value(QStringLiteral("port")).toString() == state->closurePort
                    && row.value(QStringLiteral("label"))
                           .toString()
                           .startsWith(QStringLiteral("escape "))
                    && !row.value(QStringLiteral("closure")).toString().isEmpty();
            }
            check(state->closureRefreshes > 0,
                  QStringLiteral("close: the change refreshes the panel"));
            check(listed, QStringLiteral("close: the panel lists the closure without an "
                                         "extra inspectComponent"));
            state->session->undo();
            check(state->session->inspectorClosures().isEmpty(),
                  QStringLiteral("close: the panel clears with the undo"));
            state->session->closeInspector();
        });
        steps->push_back([state, check] {
            int cylinder = -1;
            char id[128] = {};
            const uint32_t count = pf_session_component_count(state->session->handle());
            for (uint32_t slot = 0; slot < count; ++slot) {
                id[0] = '\0';
                pf_session_component_id(state->session->handle(), slot, id, sizeof id);
                if (QString::fromUtf8(id).startsWith(QLatin1String("cylinder"))) {
                    cylinder = static_cast<int>(slot);
                }
            }
            check(cylinder >= 0, QStringLiteral("close: the cylinder is placed"));
            if (cylinder < 0 || state->closurePort.isEmpty()) {
                return;
            }
            state->session->setPortClosure(state->valveIndex, state->closurePort,
                                           QStringLiteral("plug.blind"));
            check(state->session->portClosure(state->valveIndex, state->closurePort)
                      == QStringLiteral("plug.blind"),
                  QStringLiteral("close: the exhaust is plugged for the pipe probe"));
            const uint32_t before = pf_session_connection_count(state->session->handle());
            const bool joined = state->session->connectPorts(
                state->valveIndex, state->closurePort, cylinder, QStringLiteral("2"));
            check(!joined && !state->session->lastError().isEmpty()
                      && pf_session_connection_count(state->session->handle()) == before,
                  QStringLiteral("close: a pipe on a closed port is refused"));
            state->session->undo();
            check(state->session->portClosure(state->valveIndex, state->closurePort).isEmpty(),
                  QStringLiteral("close: the closed port opens with the undo"));
            // The very same pipe, now that the port is open, proves the closure refused it
            // and that the document is as the probe found it after the second undo.
            const bool reopened = state->session->connectPorts(
                state->valveIndex, state->closurePort, cylinder, QStringLiteral("2"));
            check(reopened && state->session->lastError().isEmpty(),
                  QStringLiteral("close: the same pipe joins an open port"));
            state->session->undo();
            check(pf_session_connection_count(state->session->handle()) == before,
                  QStringLiteral("close: the probe leaves the pipes as it found them"));
        });

        // Run until a state change: the transport stops on the frame the piston reaches
        // the stop, which is what the reference's own run mode does.
        steps->push_back([state, check] {
            state->session->restart();
            state->session->pressValve();
            bool changed = false;
            // The fast-forward runs on the event loop now; wait for its signal so the
            // selftest still asserts the final state instead of the frame it started on.
            QEventLoop loop;
            QObject::connect(state->session, &FluidSession::untilChangeFinished, &loop,
                             [&changed, &loop](bool reached) {
                                 changed = reached;
                                 loop.quit();
                             });
            QTimer::singleShot(5000, &loop, &QEventLoop::quit);
            state->session->runUntilChange();
            loop.exec();
            check(changed, QStringLiteral("transport: run until change stops"));
            check(state->session->positionMm() > 99.0,
                  QStringLiteral("transport: it stops at the stop"));
            check(!state->session->running(),
                  QStringLiteral("transport: it stays paused"));
            state->session->releaseValve();
            state->session->stopSimulation();
        });

        // Detener must cancel a fast-forward while it is running, not after it ends.
        steps->push_back([state, check] {
            state->session->startSimulation();
            state->session->runUntilChange();
            check(state->session->untilChangeRunning(),
                  QStringLiteral("transport: the fast-forward reports it is running"));
            state->session->stopSimulation();
            check(!state->session->untilChangeRunning() && !state->session->running()
                      && state->session->editing(),
                  QStringLiteral("transport: Detener cancels the fast-forward"));
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
            state->session->setGridStepMm(20.0);
            check(std::abs(state->session->gridStepMm() - 20.0) < 0.01,
                  QStringLiteral("view: the grid step changes"));
            state->session->setGridStepMm(5.0);
            check(std::abs(state->session->gridStepMm() - 5.0) < 0.01
                      && state->session->canUndo() == undoes,
                  QStringLiteral("view: the grid step is a view setting"));
            check(state->session->canUndo() == undoes,
                  QStringLiteral("view: zoom does not enter the history"));
        });

        // A hose is re-routed by dragging its middle segment. The press selects it and the
        // drag moves it; one undo restores the route, because one gesture is one edit.
        steps->push_back([state, check, send, toScene] {
            const double sheetWidth = state->session->sheetWidthMm();
            const double sheetHeight = state->session->sheetHeightMm();
            for (double x = 10.0; x <= sheetWidth - 10.0 && state->routePipe < 0; x += 4.0) {
                for (double y = 10.0; y <= sheetHeight - 10.0; y += 4.0) {
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
                    // already the shortest orthogonal route. The drag is measured in
                    // millimetres of the sheet, not in pixels: the fit scale changes with
                    // the window and the font, and a gesture that depends on it fails on
                    // another machine. The core is asked on a scratch edit first, and the
                    // edit is taken back at once.
                    float scratch = 0.0F;
                    pf_session_pipe_offset(state->session->handle(),
                                           static_cast<uint32_t>(hit), &scratch);
                    const double reach = 15.0;
                    if (pf_session_route_pipe(state->session->handle(),
                                              static_cast<uint32_t>(hit), x + reach,
                                              y + reach)
                        == PF_OK) {
                        float after = 0.0F;
                        pf_session_pipe_offset(state->session->handle(),
                                               static_cast<uint32_t>(hit), &after);
                        if (std::abs(after - scratch) > 0.5F) {
                            state->session->undo();
                            state->routePipe = hit;
                            state->routeScene = toScene(QPointF(x, y));
                            state->routeTo = toScene(QPointF(x + reach, y + reach));
                            break;
                        }
                        state->session->undo();
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
            send(QEvent::MouseMove, state->routeTo, Qt::NoButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            send(QEvent::MouseButtonRelease, state->routeTo, Qt::LeftButton,
                 Qt::NoButton);
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
            const double sheetWidth = state->session->sheetWidthMm();
            const double sheetHeight = state->session->sheetHeightMm();
            for (double x = 10.0; x < sheetWidth - 10.0; x += 2.0) {
                for (double y = 10.0; y < sheetHeight - 10.0; y += 2.0) {
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

        // The owner's T, built from an empty sheet so the coordinates and the pipe
        // indices are fixed here rather than found by sweeping the worked example:
        //   component 0: throttle.fixed at (80, 80), box 20x20, port 2 at (90, 80)
        //   component 1: throttle.fixed at (120, 80), port 1 at (110, 80)
        //   component 2: gauge.pressure at (100, 55), box 22x22, port 1 at (100, 66)
        //   pipe 0: throttle 2 -> throttle 1, the straight horizontal line at y = 80
        //   pipe 0 is spliced at (100, 80); component 3 is the junction, box 8x8, and
        //   the gauge joins its port 3. The branch is pipe 2.
        steps->push_back([state, check, session] {
            state->session->newDocument();
            const auto catalogIndex = [session](const QString &id) {
                for (const QVariant &entry : session->catalog()) {
                    const QVariantMap row = entry.toMap();
                    if (row.value(QStringLiteral("id")).toString() == id) {
                        return row.value(QStringLiteral("index")).toInt();
                    }
                }
                return -1;
            };
            const int throttle = catalogIndex(QStringLiteral("throttle.fixed"));
            const int gauge = catalogIndex(QStringLiteral("gauge.pressure"));
            check(throttle >= 0 && gauge >= 0,
                  QStringLiteral("tee stub: the catalog has the gauge and a restrictor"));
            if (throttle < 0 || gauge < 0) {
                return;
            }
            const int left = state->session->placeComponent(throttle, 80.0, 80.0);
            const int right = state->session->placeComponent(throttle, 120.0, 80.0);
            const int meter = state->session->placeComponent(gauge, 100.0, 55.0);
            check(left == 0 && right == 1 && meter == 2,
                  QStringLiteral("tee stub: the pieces take the fixed indices"));
            check(state->session->connectPorts(left, QStringLiteral("2"), right,
                                               QStringLiteral("1")),
                  QStringLiteral("tee stub: the horizontal line is one straight pipe"));
            check(state->session->splicePipeAt(0, 100.0, 80.0, meter, QStringLiteral("1")),
                  QStringLiteral("tee stub: the gauge lands on the line as a T"));
            check(pf_session_connection_count(state->session->handle()) == 3
                      && pf_session_component_count(state->session->handle()) == 4,
                  QStringLiteral("tee stub: two halves, the branch and the junction"));
        });
        steps->push_back([state, check] {
            // pf_session_pipe_at is the hit test the pointer uses, so the probe reads
            // exactly the route the sheet draws. 0.5 mm is the ABI's minimum reach.
            const int branch =
                pf_session_pipe_at(state->session->handle(), 100.0F, 73.0F, 0.5F);
            const int left = pf_session_pipe_at(state->session->handle(), 95.0F, 80.0F, 0.5F);
            const int right =
                pf_session_pipe_at(state->session->handle(), 105.0F, 80.0F, 0.5F);
            // The old route went from the gauge down to y = 68 and left to x = 94 before
            // turning down to the junction. A point on that horizontal lead must miss the
            // branch now; the probe that used to return it is the regression.
            const int old =
                pf_session_pipe_at(state->session->handle(), 95.0F, 68.0F, 0.5F);
            std::fprintf(stderr,
                         "selftest: tee probe branch=%d left=%d right=%d old=%d\n", branch,
                         left, right, old);
            check(branch == 2,
                  QStringLiteral("tee stub: the vertical branch is pipe 2"));
            check(left == 0 && right == 1,
                  QStringLiteral("tee stub: the two halves are pipes 0 and 1"));
            check(old < 0,
                  QStringLiteral("tee stub: the old horizontal lead is no longer a pipe"));
            state->session->newDocument();
            check(state->session->loadExample(),
                  QStringLiteral("tee stub: the worked example is restored"));
        });

        // Dragging out of the palette, which is how a circuit is actually drawn. The drop is
        // driven through the sidebar's own mouse area, so the mapping from the palette to the
        // sheet is the one the user gets.
        steps->push_back([state, check, send] {
            // The palette tiles live inside a Grid, whose delegates are children of its
            // content item rather than of the window, so the tree is walked by hand. The
            // tile names its own pointer area: the scene holds other large controls (the
            // tube gallery), so guessing by size picks the wrong one.
            QQuickItem *row = nullptr;
            std::function<void(QQuickItem *)> seek = [&](QQuickItem *item) {
                if (row == nullptr && item->isVisible()
                    && item->objectName() == QStringLiteral("paletteTile")) {
                    row = item;
                    return;
                }
                for (QQuickItem *child : item->childItems()) {
                    seek(child);
                }
            };
            seek(state->window->contentItem());
            check(row != nullptr,
                  QStringLiteral("palette: the catalog row is found"));
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

        // A real hand does not travel in one straight jump: it starts up and to the right,
        // still over the palette, and only then crosses to the sheet. The palette scrolls,
        // and a scroll container that takes the gesture cancels the drag, which is exactly
        // the "the symbol vanished on the way" the owner reported.
        steps->push_back([state, check, send] {
            state->components = pf_session_component_count(state->session->handle());
            send(QEvent::MouseButtonPress, state->paletteScene, Qt::LeftButton,
                 Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            // Two moves inside the palette, past the drag threshold, before leaving it.
            const QPointF first = state->paletteScene + QPointF(14.0, -16.0);
            send(QEvent::MouseMove, first, Qt::NoButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send] {
            const QPointF second = state->paletteScene + QPointF(26.0, -34.0);
            send(QEvent::MouseMove, second, Qt::NoButton, Qt::LeftButton);
        });
        steps->push_back([state, check, send, toScene] {
            send(QEvent::MouseMove, toScene(QPointF(170.0, 70.0)), Qt::NoButton,
                 Qt::LeftButton);
        });
        steps->push_back([state, check, send, toScene] {
            send(QEvent::MouseButtonRelease, toScene(QPointF(170.0, 70.0)), Qt::LeftButton,
                 Qt::NoButton);
        });
        steps->push_back([state, check] {
            const uint32_t now = pf_session_component_count(state->session->handle());
            check(now == state->components + 1,
                  QStringLiteral("palette: a diagonal gesture still places the component"));
            while (pf_session_can_undo(state->session->handle()) == 1
                   && pf_session_component_count(state->session->handle()) > state->components) {
                state->session->undo();
            }
        });

        // The same tile has a context menu: the right button adds the entry to the sheet
        // without carrying it, and the command is one undo.
        steps->push_back([state, check, send] {
            state->components = pf_session_component_count(state->session->handle());
            send(QEvent::MouseButtonPress, state->paletteScene, Qt::RightButton,
                 Qt::RightButton);
            send(QEvent::MouseButtonRelease, state->paletteScene, Qt::RightButton,
                 Qt::NoButton);
            QObject *menu = state->window->findChild<QObject *>(QStringLiteral("paletteMenu"));
            check(menu != nullptr && menu->property("opened").toBool(),
                  QStringLiteral("palette: the right button opens the menu"));
            check(state->window->property("dragCatalog").toInt() == -1,
                  QStringLiteral("palette: the right button does not start a carry"));
        });
        steps->push_back([state, check] {
            QObject *menu = state->window->findChild<QObject *>(QStringLiteral("paletteMenu"));
            if (menu == nullptr) {
                return;
            }
            const bool invoked = QMetaObject::invokeMethod(menu, "addToSheet");
            check(invoked, QStringLiteral("palette: the menu command runs"));
            const uint32_t now = pf_session_component_count(state->session->handle());
            check(now == state->components + 1,
                  QStringLiteral("palette: the context menu adds the entry to the sheet"));
            QObject *item = state->window->findChild<QObject *>(
                QStringLiteral("paletteAddToSheet"));
            check(item != nullptr, QStringLiteral("palette: the menu names its command"));
            check(pf_session_can_undo(state->session->handle()) == 1,
                  QStringLiteral("palette: adding from the menu is one undo"));
            state->session->undo();
            check(pf_session_component_count(state->session->handle()) == state->components,
                  QStringLiteral("palette: one undo removes what the menu added"));
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
            check(state->session->loadExample(),
                  QStringLiteral("unsaved: the next circuit test asks for its fixture"));
            QFile::remove(state->unsavedPath);
        });

        // A safety copy exists for a crash and for nothing else: saving, recovering or
        // closing cleanly must leave none behind, or the next start offers work that is
        // already safe — which is what it did, every time.
        steps->push_back([state, check] {
            // Whatever the previous steps left, an edit of its own makes the point: the copy
            // exists for unsaved work and for nothing else.
            if (!state->session->modified()) {
                state->session->setDesignation(0, QStringLiteral("1V1"));
            }
            check(state->session->modified(), QStringLiteral("safety copy: there is unsaved work"));
            state->session->autosave();
            check(QFile::exists(state->session->autosavePath()),
                  QStringLiteral("safety copy: the timer writes one"));
            const QString path = QDir::tempPath() + QStringLiteral("/punchi-close.pfluid");
            check(state->session->saveTo(path),
                  QStringLiteral("safety copy: the work is saved"));
            check(!QFile::exists(state->session->autosavePath())
                      && !state->session->recoveryOffered(),
                  QStringLiteral("safety copy: saving removes the copy"));
            // One more, and a clean close: neither may leave it behind.
            state->session->undo();
            state->session->autosave();
            check(QFile::exists(state->session->autosavePath()),
                  QStringLiteral("safety copy: it comes back with unsaved work"));
            state->session->confirmClose();
            check(!QFile::exists(state->session->autosavePath())
                      && !state->session->recoveryOffered(),
                  QStringLiteral("safety copy: a clean close leaves none"));
            QFile::remove(path);
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

            // A document that moved or was deleted stays in the list, greyed, so the row
            // says why it cannot be opened instead of vanishing.
            QFile::remove(state->recentDocument);
            check(!state->session->openRecent(0),
                  QStringLiteral("recent: opening one that is gone fails"));
            const QVariantList kept = state->session->recent();
            check(kept.size() == 1
                      && !kept.value(0).toMap().value(QStringLiteral("available")).toBool(),
                  QStringLiteral("recent: the page keeps a missing document"));
            check(state->session->recentPage().size() == 1,
                  QStringLiteral("recent: the missing document reaches the page"));

            // Quitar removes that row and nothing else, in memory and in the list file.
            check(state->session->forgetRecentAt(0),
                  QStringLiteral("recent: quitar forgets the row"));
            check(state->session->recent().isEmpty() && state->session->recentPage().isEmpty(),
                  QStringLiteral("recent: quitar forgets only that row"));
            check(!state->session->forgetRecentAt(0),
                  QStringLiteral("recent: an unknown slot is refused"));
        });
        // The page lists a slice of the list: ten are remembered for Archivo, six are shown
        // under the templates, and the number comes from the ABI, never from QML.
        steps->push_back([state, check] {
            state->session->forgetRecent();
            state->recentPages.clear();
            for (int index = 0; index < 8; ++index) {
                const QString path =
                    QDir::tempPath() + QStringLiteral("/punchi-reciente-%1.pfluid").arg(index);
                state->recentPages.append(path);
                check(state->session->saveTo(path),
                      QStringLiteral("recent page: file %1 is written").arg(index));
            }
            check(state->session->recent().size() == 8,
                  QStringLiteral("recent: the list remembers every file"));
            check(state->session->recentPage().size() == state->session->recentPageLimit(),
                  QStringLiteral("recent: the page shows at most its limit"));
            check(state->session->recentPageLimit() == static_cast<int>(PF_RECENT_PAGE_LIMIT),
                  QStringLiteral("recent: the page limit comes from the ABI"));
            const QVariantMap first = state->session->recentPage().value(0).toMap();
            check(first.value(QStringLiteral("slot")).toInt() == 0
                      && first.value(QStringLiteral("available")).toBool(),
                  QStringLiteral("recent: the page names the slot it came from"));
            state->session->forgetRecent();
            for (const QString &path : state->recentPages) {
                QFile::remove(path);
            }
            state->recentPages.clear();
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
                if (pf_session_build_frame(state->session->handle(), 4.0, 0.0, 0, &frame)
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
            check(state->session->exportTo(path, 2, qMax(state->session->sheetPaper(), 0), false,
                                           300.0),
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
            options.insert(QStringLiteral("custom_paper"), true);
            // Large enough for the 170 × 55 mm block and still a little drawing space.
            options.insert(QStringLiteral("width"), 120);
            options.insert(QStringLiteral("height"), 90);
            options.insert(QStringLiteral("landscape"), false);
            options.insert(QStringLiteral("dpi"), 300);
            options.insert(QStringLiteral("parts_list"), false);
            options.insert(QStringLiteral("margins"),
                           QVariantList { 5, 5, 5, 5 });
            check(state->session->exportWithOptions(options),
                  QStringLiteral("export: a custom print paper is accepted"));
            QFile file(path);
            const bool opened = file.open(QIODevice::ReadOnly);
            const QString svg = opened ? QString::fromUtf8(file.readAll()) : QString();
            check(svg.contains(QStringLiteral("viewBox=\"0 0 120.0 90.0\"")),
                  QStringLiteral("export: the custom print paper reaches the page"));
            check(!svg.contains(QStringLiteral("lista de piezas")),
                  QStringLiteral("export: the list can be left off"));
            QFile::remove(path);
        });

        // The title block is per circuit and one edit is one undo; the sheet number is the
        // circuit's place in the project, not a stored field.
        steps->push_back([state, check] {
            const QString circuit = QString::number(state->session->activeCircuit());
            check(state->session->blockTitle() == QStringLiteral("Circuito 1"),
                  QStringLiteral("title block: a new circuit starts with its own title"));
            const QVariantList circuits = state->session->circuits();
            check(state->session->blockSheet()
                      == QStringLiteral("1/%1").arg(circuits.size()),
                  QStringLiteral("title block: the sheet number is derived from the project"));
            state->session->setBlockField(QStringLiteral("owner"),
                                          QStringLiteral("Curso de Neumática"));
            state->session->setBlockField(QStringLiteral("number"),
                                          QStringLiteral("PF-2026-07"));
            check(state->session->blockOwner() == QStringLiteral("Curso de Neumática"),
                  QStringLiteral("title block: the owner is read back"));
            check(state->session->blockNumber() == QStringLiteral("PF-2026-07"),
                  QStringLiteral("title block: the document number is read back"));
            check(state->session->modified(),
                  QStringLiteral("title block: an edited field marks the document"));
            state->session->undo();
            check(state->session->blockNumber().isEmpty(),
                  QStringLiteral("title block: one undo takes the last field back"));
            check(state->session->blockOwner() == QStringLiteral("Curso de Neumática"),
                  QStringLiteral("title block: the previous field stays"));
            state->session->setBlockField(QStringLiteral("owner"), QString());
            state->session->setBlockField(QStringLiteral("number"), QString());
            check(circuit == QString::number(state->session->activeCircuit()),
                  QStringLiteral("title block: editing stays on the circuit in view"));
        });

        // The rest of the guide fields travel the same path, and the preset moves the block
        // without losing them.
        steps->push_back([state, check] {
            check(state->session->blockScale() == QStringLiteral("S/E"),
                  QStringLiteral("title block: the scale starts at S/E"));
            check(state->session->blockProject().isEmpty(),
                  QStringLiteral("title block: the project starts empty"));
            state->session->setBlockField(QStringLiteral("project"),
                                          QStringLiteral("Línea 2"));
            state->session->setBlockField(QStringLiteral("equipment"),
                                          QStringLiteral("Prensa 4"));
            state->session->setBlockField(QStringLiteral("norms"),
                                          QStringLiteral("NCh 14 · NCh 1768/1"));
            check(state->session->blockProject() == QStringLiteral("Línea 2")
                      && state->session->blockEquipment() == QStringLiteral("Prensa 4"),
                  QStringLiteral("title block: the guide fields are read back"));
            check(state->session->blockNorms() == QStringLiteral("NCh 14 · NCh 1768/1"),
                  QStringLiteral("title block: the norms are read back"));
            state->session->undo();
            check(state->session->blockNorms().isEmpty(),
                  QStringLiteral("title block: one undo takes the norms back"));
            check(state->session->blockEquipment() == QStringLiteral("Prensa 4"),
                  QStringLiteral("title block: the previous field stays"));
            state->session->setBlockPreset(0);
            check(state->session->blockPreset() == 0,
                  QStringLiteral("title block: the preset moves the block to the top"));
            state->session->undo();
            check(state->session->blockPreset() == 1,
                  QStringLiteral("title block: one undo returns the block"));
            state->session->setBlockField(QStringLiteral("scale"), QStringLiteral("1:50"));
            check(state->session->blockScale() == QStringLiteral("1:50"),
                  QStringLiteral("title block: the scale is editable"));
            state->session->setBlockField(QStringLiteral("scale"), QStringLiteral("S/E"));
            state->session->setBlockField(QStringLiteral("project"), QString());
            state->session->setBlockField(QStringLiteral("equipment"), QString());
        });

        // Content models: the same values print in fewer rows, and Personalizar hides one
        // field without losing what was typed.
        steps->push_back([state, check] {
            check(state->session->blockModel() == 2 && !state->session->blockCustomized(),
                  QStringLiteral("title block: the industrial model starts"));
            state->session->setBlockModel(0);
            check(state->session->blockModel() == 0 && state->session->blockFields().size() == 4,
                  QStringLiteral("title block: school keeps four fields"));
            check(state->session->blockPreset() == 1,
                  QStringLiteral("title block: choosing content shows the block"));
            state->session->setBlockFieldVisible(PF_BLOCK_TITLE, false);
            check(state->session->blockCustomized(),
                  QStringLiteral("title block: hiding a field personalizes the model"));
            check(state->session->blockTitle() == QStringLiteral("Circuito 1"),
                  QStringLiteral("title block: a hidden field keeps its value"));
            state->session->resetBlockVisibility();
            check(!state->session->blockCustomized(),
                  QStringLiteral("title block: Restablecer returns the model"));
            state->session->setBlockPreset(3);
            state->session->setBlockModel(1);
            check(state->session->blockPreset() == 1 && state->session->blockModel() == 1,
                  QStringLiteral("title block: choosing a model brings the block back"));
            state->session->setBlockModel(2);
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

        // The tube diameter: a pneumatic hose starts at 10 mm, the gallery takes it to
        // 16 mm as one undo, and the status bar names the size. An old pipe says it is
        // unspecified instead of hiding the tubing it needs.
        steps->push_back([state, check] {
            state->session->selectPipe(0);
            check(state->session->pipeTube(0) == 1000,
                  QStringLiteral("tube: a pneumatic hose starts at 10 mm"));
            state->session->setPipeTube(0, 1600);
            check(state->session->pipeTube(0) == 1600,
                  QStringLiteral("tube: the hose takes 16 mm"));
            QObject *reading =
                state->window->findChild<QObject *>(QStringLiteral("pipeTubeReading"));
            check(reading != nullptr
                      && reading->property("visible").toBool()
                      && reading->property("text").toString().contains(QStringLiteral("16 mm")),
                  QStringLiteral("tube: the status bar shows 16 mm"));
            state->session->undo();
            check(state->session->pipeTube(0) == 1000,
                  QStringLiteral("tube: one undo returns the 10 mm"));
            // An old sheet has no size, and the status says so instead of showing 10 mm.
            state->session->setPipeTube(0, 0);
            check(state->session->pipeTube(0) == 0,
                  QStringLiteral("tube: a pipe can be left unspecified"));
            reading = state->window->findChild<QObject *>(QStringLiteral("pipeTubeReading"));
            check(reading != nullptr
                      && reading->property("text").toString().contains(
                          QStringLiteral("Sin especificar")),
                  QStringLiteral("tube: the status says Sin especificar"));
            state->session->setPipeTube(0, 1000);
            state->session->selectPipe(-1);
        });

        // A pipe the user makes gets the documented default too.
        steps->push_back([state, check] {
            int supply = -1;
            int valve = -1;
            const QVariantList entries = state->session->catalog();
            for (const QVariant &entry : entries) {
                const QVariantMap row = entry.toMap();
                const QString id = row.value(QStringLiteral("id")).toString();
                if (id == QStringLiteral("supply.compressor")) {
                    supply = row.value(QStringLiteral("index")).toInt();
                }
                if (id == QStringLiteral("valve.3_2.spring_return")) {
                    valve = row.value(QStringLiteral("index")).toInt();
                }
            }
            check(supply >= 0 && valve >= 0,
                  QStringLiteral("tube: the catalog offers a supply and a valve"));
            const int supply_index =
                supply >= 0 ? state->session->placeComponent(supply, 110.0, 40.0) : -1;
            const int valve_index =
                valve >= 0 ? state->session->placeComponent(valve, 150.0, 40.0) : -1;
            const int before = static_cast<int>(state->session->connectionCount());
            check(state->session->connectPorts(supply_index, QStringLiteral("1"), valve_index,
                                               QStringLiteral("1")),
                  QStringLiteral("tube: a new pipe can be made"));
            check(state->session->connectionCount() == before + 1
                      && state->session->pipeTube(before) == 1000,
                  QStringLiteral("tube: the new pipe is 10 mm"));
            state->session->removeComponent(valve_index);
            state->session->removeComponent(supply_index);
        });

        // An electrical wire is not a tube: it has no diameter and the status shows none.
        steps->push_back([state, check] {
            int rail = -1;
            int button = -1;
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
            }
            check(rail >= 0 && button >= 0,
                  QStringLiteral("tube: the catalog offers the control parts"));
            const int rail_index =
                rail >= 0 ? state->session->placeComponent(rail, 40.0, 110.0) : -1;
            const int button_index =
                button >= 0 ? state->session->placeComponent(button, 90.0, 110.0) : -1;
            const int before = static_cast<int>(state->session->connectionCount());
            check(state->session->connectPorts(rail_index, QStringLiteral("L"), button_index,
                                               QStringLiteral("1")),
                  QStringLiteral("tube: the wire can be made"));
            const int wire = before;
            check(state->session->connectionCount() == before + 1
                      && state->session->pipeTube(wire) == 0,
                  QStringLiteral("tube: an electrical wire has no diameter"));
            check(state->session->pipeIsElectrical(wire),
                  QStringLiteral("tube: the wire is recognised by its port roles"));
            state->session->selectPipe(wire);
            QObject *reading =
                state->window->findChild<QObject *>(QStringLiteral("pipeTubeReading"));
            check(reading != nullptr && !reading->property("visible").toBool(),
                  QStringLiteral("tube: a wire shows no size in the status bar"));
            state->session->selectPipe(-1);
            state->session->removeComponent(button_index);
            state->session->removeComponent(rail_index);
        });

        // The two terminations are files like everything else: they are placed, inspected
        // and removed, and the inspector says what they do. The palette no longer lists
        // them, because a closure belongs to a port, so their ABI index comes from the
        // session's catalog: it still carries them for an old sheet, and it is the same
        // index the palette row would have kept.
        steps->push_back([state, check] {
            int plug = -1;
            int silencer = -1;
            bool hidden = true;
            for (const QVariant &entry : state->session->catalog()) {
                const QString id =
                    entry.toMap().value(QStringLiteral("id")).toString();
                if (id == QStringLiteral("plug.blind")
                    || id == QStringLiteral("silencer.pneumatic")) {
                    hidden = false;
                }
            }
            PfCatalog catalog {};
            if (pf_session_catalog(state->session->handle(), &catalog) == PF_OK
                && catalog.items != nullptr && catalog.text != nullptr) {
                for (uint32_t slot = 0; slot < catalog.items_len; ++slot) {
                    const PfCatalogItem &item = catalog.items[slot];
                    const QString id = QString::fromUtf8(
                        reinterpret_cast<const char *>(catalog.text) + item.id_at,
                        static_cast<int>(item.id_len));
                    if (id == QStringLiteral("plug.blind")) {
                        plug = static_cast<int>(slot);
                    }
                    if (id == QStringLiteral("silencer.pneumatic")) {
                        silencer = static_cast<int>(slot);
                    }
                }
            }
            check(plug >= 0 && silencer >= 0,
                  QStringLiteral("terminations: the catalog offers both"));
            check(hidden,
                  QStringLiteral("terminations: the palette hides the two closures"));
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
            if (pf_session_build_frame(state->session->handle(), 4.0, 0.0, 0, &frame)
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

        // The view: Ctrl and the wheel magnify at the pointer, the wheel pans, the middle
        // button drags the sheet, the arrows pan from the keyboard, and Ctrl+0 returns to
        // 1:1. None of it is a document change, so none of it may enter the history.
        steps->push_back([state, check, toScene] {
            state->session->stopSimulation();
            state->session->zoomReset();
            state->viewScene = toScene(QPointF(150.0, 30.0));
            state->viewSheet = state->sheet->sheetPointForScene(state->viewScene);
            state->viewComponents = pf_session_component_count(state->session->handle());
            state->viewCanUndo = state->session->canUndo();
            std::fprintf(stderr, "selftest: view probe at (%.0f, %.0f), sheet (%.1f, %.1f)\n",
                         state->viewScene.x(), state->viewScene.y(), state->viewSheet.x(),
                         state->viewSheet.y());
            check(std::abs(state->viewSheet.x() - 150.0) < 0.5
                      && std::abs(state->viewSheet.y() - 30.0) < 0.5,
                  QStringLiteral("view: the probe is where the sheet says"));
        });
        steps->push_back([state, sendWheel] {
            sendWheel(state->viewScene, 120, Qt::ControlModifier);
        });
        steps->push_back([state, check] {
            const QPointF under = state->sheet->sheetPointForScene(state->viewScene);
            std::fprintf(stderr, "selftest: zoom %.2f, probe under (%.2f, %.2f)\n",
                         state->session->zoom(), under.x(), under.y());
            check(state->session->zoom() > 1.2,
                  QStringLiteral("view: Ctrl and the wheel magnify"));
            check((under - state->viewSheet).manhattanLength() < 0.5,
                  QStringLiteral("view: the point under the pointer stays under it"));
            check(pf_session_component_count(state->session->handle()) == state->viewComponents
                      && state->session->canUndo() == state->viewCanUndo,
                  QStringLiteral("view: zooming is not a document change"));
            state->viewSheet = under;
        });
        steps->push_back([state, sendWheel] {
            sendWheel(state->viewScene, -120, Qt::NoModifier);
        });
        steps->push_back([state, check] {
            const QPointF under = state->sheet->sheetPointForScene(state->viewScene);
            check(under.y() > state->viewSheet.y() + 0.1,
                  QStringLiteral("view: the wheel pans the sheet vertically"));
            state->viewSheet = under;
        });
        steps->push_back([state, check] {
            QKeyEvent key(QEvent::KeyPress, Qt::Key_Right, Qt::NoModifier);
            QCoreApplication::sendEvent(state->sheet, &key);
            const QPointF under = state->sheet->sheetPointForScene(state->viewScene);
            check(under.x() > state->viewSheet.x() + 0.1,
                  QStringLiteral("view: the arrow key pans the sheet"));
            state->viewSheet = under;
        });
        steps->push_back([state, send] {
            send(QEvent::MouseButtonPress, state->viewScene, Qt::MiddleButton, Qt::MiddleButton);
            send(QEvent::MouseMove, state->viewScene + QPointF(40.0, 0.0), Qt::NoButton,
                 Qt::MiddleButton);
            send(QEvent::MouseButtonRelease, state->viewScene + QPointF(40.0, 0.0),
                 Qt::MiddleButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            const QPointF under = state->sheet->sheetPointForScene(state->viewScene);
            check(under.x() < state->viewSheet.x() - 0.1,
                  QStringLiteral("view: the middle button drags the sheet"));
            check(pf_session_component_count(state->session->handle()) == state->viewComponents
                      && state->session->canUndo() == state->viewCanUndo,
                  QStringLiteral("view: panning is not a document change"));
        });
        steps->push_back([state, check] {
            state->session->zoomReset();
            const QPointF under = state->sheet->sheetPointForScene(state->viewScene);
            check(qFuzzyCompare(state->session->zoom(), 1.0)
                      && (under - QPointF(150.0, 30.0)).manhattanLength() < 0.5,
                  QStringLiteral("view: Ctrl+0 returns to 1:1 and drops the pan"));
        });

        // The middle double click fits the whole sheet, margins included, and one step back
        // returns to the view before it.
        steps->push_back([state, send, toScene] {
            state->session->zoomReset();
            state->session->zoomIn();
            state->viewZoom = state->session->zoom();
            state->viewScene = toScene(QPointF(148.0, 105.0));
            // Two clicks in place, the way a hand produces them: the item counts the
            // repeat itself, so a replayed sequence needs no platform double click.
            send(QEvent::MouseButtonPress, state->viewScene, Qt::MiddleButton, Qt::MiddleButton);
            send(QEvent::MouseButtonRelease, state->viewScene, Qt::MiddleButton, Qt::NoButton);
            send(QEvent::MouseButtonPress, state->viewScene, Qt::MiddleButton, Qt::MiddleButton);
            send(QEvent::MouseButtonRelease, state->viewScene, Qt::MiddleButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            const Fit fit = state->sheet->viewPlacement();
            const double paperWidth = state->session->sheetWidthMm() * fit.pixelsPerMm;
            const double paperHeight = state->session->sheetHeightMm() * fit.pixelsPerMm;
            const QPointF centre(fit.originX + paperWidth * 0.5,
                                 fit.originY + paperHeight * 0.5);
            std::fprintf(stderr, "selftest: fit zoom %.2f, paper %.0fx%.0f in %.0fx%.0f\n",
                         state->session->zoom(), paperWidth, paperHeight, state->sheet->width(),
                         state->sheet->height());
            check(qFuzzyCompare(state->session->zoom(), 1.0)
                      && !qFuzzyCompare(state->session->zoom(), state->viewZoom)
                      && paperWidth <= state->sheet->width() + 1.0
                      && paperHeight <= state->sheet->height() + 1.0
                      && std::abs(centre.x() - state->sheet->width() * 0.5) <= 1.0
                      && std::abs(centre.y() - state->sheet->height() * 0.5) <= 1.0,
                  QStringLiteral("view: the middle double click fits the sheet, edges included"));
        });
        steps->push_back([state, check] {
            state->sheet->previousView();
            check(qFuzzyCompare(state->session->zoom(), state->viewZoom),
                  QStringLiteral("view: one step goes back to the previous view"));
        });

        // The strip over the right edge drives the same operations, never a second copy of
        // them, and folding it is view state like the panels.
        steps->push_back([state, check] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            check(strip != nullptr,
                  QStringLiteral("view: the strip floats over the drawing area"));
            if (strip == nullptr) {
                return;
            }
            state->session->zoomReset();
            state->session->zoomIn();
            state->session->zoomIn();
            state->viewComponents = pf_session_component_count(state->session->handle());
            state->viewCanUndo = state->session->canUndo();
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("fit")));
        });
        steps->push_back([state, check] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            check(qFuzzyCompare(state->session->zoom(), 1.0)
                      && pf_session_component_count(state->session->handle())
                             == state->viewComponents
                      && state->session->canUndo() == state->viewCanUndo,
                  QStringLiteral("view: the strip fits the sheet without editing"));
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("zoom-in")));
        });
        steps->push_back([state, check] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            const double magnified = state->session->zoom();
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("back")));
            check(!qFuzzyCompare(state->session->zoom(), magnified)
                      && !state->sheet->canGoBack(),
                  QStringLiteral("view: the strip steps back to the previous view"));
            state->stripHeight = strip->property("height").toReal();
            state->window->setProperty("viewStripFolded", true);
        });
        steps->push_back([state, check] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            const qreal folded = strip->property("height").toReal();
            check(folded < state->stripHeight && folded > 0.0,
                  QStringLiteral("view: the strip folds to one button"));
            state->window->setProperty("viewStripFolded", false);
        });

        // The block's show toggle is view state, and the transport is one button: start,
        // then pause where it stands instead of stopping and losing the run.
        steps->push_back([state] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            state->panelModified = state->session->modified();
            state->panelCanUndo = state->session->canUndo();
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("block")));
        });
        steps->push_back([state, check] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            check(!state->session->blockVisible()
                      && state->session->modified() == state->panelModified
                      && state->session->canUndo() == state->panelCanUndo,
                  QStringLiteral("view: hiding the block changes nothing in the document"));
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("block")));
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("play")));
        });
        // Start and pause in one step, without an event loop between them: the solver does
        // not advance, so the gestures that follow see the opening model untouched.
        steps->push_back([state, check] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            // From editing, so the button starts whatever an earlier step left running.
            state->session->stopSimulation();
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("play")));
            const bool started = state->session->running();
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("play")));
            check(started && !state->session->running() && !state->session->editing(),
                  QStringLiteral("view: the same button starts and pauses where it stands"));
            state->session->stopSimulation();
        });

        // The hand mode: a visible pointer mode that moves the sheet with the left button,
        // marks itself in the strip, leaves with Esc and never edits.
        steps->push_back([state, check, toScene] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            // A sheet that fits has nothing to pan, so the drag is proved magnified.
            state->session->setZoom(1.6);
            state->viewScene = toScene(QPointF(150.0, 100.0));
            state->viewSheet = state->sheet->sheetPointForScene(state->viewScene);
            state->viewComponents = pf_session_component_count(state->session->handle());
            state->viewCanUndo = state->session->canUndo();
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("hand")));
            check(state->session->handMode(),
                  QStringLiteral("view: the strip turns the hand on"));
        });
        steps->push_back([state, send] {
            send(QEvent::MouseButtonPress, state->viewScene, Qt::LeftButton, Qt::LeftButton);
            send(QEvent::MouseMove, state->viewScene + QPointF(40.0, 0.0), Qt::NoButton,
                 Qt::LeftButton);
            send(QEvent::MouseButtonRelease, state->viewScene + QPointF(40.0, 0.0),
                 Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            const QPointF under = state->sheet->sheetPointForScene(state->viewScene);
            check(under.x() < state->viewSheet.x() - 0.5
                      && pf_session_component_count(state->session->handle())
                             == state->viewComponents
                      && state->session->canUndo() == state->viewCanUndo,
                  QStringLiteral("view: the hand drags the sheet without editing"));
            QKeyEvent escape(QEvent::KeyPress, Qt::Key_Escape, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &escape);
        });
        // The strip at the foot: renaming in the tab is a document edit with its undo, and
        // the plus adds a circuit that becomes the one in view.
        steps->push_back([state, check] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("circuitTabs"));
            check(strip != nullptr,
                  QStringLiteral("circuits: the strip is at the foot of the sheet"));
            if (strip == nullptr) {
                return;
            }
            const int id = state->session->activeCircuit();
            strip->setProperty("editing", id);
            strip->setProperty("draft", QStringLiteral("Mando"));
            QMetaObject::invokeMethod(strip, "commitRename");
            bool named = false;
            for (const QVariant &entry : state->session->circuits()) {
                const QVariantMap map = entry.toMap();
                if (map.value(QStringLiteral("id")).toInt() == id) {
                    named = map.value(QStringLiteral("name")).toString()
                            == QStringLiteral("Mando");
                }
            }
            check(named && state->session->canUndo(),
                  QStringLiteral("circuits: the tab renames the circuit with undo"));
            state->session->undo();
            const int added = state->session->addCircuit();
            check(added != id && state->session->activeCircuit() == added,
                  QStringLiteral("circuits: the plus adds a circuit and shows it"));
            // The gestures that follow work on the circuit the test started with.
            state->session->undo();
            state->session->selectCircuit(id);
        });
        // Closing circuits from the strip: empty closes immediately; populated asks,
        // names the circuit and can be cancelled; one undo returns all of it.
        steps->push_back([state, check] {
            QObject *window = state->window;
            QObject *tabs = window->findChild<QObject *>(QStringLiteral("circuitTabs"));
            const int before = state->session->circuits().size();
            // Closing the tab in view hands over to the right neighbour, or to the left one
            // when it was the last: the new circuit is added last, so it is the left one.
            const int left = state->session->circuits().last().toMap().value("id").toInt();
            const int empty = state->session->addCircuit();
            QVariant id = empty;
            QVariant name = QStringLiteral("Vacío");
            QVariant count = 0;
            QVariant removed;
            const bool invoked = QMetaObject::invokeMethod(
                window, "requestCircuitRemoval", Q_RETURN_ARG(QVariant, removed),
                Q_ARG(QVariant, id), Q_ARG(QVariant, name), Q_ARG(QVariant, count));
            check(invoked && removed.toBool()
                      && state->session->circuits().size() == before
                      && state->session->activeCircuit() == left
                      && window->property("pendingCircuitRemoval").toInt() < 0,
                  QStringLiteral("circuits: an empty tab closes without asking"));
            state->session->undo();
            check(state->session->circuits().size() == before + 1
                      && state->session->activeCircuit() == empty,
                  QStringLiteral("circuits: undo returns an empty closed tab"));
            state->session->undo();
            check(state->session->circuits().size() == before,
                  QStringLiteral("circuits: undo removes the added empty tab"));
            check(tabs->property("canCloseCircuit").toBool()
                      == (state->session->circuits().size() > 1),
                  QStringLiteral("circuits: close control visibility follows the last-tab rule"));
        });
        steps->push_back([state, check] {
            QObject *window = state->window;
            const int before = state->session->circuits().size();
            const int first = state->session->activeCircuit();
            const int components =
                static_cast<int>(pf_session_component_count(state->session->handle()));
            check(components > 0, QStringLiteral("circuits: the active test circuit has work"));
            const int neighbour = state->session->addCircuit();
            check(neighbour >= 0, QStringLiteral("circuits: a right neighbour is added"));
            state->session->selectCircuit(first);
            const QVariant id = first;
            const QVariant name = state->session->circuits().first().toMap().value("name");
            const QVariant count = components;
            QMetaObject::invokeMethod(window, "requestCircuitRemoval", Q_ARG(QVariant, id),
                                      Q_ARG(QVariant, name), Q_ARG(QVariant, count));
            check(window->property("pendingCircuitRemoval").toInt() == first
                      && state->session->circuits().size() == before + 1,
                  QStringLiteral("circuits: a populated tab asks before closing"));
            QMetaObject::invokeMethod(window, "cancelCircuitRemoval");
            check(state->session->circuits().size() == before + 1
                      && state->session->activeCircuit() == first,
                  QStringLiteral("circuits: cancelling preserves the tab and its contents"));
            QMetaObject::invokeMethod(window, "requestCircuitRemoval", Q_ARG(QVariant, id),
                                      Q_ARG(QVariant, name), Q_ARG(QVariant, count));
            QVariant removed;
            const QVariantList ordered = state->session->circuits();
            int firstIndex = -1;
            for (int i = 0; i < ordered.size(); ++i) {
                if (ordered.at(i).toMap().value("id").toInt() == first) {
                    firstIndex = i;
                }
            }
            const int expectedNext = firstIndex + 1 < ordered.size()
                                         ? ordered.at(firstIndex + 1).toMap().value("id").toInt()
                                         : ordered.at(firstIndex - 1).toMap().value("id").toInt();
            QMetaObject::invokeMethod(window, "confirmCircuitRemoval", Q_RETURN_ARG(QVariant, removed));
            check(removed.toBool() && state->session->circuits().size() == before
                      && state->session->activeCircuit() == expectedNext,
                  QStringLiteral("circuits: confirming selects the right neighbour"));
            state->session->undo();
            check(state->session->circuits().size() == before + 1
                      && state->session->activeCircuit() == first
                      && static_cast<int>(pf_session_component_count(state->session->handle()))
                             == components,
                  QStringLiteral("circuits: one undo restores a closed populated tab"));
            state->session->undo();
            check(state->session->circuits().size() == before
                      && state->session->activeCircuit() == first,
                  QStringLiteral("circuits: undo removes the added neighbour"));
        });
        steps->push_back([state, check] {
            const int original = state->session->circuits().size();
            int removed = 0;
            while (state->session->circuits().size() > 1) {
                const QVariantList circuits = state->session->circuits();
                const int id = circuits.last().toMap().value("id").toInt();
                if (!state->session->removeCircuit(id)) {
                    break;
                }
                ++removed;
            }
            QObject *tabs = state->window->findChild<QObject *>(QStringLiteral("circuitTabs"));
            check(state->session->circuits().size() == 1 && tabs != nullptr
                      && !tabs->property("canCloseCircuit").toBool(),
                  QStringLiteral("circuits: the last tab has no close control"));
            for (int i = 0; i < removed; ++i) {
                state->session->undo();
            }
            check(state->session->circuits().size() == original,
                  QStringLiteral("circuits: undo restores the tabs after the last-tab check"));
        });
        steps->push_back([state, check] {
            const int before = state->session->circuits().size();
            const int limit = state->session->circuitLimit();
            while (state->session->circuits().size() < limit) {
                if (state->session->addCircuit() < 0) {
                    break;
                }
            }
            QObject *tabs = state->window->findChild<QObject *>(QStringLiteral("circuitTabs"));
            QObject *plus = tabs ? tabs->findChild<QObject *>(QStringLiteral("addCircuitButton"))
                                 : nullptr;
            const bool capped = state->session->circuits().size() == limit && plus != nullptr
                                && !plus->property("enabled").toBool();
            check(capped, QStringLiteral("circuits: plus disables at the project limit"));
            while (state->session->circuits().size() > before) {
                state->session->undo();
            }
        });

        // The strip is the list. The ribbon's Circuito group is three commands and does not
        // grow with the project, and its Cerrar reaches the same removal the strip's x does.
        steps->push_back([state, check] {
            const int before = state->session->circuits().size();
            check(state->session->addCircuit() >= 0,
                  QStringLiteral("ribbon: a circuit opens from its command"));
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            const QVariantList tabs = ribbon ? ribbon->property("tabs").toList() : QVariantList();
            const QVariantList groups = tabs.value(0).toMap().value("groups").toList();
            const QVariantList items = groups.value(1).toMap().value("items").toList();
            bool listsCircuits = false;
            for (const QVariant &item : items) {
                if (item.toMap().value("id").toString().startsWith(QStringLiteral("circuit:"))) {
                    listsCircuits = true;
                }
            }
            check(tabs.size() == 5 && groups.size() >= 2 && items.size() == 3 && !listsCircuits,
                  QStringLiteral("ribbon: the Circuito group is commands, not the list"));
            state->session->undo();
            check(state->session->circuits().size() == before,
                  QStringLiteral("ribbon: the added circuit leaves with the undo"));
        });

        // The ribbon folds at 1280 (P-027): the tab row is what is on screen and the menu
        // carries the Conducto group, including the P-024a Diámetro, instead of clipping it
        // past the right edge. At 1440 the unfolded row is back. The window's size and the
        // remembered fold are restored for the gestures that follow.
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            check(ribbon != nullptr && ribbon->property("collapsed").toBool()
                      && !ribbon->property("bodyShown").toBool(),
                  QStringLiteral("ribbon: 1280 folds to the tab row"));
            if (ribbon == nullptr) {
                return;
            }
            state->ribbonMinimized = state->window->property("ribbonMinimized").toBool();
            ribbon->setProperty("menuOpen", true);
        });
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            if (ribbon == nullptr) {
                check(false, QStringLiteral("ribbon: Conducto and Diámetro reachable at 1280"));
                return;
            }
            QVariant flat;
            const bool listed = QMetaObject::invokeMethod(ribbon, "flatActions",
                                                          Q_RETURN_ARG(QVariant, flat));
            bool tube = false;
            bool diameter = false;
            const QVariantList actions = listed ? flat.toList() : QVariantList();
            for (const QVariant &action : actions) {
                const QString id = action.toMap().value(QStringLiteral("id")).toString();
                tube = tube || id == QLatin1String("pipe-working");
                diameter = diameter || id == QLatin1String("tube-diameter");
            }
            QObject *control = ribbon->property("tubeControl").value<QObject *>();
            check(listed && tube && diameter && control != nullptr,
                  QStringLiteral("ribbon: Conducto and Diámetro reachable at 1280"));
            ribbon->setProperty("menuOpen", false);
            state->window->setProperty("ribbonMinimized", false);
            state->window->resize(1440, 900);
        });
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            check(ribbon != nullptr && !ribbon->property("collapsed").toBool()
                      && ribbon->property("bodyShown").toBool(),
                  QStringLiteral("ribbon: 1440 shows the unfolded row"));
            state->window->resize(1280, 800);
            state->window->setProperty("ribbonMinimized", state->ribbonMinimized);
        });

        // The support floor (P-028): at 1366 the ribbon is one row with the seven groups and
        // the measured collapse set, and a real Enter on a collapsed group's titled button
        // opens its flyout. At 1920 every group is expanded; at 1280 the P-027 fold is still
        // the fallback. The window's size and remembered fold are restored afterwards.
        steps->push_back([state, check] {
            state->ribbonMinimized = state->window->property("ribbonMinimized").toBool();
            state->window->resize(1366, 768);
            check(state->window->width() == 1366 && state->window->height() == 768,
                  QStringLiteral("ribbon: the window reaches 1366x768"));
        });
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            check(ribbon != nullptr && !ribbon->property("collapsed").toBool()
                      && ribbon->property("bodyShown").toBool(),
                  QStringLiteral("ribbon: 1366 keeps the one-row ribbon"));
            QQuickItem *groupsRow =
                state->window->findChild<QQuickItem *>(QStringLiteral("ribbonGroupsRow"));
            QStringList titles;
            QStringList collapsedTitles;
            QQuickItem *alinear = nullptr;
            if (groupsRow != nullptr) {
                for (QQuickItem *group : groupsRow->childItems()) {
                    if (group->objectName() != QLatin1String("ribbonGroup")
                        || !group->isVisible()) {
                        continue;
                    }
                    const QString title = group->property("title").toString();
                    titles.append(title);
                    if (group->property("collapsed").toBool()) {
                        collapsedTitles.append(title);
                    }
                    if (title == QLatin1String("Alinear")) {
                        alinear = group;
                    }
                }
            }
            const QStringList order = {
                QStringLiteral("Historial"), QStringLiteral("Circuito"),
                QStringLiteral("Componente"), QStringLiteral("Alinear"),
                QStringLiteral("Conducto"), QStringLiteral("Repartir"),
                QStringLiteral("Paleta"),
            };
            const QStringList collapsed = {
                QStringLiteral("Historial"), QStringLiteral("Alinear"),
                QStringLiteral("Repartir"),
            };
            check(ribbon != nullptr && titles == order,
                  QStringLiteral("ribbon: 1366 shows the seven groups"));
            check(collapsedTitles == collapsed,
                  QStringLiteral("ribbon: 1366 collapses Historial, Alinear and Repartir"));
            QQuickItem *button =
                alinear != nullptr
                    ? alinear->findChild<QQuickItem *>(QStringLiteral("ribbonGroupButton"))
                    : nullptr;
            if (button != nullptr && button->isVisible()) {
                button->forceActiveFocus();
            }
            check(button != nullptr && button->isVisible() && button->hasActiveFocus(),
                  QStringLiteral("ribbon: 1366 focuses Alinear's group button"));
        });
        steps->push_back([state, check] {
            QQuickItem *groupsRow =
                state->window->findChild<QQuickItem *>(QStringLiteral("ribbonGroupsRow"));
            QQuickItem *alinear = nullptr;
            if (groupsRow != nullptr) {
                for (QQuickItem *group : groupsRow->childItems()) {
                    if (group->objectName() == QLatin1String("ribbonGroup")
                        && group->property("title").toString() == QLatin1String("Alinear")) {
                        alinear = group;
                    }
                }
            }
            QKeyEvent key(QEvent::KeyPress, Qt::Key_Return, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &key);
            QVariant commands;
            const bool counted = alinear != nullptr
                                 && QMetaObject::invokeMethod(
                                     alinear, "flyoutCommandCount",
                                     Q_RETURN_ARG(QVariant, commands));
            check(alinear != nullptr && alinear->property("flyoutOpen").toBool() && counted
                      && commands.toInt() > 0,
                  QStringLiteral("ribbon: Enter opens Alinear's flyout with its commands"));
            if (alinear != nullptr) {
                QMetaObject::invokeMethod(alinear, "closeFlyout");
            }
            state->window->resize(1920, 1080);
        });
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            QQuickItem *groupsRow =
                state->window->findChild<QQuickItem *>(QStringLiteral("ribbonGroupsRow"));
            int groups = 0;
            int collapsed = 0;
            if (groupsRow != nullptr) {
                for (QQuickItem *group : groupsRow->childItems()) {
                    if (group->objectName() != QLatin1String("ribbonGroup")
                        || !group->isVisible()) {
                        continue;
                    }
                    ++groups;
                    if (group->property("collapsed").toBool()) {
                        ++collapsed;
                    }
                }
            }
            check(ribbon != nullptr && !ribbon->property("collapsed").toBool()
                      && groups == 7 && collapsed == 0,
                  QStringLiteral("ribbon: 1920 expands all seven groups"));
            state->window->resize(1280, 800);
        });
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            check(ribbon != nullptr && ribbon->property("collapsed").toBool()
                      && !ribbon->property("bodyShown").toBool(),
                  QStringLiteral("ribbon: 1280 still folds to the tab row"));
            state->window->setProperty("ribbonMinimized", state->ribbonMinimized);
        });

        // The first window (P-028 §1b): the size is computed from the screen area, not from
        // the machine's own screen, so these probes feed synthetic areas and the unit the
        // rule names. 1366 opens above the 1280 fold; 1280 is the recorded folded case; a
        // remembered 1920x1080 is clamped to the 1366 area with the same margin.
        const auto defaultSize = [state](int areaWidth, int areaHeight, int unit) {
            QVariant result;
            const bool called = QMetaObject::invokeMethod(
                state->window, "defaultWindowSize", Q_RETURN_ARG(QVariant, result),
                Q_ARG(QVariant, areaWidth), Q_ARG(QVariant, areaHeight),
                Q_ARG(QVariant, unit));
            return called ? result.toMap() : QVariantMap();
        };
        steps->push_back([check, defaultSize] {
            const QVariantMap size = defaultSize(1920, 1080, 18);
            check(size.value(QStringLiteral("width")).toInt() == 1440
                      && size.value(QStringLiteral("height")).toInt() == 900,
                  QStringLiteral("window: 1920x1080 gives 1440x900"));
        });
        steps->push_back([check, defaultSize] {
            const QVariantMap size = defaultSize(1366, 768, 18);
            check(size.value(QStringLiteral("width")).toInt() == 1330
                      && size.value(QStringLiteral("width")).toInt() > 1280
                      && size.value(QStringLiteral("height")).toInt() == 732,
                  QStringLiteral("window: 1366x768 fits above the fold"));
        });
        steps->push_back([check, defaultSize] {
            const QVariantMap size = defaultSize(1280, 800, 18);
            check(size.value(QStringLiteral("width")).toInt() == 1244
                      && size.value(QStringLiteral("width")).toInt() < 1280
                      && size.value(QStringLiteral("height")).toInt() == 764,
                  QStringLiteral("window: 1280x800 is the folded pair"));
        });
        steps->push_back([state, check] {
            QVariant result;
            const bool called = QMetaObject::invokeMethod(
                state->window, "clampWindowSize", Q_RETURN_ARG(QVariant, result),
                Q_ARG(QVariant, 1920), Q_ARG(QVariant, 1080),
                Q_ARG(QVariant, 1366), Q_ARG(QVariant, 768), Q_ARG(QVariant, 18));
            const QVariantMap size = result.toMap();
            check(called && size.value(QStringLiteral("width")).toInt() == 1330
                      && size.value(QStringLiteral("height")).toInt() == 732,
                  QStringLiteral("window: restored 1920x1080 fits 1366x768"));
        });

        steps->push_back([state, check] {
            check(!state->session->handMode(),
                  QStringLiteral("view: Esc leaves the hand mode"));
            // Back to the opening view, so the gestures that follow see the same sheet the
            // earlier steps did.
            state->session->zoomReset();
        });

        // The zoom window: the panel arms it, a drag chooses the rectangle, and the chosen
        // centre lands on the canvas centre. A stray click or Esc leaves the view alone.
        steps->push_back([state, check, send, toScene] {
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            state->session->zoomReset();
            state->viewZoom = state->session->zoom();
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("window")));
            check(state->sheet->zoomWindowActive(),
                  QStringLiteral("view: the strip arms the zoom window"));
            const QPointF from = toScene(QPointF(120.0, 80.0));
            const QPointF to = toScene(QPointF(180.0, 130.0));
            send(QEvent::MouseButtonPress, from, Qt::LeftButton, Qt::LeftButton);
            send(QEvent::MouseMove, to, Qt::NoButton, Qt::LeftButton);
            send(QEvent::MouseButtonRelease, to, Qt::LeftButton, Qt::NoButton);
        });
        steps->push_back([state, check] {
            const Fit fit = state->sheet->viewPlacement();
            const QPointF centre(fit.originX + 150.0 * fit.pixelsPerMm,
                                 fit.originY + 105.0 * fit.pixelsPerMm);
            check(!state->sheet->zoomWindowActive()
                      && state->session->zoom() > state->viewZoom,
                  QStringLiteral("view: the zoom window magnifies the rectangle"));
            check(std::abs(centre.x() - state->sheet->width() * 0.5) <= 1.5
                      && std::abs(centre.y() - state->sheet->height() * 0.5) <= 1.5,
                  QStringLiteral("view: the rectangle lands centred"));
            QObject *strip = state->window->findChild<QObject *>(QStringLiteral("viewStrip"));
            QMetaObject::invokeMethod(strip, "invoke", Q_ARG(QVariant, QStringLiteral("window")));
            const double before = state->session->zoom();
            QKeyEvent escape(QEvent::KeyPress, Qt::Key_Escape, Qt::NoModifier);
            QCoreApplication::sendEvent(state->sheet, &escape);
            check(!state->sheet->zoomWindowActive()
                      && qFuzzyCompare(state->session->zoom(), before),
                  QStringLiteral("view: Esc cancels the zoom window"));
            state->session->zoomReset();
        });

        // The sheet is the paper: choosing A3 changes the document, the canvas and the
        // check together, and every choice is one undo.
        steps->push_back([state, check] {
            // Nothing changed the sheet before this step: this is the opening document.
            check(state->session->sheetPaper() == 1 && state->session->sheetLandscape(),
                  QStringLiteral("sheet: the document opens A4 landscape"));
            check(std::abs(state->session->sheetWidthMm() - 297.0) < 0.01
                      && std::abs(state->session->sheetHeightMm() - 210.0) < 0.01,
                  QStringLiteral("sheet: A4 landscape is 297 x 210 mm"));
            check(std::abs(state->session->sheetMarginLeft() - 20.0) < 0.01
                      && std::abs(state->session->sheetMarginTop() - 10.0) < 0.01,
                  QStringLiteral("sheet: the frame is the ISO 5457 one"));
            // A4 normal is still portrait, one click away from the opening sheet.
            state->session->resetSheet();
        });
        steps->push_back([state, check] {
            check(state->session->sheetPaper() == 1 && !state->session->sheetLandscape(),
                  QStringLiteral("sheet: A4 normal returns to portrait"));
            check(std::abs(state->session->sheetWidthMm() - 210.0) < 0.01
                      && std::abs(state->session->sheetHeightMm() - 297.0) < 0.01,
                  QStringLiteral("sheet: A4 portrait is 210 x 297 mm"));
            state->session->setSheetPaper(2);
        });
        steps->push_back([state, check] {
            check(std::abs(state->session->sheetWidthMm() - 297.0) < 0.01
                      && std::abs(state->session->sheetHeightMm() - 420.0) < 0.01,
                  QStringLiteral("sheet: A3 keeps the portrait orientation"));
            state->session->toggleSheetLandscape();
        });
        steps->push_back([state, check] {
            check(std::abs(state->session->sheetWidthMm() - 420.0) < 0.01
                      && std::abs(state->session->sheetHeightMm() - 297.0) < 0.01,
                  QStringLiteral("sheet: landscape swaps the sides"));
            state->session->undo();
            state->session->undo();
            state->session->undo();
        });
        steps->push_back([state, check] {
            check(state->session->sheetPaper() == 1 && state->session->sheetLandscape(),
                  QStringLiteral("sheet: undo returns the opening A4 landscape"));
            state->session->setSheetMargin(0, 15.0);
        });
        steps->push_back([state, check] {
            check(std::abs(state->session->sheetMarginTop() - 15.0) < 0.01,
                  QStringLiteral("sheet: a frame margin is a document change"));
            state->session->undo();
            check(std::abs(state->session->sheetMarginTop() - 10.0) < 0.01,
                  QStringLiteral("sheet: undo returns the ISO margin"));
        });

        // The reference zones read from the same helpers the canvas draws from: the axes
        // are not swapped (letters name rows), I and O are skipped, and A4 is two-sided.
        steps->push_back([check] {
            check(sheetZoneBands(QStringLiteral("A3"), 420.0, 297.0) == ZoneBands::FourSides
                      && sheetRowLabel(0) == QStringLiteral("A")
                      && sheetRowLabel(7) == QStringLiteral("H")
                      && sheetRowLabel(8) == QStringLiteral("J")
                      && sheetRowLabel(13) == QStringLiteral("P")
                      && sheetRowLabel(16) == QStringLiteral("S")
                      && sheetRowLabel(8) != QStringLiteral("I")
                      && sheetRowLabel(13) != QStringLiteral("O")
                      && sheetColumnLabel(0) == QStringLiteral("1"),
                  QStringLiteral(
                      "zones: A3 names the rows with letters and the columns with numerals"));
        });
        steps->push_back([check] {
            check(sheetZoneBands(QStringLiteral("A4"), 297.0, 210.0) == ZoneBands::A4
                      && sheetZoneBands(QStringLiteral("A4"), 210.0, 297.0) == ZoneBands::A4
                      && sheetZoneBands(QStringLiteral("Letter"), 279.4, 215.9)
                          == ZoneBands::None
                      && sheetZoneBands(QStringLiteral("Personalizado"), 297.0, 420.0)
                          == ZoneBands::FourSides,
                  QStringLiteral("zones: A4 uses only the top and right bands"));
        });

        // The page controls keep the common name beside the normalised one, and the
        // sheet's own name stays canonical for the status bar and the file.
        steps->push_back([state, check] {
            const QVariantList papers = state->session->papers();
            check(papers.size() == 9,
                  QStringLiteral("page setup: the core lists nine papers"));
            QString letter;
            QString legal;
            QString oficio;
            for (const QVariant &entry : papers) {
                const QVariantMap row = entry.toMap();
                const QString name = row.value(QStringLiteral("name")).toString();
                if (name == QStringLiteral("Letter")) {
                    letter = row.value(QStringLiteral("label")).toString();
                } else if (name == QStringLiteral("Legal")) {
                    legal = row.value(QStringLiteral("label")).toString();
                } else if (name == QStringLiteral("Oficio")) {
                    oficio = row.value(QStringLiteral("label")).toString();
                }
            }
            check(letter == QStringLiteral("Letter (Carta)")
                      && legal == QStringLiteral("Legal (Oficio)")
                      && oficio == QStringLiteral("Oficio chileno (Folio)"),
                  QStringLiteral("page setup: the common names sit in parentheses"));
            check(state->session->formatPaperSize(215.9, 279.4)
                      == QStringLiteral("21,59 × 27,94 cm"),
                  QStringLiteral("page setup: a paper is measured in centimetres with a comma"));
            check(state->session->formatPaperSize(210.0, 297.0)
                      == QStringLiteral("21 × 29,7 cm"),
                  QStringLiteral("page setup: trailing zeros do not clutter the measure"));
            check(state->session->sheetPaperName() == QStringLiteral("A4"),
                  QStringLiteral("page setup: the status bar keeps the canonical name"));
        });

        // A custom paper is the document's, not a print override: saved like any sheet and
        // one undo away.
        steps->push_back([state, check] {
            state->session->setSheetCustomPaper(180.0, 250.0);
            check(state->session->sheetPaper() == -1
                      && std::abs(state->session->sheetPaperWidthMm() - 180.0) < 0.01
                      && std::abs(state->session->sheetPaperHeightMm() - 250.0) < 0.01,
                  QStringLiteral("page setup: a custom paper is the document's"));
            check(state->session->sheetPaperDisplay() == QStringLiteral("Personalizado"),
                  QStringLiteral("page setup: the control names the custom paper"));
            state->session->undo();
            check(state->session->sheetPaper() == 1 && state->session->sheetLandscape(),
                  QStringLiteral("page setup: one undo returns the opening sheet"));
        });

        // Margin presets are one change each: four margins, one undo.
        steps->push_back([state, check] {
            state->session->applyMarginPreset(1);
            check(state->session->sheetMarginPreset() == 1
                      && std::abs(state->session->sheetMarginTop() - 10.0) < 0.01
                      && std::abs(state->session->sheetMarginLeft() - 10.0) < 0.01,
                  QStringLiteral("page setup: Estrecho applies four margins at once"));
            state->session->undo();
            check(state->session->sheetMarginPreset() == 0,
                  QStringLiteral("page setup: one undo returns the ISO frame"));
            state->session->applyMarginPreset(2);
            check(state->session->sheetMarginPreset() == 2
                      && std::abs(state->session->sheetMarginLeft() - 20.0) < 0.01,
                  QStringLiteral("page setup: Amplio applies the wide frame"));
            state->session->undo();
            check(state->session->sheetMarginPreset() == 0,
                  QStringLiteral("page setup: the ISO frame comes back"));
            check(std::abs(state->session->paperMinMm() - 50.0) < 0.01
                      && std::abs(state->session->paperMaxMm() - 2000.0) < 0.01,
                  QStringLiteral("page setup: the custom bounds come from the core"));
        });

        // A custom size outside the bounds is refused and leaves the sheet alone.
        steps->push_back([state, check] {
            state->session->setSheetCustomPaper(10.0, 10.0);
            check(state->session->sheetPaper() == 1,
                  QStringLiteral("page setup: an impossible custom paper is refused"));
            check(!state->session->lastError().isEmpty(),
                  QStringLiteral("page setup: the refusal explains itself"));
            state->session->setSheetPaper(1);
        });

        // The block protects its own rectangle, not a band: no example box may sit under
        // it, and the strip beside it is drawing space.
        steps->push_back([state, check, componentRect] {
            PfBlockLayout block {};
            const bool hasBlock =
                state->session->blockPreset() != 3
                && pf_session_block_layout(state->session->handle(), &block) == PF_OK;
            const double frameLeft = state->session->sheetMarginLeft();
            const double frameRight = state->session->sheetWidthMm()
                - state->session->sheetMarginRight();
            const double frameTop = state->session->sheetMarginTop();
            const double frameBottom = state->session->sheetHeightMm()
                - state->session->sheetMarginBottom();
            const int preset = state->session->blockPreset();
            const double width = preset == 2 ? frameRight - frameLeft
                                             : std::min<double>(block.width_mm,
                                                                frameRight - frameLeft);
            const double height = std::min<double>(block.height_mm, frameBottom - frameTop);
            const double left = preset == 2 ? frameLeft : frameRight - width;
            const double top = preset == 0 ? frameTop : frameBottom - height;
            const QRectF protectedArea(left, top, width, height);
            bool clear = true;
            for (int index = 0; index < state->session->componentCount(); ++index) {
                clear = clear && !componentRect(index).intersects(protectedArea);
            }
            check(!hasBlock || clear,
                  QStringLiteral("title block: the example stays clear of the block"));
        });

        // A drop in the lower left, below the band the old rule reserved, is drawing space:
        // the block does not own that column (ADR 0030).
        steps->push_back([state, check, componentRect] {
            int compressor = -1;
            const QVariantList entries = state->session->catalog();
            for (const QVariant &entry : entries) {
                const QVariantMap row = entry.toMap();
                if (row.value(QStringLiteral("id")).toString()
                    == QStringLiteral("supply.compressor")) {
                    compressor = row.value(QStringLiteral("index")).toInt();
                }
            }
            PfBlockLayout block {};
            const bool hasBlock =
                state->session->blockPreset() != 3
                && pf_session_block_layout(state->session->handle(), &block) == PF_OK;
            const double frameBottom = state->session->sheetHeightMm()
                - state->session->sheetMarginBottom();
            const double blockTop = frameBottom - block.height_mm;
            const double blockLeft =
                state->session->sheetWidthMm() - state->session->sheetMarginRight()
                - block.width_mm;
            const int before = state->session->componentCount();
            const int placed = compressor >= 0
                                   ? state->session->placeComponent(compressor, 60.0, 175.0)
                                   : -1;
            const QRectF box = placed >= 0 ? componentRect(placed) : QRectF();
            check(!hasBlock
                      || (placed >= 0 && box.top() > blockTop && box.right() < blockLeft),
                  QStringLiteral("title block: the strip beside it takes a drop"));
            while (state->session->componentCount() > before) {
                state->session->undo();
            }
        });

        // The port magnet: a hose between two ports on the same axis becomes straight when
        // the dragged component's port comes within reach, even though the grid could never
        // land on the 8.02 mm offset. Then the same move from the keyboard.
        steps->push_back([state, check, send, toScene, componentRect] {
            auto componentNamed = [state](const QString &prefix) {
                const uint32_t count = pf_session_component_count(state->session->handle());
                for (uint32_t index = 0; index < count; ++index) {
                    char id[128] = {};
                    if (pf_session_component_id(state->session->handle(), index, id, sizeof id) >= 0
                        && QString::fromUtf8(id).startsWith(prefix)) {
                        return static_cast<int>(index);
                    }
                }
                return -1;
            };
            const int valve = componentNamed(QStringLiteral("valve."));
            const int cylinder = componentNamed(QStringLiteral("cylinder."));
            check(valve >= 0 && cylinder >= 0,
                  QStringLiteral("magnet: the example has a valve and a cylinder"));
            if (valve < 0 || cylinder < 0) {
                return;
            }
            QPointF valvePort;
            QPointF cylinderPort;
            check(state->session->portPoint(valve, QStringLiteral("2"), &valvePort)
                      && state->session->portPoint(cylinder, QStringLiteral("1"), &cylinderPort),
                  QStringLiteral("magnet: both ports are found"));
            const QPointF centre = componentRect(cylinder).center();
            const QPointF target(
                valvePort.x() - (cylinderPort.x() - centre.x()) + 2.0, centre.y());
            send(QEvent::MouseButtonPress, toScene(centre), Qt::LeftButton, Qt::LeftButton);
            send(QEvent::MouseMove, toScene(target), Qt::NoButton, Qt::LeftButton);
            send(QEvent::MouseButtonRelease, toScene(target), Qt::LeftButton, Qt::NoButton);
            QPointF moved;
            state->session->portPoint(cylinder, QStringLiteral("1"), &moved);
            std::fprintf(stderr, "selftest: magnet port x %.2f against %.2f (hand %d, zoom %.2f)\n",
                         moved.x(), valvePort.x(), state->session->handMode() ? 1 : 0,
                         state->session->zoom());
            check(std::abs(moved.x() - valvePort.x()) < 0.05,
                  QStringLiteral("magnet: the dragged port lands on the other end's axis"));
            state->session->undo();
        });
        steps->push_back([state, check] {
            int valve = -1;
            int cylinder = -1;
            const uint32_t count = pf_session_component_count(state->session->handle());
            for (uint32_t index = 0; index < count; ++index) {
                char id[128] = {};
                if (pf_session_component_id(state->session->handle(), index, id, sizeof id) < 0) {
                    continue;
                }
                const QString text = QString::fromUtf8(id);
                if (valve < 0 && text.startsWith(QStringLiteral("valve."))) {
                    valve = static_cast<int>(index);
                }
                if (cylinder < 0 && text.startsWith(QStringLiteral("cylinder."))) {
                    cylinder = static_cast<int>(index);
                }
            }
            QPointF valvePort;
            QPointF cylinderPort;
            if (valve < 0 || cylinder < 0
                || !state->session->portPoint(valve, QStringLiteral("2"), &valvePort)
                || !state->session->portPoint(cylinder, QStringLiteral("1"), &cylinderPort)) {
                check(false, QStringLiteral("magnet: the hose can be selected"));
                return;
            }
            const double mid = (valvePort.y() + cylinderPort.y()) * 0.5;
            const int pipe = state->session->pipeAt(valvePort.x(), mid, 2.0);
            check(pipe >= 0, QStringLiteral("magnet: the hose is under the point"));
            if (pipe < 0) {
                return;
            }
            state->session->selectPipe(pipe);
            state->session->straightenSelectedPipe();
            QPointF moved;
            state->session->portPoint(cylinder, QStringLiteral("1"), &moved);
            check(std::abs(moved.x() - valvePort.x()) < 0.05,
                  QStringLiteral("magnet: Alt+A straightens the selected hose"));
            state->session->undo();
        });

        // The two help links are text in the tab row, not tabs. Opening each popup proves
        // the link reaches the window, that the version on screen is the core's own and
        // that the licence page is the GPL text compiled into the binary, not a file the
        // machine happens to have.
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            if (ribbon == nullptr) {
                check(false, QStringLiteral("help: the ribbon is in the scene"));
                return;
            }
            QMetaObject::invokeMethod(ribbon, "invoke", Q_ARG(QVariant, QStringLiteral("about")));
            check(state->window->property("aboutOpen").toBool()
                      && !state->window->property("licenseOpen").toBool(),
                  QStringLiteral("help: Acerca de opens its popup"));
        });
        steps->push_back([state, check] {
            QObject *about = state->window->findChild<QObject *>(QStringLiteral("aboutPanel"));
            QObject *version = state->window->findChild<QObject *>(QStringLiteral("aboutVersion"));
            const QString shown = version != nullptr ? version->property("text").toString()
                                                     : QString();
            std::fprintf(stderr, "selftest: about version \"%s\"\n", qUtf8Printable(shown));
            check(about != nullptr && about->property("visible").toBool(),
                  QStringLiteral("help: the About popup is on screen"));
            check(shown == QString::fromUtf8(pf_version()) && !shown.isEmpty(),
                  QStringLiteral("help: the version is the core's own"));
        });
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            QMetaObject::invokeMethod(ribbon, "invoke",
                                      Q_ARG(QVariant, QStringLiteral("license")));
            QObject *page = state->window->findChild<QObject *>(QStringLiteral("licenseText"));
            const QString text = page != nullptr ? page->property("text").toString() : QString();
            check(state->window->property("licenseOpen").toBool()
                      && !state->window->property("aboutOpen").toBool(),
                  QStringLiteral("help: Licencia opens its popup"));
            check(text.contains(QStringLiteral("GNU GENERAL PUBLIC LICENSE"))
                      && text.contains(QStringLiteral("Version 3")),
                  QStringLiteral("help: the licence is the full GPL text"));
        });
        steps->push_back([state, check] {
            QKeyEvent key(QEvent::KeyPress, Qt::Key_Escape, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &key);
            check(!state->window->property("aboutOpen").toBool()
                      && !state->window->property("licenseOpen").toBool(),
                  QStringLiteral("help: Escape closes the help popups"));
        });

        // A panel is chrome: Ctrl+B folds the catalog to its rail, the sheet takes the
        // room, and the document does not notice. The rail is the same target that brings
        // it back, so a hidden panel is never a mode the user cannot find.
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            check(dock != nullptr && state->window->property("catalogOpen").toBool(),
                  QStringLiteral("panels: the catalog starts open"));
            if (dock == nullptr) {
                return;
            }
            state->panelModified = state->session->modified();
            state->panelCanUndo = state->session->canUndo();
            state->panelPaints = state->sheet->paintCount();
            QKeyEvent key(QEvent::KeyPress, Qt::Key_B, Qt::ControlModifier);
            QCoreApplication::sendEvent(state->window, &key);
        });
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            check(!state->window->property("catalogOpen").toBool() && width <= 30.0,
                  QStringLiteral("panels: Ctrl+B folds it to the rail"));
            check(!state->session->viewSetting(QStringLiteral("panel/catalogOpen"), true).toBool(),
                  QStringLiteral("panels: the catalog state is remembered"));
            const int paints = state->sheet->paintCount() - state->panelPaints;
            std::fprintf(stderr, "selftest: the fold painted %d frame(s)\n", paints);
            check(paints <= 40, QStringLiteral("panels: the fold is a handful of frames"));
            check(state->session->modified() == state->panelModified
                      && state->session->canUndo() == state->panelCanUndo,
                  QStringLiteral("panels: hiding is not a document change"));
            QKeyEvent key(QEvent::KeyPress, Qt::Key_B, Qt::ControlModifier);
            QCoreApplication::sendEvent(state->window, &key);
        });
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            check(state->window->property("catalogOpen").toBool() && width > 200.0,
                  QStringLiteral("panels: Ctrl+B brings it back"));
        });

        // The right dock holds the properties and the title block. It appears when one of
        // them opens, folds to the rail with Ctrl+J, and yields its width when it has
        // nothing to show. None of it is a document change.
        steps->push_back([state, check] {
            state->session->closeInspector();
            check(!state->window->property("rightDockHasContent").toBool(),
                  QStringLiteral("panels: the right dock starts without content"));
            state->session->inspectComponent(2);
            check(state->window->property("rightDockHasContent").toBool(),
                  QStringLiteral("panels: the inspector fills the right dock"));
        });
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("rightDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            check(dock != nullptr && width > 300.0,
                  QStringLiteral("panels: the right dock shows the properties"));
            state->panelModified = state->session->modified();
            state->panelCanUndo = state->session->canUndo();
            QKeyEvent key(QEvent::KeyPress, Qt::Key_J, Qt::ControlModifier);
            QCoreApplication::sendEvent(state->window, &key);
        });
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("rightDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            check(!state->window->property("rightDockOpen").toBool() && width <= 30.0,
                  QStringLiteral("panels: Ctrl+J folds the right dock"));
            check(state->session->modified() == state->panelModified
                      && state->session->canUndo() == state->panelCanUndo,
                  QStringLiteral("panels: folding the right dock is not a document change"));
            QKeyEvent key(QEvent::KeyPress, Qt::Key_J, Qt::ControlModifier);
            QCoreApplication::sendEvent(state->window, &key);
        });
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("rightDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            check(state->window->property("rightDockOpen").toBool() && width > 300.0,
                  QStringLiteral("panels: Ctrl+J brings the right dock back"));
            state->session->closeInspector();
        });
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("rightDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            const bool blockOpen = state->window->property("titleBlockOpen").toBool();
            check(blockOpen || width <= 1.0,
                  QStringLiteral("panels: the right dock yields when it has no content"));
        });

        // Auto-hide is opt-in and remembered. Leaving the rail hides the panel, a dwell on
        // the rail brings it back, and a manual hide does not reopen under the pointer.
        steps->push_back([state, check] {
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            QMetaObject::invokeMethod(ribbon, "invoke",
                                      Q_ARG(QVariant, QStringLiteral("auto-hide")));
            check(state->window->property("panelAutoHide").toBool(),
                  QStringLiteral("panels: auto-hide is opt-in"));
            check(state->session->viewSetting(QStringLiteral("panel/autoHide"), false).toBool(),
                  QStringLiteral("panels: the auto-hide flag is remembered"));
            state->window->setProperty("catalogOpen", true);
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            if (dock != nullptr) {
                dock->setProperty("hovered", false);
            }
        });
        steps->push_back([] {});
        steps->push_back([] {});
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            check(!state->window->property("catalogOpen").toBool(),
                  QStringLiteral("panels: leaving the rail hides it"));
            if (dock != nullptr) {
                dock->setProperty("hovered", true);
            }
        });
        steps->push_back([] {});
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            check(state->window->property("catalogOpen").toBool() && width > 200.0,
                  QStringLiteral("panels: the edge opens it after the dwell"));
            state->window->setProperty("catalogOpen", false);
        });
        steps->push_back([] {});
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            check(!state->window->property("catalogOpen").toBool(),
                  QStringLiteral("panels: a manual hide stays closed under the pointer"));
            if (dock != nullptr) {
                dock->setProperty("hovered", false);
            }
        });

        // Restablecer vista puts the panels, the zoom and the pan back and turns the opt-in
        // auto-hide off, so a remembered state is never a trap.
        steps->push_back([state, check] {
            state->session->setZoom(1.5);
            state->window->setProperty("rightDockOpen", false);
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            QMetaObject::invokeMethod(ribbon, "invoke",
                                      Q_ARG(QVariant, QStringLiteral("reset-view")));
            check(state->window->property("catalogOpen").toBool()
                      && state->window->property("rightDockOpen").toBool(),
                  QStringLiteral("panels: reset view opens the panels"));
            check(!state->window->property("panelAutoHide").toBool(),
                  QStringLiteral("panels: reset view turns auto-hide off"));
            check(std::abs(state->session->zoom() - 1.0) < 1e-9,
                  QStringLiteral("panels: reset view returns to 1:1"));
        });

        // The catalog header filters the groups and remembers a collapsed one. Search is
        // a view state: it never changes the document until a match is placed.
        steps->push_back([state, check] {
            QObject *sidebar =
                state->window->findChild<QObject *>(QStringLiteral("catalogSidebar"));
            QObject *search =
                state->window->findChild<QObject *>(QStringLiteral("catalogSearch"));
            check(sidebar != nullptr && search != nullptr,
                  QStringLiteral("catalog: the search and the groups are in the scene"));
            if (sidebar != nullptr && search != nullptr) {
                search->setProperty("text", QStringLiteral("cilindro"));
            }
        });
        steps->push_back([state, check] {
            QObject *sidebar =
                state->window->findChild<QObject *>(QStringLiteral("catalogSidebar"));
            QObject *search =
                state->window->findChild<QObject *>(QStringLiteral("catalogSearch"));
            const int matches =
                sidebar != nullptr ? sidebar->property("matchCount").toInt() : -1;
            const int total = state->session->catalog().size();
            check(matches >= 2 && matches < total,
                  QStringLiteral("catalog: the search narrows the groups"));
            if (search != nullptr) {
                search->setProperty("text", QString());
            }
        });
        steps->push_back([state, check] {
            QObject *sidebar =
                state->window->findChild<QObject *>(QStringLiteral("catalogSidebar"));
            check(sidebar != nullptr
                      && sidebar->property("matchCount").toInt()
                             == state->session->catalog().size(),
                  QStringLiteral("catalog: clearing the search brings everything back"));
            if (sidebar != nullptr) {
                QMetaObject::invokeMethod(sidebar, "toggleGroup", Q_ARG(QVariant, 0));
            }
        });
        steps->push_back([state, check] {
            QObject *sidebar =
                state->window->findChild<QObject *>(QStringLiteral("catalogSidebar"));
            check(!state->session->viewSetting(QStringLiteral("catalog/group-0"), true).toBool(),
                  QStringLiteral("catalog: a collapsed group is remembered"));
            if (sidebar != nullptr) {
                QMetaObject::invokeMethod(sidebar, "toggleGroup", Q_ARG(QVariant, 0));
            }
        });
        steps->push_back([state, check] {
            QObject *search =
                state->window->findChild<QObject *>(QStringLiteral("catalogSearch"));
            state->components = pf_session_component_count(state->session->handle());
            if (search != nullptr) {
                search->setProperty("text", QStringLiteral("compresor"));
                auto *item = qobject_cast<QQuickItem *>(search);
                if (item != nullptr) {
                    item->forceActiveFocus();
                }
            }
            check(search != nullptr, QStringLiteral("catalog: the search field takes focus"));
        });
        steps->push_back([state, check] {
            QKeyEvent key(QEvent::KeyPress, Qt::Key_Return, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &key);
            check(true, QStringLiteral("catalog: Enter accepts the search"));
        });
        steps->push_back([state, check] {
            const uint32_t now =
                pf_session_component_count(state->session->handle());
            check(now == state->components + 1,
                  QStringLiteral("catalog: Enter places the first match"));
            QObject *search =
                state->window->findChild<QObject *>(QStringLiteral("catalogSearch"));
            if (search != nullptr) {
                search->setProperty("text", QString());
            }
        });

        // A tile is keyboard reachable: focus it and Return places it, like a click.
        steps->push_back([state, check] {
            QQuickItem *mouse = nullptr;
            std::function<void(QQuickItem *)> seek = [&](QQuickItem *item) {
                if (mouse == nullptr && item->isVisible()
                    && item->objectName() == QStringLiteral("paletteTile")) {
                    mouse = item;
                    return;
                }
                for (QQuickItem *child : item->childItems()) {
                    seek(child);
                }
            };
            seek(state->window->contentItem());
            check(mouse != nullptr,
                  QStringLiteral("catalog: a tile is in the scene"));
            if (mouse != nullptr) {
                if (QQuickItem *tile = mouse->parentItem(); tile != nullptr) {
                    tile->forceActiveFocus();
                }
                state->components = pf_session_component_count(state->session->handle());
            }
        });
        steps->push_back([state, check] {
            QKeyEvent key(QEvent::KeyPress, Qt::Key_Return, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &key);
            check(true, QStringLiteral("catalog: Return accepts a focused tile"));
        });
        steps->push_back([state, check] {
            const uint32_t now = pf_session_component_count(state->session->handle());
            check(now == state->components + 1,
                  QStringLiteral("catalog: Return places the focused tile"));
            state->window->setProperty("catalogWidth", 300);
        });
        steps->push_back([state, check] {
            QObject *dock =
                state->window->findChild<QObject *>(QStringLiteral("catalogDock"));
            const double width = dock != nullptr ? dock->property("width").toDouble() : -1.0;
            check(width >= 295.0 && width <= 305.0,
                  QStringLiteral("catalog: the width is the one chosen"));
            check(state->session->viewSetting(QStringLiteral("panel/catalogWidth"), 0).toInt()
                      == 300,
                  QStringLiteral("catalog: the width is remembered"));
            state->window->setProperty("catalogWidth", 18 * state->session->unit());
        });

        // Closing from the ribbon reaches the same removal the strip's x does. It runs last,
        // because the circuit in view changes and the gestures above measure that sheet.
        steps->push_back([state, check] {
            const int before = state->session->circuits().size();
            check(state->session->addCircuit() >= 0,
                  QStringLiteral("ribbon: a second circuit for the close"));
            QObject *ribbon = state->window->findChild<QObject *>(QStringLiteral("ribbon"));
            QMetaObject::invokeMethod(ribbon, "invoke",
                                      Q_ARG(QVariant, QStringLiteral("circuit-close")));
            check(state->session->circuits().size() == before,
                  QStringLiteral("ribbon: Cerrar closes the circuit in view"));
            state->session->undo();
            check(state->session->circuits().size() == before + 1,
                  QStringLiteral("ribbon: the closed circuit returns with the undo"));
            state->session->undo();
        });

        // Inicio offers the authored circuits and loads the selected one as a clean project.
        steps->push_back([state, check] {
            state->session->newDocument();
            state->window->setProperty("startViewOpen", true);
            QObject *start = state->window->findChild<QObject *>(QStringLiteral("startView"));
            check(start != nullptr && start->property("visible").toBool(),
                  QStringLiteral("templates: Inicio is visible"));
            check(state->session->templateCount() == static_cast<int>(PF_TEMPLATE_COUNT),
                  QStringLiteral("templates: the ABI count reaches the session"));
        });
        // A palette change must replace the visible icons: the URL carries the revision,
        // so QML asks the provider again instead of drawing the previous tint.
        steps->push_back([state, check] {
            QObject *icon = state->window->findChild<QObject *>(QStringLiteral("railHomeIcon"));
            check(icon != nullptr, QStringLiteral("theme: the rail home icon is in the scene"));
            if (icon == nullptr) {
                return;
            }
            const QString before = icon->property("source").toUrl().toString();
            check(before.contains(QStringLiteral("v=")
                                  + QString::number(state->session->themeRevision())),
                  QStringLiteral("theme: the icon URL carries the revision"));
            state->paletteBeforeTheme = QGuiApplication::palette();
            QPalette changed = state->paletteBeforeTheme;
            changed.setColor(QPalette::Highlight, QColor(190, 40, 40));
            QGuiApplication::setPalette(changed);
            state->session->reloadTheme();
        });
        steps->push_back([state, check] {
            QObject *icon = state->window->findChild<QObject *>(QStringLiteral("railHomeIcon"));
            if (icon == nullptr) {
                check(false, QStringLiteral("theme: the rail icon survives the change"));
                return;
            }
            const int revision = state->session->themeRevision();
            const QString source = icon->property("source").toUrl().toString();
            check(source.contains(QStringLiteral("v=") + QString::number(revision)),
                  QStringLiteral("theme: the icon URL carries the new revision"));
            const qreal dpr = QGuiApplication::primaryScreen()
                                  ? QGuiApplication::primaryScreen()->devicePixelRatio()
                                  : 1.0;
            const int pixels = qRound(state->session->unit() * 1.1 * dpr);
            check(state->icons != nullptr
                      && state->icons->contains(QStringLiteral("go-home?tone=onaccent"),
                                                pixels),
                  QStringLiteral("theme: the icon is requested again"));
            QGuiApplication::setPalette(state->paletteBeforeTheme);
            state->session->reloadTheme();
        });
        steps->push_back([state, check] {
            QObject *start = state->window->findChild<QObject *>(QStringLiteral("startView"));
            check(start != nullptr, QStringLiteral("templates: the start page remains available"));
            if (start == nullptr) {
                return;
            }
            check(start->property("templateCardCount").toInt() == 3,
                  QStringLiteral("templates: Inicio offers three cards"));
            // The electrical template is compiled in and PF_TEMPLATE_COUNT stays four, but
            // its card is not offered until P-026. Reading the delegates proves the absence
            // instead of trusting the array's length.
            QObject *repeater =
                start->findChild<QObject *>(QStringLiteral("templateRepeater"));
            bool electrical = false;
            const int cards = repeater != nullptr ? repeater->property("count").toInt() : 0;
            for (int index = 0; index < cards; ++index) {
                QQuickItem *card = nullptr;
                if (QMetaObject::invokeMethod(repeater, "itemAt",
                                              Q_RETURN_ARG(QQuickItem *, card),
                                              Q_ARG(int, index))
                    && card != nullptr
                    && card->objectName() == QStringLiteral("templateCard3")) {
                    electrical = true;
                }
            }
            check(cards == 3 && !electrical,
                  QStringLiteral("templates: the electrical card is absent"));
            const bool requested = QMetaObject::invokeMethod(
                start, "activateTemplate", Q_ARG(QVariant, QVariant(0)));
            check(requested, QStringLiteral("templates: a card can request a template"));
            check(!state->window->property("startViewOpen").toBool(),
                  QStringLiteral("templates: choosing one returns to its project"));
            check(state->session->componentCount() == 6,
                  QStringLiteral("templates: the OR circuit is copied into the project"));
            check(!state->session->modified(),
                  QStringLiteral("templates: a starting circuit is clean, not an edit"));

            state->session->setDesignation(0, QStringLiteral("pendiente"));
            state->window->setProperty("startViewOpen", true);
            QMetaObject::invokeMethod(start, "activateTemplate", Q_ARG(QVariant, QVariant(1)));
            check(state->window->property("pendingCommand").toString()
                      == QStringLiteral("template")
                      && state->window->property("pendingTemplate").toInt() == 1,
                  QStringLiteral("templates: replacing unsaved work asks for confirmation"));
            check(state->session->componentCount() == 4 && state->session->modified(),
                  QStringLiteral("templates: work stays intact before confirmation"));
            const bool discarded =
                QMetaObject::invokeMethod(state->window, "discardAndContinue");
            check(discarded, QStringLiteral("templates: the confirmation can continue"));
            check(state->session->componentCount() == 3 && !state->session->modified()
                      && !state->window->property("startViewOpen").toBool(),
                  QStringLiteral("templates: discard opens the requested template cleanly"));
        });

        // Inicio's own visual pass: two buttons, three offered templates, the recent row with
        // its stat'd time and size, the missing row and its quitar, and Esc. The fixture is a
        // file the session saved; the page never opens or parses it.
        steps->push_back([state, check, accessibleName] {
            state->window->setProperty("startViewOpen", true);
            QObject *start = state->window->findChild<QObject *>(QStringLiteral("startView"));
            check(start != nullptr, QStringLiteral("start: the page is in the scene"));
            if (start == nullptr) {
                return;
            }
            QQuickItem *newCard =
                start->findChild<QQuickItem *>(QStringLiteral("startCardNew"));
            QQuickItem *openCard =
                start->findChild<QQuickItem *>(QStringLiteral("startCardOpen"));
            check(newCard != nullptr && openCard != nullptr && newCard->isVisible()
                      && openCard->isVisible(),
                  QStringLiteral("start: the two cards exist"));
            check(accessibleName(newCard) == QStringLiteral("Nuevo proyecto")
                      && accessibleName(openCard) == QStringLiteral("Abrir…"),
                  QStringLiteral("start: the cards are named for assistive technology"));
            check(start->property("templateCardCount").toInt() == 3,
                  QStringLiteral("start: three templates are offered"));
            QObject *repeater =
                start->findChild<QObject *>(QStringLiteral("templateRepeater"));
            bool electrical = false;
            const int cards = repeater != nullptr ? repeater->property("count").toInt() : 0;
            for (int index = 0; index < cards; ++index) {
                QQuickItem *card = nullptr;
                if (QMetaObject::invokeMethod(repeater, "itemAt",
                                              Q_RETURN_ARG(QQuickItem *, card),
                                              Q_ARG(int, index))
                    && card != nullptr
                    && card->objectName() == QStringLiteral("templateCard3")) {
                    electrical = true;
                }
            }
            check(cards == 3 && !electrical,
                  QStringLiteral("start: the electrical template is not offered"));
        });
        steps->push_back([state, check] {
            state->session->forgetRecent();
            state->startRecent =
                QDir::tempPath() + QStringLiteral("/punchi-start-reciente.pfluid");
            check(state->session->saveTo(state->startRecent),
                  QStringLiteral("start: a recent fixture is written"));
            check(state->session->recentPage().size() == 1,
                  QStringLiteral("start: the fixture reaches the page"));
            const QVariantMap row = state->session->recentPage().value(0).toMap();
            check(row.value(QStringLiteral("modified")).toDateTime().isValid()
                      && row.value(QStringLiteral("size")).toLongLong() > 0,
                  QStringLiteral("start: the row carries real modified time and size"));
            state->window->setProperty("startViewOpen", true);
        });
        steps->push_back([state, check] {
            QObject *start = state->window->findChild<QObject *>(QStringLiteral("startView"));
            QQuickItem *scroll =
                start != nullptr ? start->findChild<QQuickItem *>(
                    QStringLiteral("startScroll")) : nullptr;
            if (scroll != nullptr) {
                scroll->setProperty("contentY",
                                    qMax(0.0, scroll->property("contentHeight").toReal()
                                                   - scroll->height()));
            }
            check(start != nullptr && scroll != nullptr,
                  QStringLiteral("start: the page can scroll to the rows"));
            QObject *repeater =
                start != nullptr ? start->findChild<QObject *>(
                    QStringLiteral("recentRepeater")) : nullptr;
            QQuickItem *row = nullptr;
            const bool got = repeater != nullptr
                             && QMetaObject::invokeMethod(repeater, "itemAt",
                                                          Q_RETURN_ARG(QQuickItem *, row),
                                                          Q_ARG(int, 0));
            check(got && row != nullptr && row->isVisible(),
                  QStringLiteral("start: the recent row is on the page"));
            QQuickItem *open =
                row != nullptr ? row->findChild<QQuickItem *>(
                    QStringLiteral("recentOpen0")) : nullptr;
            check(open != nullptr && open->isVisible(),
                  QStringLiteral("start: the row is a focusable button"));
            state->startRow = row;
        });
        steps->push_back([state, check, clickItem] {
            clickItem(state->startRow);
            check(!state->window->property("startViewOpen").toBool(),
                  QStringLiteral("start: a recent row opens"));
            check(state->session->currentPath() == state->startRecent,
                  QStringLiteral("start: the row opens the file it names"));
        });
        steps->push_back([state, check] {
            state->startMissing =
                QDir::tempPath() + QStringLiteral("/punchi-start-falta.pfluid");
            check(state->session->saveTo(state->startMissing),
                  QStringLiteral("start: a file is written to go missing"));
            check(QFile::remove(state->startMissing),
                  QStringLiteral("start: the file is removed behind the page"));
            // The failed open is what makes the session re-stat the rows; it never parses.
            check(!state->session->openRecent(0),
                  QStringLiteral("start: a missing row does not open"));
            state->window->setProperty("startViewOpen", true);
        });
        steps->push_back([state, check] {
            QObject *start = state->window->findChild<QObject *>(QStringLiteral("startView"));
            QObject *repeater =
                start != nullptr ? start->findChild<QObject *>(
                    QStringLiteral("recentRepeater")) : nullptr;
            QQuickItem *row = nullptr;
            const bool got = repeater != nullptr
                             && QMetaObject::invokeMethod(repeater, "itemAt",
                                                          Q_RETURN_ARG(QQuickItem *, row),
                                                          Q_ARG(int, 0));
            QQuickItem *stateText =
                row != nullptr ? row->findChild<QQuickItem *>(
                    QStringLiteral("recentState0")) : nullptr;
            check(got && row != nullptr && stateText != nullptr
                      && stateText->property("text").toString()
                             == QStringLiteral("no encontrado"),
                  QStringLiteral("start: a missing recent says no encontrado"));
            state->startRow = row;
        });
        steps->push_back([state, check, clickItem] {
            QQuickItem *forget =
                state->startRow != nullptr ? state->startRow->findChild<QQuickItem *>(
                    QStringLiteral("recentForget0")) : nullptr;
            check(forget != nullptr, QStringLiteral("start: quitar is on the missing row"));
            clickItem(forget);
            check(state->session->recentPage().size() == 1
                      && state->session->recent().value(0).toMap()
                             .value(QStringLiteral("path")).toString()
                         == state->startRecent,
                  QStringLiteral("start: quitar removes the row"));
        });
        steps->push_back([state, check] {
            // Leave the list as the rest of the battery expects it and remove the fixture.
            state->session->forgetRecent();
            check(QFile::remove(state->startRecent),
                  QStringLiteral("start: the fixture is removed"));
            state->window->setProperty("startViewOpen", true);
            QKeyEvent escape(QEvent::KeyPress, Qt::Key_Escape, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &escape);
        });
        steps->push_back([state, check] {
            check(!state->window->property("startViewOpen").toBool(),
                  QStringLiteral("start: Esc returns"));
        });

        // The strip's home is the way back, and it is not a circuit: these steps prove the
        // control, Volver, Ctrl+Home, the empty project and the strip's keyboard order.
        steps->push_back([state, check, accessibleName, clickItem] {
            state->session->newDocument();
            while (state->session->circuits().size() < 2) {
                if (state->session->addCircuit() < 0) {
                    break;
                }
            }
            state->window->setProperty("startViewOpen", false);
            state->homeCircuits = static_cast<int>(state->session->circuits().size());
            state->homeActive = state->session->activeCircuit();
            state->homeCanUndo = state->session->canUndo();
            QQuickItem *home =
                state->window->findChild<QQuickItem *>(QStringLiteral("homeTab"));
            check(home != nullptr && home->isVisible(),
                  QStringLiteral("home: the strip shows the home control"));
            if (home == nullptr) {
                return;
            }
            check(home->property("accessibleLabel").toString() == QStringLiteral("Inicio")
                      && home->property("tooltipText").toString() == QStringLiteral("Inicio"),
                  QStringLiteral("home: the tooltip and the mirrored label are Inicio"));
            check(accessibleName(home) == QStringLiteral("Inicio"),
                  QStringLiteral("home: assistive technology reads Inicio"));
            check(home->property("text").toString().isEmpty(),
                  QStringLiteral("home: the control carries no visible text"));
            clickItem(home);
        });
        steps->push_back([state, check] {
            QObject *start =
                state->window->findChild<QObject *>(QStringLiteral("startView"));
            check(state->window->property("startViewOpen").toBool() && start != nullptr
                      && start->property("visible").toBool(),
                  QStringLiteral("home: a click opens Inicio"));
            check(static_cast<int>(state->session->circuits().size()) == state->homeCircuits,
                  QStringLiteral("home: the control is not a circuit"));
            check(state->session->activeCircuit() == state->homeActive,
                  QStringLiteral("home: opening Inicio leaves the circuit in view"));
            check(state->session->canUndo() == state->homeCanUndo,
                  QStringLiteral("home: opening Inicio adds no undo entry"));
            if (start != nullptr) {
                QMetaObject::invokeMethod(start, "returnRequested");
            }
        });
        steps->push_back([state, check] {
            check(!state->window->property("startViewOpen").toBool(),
                  QStringLiteral("home: Volver returns to the sheet"));
            check(state->session->activeCircuit() == state->homeActive
                      && static_cast<int>(state->session->circuits().size())
                             == state->homeCircuits,
                  QStringLiteral("home: Volver keeps the same circuit"));
        });
        steps->push_back([state, check, clickItem] {
            state->session->newDocument();
            state->window->setProperty("startViewOpen", false);
            check(static_cast<int>(state->session->circuits().size()) == 1
                      && state->session->currentPath().isEmpty()
                      && !state->session->hasModel(),
                  QStringLiteral("home: a new project is empty and has no path"));
            clickItem(state->window->findChild<QQuickItem *>(QStringLiteral("homeTab")));
        });
        steps->push_back([state, check] {
            QObject *start =
                state->window->findChild<QObject *>(QStringLiteral("startView"));
            check(state->window->property("startViewOpen").toBool() && start != nullptr
                      && start->property("canReturn").toBool(),
                  QStringLiteral("home: an empty project still shows Volver"));
            if (start != nullptr) {
                QMetaObject::invokeMethod(start, "returnRequested");
            }
        });
        steps->push_back([state, check] {
            QQuickItem *home =
                state->window->findChild<QQuickItem *>(QStringLiteral("homeTab"));
            if (home != nullptr) {
                home->forceActiveFocus();
            }
            QKeyEvent key(QEvent::KeyPress, Qt::Key_Home, Qt::ControlModifier);
            QCoreApplication::sendEvent(state->window, &key);
        });
        steps->push_back([state, check] {
            check(state->window->property("startViewOpen").toBool(),
                  QStringLiteral("home: Ctrl+Home opens Inicio"));
            state->window->setProperty("startViewOpen", false);
        });
        steps->push_back([state, check] {
            QObject *tabs =
                state->window->findChild<QObject *>(QStringLiteral("circuitTabs"));
            const QVariantList circuits = state->session->circuits();
            check(tabs != nullptr && !circuits.isEmpty(),
                  QStringLiteral("home: the strip is there for the rename"));
            if (tabs == nullptr || circuits.isEmpty()) {
                return;
            }
            const QVariantMap first = circuits.constFirst().toMap();
            check(QMetaObject::invokeMethod(tabs, "beginRename",
                                            Q_ARG(QVariant, first.value(QStringLiteral("id"))),
                                            Q_ARG(QVariant, first.value(QStringLiteral("name")))),
                  QStringLiteral("home: the circuit rename editor opens"));
        });
        steps->push_back([state, check] {
            QQuickItem *focus = state->window->activeFocusItem();
            check(focus != nullptr && focus->inherits("QQuickTextInput"),
                  QStringLiteral("home: the rename editor owns the keyboard"));
            QKeyEvent key(QEvent::KeyPress, Qt::Key_Home, Qt::ControlModifier);
            QCoreApplication::sendEvent(state->window, &key);
        });
        steps->push_back([state, check] {
            check(!state->window->property("startViewOpen").toBool(),
                  QStringLiteral("home: Ctrl+Home yields to a text editor"));
            QObject *tabs =
                state->window->findChild<QObject *>(QStringLiteral("circuitTabs"));
            if (tabs != nullptr) {
                QMetaObject::invokeMethod(tabs, "cancelRename");
            }
        });
        steps->push_back([state, check] {
            while (state->session->circuits().size() < 2) {
                if (state->session->addCircuit() < 0) {
                    break;
                }
            }
            state->window->setProperty("startViewOpen", false);
            QQuickItem *home =
                state->window->findChild<QQuickItem *>(QStringLiteral("homeTab"));
            if (home != nullptr) {
                home->forceActiveFocus();
            }
        });
        steps->push_back([state, check, accessibleName] {
            QQuickItem *home =
                state->window->findChild<QQuickItem *>(QStringLiteral("homeTab"));
            QQuickItem *plus =
                state->window->findChild<QQuickItem *>(QStringLiteral("addCircuitButton"));
            const QVariantList circuits = state->session->circuits();
            check(circuits.size() == 2,
                  QStringLiteral("home: the keyboard order uses two circuits"));
            if (home == nullptr || plus == nullptr || circuits.size() != 2) {
                return;
            }
            check(state->window->activeFocusItem() == home,
                  QStringLiteral("home: the home control is the first focusable item"));
            const QVariantMap firstRow = circuits.at(0).toMap();
            const QVariantMap secondRow = circuits.at(1).toMap();
            auto *repeater =
                state->window->findChild<QObject *>(QStringLiteral("tabRepeater"));
            QQuickItem *first = nullptr;
            QQuickItem *second = nullptr;
            if (repeater != nullptr) {
                QMetaObject::invokeMethod(repeater, "itemAt", Q_RETURN_ARG(QQuickItem *, first),
                                          Q_ARG(int, 0));
                QMetaObject::invokeMethod(repeater, "itemAt", Q_RETURN_ARG(QQuickItem *, second),
                                          Q_ARG(int, 1));
            }
            auto *close = first != nullptr
                              ? first->findChild<QQuickItem *>(
                                    QStringLiteral("circuitCloseButton_")
                                    + QString::number(firstRow.value(QStringLiteral("id")).toInt()))
                              : nullptr;
            check(accessibleName(first)
                      == firstRow.value(QStringLiteral("name")).toString(),
                  QStringLiteral("home: the first circuit tab is named"));
            check(accessibleName(second)
                      == secondRow.value(QStringLiteral("name")).toString(),
                  QStringLiteral("home: the second circuit tab is named"));
            check(accessibleName(plus) == QStringLiteral("Nuevo circuito"),
                  QStringLiteral("home: the plus is named"));
            check(accessibleName(close)
                      == QStringLiteral("Cerrar %1").arg(
                             firstRow.value(QStringLiteral("name")).toString()),
                  QStringLiteral("home: the close control is named"));
            QKeyEvent tabOne(QEvent::KeyPress, Qt::Key_Tab, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &tabOne);
            check(state->window->activeFocusItem() == first,
                  QStringLiteral("home: Tab reaches the first circuit"));
            QKeyEvent tabTwo(QEvent::KeyPress, Qt::Key_Tab, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &tabTwo);
            check(state->window->activeFocusItem() == second,
                  QStringLiteral("home: Tab reaches the second circuit"));
            QKeyEvent tabThree(QEvent::KeyPress, Qt::Key_Tab, Qt::NoModifier);
            QCoreApplication::sendEvent(state->window, &tabThree);
            check(state->window->activeFocusItem() == plus,
                  QStringLiteral("home: Tab reaches the plus"));
        });

        // A hose to a dead end carries no flow, so it carries no pulse even while the
        // transport runs and the phase advances. The fixture is two components and one
        // hose, so the only accent-coloured line the frame could contain is the pulse.
        steps->push_back([state, check] {
            const auto catalogIndex = [state](const QString &id) {
                const QVariantList rows = state->session->catalog();
                for (const QVariant &entry : rows) {
                    const QVariantMap row = entry.toMap();
                    if (row.value(QStringLiteral("id")).toString() == id) {
                        return row.value(QStringLiteral("index")).toInt();
                    }
                }
                return -1;
            };
            state->session->stopSimulation();
            state->session->newDocument();
            const int compressor = catalogIndex(QStringLiteral("supply.compressor"));
            const int gauge = catalogIndex(QStringLiteral("gauge.pressure"));
            check(compressor >= 0 && gauge >= 0,
                  QStringLiteral("animation: the dead-end fixture exists"));
            if (compressor < 0 || gauge < 0) {
                return;
            }
            const int source = state->session->placeComponent(compressor, 30.0, 40.0);
            const int dead_end = state->session->placeComponent(gauge, 120.0, 40.0);
            check(source >= 0 && dead_end >= 0,
                  QStringLiteral("animation: the dead-end hose is placed"));
            if (source < 0 || dead_end < 0) {
                return;
            }
            check(pf_session_connect(state->session->handle(), source, "1", dead_end, "1")
                      == PF_OK,
                  QStringLiteral("animation: the gauge is connected as a dead end"));
            // A placed component is selected, and a selected symbol is drawn in the accent;
            // clear the selection so the only accent the frame can carry is the pulse.
            state->session->select(-1);
            state->session->startSimulation();
            for (int i = 0; i < 20; ++i) {
                state->session->advance(0.016);
            }
            PfFrame frame {};
            const int32_t code = pf_session_build_frame(
                state->session->handle(), 4.0F, static_cast<float>(state->session->phase()),
                state->session->motionActive() ? 1 : 0, &frame);
            check(code == PF_OK, QStringLiteral("animation: the dead-end frame is built"));
            const QColor accent = state->session->accent().lighter(150);
            const auto pulse_rgb = [&accent](const PfRgb &rgb) {
                return rgb.r == static_cast<uint8_t>(accent.red())
                       && rgb.g == static_cast<uint8_t>(accent.green())
                       && rgb.b == static_cast<uint8_t>(accent.blue());
            };
            bool pulse = false;
            if (code == PF_OK) {
                for (uint32_t i = 0; i < frame.calls_len; ++i) {
                    const PfCall &call = frame.calls[i];
                    if (call.kind == PF_KIND_LINE && pulse_rgb(call.stroke)) {
                        pulse = true;
                    }
                }
            }
            check(!pulse, QStringLiteral("animation: a dead-end draws no pulse"));
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
