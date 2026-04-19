#include <fstream>
#include <iostream>
#include <iterator>
#include <string>

#include "Idot/Diagnostics.h"
#include "Idot/Runner.h"

namespace {

int RunFile(const std::string& path) {
  std::ifstream input(path, std::ios::in | std::ios::binary);
  if (!input.is_open()) {
    std::cerr << "Failed to open file: " << path << '\n';
    return 1;
  }

  std::string source((std::istreambuf_iterator<char>(input)), std::istreambuf_iterator<char>());
  idot::Session session;
  try {
    session.Execute(source, std::cout);
    return 0;
  } catch (const idot::DiagnosticError& error) {
    std::cerr << error.what() << '\n';
    return 1;
  }
}

int RunRepl() {
  idot::Session session;
  std::string line;

  while (true) {
    std::cout << "idot> ";
    if (!std::getline(std::cin, line)) {
      std::cout << '\n';
      break;
    }

    if (line == "exit" || line == "quit") {
      break;
    }

    if (line.empty()) {
      continue;
    }

    try {
      session.Execute(line, std::cout);
    } catch (const idot::DiagnosticError& error) {
      std::cerr << error.what() << '\n';
    }
  }

  return 0;
}

}  // namespace

int main(int argc, char** argv) {
  if (argc > 2) {
    std::cerr << "Usage: Idot [file.idot]\n";
    return 1;
  }

  if (argc == 2) {
    return RunFile(argv[1]);
  }

  return RunRepl();
}
