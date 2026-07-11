cmake_minimum_required(VERSION 3.28)
project(godot_wtransport LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(GWT_BUILD_FFI_HARNESS "Build the C ABI lifecycle harness" ON)

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
