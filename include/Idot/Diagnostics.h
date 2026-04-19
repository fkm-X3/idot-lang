#pragma once

#include <stdexcept>
#include <string>

namespace idot {

enum class ErrorPhase {
  Lex,
  Parse,
  Runtime
};

inline const char* PhaseName(ErrorPhase phase) {
  switch (phase) {
    case ErrorPhase::Lex:
      return "Lex";
    case ErrorPhase::Parse:
      return "Parse";
    case ErrorPhase::Runtime:
      return "Runtime";
  }
  return "Unknown";
}

class DiagnosticError : public std::runtime_error {
 public:
  DiagnosticError(ErrorPhase phase, int line, int column, const std::string& message)
      : std::runtime_error(Format(phase, line, column, message)),
        phase_(phase),
        line_(line),
        column_(column),
        message_(message) {}

  ErrorPhase phase() const { return phase_; }
  int line() const { return line_; }
  int column() const { return column_; }
  const std::string& message() const { return message_; }

  static std::string Format(ErrorPhase phase, int line, int column, const std::string& message) {
    return std::string(PhaseName(phase)) + " error at " + std::to_string(line) + ":" +
           std::to_string(column) + ": " + message;
  }

 private:
  ErrorPhase phase_;
  int line_;
  int column_;
  std::string message_;
};

}  // namespace idot
