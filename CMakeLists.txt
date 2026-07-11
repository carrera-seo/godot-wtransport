cmake_minimum_required(VERSION 3.28)
project(godot_wtransport LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(GWT_BUILD_FFI_HARNESS "Build the C ABI lifecycle harness" ON)
option(GWT_BUILD_EXTENSION "Build the Godot GDExtension" ON)
set(GWT_GODOT_CPP_SOURCE "" CACHE PATH "Use an existing godot-cpp checkout")

if(GWT_BUILD_EXTENSION)
    include(FetchContent)
    set(GODOTCPP_BUILD_PROFILE "${CMAKE_SOURCE_DIR}/cmake/godot-cpp-profile.json" CACHE PATH "godot-cpp build profile" FORCE)
    set(GODOTCPP_USE_HOT_RELOAD ON CACHE BOOL "Enable editor hot reload" FORCE)
    if(GWT_GODOT_CPP_SOURCE)
        add_subdirectory("${GWT_GODOT_CPP_SOURCE}" "${CMAKE_BINARY_DIR}/godot-cpp")
    else()
        FetchContent_Declare(
            godot_cpp
            GIT_REPOSITORY https://github.com/godotengine/godot-cpp.git
            GIT_TAG e83fd0904c13356ed1d4c3d09f8bb9132bdc6b77
            GIT_SHALLOW TRUE
        )
        FetchContent_MakeAvailable(godot_cpp)
    endif()

    add_custom_target(
        gwt_rust_ffi
        COMMAND cargo build --release -p godot-wtransport-ffi
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        BYPRODUCTS "${CMAKE_SOURCE_DIR}/target/release/libgodot_wtransport_ffi.a"
    )
    add_library(godot_wtransport SHARED extension/src/register_types.cpp extension/src/webtransport.cpp)
    add_dependencies(godot_wtransport gwt_rust_ffi)
    target_include_directories(godot_wtransport PRIVATE extension/include rust/ffi/include)
    target_compile_options(godot_wtransport PRIVATE
        -Wall -Wextra -Werror
        -Wno-unused-parameter -Wno-gnu-anonymous-struct -Wno-nested-anon-types
    )
    target_link_libraries(
        godot_wtransport PRIVATE godot-cpp
        "${CMAKE_SOURCE_DIR}/target/release/libgodot_wtransport_ffi.a"
        "-framework Security" "-framework CoreFoundation" "-framework SystemConfiguration" resolv
    )
    set_target_properties(godot_wtransport PROPERTIES
        OUTPUT_NAME "godot_wtransport.macos.debug.arm64"
        LIBRARY_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/demo/addons/godot_wtransport/bin"
    )
endif()

if(GWT_BUILD_FFI_HARNESS)
    add_executable(gwt_ffi_lifecycle tests/ffi/lifecycle.c)
    add_executable(gwt_ffi_integration tests/ffi/integration.c)
    target_include_directories(gwt_ffi_lifecycle PRIVATE rust/ffi/include)
    target_include_directories(gwt_ffi_integration PRIVATE rust/ffi/include)
    target_link_libraries(
        gwt_ffi_lifecycle
        PRIVATE
        "${CMAKE_SOURCE_DIR}/target/release/libgodot_wtransport_ffi.a"
        "-framework Security"
        "-framework CoreFoundation"
        "-framework SystemConfiguration"
        resolv
    )
    target_link_libraries(
        gwt_ffi_integration
        PRIVATE
        "${CMAKE_SOURCE_DIR}/target/release/libgodot_wtransport_ffi.a"
        "-framework Security"
        "-framework CoreFoundation"
        "-framework SystemConfiguration"
        resolv
    )
endif()
