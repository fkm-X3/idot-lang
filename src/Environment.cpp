#include "Idot/Environment.h"

#include "Idot/Diagnostics.h"

namespace idot {

Environment::Environment(std::shared_ptr<Environment> enclosing) : enclosing_(std::move(enclosing)) {}

void Environment::Define(const std::string& name, const Value& value) { values_[name] = value; }

void Environment::Assign(const Token& name, const Value& value) {
  auto found = values_.find(name.lexeme);
  if (found != values_.end()) {
    found->second = value;
    return;
  }

  if (enclosing_) {
    enclosing_->Assign(name, value);
    return;
  }

  throw DiagnosticError(ErrorPhase::Runtime, name.line, name.column,
                        "Undefined variable '" + name.lexeme + "'.");
}

Value Environment::Get(const Token& name) const {
  auto found = values_.find(name.lexeme);
  if (found != values_.end()) {
    return found->second;
  }

  if (enclosing_) {
    return enclosing_->Get(name);
  }

  throw DiagnosticError(ErrorPhase::Runtime, name.line, name.column,
                        "Undefined variable '" + name.lexeme + "'.");
}

}  // namespace idot
