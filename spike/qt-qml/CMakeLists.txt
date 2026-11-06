# SPDX-FileCopyrightText: 2026 PunchiSoft
# SPDX-License-Identifier: GPL-3.0-or-later
#
# The two-day spike that decides ADR 0005: can a Qt Quick front end drive the Rust core
# through the C ABI, draw the catalog symbols with QPainter, take a click on the valve
# and get its colours from the desktop theme?
#
#   cmake -S spike/qt-qml -B spike/qt-qml/build -G Ninja
#   cmake --build spike/qt-qml/build
#   QT_QPA_PLATFORM=offscreen spike/qt-qml/build/punchi-fluid-qt --grab shot.png
#
# The Rust core is linked as a static library, so the result is one binary with no
# runtime search path to get wrong.

cmake_minimum_required(VERSION 3.21)
project(punchi-fluid-qt LANGUAGES CXX)

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

qt_add_executable(punchi-fluid-qt
    src/main.cpp
    src/session.cpp
    src/session.h
    src/canvasitem.cpp
    src/canvasitem.h
)

add_dependencies(punchi-fluid-qt rust_core)

qt_add_qml_module(punchi-fluid-qt
    URI PunchiFluid
    VERSION 1.0
    QML_FILES Main.qml
)

target_include_directories(punchi-fluid-qt PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}/src"
    "${REPO_ROOT}/crates/punchi-fluid-ffi/include"
)

# A Rust static library still needs the system libraries its own std and the solver use.
target_link_libraries(punchi-fluid-qt PRIVATE
    Qt6::Core Qt6::Gui Qt6::Qml Qt6::Quick
    "${RUST_STATIC}" m dl pthread
)
