cmake_minimum_required(VERSION 3.28)
project(godot_wtransport LANGUAGES C CXX)

if(MSVC)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreadedDLL")
endif()

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(GWT_BUILD_FFI_HARNESS "Build the C ABI lifecycle harness" ON)
option(GWT_BUILD_EXTENSION "Build the Godot GDExtension" ON)
option(GWT_ENABLE_INSECURE_TESTING "Expose certificate bypass in debug builds" OFF)
set(GWT_GODOT_CPP_SOURCE "" CACHE PATH "Use an existing godot-cpp checkout")
set(GWT_RUST_TARGET "" CACHE STRING "Optional Rust target triple")
set(GWT_RUST_LINKER_ENV "" CACHE STRING "Optional Cargo target linker environment assignment")
set(GWT_OUTPUT_ARCH "" CACHE STRING "Architecture tag used in artifact filenames")
set(GWT_MACOS_DEPLOYMENT_TARGET "12.0" CACHE STRING "Minimum supported macOS version")
if(APPLE AND NOT CMAKE_SYSTEM_NAME STREQUAL "iOS")
    set(CMAKE_OSX_DEPLOYMENT_TARGET "${GWT_MACOS_DEPLOYMENT_TARGET}" CACHE STRING "Minimum macOS version" FORCE)
endif()

if(GWT_BUILD_EXTENSION)
    include(FetchContent)
    set(GODOTCPP_BUILD_PROFILE "${CMAKE_SOURCE_DIR}/cmake/godot-cpp-profile.json" CACHE PATH "godot-cpp build profile" FORCE)
    set(GODOTCPP_USE_HOT_RELOAD ON CACHE BOOL "Enable editor hot reload" FORCE)
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        set(GODOTCPP_TARGET "template_debug" CACHE STRING "godot-cpp target" FORCE)
    else()
        set(GODOTCPP_TARGET "template_release" CACHE STRING "godot-cpp target" FORCE)
    endif()
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

    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        set(GWT_RUST_PROFILE_DIR "debug")
        set(GWT_RUST_BUILD_ARGS build -p godot-wtransport-ffi)
    else()
        set(GWT_RUST_PROFILE_DIR "release")
        set(GWT_RUST_BUILD_ARGS build --release -p godot-wtransport-ffi)
    endif()
    if(GWT_ENABLE_INSECURE_TESTING)
        if(NOT CMAKE_BUILD_TYPE STREQUAL "Debug")
            message(FATAL_ERROR "GWT_ENABLE_INSECURE_TESTING is only permitted in Debug builds")
        endif()
        list(APPEND GWT_RUST_BUILD_ARGS --features godot-wtransport-ffi/dangerous-insecure)
    endif()
    if(GWT_RUST_TARGET)
        list(APPEND GWT_RUST_BUILD_ARGS --target ${GWT_RUST_TARGET})
        set(GWT_RUST_OUTPUT_DIR "${CMAKE_SOURCE_DIR}/target/${GWT_RUST_TARGET}/${GWT_RUST_PROFILE_DIR}")
    else()
        set(GWT_RUST_OUTPUT_DIR "${CMAKE_SOURCE_DIR}/target/${GWT_RUST_PROFILE_DIR}")
    endif()
    if(WIN32)
        set(GWT_RUST_FFI_LIBRARY "${GWT_RUST_OUTPUT_DIR}/godot_wtransport_ffi.lib")
    else()
        set(GWT_RUST_FFI_LIBRARY "${GWT_RUST_OUTPUT_DIR}/libgodot_wtransport_ffi.a")
    endif()

    if(GWT_RUST_LINKER_ENV)
        set(GWT_RUST_BUILD_COMMAND ${CMAKE_COMMAND} -E env
            ${GWT_RUST_LINKER_ENV} cargo ${GWT_RUST_BUILD_ARGS})
    elseif(CMAKE_SYSTEM_NAME STREQUAL "iOS")
        set(GWT_RUST_BUILD_COMMAND ${CMAKE_COMMAND} -E env
            IPHONEOS_DEPLOYMENT_TARGET=${CMAKE_OSX_DEPLOYMENT_TARGET} cargo ${GWT_RUST_BUILD_ARGS})
    elseif(APPLE)
        set(GWT_RUST_BUILD_COMMAND ${CMAKE_COMMAND} -E env
            MACOSX_DEPLOYMENT_TARGET=${GWT_MACOS_DEPLOYMENT_TARGET} cargo ${GWT_RUST_BUILD_ARGS})
    else()
        set(GWT_RUST_BUILD_COMMAND cargo ${GWT_RUST_BUILD_ARGS})
    endif()
    add_custom_target(
        gwt_rust_ffi
        COMMAND ${GWT_RUST_BUILD_COMMAND}
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        BYPRODUCTS "${GWT_RUST_FFI_LIBRARY}"
    )
    if(CMAKE_SYSTEM_NAME STREQUAL "iOS")
        add_library(godot_wtransport STATIC extension/src/register_types.cpp extension/src/webtransport.cpp)
    else()
        add_library(godot_wtransport SHARED extension/src/register_types.cpp extension/src/webtransport.cpp)
    endif()
    add_dependencies(godot_wtransport gwt_rust_ffi)
    target_include_directories(godot_wtransport PRIVATE extension/include rust/ffi/include)
    if(MSVC)
        target_compile_options(godot_wtransport PRIVATE
            /W4 /WX /wd4100 /external:anglebrackets /external:W0
        )
    else()
        target_compile_options(godot_wtransport PRIVATE
            -Wall -Wextra -Werror -Wno-unused-parameter
        )
        if(CMAKE_CXX_COMPILER_ID MATCHES "Clang")
            target_compile_options(godot_wtransport PRIVATE
                -Wno-gnu-anonymous-struct -Wno-nested-anon-types
            )
        elseif(CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
            target_compile_options(godot_wtransport PRIVATE
                -Wno-unused-but-set-parameter
            )
        endif()
    endif()
    if(GWT_ENABLE_INSECURE_TESTING)
        target_compile_definitions(godot_wtransport PRIVATE GWT_ENABLE_INSECURE=1)
    endif()
    target_link_libraries(godot_wtransport PRIVATE godot-cpp "${GWT_RUST_FFI_LIBRARY}")
    if(ANDROID)
        target_link_libraries(godot_wtransport PRIVATE android log dl m)
        set(GWT_PLATFORM_NAME "android")
    elseif(CMAKE_SYSTEM_NAME STREQUAL "iOS")
        target_link_libraries(godot_wtransport PRIVATE
            "-framework Security" "-framework CoreFoundation" "-framework SystemConfiguration"
        )
        set(GWT_PLATFORM_NAME "ios")
    elseif(APPLE)
        target_link_libraries(godot_wtransport PRIVATE
            "-framework Security" "-framework CoreFoundation" "-framework SystemConfiguration" resolv
        )
        set(GWT_PLATFORM_NAME "macos")
    elseif(WIN32)
        target_link_libraries(godot_wtransport PRIVATE
            ws2_32 bcrypt crypt32 ncrypt secur32 ntdll userenv
        )
        set(GWT_PLATFORM_NAME "windows")
    elseif(UNIX)
        target_link_libraries(godot_wtransport PRIVATE pthread dl m)
        set(GWT_PLATFORM_NAME "linux")
    else()
        message(FATAL_ERROR "Unsupported desktop platform")
    endif()
    if(CMAKE_BUILD_TYPE STREQUAL "Debug")
        set(GWT_BUILD_FLAVOR "debug")
    else()
        set(GWT_BUILD_FLAVOR "release")
    endif()
    if(NOT GWT_OUTPUT_ARCH)
        if(CMAKE_SYSTEM_PROCESSOR MATCHES "^(arm64|aarch64)$")
            set(GWT_OUTPUT_ARCH "arm64")
        else()
            set(GWT_OUTPUT_ARCH "x86_64")
        endif()
    endif()

    set_target_properties(godot_wtransport PROPERTIES
        OUTPUT_NAME "godot_wtransport.${GWT_PLATFORM_NAME}.${GWT_BUILD_FLAVOR}.${GWT_OUTPUT_ARCH}"
        LIBRARY_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/demo/addons/godot_wtransport/bin"
        ARCHIVE_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/mobile/ios/lib"
        RUNTIME_OUTPUT_DIRECTORY "${CMAKE_SOURCE_DIR}/demo/addons/godot_wtransport/bin"
        RUNTIME_OUTPUT_DIRECTORY_DEBUG "${CMAKE_SOURCE_DIR}/demo/addons/godot_wtransport/bin"
        RUNTIME_OUTPUT_DIRECTORY_RELEASE "${CMAKE_SOURCE_DIR}/demo/addons/godot_wtransport/bin"
    )
endif()

if(GWT_BUILD_FFI_HARNESS)
    if(NOT APPLE)
        message(FATAL_ERROR "The Phase 1 FFI harness currently supports macOS only")
    endif()
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
