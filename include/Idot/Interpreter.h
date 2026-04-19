#pragma once

#include <memory>
#include <ostream>
#include <vector>

#include "Idot/Ast.h"
#include "Idot/Environment.h"

namespace idot {

class Interpreter {
 public:
  Interpreter();

  void Execute(const std::vector<StmtPtr>& statements, std::ostream& output);

 private:
  void Execute(const Stmt& statement, std::ostream& output);
  void ExecuteBlock(const std::vector<StmtPtr>& statements, std::shared_ptr<Environment> environment,
                    std::ostream& output);
  Value Evaluate(const Expr& expression, std::ostream& output);
  bool IsTruthy(const Value& value) const;
  double RequireNumber(const Value& value, const Token& token, const char* context) const;

  std::shared_ptr<Environment> environment_;
};

}  // namespace idot
