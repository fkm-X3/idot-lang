#include <iostream>
#include <sstream>
#include <string>

#include "Idot/Diagnostics.h"
#include "Idot/Runner.h"

namespace {

int failures = 0;

void Fail(const std::string& testName, const std::string& message) {
  ++failures;
  std::cerr << "[FAIL] " << testName << ": " << message << '\n';
}

void ExpectEqual(const std::string& testName, const std::string& actual, const std::string& expected) {
  if (actual != expected) {
    Fail(testName, "expected `" + expected + "` but got `" + actual + "`");
  }
}

void ExpectTrue(const std::string& testName, bool condition, const std::string& message) {
  if (!condition) {
    Fail(testName, message);
  }
}

void TestConditionalsAndVariables() {
  const std::string testName = "ConditionalsAndVariables";
  idot::Session session;
  std::ostringstream output;
  session.Execute(
      "let x = 3;\n"
      "if (x > 2) { print \"big\"; } else { print \"small\"; }\n"
      "print x + 1;\n",
      output);

  ExpectEqual(testName, output.str(), "big\n4\n");
}

void TestBlockScope() {
  const std::string testName = "BlockScope";
  idot::Session session;
  std::ostringstream output;
  session.Execute(
      "let x = 1;\n"
      "{\n"
      "  let x = 2;\n"
      "  print x;\n"
      "}\n"
      "print x;\n",
      output);

  ExpectEqual(testName, output.str(), "2\n1\n");
}

void TestRuntimeErrorForUndefinedVariable() {
  const std::string testName = "RuntimeErrorForUndefinedVariable";
  idot::Session session;
  std::ostringstream output;
  bool threw = false;
  try {
    session.Execute("print missing;\n", output);
  } catch (const idot::DiagnosticError& error) {
    threw = true;
    ExpectTrue(testName, error.phase() == idot::ErrorPhase::Runtime, "error phase should be runtime");
  }
  ExpectTrue(testName, threw, "expected runtime error for undefined variable");
}

void TestParseErrorForInvalidDeclaration() {
  const std::string testName = "ParseErrorForInvalidDeclaration";
  idot::Session session;
  std::ostringstream output;
  bool threw = false;
  try {
    session.Execute("let x = ;\n", output);
  } catch (const idot::DiagnosticError& error) {
    threw = true;
    ExpectTrue(testName, error.phase() == idot::ErrorPhase::Parse, "error phase should be parse");
  }
  ExpectTrue(testName, threw, "expected parse error for invalid declaration");
}

}  // namespace

int main() {
  TestConditionalsAndVariables();
  TestBlockScope();
  TestRuntimeErrorForUndefinedVariable();
  TestParseErrorForInvalidDeclaration();

  if (failures > 0) {
    std::cerr << failures << " test(s) failed.\n";
    return 1;
  }

  std::cout << "All tests passed.\n";
  return 0;
}
