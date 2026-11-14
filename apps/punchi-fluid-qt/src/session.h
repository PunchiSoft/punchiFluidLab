// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <QColor>
#include <cmath>
#include <QObject>
#include <QPalette>
#include <QQmlEngine>
#include <QString>
#include <QVariantList>
#include <QPointF>

#include "punchi_fluid.h"
#include "theme.h"

/// One simulation, owned by the Rust core and seen from QML.
///
/// The properties are the whole of what the interface reads; everything else lives
/// behind the opaque `PfSession`. Colours are pushed the other way, from the desktop
/// palette into the core, so the drawing never picks one itself.
class FluidSession : public QObject
{
    Q_OBJECT
    QML_ELEMENT

    Q_PROPERTY(double pressureBar READ pressureBar NOTIFY changed)
    Q_PROPERTY(double positionMm READ positionMm NOTIFY changed)
    Q_PROPERTY(double velocityMmS READ velocityMmS NOTIFY changed)
    Q_PROPERTY(double timeS READ timeS NOTIFY changed)
    Q_PROPERTY(int valvePosition READ valvePosition NOTIFY changed)
    Q_PROPERTY(bool valveSpringReturn READ valveSpringReturn NOTIFY changed)
    Q_PROPERTY(bool buzzerActive READ buzzerActive NOTIFY buzzerChanged)

    /// `true` in edit mode: the animation does not run and a click opens properties
    /// instead of actuating, which is the split a schematic editor has.
    ///
    /// The mode is not a switch of its own: it is what the transport leaves behind.
    /// Starting or pausing enters simulation; stopping returns to editing.
    Q_PROPERTY(bool editing READ editing NOTIFY modeChanged)
    Q_PROPERTY(int componentCount READ componentCount NOTIFY documentChanged)
    Q_PROPERTY(int selected READ selected NOTIFY documentChanged)
    /// The selected pipe, or `-1`; a pipe and a component are never both selected.
    Q_PROPERTY(int selectedPipe READ selectedPipe NOTIFY documentChanged)
    Q_PROPERTY(int selectionCount READ selectionCount NOTIFY documentChanged)
    Q_PROPERTY(double zoom READ zoom WRITE setZoom NOTIFY viewChanged)
    Q_PROPERTY(bool grid READ grid WRITE setGrid NOTIFY viewChanged)
    /// The sheet the document is drawn on: the same paper the export prints.
    Q_PROPERTY(int sheetPaper READ sheetPaper NOTIFY sheetChanged)
    Q_PROPERTY(bool sheetLandscape READ sheetLandscape NOTIFY sheetChanged)
    Q_PROPERTY(QString sheetPaperName READ sheetPaperName NOTIFY sheetChanged)
    Q_PROPERTY(double sheetWidthMm READ sheetWidthMm NOTIFY sheetChanged)
    Q_PROPERTY(double sheetHeightMm READ sheetHeightMm NOTIFY sheetChanged)
    Q_PROPERTY(double sheetMarginTop READ sheetMarginTop NOTIFY sheetChanged)
    Q_PROPERTY(double sheetMarginRight READ sheetMarginRight NOTIFY sheetChanged)
    Q_PROPERTY(double sheetMarginBottom READ sheetMarginBottom NOTIFY sheetChanged)
    Q_PROPERTY(double sheetMarginLeft READ sheetMarginLeft NOTIFY sheetChanged)
    Q_PROPERTY(bool canUndo READ canUndo NOTIFY documentChanged)
    Q_PROPERTY(bool canRedo READ canRedo NOTIFY documentChanged)
    Q_PROPERTY(bool inspectorOpen READ inspectorOpen NOTIFY inspectorChanged)
    Q_PROPERTY(bool checkOpen READ checkOpen NOTIFY inspectorChanged)
    Q_PROPERTY(QString checkReport READ checkReport NOTIFY inspectorChanged)
    Q_PROPERTY(int checkCount READ checkCount NOTIFY inspectorChanged)
    Q_PROPERTY(QString inspectorText READ inspectorText NOTIFY inspectorChanged)
    Q_PROPERTY(QVariantList inspectorFields READ inspectorFields NOTIFY inspectorChanged)
    /// The sources a selected recorder can chart, as `placement`, `quantity` and `label`.
    Q_PROPERTY(QVariantList chartCandidates READ chartCandidates NOTIFY inspectorChanged)
    /// Identity of the charted placement, or `-1` when the recorder is not bound.
    Q_PROPERTY(int chartSourcePlacement READ chartSourcePlacement NOTIFY inspectorChanged)
    Q_PROPERTY(bool hasModel READ hasModel NOTIFY changed)
    Q_PROPERTY(QString lastError READ lastError NOTIFY changed)
    Q_PROPERTY(QString currentPath READ currentPath NOTIFY documentChanged)
    Q_PROPERTY(bool modified READ modified NOTIFY documentChanged)
    Q_PROPERTY(QString title READ title NOTIFY documentChanged)
    Q_PROPERTY(bool recoveryOffered READ recoveryOffered NOTIFY documentChanged)
    Q_PROPERTY(bool closeConfirmed READ closeConfirmed NOTIFY changed)
    Q_PROPERTY(QVariantList recent READ recent NOTIFY recentChanged)
    Q_PROPERTY(QVariantList circuits READ circuits NOTIFY changed)
    Q_PROPERTY(int activeCircuit READ activeCircuit NOTIFY changed)

    Q_PROPERTY(bool running READ running WRITE setRunning NOTIFY runningChanged)
    Q_PROPERTY(double speed READ speed WRITE setSpeed NOTIFY speedChanged)

    Q_PROPERTY(QVariantList catalog READ catalog NOTIFY catalogChanged)
    Q_PROPERTY(QString contrastSummary READ contrastSummary NOTIFY changed)
    Q_PROPERTY(bool themePasses READ themePasses NOTIFY changed)
    Q_PROPERTY(bool settled READ settled NOTIFY settledChanged)

    Q_PROPERTY(QColor background READ background NOTIFY changed)
    Q_PROPERTY(QColor panel READ panel NOTIFY changed)
    Q_PROPERTY(QColor foreground READ foreground NOTIFY changed)
    Q_PROPERTY(QColor muted READ muted NOTIFY changed)
    Q_PROPERTY(QColor accent READ accent NOTIFY changed)

public:
    explicit FluidSession(QObject *parent = nullptr);
    ~FluidSession() override;

    PfSession *handle() const { return m_session; }
    const ThemeTokens &tokens() const { return m_tokens; }

    /// Gauge pressure in the cylinder, the way a pneumatic gauge reads: zero at rest.
    double pressureBar() const
    {
        const double bar = (m_telemetry.pressure_pa - PF_ATMOSPHERE_PA) / 1.0e5;
        // A chamber at rest can sit a fraction of a pascal below the atmosphere the model
        // uses, and "-0,00 bar" is a reading no gauge would show.
        return std::abs(bar) < 0.005 ? 0.0 : bar;
    }
    double positionMm() const { return m_telemetry.position_m * 1.0e3; }
    double velocityMmS() const { return m_telemetry.velocity_m_s * 1.0e3; }
    double timeS() const { return m_telemetry.time_s; }
    int valvePosition() const { return static_cast<int>(m_telemetry.valve_position); }
    bool hasModel() const { return m_telemetry.has_model == 1; }
    QString lastError() const { return m_error; }
    /// File the document was opened from or last saved to; empty until then.
    QString currentPath() const { return m_currentPath; }
    /// `true` when the sheet holds work that is not in the file it came from.
    bool modified() const { return m_modified; }
    /// The window title: the file name, and a mark when there is unsaved work.
    QString title() const;
    /// `true` when a safety copy from a previous run is waiting to be recovered.
    bool recoveryOffered() const { return m_recoveryOffered; }
    /// Where the safety copy lives: one file per user, beside the application data.
    QString autosavePath() const;
    /// `true` once the user said the unsaved work may be lost.
    bool closeConfirmed() const { return m_closeConfirmed; }
    /// Loads the safety copy and leaves it unsaved, as the core asks.
    Q_INVOKABLE bool recoverAutosave();
    /// Throws the safety copy away.
    Q_INVOKABLE void discardAutosave();
    /// Writes the safety copy if there is unsaved work. Called by the window's timer.
    Q_INVOKABLE void autosave();
    /// Starts over with the worked example. The caller warns first when `modified`.
    Q_INVOKABLE void discardAndNewDocument();
    /// Opens a file, discarding unsaved work. The caller warns first.
    Q_INVOKABLE bool discardAndOpen(const QString &path);
    /// `true` when closing right now would lose work.
    Q_INVOKABLE bool hasUnsavedWork() const { return m_modified; }
    /// Called by the window once the close was confirmed.
    Q_INVOKABLE void confirmClose();
    /// Writes the sheet; `path` may be a local path or a `file:` URL.
    Q_INVOKABLE bool saveTo(const QString &path);
    /// Reads a sheet, replacing the document. On failure the document is untouched.
    Q_INVOKABLE bool openFrom(const QString &path);
    /// Saves to the current file. `false` when there is none yet.
    Q_INVOKABLE bool saveCurrent();
    /// Writes the sheet, as it is drawn, to `path`.
    ///
    /// `pdf` is `true` for a vector page and `false` for a raster at `dpi`. `paper` is an index
    /// into the list the export offers, which `paperNames` mirrors for the panel.
    /// Writes the sheet with the panel's settings, as a map of `path`, `kind`, `paper`,
    /// `width`, `height`, `landscape`, `dpi`, `margins` (four values) and `parts_list`.
    Q_INVOKABLE bool exportWithOptions(const QVariantMap &options);
    /// Writes the sheet: `kind` 0 PNG, 1 PDF, 2 SVG.
    Q_INVOKABLE bool exportTo(const QString &path, int kind, int paper, bool landscape,
                              qreal dpi);
    /// The papers the export offers, in the order the panel lists them.
    Q_INVOKABLE QStringList paperNames();
    /// The documents opened or saved recently, newest first: `path` and `name`.
    QVariantList recent() const { return m_recent; }
    /// Opens one of them by its place in the list.
    Q_INVOKABLE bool selectCircuit(int id);
    Q_INVOKABLE int addCircuit();
    Q_INVOKABLE bool renameCircuit(int id, const QString &title);
    Q_INVOKABLE QVariantList circuits() const;
    Q_INVOKABLE int activeCircuit() const;
    Q_INVOKABLE bool openRecent(int slot);
    /// Forgets every remembered document.
    Q_INVOKABLE void forgetRecent();
    /// Starts a new document and forgets the current file.
    Q_INVOKABLE void newDocument();

    /// `true` while the transport is playing. Pausing stops the sheet loop without
    /// touching the model, so resuming continues from the same instant.
    bool running() const { return m_running; }
    bool buzzerActive() const { return m_buzzerActive; }
    void setRunning(bool running);
    Q_INVOKABLE void toggleRunning();

    /// Simulated seconds per real second. Clamped to `0.05 ..= 8`.
    double speed() const { return m_speed; }
    void setSpeed(double speed);

    /// Simulated seconds one frame advances, before the speed is applied.
    ///
    /// A step-by-step advance uses this same number, so a step is exactly what playing
    /// would have done in one frame — the two modes cannot diverge.
    static constexpr double kFrameSeconds = 0.016;
    Q_PROPERTY(double frameSeconds READ frameSeconds CONSTANT)
    /// One frame of simulated time at the current speed.
    double frameSeconds() const { return kFrameSeconds; }

    /// Advances exactly one frame and leaves the transport paused, the way a simulator's
    /// step button behaves.
    Q_INVOKABLE void stepForward();

    /// Runs until the state changes: a piston reaches an end or a valve moves.
    ///
    /// The reference documents this as a transport mode (§3.2). It computes the run at
    /// once instead of animating it, and leaves the transport paused at the frame where
    /// the change happened. Returns `false` when four simulated seconds pass with no
    /// change.
    Q_INVOKABLE bool runUntilChange();

    /// `true` when the last frames left the state unchanged: the sheet loop can stop.
    ///
    /// Without this the window repaints a CPU-rasterised item sixty times a second
    /// for ever, and on a 144 Hz panel the compositor then recomposites every frame —
    /// which is how an idle simulator ends up looking like a busy GPU.
    bool settled() const { return m_settled; }

    QVariantList catalog() const { return m_catalog; }
    QString contrastSummary() const { return describe(m_contrast); }
    /// `true` when every pairing meets the criterion of `docs/ui-ux.md`.
    [[nodiscard]] bool themePasses() const { return m_contrast.passes(); }

    QColor background() const { return m_tokens.background; }
    QColor panel() const { return m_tokens.panel; }
    QColor foreground() const { return m_tokens.ink; }
    QColor muted() const { return m_tokens.inkMuted; }
    QColor accent() const { return m_tokens.accent; }

    /// Advances the model and republishes the telemetry.
    Q_INVOKABLE void advance(double dt);
    /// The gesture that actuates the valve, as the schematic expects it: held while the
    /// pointer or the key is down for a spring-return valve, latched for the others.
    Q_INVOKABLE void pressValve();
    Q_INVOKABLE void releaseValve();
    /// The same gesture on the valve of one placement: a sheet can hold more than one.
    /// Closes or opens the contact at one placement; false when it is not one.
    Q_INVOKABLE bool toggleContactAt(int index);
    Q_INVOKABLE void pressValveAt(int index);
    Q_INVOKABLE void releaseValveAt(int index);
    /// `-1` when that placement is not a valve.
    Q_INVOKABLE int valvePositionAt(int index) const;
    Q_INVOKABLE bool valveSpringReturnAt(int index) const;

    /// `true` when the valve returns to rest on release, so the front end holds it.
    bool valveSpringReturn() const { return m_valveSpringReturn; }

    bool editing() const { return m_editing; }

    /// Enters simulation and starts: the reference's Iniciar (F9).
    Q_INVOKABLE void startSimulation();
    /// Stays in simulation and stops the animation. From editing, it enters simulation
    /// without starting, which is how the reference lets settings be checked first.
    Q_INVOKABLE void pauseSimulation();
    /// Leaves simulation and goes back to editing: the reference's Detener (F5).
    Q_INVOKABLE void stopSimulation();

    int componentCount() const { return m_componentCount; }
    int selected() const { return m_selected; }
    bool canUndo() const { return m_canUndo; }
    bool canRedo() const { return m_canRedo; }

    /// Places a catalog entry at a point of the sheet, snapped by the core.
    Q_INVOKABLE int placeComponent(int catalogIndex, double xMm, double yMm);
    Q_INVOKABLE void moveComponent(int index, double xMm, double yMm);
    /// The same, as a drag: the core's grid first and then a wired port pulls onto the
    /// other end's axis, so a hose between parallel edges becomes a straight run.
    Q_INVOKABLE void dragComponent(int index, double xMm, double yMm);
    /// Moves one end of the selected hose onto the other's axis, in one undo.
    Q_INVOKABLE void straightenSelectedPipe();
    Q_INVOKABLE void removeComponent(int index);
    Q_INVOKABLE void removeSelected();
    /// Keeps the selected component for a later paste.
    Q_INVOKABLE void copySelected();
    /// Pastes at a point of the sheet, snapped by the core.
    Q_INVOKABLE int pasteAt(double xMm, double yMm);
    /// Pastes ten millimetres right and down from the copied component.
    Q_INVOKABLE void pasteOffset();
    /// Copies the selected component ten millimetres away and selects the copy.
    Q_INVOKABLE void duplicateSelected();
    /// Copies a placement displaced by a delta; the Shift+drag gesture.
    Q_INVOKABLE int duplicateComponent(int index, double dxMm, double dyMm);
    /// Turns the selected component a quarter; `anticlockwise` is the Ctrl and double
    /// click gesture of the reference, the plain command is the clockwise turn.
    Q_INVOKABLE void rotateSelected(bool anticlockwise);
    /// Reflects the selected component on its own vertical axis.
    Q_INVOKABLE void mirrorSelected();
    Q_INVOKABLE void selectPipe(int index);
    Q_INVOKABLE int pipeAt(double xMm, double yMm, double toleranceMm);
    /// Drags the middle segment of a pipe so it passes through a point.
    Q_INVOKABLE void routePipe(int index, double xMm, double yMm);
    Q_INVOKABLE void select(int index);
    /// Adds or removes one component from the selection: `Mayús` and a click.
    Q_INVOKABLE void toggleSelect(int index);
    /// Marks every component: `Ctrl+E`.
    Q_INVOKABLE void selectAll();
    /// Marks every component inside a rectangle of the sheet: the rubber band.
    Q_INVOKABLE void selectRect(double x0Mm, double y0Mm, double x1Mm, double y1Mm);
    /// How many components are selected.
    Q_INVOKABLE int selectionCount() const;
    /// One selected component by slot, or `-1`.
    Q_INVOKABLE int selectionAt(int slot) const;
    /// `true` when the placement is part of the selection.
    Q_INVOKABLE bool isSelected(int index) const;
    /// Moves the whole selection by a delta; one drag, one undo.
    Q_INVOKABLE void moveSelection(double rightMm, double downMm);
    /// Copies the whole selection in place and selects the copies.
    Q_INVOKABLE void duplicateSelection(double rightMm, double downMm);
    /// Removes every selected component, as one change.
    Q_INVOKABLE void removeSelection();
    /// Lines the selection up on the component furthest out: 0 left, 1 right, 2 top,
    /// 3 bottom.
    Q_INVOKABLE void alignSelection(int edge);
    /// Spreads the selection with equal gaps: `axis` 0 across, 1 down.
    Q_INVOKABLE void spaceSelection(int axis);
    /// Gives the selected pipe a line class: 0 working, 1 pilot, 2 drain.
    Q_INVOKABLE void setPipeClass(int lineClass);
    /// The selected pipe, or `-1`.
    int selectedPipe() const;
    Q_INVOKABLE int componentAt(double xMm, double yMm);

    /// The port nearest to a point, if one is within `toleranceMm`.
    bool portAt(double xMm, double yMm, double toleranceMm, int *index, QString *port) const;
    /// Splits a pipe at a point and joins `fromIndex:fromPort` to the junction it leaves.
    bool splicePipeAt(int pipeIndex, double xMm, double yMm, int fromIndex,
                      const QString &fromPort);

    /// Draws a pipe between two ports. `false` when the model refuses it.
    bool connectPorts(int fromIndex, const QString &fromPort, int toIndex, const QString &toPort);
    /// Where a port sits, in millimetres of the sheet.
    bool portPoint(int index, const QString &port, QPointF *point) const;
    int connectionCount() const;
    /// The view: how far the sheet is magnified and whether its grid shows. Both are
    /// view state, not document state, so neither touches the undo history.
    ///
    /// The limits and the step a button takes: a quarter per press doubles the size in
    /// five presses. Public because the sheet anchors one wheel notch to the pointer with
    /// the same step the buttons and the keys use.
    static constexpr double kMinZoom = 0.4;
    static constexpr double kMaxZoom = 6.0;
    static constexpr double kZoomStep = 1.25;
    double zoom() const { return m_zoom; }
    void setZoom(double zoom);
    bool grid() const { return m_grid; }
    void setGrid(bool on);
    Q_INVOKABLE void zoomIn();
    Q_INVOKABLE void zoomOut();
    Q_INVOKABLE void zoomReset();
    Q_INVOKABLE void toggleGrid();

    /// The sheet, as the document stores it. `sheetPaper` is the index the paper list
    /// uses, or `-1` for a custom size; the margins are the ISO 5457 frame by default.
    int sheetPaper() const;
    bool sheetLandscape() const { return m_sheet.landscape != 0; }
    QString sheetPaperName() const { return m_sheetPaperName; }
    double sheetWidthMm() const { return m_sheetWidthMm; }
    double sheetHeightMm() const { return m_sheetHeightMm; }
    double sheetMarginTop() const { return m_sheet.margin_top; }
    double sheetMarginRight() const { return m_sheet.margin_right; }
    double sheetMarginBottom() const { return m_sheet.margin_bottom; }
    double sheetMarginLeft() const { return m_sheet.margin_left; }
    /// One undoable document change each: the paper, the orientation or one margin.
    Q_INVOKABLE void setSheetPaper(int index);
    Q_INVOKABLE void setSheetLandscape(bool landscape);
    Q_INVOKABLE void toggleSheetLandscape();
    /// `side` is 0 top, 1 right, 2 bottom, 3 left, as the export panel reads them.
    Q_INVOKABLE void setSheetMargin(int side, double mm);
    /// Back to the A4 with the ISO 5457 frame a new document starts on.
    Q_INVOKABLE void resetSheet();

    Q_INVOKABLE void undo();
    Q_INVOKABLE void redo();

    bool inspectorOpen() const { return m_inspectorOpen; }
    QString inspectorText() const { return m_inspectorText; }
    QVariantList inspectorFields() const { return m_inspectorFields; }
    /// Opens the properties of one placement: what its definition and its own edits say.
    Q_INVOKABLE void inspectComponent(int index);
    /// Opens the properties of the selected component, or of the first valve.
    Q_INVOKABLE void inspectValve();
    /// Applies the edited fields (`key` and `value` per entry) as one undoable edit.
    Q_INVOKABLE void applyInspectorEdits(const QVariantList &values);
    /// The designation of one placement, empty when it has none.
    Q_INVOKABLE QString designationOf(int index) const;
    /// Names one placement; an empty string takes the name away.
    Q_INVOKABLE void setDesignation(int index, const QString &text);
    /// Binds the recorder in the inspector to one source, as one undoable edit.
    Q_INVOKABLE bool setChartSource(int placement, int quantity);
    /// Unbinds the recorder in the inspector.
    Q_INVOKABLE bool clearChartSource();
    /// The sources the recorder in the inspector can chart.
    QVariantList chartCandidates();
    /// The placement the recorder in the inspector charts, or `-1`.
    int chartSourcePlacement();
    /// Checks the circuit and opens the report. The faults are counted, not judged: an
    /// empty report is the only clean one.
    Q_INVOKABLE void checkCircuit();
    bool checkOpen() const { return m_checkOpen; }
    QString checkReport() const { return m_checkReport; }
    int checkCount() const { return m_checkCount; }
    Q_INVOKABLE void closeInspector();
    /// Back to t = 0.
    Q_INVOKABLE void restart();
    /// Reads the desktop palette into the core. Called on start and on theme change.
    Q_INVOKABLE void loadTheme();
    /// Uses `palette` instead of the application palette, for a forced light or dark run.
    void applyPalette(const QPalette &palette);

    /// Re-reads the palette when the desktop changes it, never per frame.
    bool event(QEvent *event) override;

signals:
    void changed();
    void viewChanged();
    void sheetChanged();
    void documentChanged();
    void recentChanged();
    void modeChanged();
    void inspectorChanged();
    void catalogChanged();
    void settledChanged();
    void runningChanged();
    void speedChanged();

    /// The valve moved: the spool has actually changed position, not merely been pressed.
    ///
    /// A spring-return valve returns on release, so one gesture produces two of these, and
    /// the front end plays the sample each time — that is the tick-tick a real one makes.
    void valveActuated();
    /// The piston reached an end of its stroke **and got there moving**.
    ///
    /// The reference plays `cylinder.wav` at the cylinder's end positions, and that is the
    /// sound a pneumatic circuit actually makes: the spool is quiet, the impact is not. A
    /// piston that creeps up to the end has nothing to report, so the arrival is only an
    /// event when it carries speed.
    void cylinderReachedEnd();
    /// The transport started or stopped, which is the front end's cue for the air.
    void transportStarted();
    void transportStopped();
    /// An audible indicator started or stopped sounding; the QML loop follows this.
    void buzzerChanged();

private:
    /// Republishes the telemetry. `forced` repaints even when nothing moved, which is
    /// what a single step must do.
    void refresh(bool forced = false);
    /// Announces an arrival at the end of the stroke, at most once per arrival.
    void announceImpact(const PfTelemetry &telemetry);
    /// Polls the buzzer state and announces a change, so the QML loop can start or stop.
    void refreshBuzzer();
    /// Republishes what the document reports, after any change to it.
    void refreshDocument();
    /// Re-reads the sheet from the core; `true` when it changed.
    bool refreshSheet();
    /// Applies a sheet, keeping the cached copy in step with what the core stored.
    void applySheet(const PfSheet &sheet);
    /// Where the list of remembered documents lives, beside the safety copy.
    QString recentPath() const;
    /// Re-reads the list from the core into what the ribbon shows.
    void refreshRecent();
    /// Writes the safety copy and announces whether one exists.
    void writeAutosave();
    /// Republishes the selection alone: marking is not a change to the document.
    void refreshSelection();
    void advanceInternal(double dt, bool forced);
    void readCatalog();

    PfSession *m_session = nullptr;
    PfTelemetry m_telemetry {};
    QString m_error;
    QString m_currentPath;
    bool m_modified = false;
    bool m_recoveryOffered = false;
    bool m_closeConfirmed = false;
    QVariantList m_recent;
    bool m_valveSpringReturn = true;
    /// The program starts in edit mode, as the reference does: nothing moves and a click on
    /// a valve selects it. Simulation is entered deliberately, with Iniciar or Pausa.
    bool m_editing = true;
    int m_componentCount = 0;
    int m_selected = -1;
    bool m_canUndo = false;
    bool m_canRedo = false;
    bool m_inspectorOpen = false;
    double m_zoom = 1.0;
    bool m_grid = true;
    /// The sheet as the core reported it, and the label and size the interface reads.
    PfSheet m_sheet {};
    QString m_sheetPaperName;
    double m_sheetWidthMm = 210.0;
    double m_sheetHeightMm = 297.0;
    bool m_checkOpen = false;
    QString m_checkReport;
    int m_checkCount = 0;
    QString m_inspectorText;
    QVariantList m_inspectorFields;
    int m_inspectorIndex = -1;
    QVariantList m_catalog;
    ThemeTokens m_tokens;
    Contrast m_contrast;
    bool m_settled = false;
    bool m_running = false;
    bool m_buzzerActive = false;
    double m_speed = 1.0;
    int m_steadyFrames = 0;
    /// How fast the piston was moving on the last frame. The solver clamps the velocity to
    /// zero in the same step that stops the piston, so the speed that carried it into the
    /// stop — the impact energy — only exists on the frame before.
    double m_lastVelocity = 0.0;
    /// How far the piston was on the last frame, so an arrival can be recognised at all.
    double m_lastExtension = 0.0;
    /// Which end the piston last announced, so one impact is one sound rather than sixty.
    int m_lastEnd = -1;
    /// When the piston was last seen away from that end, in simulated seconds.
    ///
    /// The model can rebound a few millimetres off the stop before the chamber charges and
    /// it settles there, which is several arrivals a few milliseconds apart. To a listener
    /// that is one hit, so the arrival is only news again after the piston has been away
    /// from that end for longer than a rebound lasts.
    double m_awaySince = 0.0;
    /// Whether the piston was at an end of its stroke on the frame before.
    bool m_wasAtEnd = false;
};
