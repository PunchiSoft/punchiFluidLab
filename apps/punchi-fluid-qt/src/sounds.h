// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QObject>
#include <QQmlEngine>
#include <QString>
#include <QUrl>

/// One event sample and where it came from.
struct SoundSource
{
    QUrl url;
    QString origin;
};

/// The samples the window may play, resolved once on start.
struct SoundSources
{
    SoundSource valve;
    SoundSource air;
    SoundSource cylinder;
};

/// Resolves every sample: a file in `$XDG_DATA_HOME/punchi-fluid-lab/sounds/` if it
/// exists, the built-in resource otherwise.
///
/// The priority rule is the catalog's, and it is the catalog's for the same reason: a
/// recording made in a real workshop beats anything generated, and trying one must not
/// need a fork. The built-in pair is compiled into the binary, so an untouched checkout
/// still makes a sound on a machine with no data folder at all.
SoundSources resolveSoundSources();

/// `true` when the source opens and starts with a RIFF/WAVE header.
///
/// A truncated file, a renamed `.mp3`, or a source that never made it into the binary all
/// fail the same way in the mixer — silently — so the check happens where it can be
/// reported.
bool isWaveFile(const QUrl &source);

/// The samples for QML. Resolved once, because the data folder does not change under a
/// running window and a QSoundEffect re-reads its file on every source change.
class SoundLibrary : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(QUrl valve READ valve CONSTANT)
    Q_PROPERTY(QUrl air READ air CONSTANT)
    Q_PROPERTY(QUrl cylinder READ cylinder CONSTANT)

public:
    explicit SoundLibrary(QObject *parent = nullptr);

    QUrl valve() const { return m_sources.valve.url; }
    QUrl air() const { return m_sources.air.url; }
    QUrl cylinder() const { return m_sources.cylinder.url; }

private:
    SoundSources m_sources;
};
