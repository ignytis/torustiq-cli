cmake_minimum_required(VERSION 3.20)

# Read project info
file(READ project.json PROJECT_INFO)
string(JSON APP_NAME GET ${PROJECT_INFO} name)
string(JSON APP_VERSION GET ${PROJECT_INFO} version)
string(JSON APP_DESCRIPTION GET ${PROJECT_INFO} description)

string(TOLOWER ${APP_NAME} APP_NAME_LOWERCASE)

project(${APP_NAME} VERSION ${APP_VERSION} LANGUAGES C CXX)

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_RUNTIME_OUTPUT_DIRECTORY "build")
# Prefer static libraries when searching
# TODO: Linux-specific suffix here. Need to replace with something more cross-platform.
if (LINUX)
    set(CMAKE_FIND_LIBRARY_SUFFIXES ".a;.so")
endif()
# Helps with debugging
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

find_package(CLI11 REQUIRED)
find_package(spdlog REQUIRED)
find_package(yaml-cpp REQUIRED)
find_package(Lua REQUIRED)

# SDK library
find_path(TORUSTIQ_SDK_INCLUDE_DIR
    NAMES torustiq_sdk/plugins/typedefs.h
    PATHS ${CMAKE_CURRENT_SOURCE_DIR}/../torustiq-cpp-lib-sdk/include
    ENV TORUSTIQ_SDK_DIR
    REQUIRED
)

add_library(torustiq_sdk STATIC
    # Could this list be generated the same way as SOURCES?
    ${TORUSTIQ_SDK_INCLUDE_DIR}/../src/message.c
)
target_include_directories(torustiq_sdk PUBLIC ${TORUSTIQ_SDK_INCLUDE_DIR})

file(GLOB_RECURSE SOURCES "src/*.cpp")
add_executable(${APP_NAME_LOWERCASE} ${SOURCES})

if(Lua_FOUND AND NOT TARGET Lua::Lua)
  add_library(Lua::Lua INTERFACE IMPORTED)
  set_target_properties(
    Lua::Lua
    PROPERTIES
      INTERFACE_INCLUDE_DIRECTORIES "${LUA_INCLUDE_DIR}"
      INTERFACE_LINK_LIBRARIES "${LUA_LIBRARIES}"
  )
endif()

target_include_directories(${APP_NAME_LOWERCASE} PRIVATE src ${LUA_INCLUDE_DIR})
target_link_libraries(${APP_NAME_LOWERCASE} PRIVATE CLI11::CLI11)
target_link_libraries(${APP_NAME_LOWERCASE} PRIVATE spdlog::spdlog)
target_link_libraries(${APP_NAME_LOWERCASE} PRIVATE torustiq_sdk)
# TODO: Linux-specific suffix here. Need to replace with something more cross-platform.
if (LINUX)
    target_link_libraries(${APP_NAME_LOWERCASE} PRIVATE
        libyaml-cpp.lib
        lua.lib
    )
elseif (WIN32)
    target_link_libraries(${APP_NAME_LOWERCASE} PRIVATE Lua::Lua)
    target_link_libraries(${APP_NAME_LOWERCASE} PRIVATE yaml-cpp::yaml-cpp)
endif()

add_compile_definitions(APP_NAME="${APP_NAME}")
add_compile_definitions(APP_VERSION="${APP_VERSION}")
add_compile_definitions(APP_DESCRIPTION="${APP_DESCRIPTION}")

# Platform-specific configuration
if (WIN32)
    # To avoid type collistion with std::byte
    # https://stackoverflow.com/a/58060367
    add_compile_definitions(_HAS_STD_BYTE=0)

    set_target_properties(${APP_NAME_LOWERCASE} PROPERTIES
        WIN32_EXECUTABLE ON
        LINK_FLAGS /SUBSYSTEM:CONSOLE
    )
elseif(APPLE)
    set_target_properties(${APP_NAME_LOWERCASE} PROPERTIES
        MACOSX_BUNDLE ON
    )
endif()