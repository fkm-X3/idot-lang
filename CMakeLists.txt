cmake_minimum_required(VERSION 3.10)

if (POLICY CMP0141)
  cmake_policy(SET CMP0141 NEW)
  set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT "$<IF:$<AND:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>,$<$<CONFIG:Debug,RelWithDebInfo>:EditAndContinue>,$<$<CONFIG:Debug,RelWithDebInfo>:ProgramDatabase>>")
endif()

project(Idot LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

option(IDOT_BUILD_RUST "Build Rust compiler targets" ON)

if (IDOT_BUILD_RUST)
  find_program(CARGO_EXECUTABLE cargo REQUIRED)

  set(IDOT_RUST_MANIFEST "${CMAKE_CURRENT_SOURCE_DIR}/compiler/Cargo.toml")
  set(IDOT_RUST_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo-target")
  set(IDOT_RUST_PROFILE "$<IF:$<OR:$<CONFIG:Release>,$<CONFIG:RelWithDebInfo>,$<CONFIG:MinSizeRel>>,release,dev>")

  add_custom_target(idotc ALL
    COMMAND ${CMAKE_COMMAND} -E env
      "CARGO_TARGET_DIR=${IDOT_RUST_TARGET_DIR}"
      ${CARGO_EXECUTABLE} build --manifest-path "${IDOT_RUST_MANIFEST}" --profile ${IDOT_RUST_PROFILE}
    WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}"
    COMMENT "Building Rust compiler (idotc)"
    VERBATIM
  )
endif()

add_library(idot_core
  src/Environment.cpp
  src/Interpreter.cpp
  src/Lexer.cpp
  src/Parser.cpp
  src/Runner.cpp
  src/Value.cpp
)

target_include_directories(idot_core PUBLIC include)

add_executable(Idot src/main.cpp)
target_link_libraries(Idot PRIVATE idot_core)

enable_testing()
add_executable(idot_tests tests/IdotTests.cpp)
target_link_libraries(idot_tests PRIVATE idot_core)
add_test(NAME idot_tests COMMAND idot_tests)

if (IDOT_BUILD_RUST)
  add_test(
    NAME idotc_tests
    COMMAND ${CMAKE_COMMAND} -E env
      "CARGO_TARGET_DIR=${IDOT_RUST_TARGET_DIR}"
      ${CARGO_EXECUTABLE} test --manifest-path "${IDOT_RUST_MANIFEST}" --profile ${IDOT_RUST_PROFILE}
  )
  set_tests_properties(idotc_tests PROPERTIES WORKING_DIRECTORY "${CMAKE_CURRENT_SOURCE_DIR}")
endif()
