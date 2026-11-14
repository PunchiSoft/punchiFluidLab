# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The front end. The drawing, the model and the format live in Rust; this builds the
# window that drives them through the C ABI of `punchi-fluid-ffi`.
#
#   cmake -S apps/punchi-fluid-qt -B apps/punchi-fluid-qt/build -G Ninja
#   cmake --build apps/punchi-fluid-qt/build
#   apps/punchi-fluid-qt/build/punchi-fluid
#
# The Rust core is linked as a static library, so the result is one binary with no
# runtime search path to get wrong.

cmake_minimum_required(VERSION 3.21)
project(punchi-fluid LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Core Gui Qml Quick)

get_filename_component(REPO_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
set(RUST_STATIC "${REPO_ROOT}/target/release/libpunchi_fluid.a")

add_custom_target(rust_core
    COMMAND cargo build --release -p punchi-fluid-ffi
    WORKING_DIRECTORY "${REPO_ROOT}"
    BYPRODUCTS "${RUST_STATIC}"
    COMMENT "Building the Rust core (release)"
    VERBATIM
)

qt_add_executable(punchi-fluid
    src/main.cpp
    src/session.cpp
    src/session.h
    src/canvasitem.cpp
    src/sheetframe.cpp
    src/canvasitem.h
    src/previewitem.cpp
    src/previewitem.h
    src/replay.cpp
    src/replay.h
    src/icons.cpp
    src/icons.h
    src/sounds.cpp
    src/sounds.h
    src/theme.cpp
    src/theme.h
)

add_dependencies(punchi-fluid rust_core)

qt_add_qml_module(punchi-fluid
    URI PunchiFluid
    VERSION 1.0
    QML_FILES
        GridControl.qml
        HelpLink.qml
        Main.qml
        MarginGallery.qml
        OrientationControl.qml
        PageControls.qml
        PageSetupPanel.qml
        PanelButton.qml
        PaperCombo.qml
        PaperPreview.qml
        RibbonBar.qml
        RibbonButton.qml
        RibbonGroup.qml
        Sidebar.qml
        SoundFeedback.qml
        StatusBar.qml
)

# The licence travels inside the binary too: the About popup shows the whole GPL text, and
# a copy installed without the repository still carries the licence it is distributed
# under. The alias is `/LICENSE`, which `FluidSession::licenseText` reads once.
qt_add_resources(punchi-fluid "license"
    PREFIX "/"
    BASE "${REPO_ROOT}"
    FILES "${REPO_ROOT}/LICENSE"
)

# The two event samples travel inside the binary, the way the catalog does, so an
# installation is still one file and there is no data path to get wrong. `SoundLibrary`
# prefers a file of the same name in $XDG_DATA_HOME/punchi-fluid-lab/sounds/ over these.
#
# Playing them needs the Qt Multimedia module at run time — it is imported by
# SoundFeedback.qml, not linked here, so the build only needs the QML module present in the
# Qt installation.
qt_add_resources(punchi-fluid "sounds"
    PREFIX "/sounds"
    BASE "${REPO_ROOT}/assets/sounds"
    FILES
        "${REPO_ROOT}/assets/sounds/valve.wav"
        "${REPO_ROOT}/assets/sounds/air.wav"
        "${REPO_ROOT}/assets/sounds/cylinder.wav"
        "${REPO_ROOT}/assets/sounds/buzzer.wav"
)

target_include_directories(punchi-fluid PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/src"
    "${REPO_ROOT}/crates/punchi-fluid-ffi/include"
)

target_compile_options(punchi-fluid PRIVATE -Wall -Wextra)

# A Rust static library still needs the system libraries its own std and the solver use.
target_link_libraries(punchi-fluid PRIVATE
    Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick
    "${RUST_STATIC}" m dl pthread
)
