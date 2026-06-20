cmake_minimum_required(VERSION 3.24)
project(verium_qt LANGUAGES CXX)

# ---------------------------------------------------------------------------
# CMake + Corrosion integration (optional packaging / IDE path).
#
# The primary, validated build is Cargo-driven (`cargo build`), where
# cxx-qt-build compiles the Rust QObjects + QML and links Qt. This CMake file
# wraps that build with Corrosion so teams can drive it from CMake-based IDEs
# and CI, and so install()/CPack rules can hang off the produced target.
# ---------------------------------------------------------------------------

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Qt6 REQUIRED COMPONENTS Core Gui Qml Quick QuickControls2)

include(FetchContent)
FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5.1
)
FetchContent_MakeAvailable(Corrosion)

# Build the `verium-qt` binary crate via Cargo.
corrosion_import_crate(MANIFEST_PATH Cargo.toml)

# The executable produced by Cargo is exposed as the imported target `verium-qt`.
# Deployment (macdeployqt / windeployqt) + installers live in packaging/.
install(IMPORTED_RUNTIME_ARTIFACTS verium-qt
        RUNTIME DESTINATION bin
        OPTIONAL)
