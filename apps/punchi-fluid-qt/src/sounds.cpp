// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#include "sounds.h"

#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QLatin1String>
#include <QStandardPaths>

namespace {

/// The built-in pair, compiled in by `qt_add_resources` in CMakeLists.txt.
constexpr QLatin1String kBuiltinPrefix("/sounds/");

SoundSource resolve(const QString &userDir, const QString &name)
{
    const QString path = userDir + QLatin1Char('/') + name;
    if (QFileInfo::exists(path)) {
        return SoundSource { QUrl::fromLocalFile(path), QStringLiteral("user:") + path };
    }
    return SoundSource { QUrl(QStringLiteral("qrc:") + kBuiltinPrefix + name),
                         QStringLiteral("built-in") };
}

QString openPath(const QUrl &source)
{
    // A qrc URL is not a file system path: QFile wants the colon form.
    return source.scheme() == QLatin1String("qrc") ? QLatin1Char(':') + source.path()
                                                   : source.toLocalFile();
}

} // namespace

SoundSources resolveSoundSources()
{
    const QString userDir = QStandardPaths::writableLocation(QStandardPaths::GenericDataLocation)
                            + QStringLiteral("/punchi-fluid-lab/sounds");
    return SoundSources { resolve(userDir, QStringLiteral("valve.wav")),
                          resolve(userDir, QStringLiteral("air.wav")),
                          resolve(userDir, QStringLiteral("cylinder.wav")) };
}

bool isWaveFile(const QUrl &source)
{
    QFile file(openPath(source));
    if (!file.open(QIODevice::ReadOnly)) {
        return false;
    }
    const QByteArray header = file.read(12);
    return header.size() == 12 && header.startsWith("RIFF") && header.mid(8, 4) == "WAVE";
}

SoundLibrary::SoundLibrary(QObject *parent)
    : QObject(parent)
    , m_sources(resolveSoundSources())
{
}
