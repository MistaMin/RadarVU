cmake_minimum_required(VERSION 3.22)
# These must be set before project(): the compiler flags are fixed there.
# macOS release builds: -DRADARVU_UNIVERSAL=ON builds arm64 + x86_64 (the Rust library too).
option(RADARVU_UNIVERSAL "Build arm64 + x86_64 on macOS" OFF)
if(APPLE AND RADARVU_UNIVERSAL)
    set(CMAKE_OSX_ARCHITECTURES "arm64;x86_64" CACHE STRING "" FORCE)
endif()
set(CMAKE_OSX_DEPLOYMENT_TARGET "11.0" CACHE STRING "Minimum macOS version")
project(RadarVU VERSION 1.0.0)
set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# JUCE: a local checkout next to this repository (the ArcVu / audiogridder layout).
set(JUCE_DIR "${CMAKE_SOURCE_DIR}/../audiogridder/JUCE" CACHE PATH "JUCE checkout")
add_subdirectory(${JUCE_DIR} ${CMAKE_BINARY_DIR}/juce EXCLUDE_FROM_ALL)

# The loudness detector and the radar drawing list are a Rust static library.
find_program(RADARVU_CARGO cargo HINTS "$ENV{HOME}/.cargo/bin" "/opt/homebrew/bin" "/usr/local/bin")
if(NOT RADARVU_CARGO)
    message(FATAL_ERROR "cargo (Rust) is required to build the RadarVU detector")
endif()
if(WIN32)
    set(RADARVU_LIB_NAME radarvu_dsp.lib)
else()
    set(RADARVU_LIB_NAME libradarvu_dsp.a)
endif()
set(RADARVU_LIB ${CMAKE_BINARY_DIR}/${RADARVU_LIB_NAME})
set(RADARVU_RUST_SOURCES ${CMAKE_SOURCE_DIR}/dsp/src/lib.rs ${CMAKE_SOURCE_DIR}/dsp/src/loudness.rs ${CMAKE_SOURCE_DIR}/dsp/src/extra.rs ${CMAKE_SOURCE_DIR}/dsp/src/skin.rs ${CMAKE_SOURCE_DIR}/dsp/Cargo.toml)
if(APPLE AND RADARVU_UNIVERSAL)
    add_custom_command(
        OUTPUT ${RADARVU_LIB}
        COMMAND ${RADARVU_CARGO} build --release --target aarch64-apple-darwin --manifest-path ${CMAKE_SOURCE_DIR}/dsp/Cargo.toml
                --target-dir ${CMAKE_BINARY_DIR}/dsp-target
        COMMAND ${RADARVU_CARGO} build --release --target x86_64-apple-darwin --manifest-path ${CMAKE_SOURCE_DIR}/dsp/Cargo.toml
                --target-dir ${CMAKE_BINARY_DIR}/dsp-target
        COMMAND lipo -create -output ${RADARVU_LIB}
                ${CMAKE_BINARY_DIR}/dsp-target/aarch64-apple-darwin/release/libradarvu_dsp.a
                ${CMAKE_BINARY_DIR}/dsp-target/x86_64-apple-darwin/release/libradarvu_dsp.a
        DEPENDS ${RADARVU_RUST_SOURCES}
        COMMENT "Building RadarVU Rust DSP (arm64 + x86_64)")
else()
    add_custom_command(
        OUTPUT ${RADARVU_LIB}
        COMMAND ${RADARVU_CARGO} build --release --manifest-path ${CMAKE_SOURCE_DIR}/dsp/Cargo.toml
                --target-dir ${CMAKE_BINARY_DIR}/dsp-target
        COMMAND ${CMAKE_COMMAND} -E copy ${CMAKE_BINARY_DIR}/dsp-target/release/${RADARVU_LIB_NAME} ${RADARVU_LIB}
        DEPENDS ${RADARVU_RUST_SOURCES}
        COMMENT "Building RadarVU Rust DSP")
endif()
add_custom_target(radarvu_dsp DEPENDS ${RADARVU_LIB})

# VST3 and Standalone everywhere, AU on macOS.
set(RADARVU_FORMATS VST3 LV2 Standalone)
if(APPLE)
    list(APPEND RADARVU_FORMATS AU)
endif()
# AAX is built when the Avid AAX SDK is present (it is not distributed with the source). An AAX bundle
# is only ever released after PACE signing.
set(RADARVU_AAX_SDK_PATH "${CMAKE_CURRENT_SOURCE_DIR}/../aax-sdk-2-8-1" CACHE PATH "Avid AAX SDK root (optional)")
option(RADARVU_BUILD_AAX "Build the AAX format when the SDK is available" ON)
if(RADARVU_BUILD_AAX AND (APPLE OR WIN32) AND EXISTS "${RADARVU_AAX_SDK_PATH}/Interfaces/AAX.h")
    juce_set_aax_sdk_path("${RADARVU_AAX_SDK_PATH}")
    list(APPEND RADARVU_FORMATS AAX)
else()
    message(STATUS "AAX SDK not used")
endif()
message(STATUS "RadarVU formats: ${RADARVU_FORMATS}")

juce_add_plugin(RadarVU
    COMPANY_NAME "OpenGrid"
    BUNDLE_ID com.opengrid.radarvu
    ICON_BIG "${CMAKE_CURRENT_SOURCE_DIR}/Resources/icon-1024.png"
    ICON_SMALL "${CMAKE_CURRENT_SOURCE_DIR}/Resources/icon-128.png"
    IS_SYNTH FALSE
    NEEDS_MIDI_INPUT FALSE
    NEEDS_MIDI_OUTPUT FALSE
    IS_MIDI_EFFECT FALSE
    EDITOR_WANTS_KEYBOARD_FOCUS FALSE
    COPY_PLUGIN_AFTER_BUILD FALSE
    PLUGIN_MANUFACTURER_CODE OpGr
    PLUGIN_CODE RdVU
    FORMATS ${RADARVU_FORMATS}
    DESCRIPTION "BS.1770-5 loudness meter with a radar display"
    VST3_CATEGORIES Fx Analyzer
    AU_MAIN_TYPE kAudioUnitType_Effect
    MICROPHONE_PERMISSION_ENABLED TRUE
    MICROPHONE_PERMISSION_TEXT "RadarVU uses audio input to meter loudness in the standalone application."
    HARDENED_RUNTIME_ENABLED TRUE
    HARDENED_RUNTIME_OPTIONS com.apple.security.device.audio-input
    LV2URI "urn:opengrid:radarvu"
    AAX_IDENTIFIER com.opengrid.radarvu
    AAX_CATEGORY AAX_ePlugInCategory_SoundField
    PRODUCT_NAME "RadarVU")

option(RADARVU_BUILD_CLAP "Build the CLAP plugin" ON)
if(RADARVU_BUILD_CLAP)
    set(RADARVU_CLAP_EXT_SOURCE "${CMAKE_SOURCE_DIR}/../clap-juce-extensions" CACHE PATH "An existing clap-juce-extensions checkout")
    include(FetchContent)
    if(EXISTS "${RADARVU_CLAP_EXT_SOURCE}/CMakeLists.txt")
        set(FETCHCONTENT_SOURCE_DIR_CLAP-JUCE-EXTENSIONS "${RADARVU_CLAP_EXT_SOURCE}")
    endif()
    FetchContent_Declare(
        clap-juce-extensions
        GIT_REPOSITORY https://github.com/free-audio/clap-juce-extensions.git
        # Pinned commit on main (the last tag predates JUCE 8).
        GIT_TAG        55525c9858d4b25687be7759a5e0f70eccef218e
        GIT_SUBMODULES_RECURSE TRUE)
    FetchContent_MakeAvailable(clap-juce-extensions)
endif()

juce_generate_juce_header(RadarVU)
target_sources(RadarVU PRIVATE
    plugin/PluginProcessor.cpp
    plugin/PluginEditor.cpp)
target_include_directories(RadarVU PRIVATE plugin)
# The AAX page table (control-surface layout) is read from the bundle's Contents/Resources.
if(TARGET RadarVU_AAX)
    add_custom_command(TARGET RadarVU_AAX POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E make_directory "$<TARGET_FILE_DIR:RadarVU_AAX>/../Resources"
        COMMAND ${CMAKE_COMMAND} -E copy "${CMAKE_CURRENT_SOURCE_DIR}/Resources/RadarVU.xml" "$<TARGET_FILE_DIR:RadarVU_AAX>/../Resources/RadarVU.xml")
endif()
target_compile_definitions(RadarVU PUBLIC
    JUCE_WEB_BROWSER=0
    JUCE_USE_CURL=0
    JUCE_VST3_CAN_REPLACE_VST2=0)
target_link_libraries(RadarVU PRIVATE
    ${RADARVU_LIB}
    juce::juce_audio_utils
    juce::juce_recommended_config_flags
    juce::juce_recommended_warning_flags)
if(WIN32)
    target_link_libraries(RadarVU PRIVATE ws2_32 userenv ntdll bcrypt advapi32)
elseif(UNIX AND NOT APPLE)
    target_link_libraries(RadarVU PRIVATE pthread dl m)
endif()
add_dependencies(RadarVU radarvu_dsp)

if(RADARVU_BUILD_CLAP)
    clap_juce_extensions_plugin(TARGET RadarVU
        CLAP_ID "com.opengrid.radarvu"
        CLAP_FEATURES audio-effect analyzer utility stereo
        CLAP_MANUAL_URL ""
        CLAP_SUPPORT_URL "")
endif()

option(RADARVU_BUILD_QA "Build the offscreen UI audit harness" OFF)
if(RADARVU_BUILD_QA)
    juce_add_console_app(RadarVU_QA PRODUCT_NAME "RadarVU QA")
    juce_generate_juce_header(RadarVU_QA)
    target_sources(RadarVU_QA PRIVATE qa/UiAudit.cpp plugin/PluginProcessor.cpp plugin/PluginEditor.cpp)
    target_include_directories(RadarVU_QA PRIVATE plugin)
    target_compile_definitions(RadarVU_QA PRIVATE JUCE_WEB_BROWSER=0 JUCE_USE_CURL=0 JUCE_VST3_CAN_REPLACE_VST2=0 JUCE_PLUGINHOST_VST3=1)
    target_link_libraries(RadarVU_QA PRIVATE ${RADARVU_LIB} juce::juce_audio_utils juce::juce_recommended_config_flags)
    if(WIN32)
        target_link_libraries(RadarVU_QA PRIVATE ws2_32 userenv ntdll bcrypt advapi32)
    endif()
    add_dependencies(RadarVU_QA radarvu_dsp)
endif()
