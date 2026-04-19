#include "Idot/Interpreter.h"

#include <cmath>
#include <memory>
#include <utility>

#include "Idot/Diagnostics.h"

namespace idot {
namespace {

template <class... Ts>
struct Overloaded : Ts... {
  using Ts::operator()...;
};

template <class... Ts>
Overloaded(Ts...) -> Overloaded<Ts...>;

}  // namespace

Interpreter::Interpreter() : environment_(std::make_shared<Environment>()) {}

void Interpreter::Execute(const std::vector<StmtPtr>& statements, std::ostream& output) {
  for (const auto& statement : statements) {
    Execute(*statement, output);
  }
}

void Interpreter::Execute(const Stmt& statement, std::ostream& output) {
  std::visit(
      Overloaded{
          [&](const BlockStmt& block) {
            ExecuteBlock(block.statements, std::make_shared<Environment>(environment_), output);
          },
          [&](const ExprStmt& expression) {
            (void)Evaluate(*expression.expression, output);
          },
          [&](const IfStmt& ifStatement) {
            const Value condition = Evaluate(*ifStatement.condition, output);
            if (IsTruthy(condition)) {
              Execute(*ifStatement.thenBranch, output);
            } else if (ifStatement.elseBranch) {
              Execute(*ifStatement.elseBranch, output);
            }
          },
          [&](const PrintStmt& printStatement) {
            const Value value = Evaluate(*printStatement.expression, output);
            output << value.ToString() << '\n';
          },
          [&](const VarStmt& variableStatement) {
            const Value value = Evaluate(*variableStatement.initializer, output);
            environment_->Define(variableStatement.name.lexeme, value);
          },
      },
      statement.node);
}

void Interpreter::ExecuteBlock(const std::vector<StmtPtr>& statements,
                               std::shared_ptr<Environment> environment, std::ostream& output) {
  auto previous = environment_;
  environment_ = std::move(environment);
  struct ScopeReset {
    std::shared_ptr<Environment>& slot;
    std::shared_ptr<Environment> previous;
    ~ScopeReset() { slot = std::move(previous); }
  } reset{environment_, previous};

  for (const auto& statement : statements) {
    Execute(*statement, output);
  }
}

Value Interpreter::Evaluate(const Expr& expression, std::ostream& output) {
  return std::visit(
      Overloaded{
          [&](const AssignExpr& assign) {
            const Value value = Evaluate(*assign.value, output);
            environment_->Assign(assign.name, value);
            return value;
          },
          [&](const BinaryExpr& binary) {
            const Value left = Evaluate(*binary.left, output);
            const Value right = Evaluate(*binary.right, output);

            switch (binary.op.type) {
              case TokenType::Plus:
                if (left.IsNumber() && right.IsNumber()) {
                  return Value(left.AsNumber() + right.AsNumber());
                }
                if (left.IsString() && right.IsString()) {
                  return Value(left.AsString() + right.AsString());
                }
                throw DiagnosticError(ErrorPhase::Runtime, binary.op.line, binary.op.column,
                                      "Operator '+' requires two numbers or two strings.");
              case TokenType::Minus:
                return Value(RequireNumber(left, binary.op, "left operand") -
                             RequireNumber(right, binary.op, "right operand"));
              case TokenType::Star:
                return Value(RequireNumber(left, binary.op, "left operand") *
                             RequireNumber(right, binary.op, "right operand"));
              case TokenType::Slash: {
                const double divisor = RequireNumber(right, binary.op, "right operand");
                if (divisor == 0.0) {
                  throw DiagnosticError(ErrorPhase::Runtime, binary.op.line, binary.op.column,
                                        "Division by zero.");
                }
                return Value(RequireNumber(left, binary.op, "left operand") / divisor);
              }
              case TokenType::Percent: {
                const double divisor = RequireNumber(right, binary.op, "right operand");
                if (divisor == 0.0) {
                  throw DiagnosticError(ErrorPhase::Runtime, binary.op.line, binary.op.column,
                                        "Modulo by zero.");
                }
                return Value(std::fmod(RequireNumber(left, binary.op, "left operand"), divisor));
              }
              case TokenType::Greater:
                return Value(RequireNumber(left, binary.op, "left operand") >
                             RequireNumber(right, binary.op, "right operand"));
              case TokenType::GreaterEqual:
                return Value(RequireNumber(left, binary.op, "left operand") >=
                             RequireNumber(right, binary.op, "right operand"));
              case TokenType::Less:
                return Value(RequireNumber(left, binary.op, "left operand") <
                             RequireNumber(right, binary.op, "right operand"));
              case TokenType::LessEqual:
                return Value(RequireNumber(left, binary.op, "left operand") <=
                             RequireNumber(right, binary.op, "right operand"));
              case TokenType::EqualEqual:
                return Value(left == right);
              case TokenType::BangEqual:
                return Value(left != right);
              default:
                break;
            }
            throw DiagnosticError(ErrorPhase::Runtime, binary.op.line, binary.op.column,
                                  "Unsupported binary operator.");
          },
          [&](const GroupingExpr& grouping) { return Evaluate(*grouping.expression, output); },
          [&](const LiteralExpr& literal) { return literal.value; },
          [&](const UnaryExpr& unary) {
            const Value right = Evaluate(*unary.right, output);
            switch (unary.op.type) {
              case TokenType::Bang:
                return Value(!IsTruthy(right));
              case TokenType::Minus:
                return Value(-RequireNumber(right, unary.op, "operand"));
              default:
                break;
            }
            throw DiagnosticError(ErrorPhase::Runtime, unary.op.line, unary.op.column,
                                  "Unsupported unary operator.");
          },
          [&](const VariableExpr& variable) { return environment_->Get(variable.name); },
      },
      expression.node);
}

bool Interpreter::IsTruthy(const Value& value) const {
  if (value.IsNil()) {
    return false;
  }
  if (value.IsBool()) {
    return value.AsBool();
  }
  return true;
}

double Interpreter::RequireNumber(const Value& value, const Token& token, const char* context) const {
  if (!value.IsNumber()) {
    throw DiagnosticError(ErrorPhase::Runtime, token.line, token.column,
                          std::string("Expected number for ") + context + ".");
  }
  return value.AsNumber();
}

}  // namespace idot
