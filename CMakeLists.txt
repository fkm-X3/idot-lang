cmake_minimum_required(VERSION 3.10)

if (POLICY CMP0141)
  cmake_policy(SET CMP0141 NEW)
  set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT "$<IF:$<AND:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>,$<$<CONFIG:Debug,RelWithDebInfo>:EditAndContinue>,$<$<CONFIG:Debug,RelWithDebInfo>:ProgramDatabase>>")
endif()

project(Idot LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

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
