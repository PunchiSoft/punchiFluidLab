// SPDX-FileCopyrightText: 2026 PunchiSoft
// SPDX-License-Identifier: GPL-3.0-or-later
//
// The Página group of the Disposición tab: the size and the orientation, side by side,
// the way a page is chosen before anything is drawn on it.

import QtQuick
import PunchiFluid

Row {
    id: pageControls

    property FluidSession session
    property bool openOnCreate: false
    signal pageSetupRequested(string section)

    spacing: 10

    PaperCombo {
        objectName: "paperCombo"
        session: pageControls.session
        openOnCreate: pageControls.openOnCreate
        onPageSetupRequested: function (section) {
            pageControls.pageSetupRequested(section);
        }
    }

    OrientationControl {
        session: pageControls.session
    }
}
