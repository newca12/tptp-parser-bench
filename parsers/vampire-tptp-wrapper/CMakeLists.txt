cmake_minimum_required(VERSION 3.14)
project(vampire_parser_wrapper LANGUAGES CXX)

set(VAMPIRE_SOURCE_DIR "${CMAKE_CURRENT_SOURCE_DIR}/vampire")
set(VAMPIRE_BUILD_DIR "${CMAKE_CURRENT_BINARY_DIR}/vampire-build")
set(VAMPIRE_BUILD_JOBS "$ENV{VAMPIRE_BUILD_JOBS}")
if(NOT VAMPIRE_BUILD_JOBS MATCHES "^[1-9][0-9]*$")
    set(VAMPIRE_BUILD_JOBS 1)
endif()

execute_process(
    COMMAND "${CMAKE_COMMAND}" -S "${VAMPIRE_SOURCE_DIR}" -B "${VAMPIRE_BUILD_DIR}"
        -DCMAKE_BUILD_TYPE=Release
        -DBUILD_SHARED_LIBS=OFF
        -DCMAKE_POSITION_INDEPENDENT_CODE=ON
    RESULT_VARIABLE configure_status
)
if(NOT configure_status EQUAL 0)
    message(FATAL_ERROR "Failed to configure Vampire")
endif()

execute_process(
    COMMAND "${CMAKE_COMMAND}" --build "${VAMPIRE_BUILD_DIR}" --target common
        --parallel "${VAMPIRE_BUILD_JOBS}"
    RESULT_VARIABLE build_status
)
if(NOT build_status EQUAL 0)
    message(FATAL_ERROR "Failed to build Vampire parser objects")
endif()

file(GLOB_RECURSE vampire_objects "${VAMPIRE_BUILD_DIR}/CMakeFiles/common.dir/*.o")
if(NOT vampire_objects)
    message(FATAL_ERROR "Vampire parser object files were not produced")
endif()

add_library(vampire_lib STATIC ${vampire_objects})
set_target_properties(vampire_lib PROPERTIES LINKER_LANGUAGE CXX)
target_include_directories(vampire_lib PUBLIC
    "${VAMPIRE_SOURCE_DIR}"
    "${VAMPIRE_SOURCE_DIR}/viras/src"
    "${VAMPIRE_SOURCE_DIR}/mini-gmp-6.3.0"
    "${VAMPIRE_SOURCE_DIR}/cadical/src"
    "${VAMPIRE_BUILD_DIR}"
)
install(TARGETS vampire_lib ARCHIVE DESTINATION lib)
