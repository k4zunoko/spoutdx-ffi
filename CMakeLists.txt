cmake_minimum_required(VERSION 3.25)

project(spoutdx_ffi
  VERSION 0.1.0
  LANGUAGES CXX
)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

# Spout source code root
set(SPOUT_SOURCE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}/third_party/Spout2/SPOUTSDK" CACHE PATH "Path to Spout2/SPOUTSDK source directory")

# Verify Spout source exists
if (NOT EXISTS "${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutCommon.h")
  message(FATAL_ERROR "Spout source not found at: ${SPOUT_SOURCE_ROOT}\nPlease place Spout2/SPOUTSDK in third_party/Spout2/")
endif()

# Spout source files
set(SPOUT_SOURCES
  # SpoutGL core files
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutCopy.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutDirectX.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutFrameCount.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutSenderNames.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutSharedMemory.cpp
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutUtils.cpp
  # SpoutDX
  ${SPOUT_SOURCE_ROOT}/SpoutDirectX/SpoutDX/SpoutDX.cpp
)

set(SPOUT_HEADERS
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutCommon.h
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutCopy.h
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutDirectX.h
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutFrameCount.h
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutSenderNames.h
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutSharedMemory.h
  ${SPOUT_SOURCE_ROOT}/SpoutGL/SpoutUtils.h
  ${SPOUT_SOURCE_ROOT}/SpoutDirectX/SpoutDX/SpoutDX.h
)

# Library (C ABI shim built from Spout source)
add_library(spoutdx_ffi SHARED
  src/spoutdx_ffi.cpp
  ${SPOUT_SOURCES}
)

target_include_directories(spoutdx_ffi
  PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
  PRIVATE
    ${SPOUT_SOURCE_ROOT}/SpoutGL
    ${SPOUT_SOURCE_ROOT}/SpoutDirectX/SpoutDX
)

# DirectX and Windows libs
if (WIN32)
  target_compile_definitions(spoutdx_ffi PRIVATE 
    NOMINMAX
    SPOUT_BUILD_DLL  # For SPOUT_DLLEXP
  )
  
  target_link_libraries(spoutdx_ffi PRIVATE 
    d3d11 
    dxgi
    psapi      # For GetModuleFileNameEx (SpoutUtils)
    version    # For version resources (SpoutUtils)
    winmm      # For timeGetTime/timeBeginPeriod/timeEndPeriod (SpoutFrameCount)
  )
endif()

