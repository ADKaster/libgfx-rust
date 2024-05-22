cmake_minimum_required(VERSION 3.25)

project(LibGfxExample
    VERSION 1.0
    DESCRIPTION "Example of using LibGfx"
    LANGUAGES CXX
)

include(FetchContent)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5 # Optionally specify a commit hash, version tag or branch here
)
FetchContent_MakeAvailable(Corrosion)

find_package(SDL2 REQUIRED)

# Import targets defined in a package or workspace manifest `Cargo.toml` file
corrosion_import_crate(MANIFEST_PATH libgfx-rust/Cargo.toml
    CRATE_TYPES staticlib
    FLAGS --crate-type=staticlib
)
corrosion_set_env_vars(libgfx_rust EXTERNAL_CBINDGEN_OUTPUT_PATH=${CMAKE_CURRENT_BINARY_DIR}/generated)
target_include_directories(libgfx_rust INTERFACE ${CMAKE_CURRENT_BINARY_DIR}/generated)

add_executable(example cpp/example.cpp)
target_link_libraries(example PUBLIC libgfx_rust SDL2::SDL2)
