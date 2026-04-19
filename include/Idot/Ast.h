#pragma once

#include <memory>
#include <utility>
#include <variant>
#include <vector>

#include "Idot/Token.h"
#include "Idot/Value.h"

namespace idot {

struct Expr;
struct Stmt;

using ExprPtr = std::unique_ptr<Expr>;
using StmtPtr = std::unique_ptr<Stmt>;

struct AssignExpr {
  Token name;
  ExprPtr value;
};

struct BinaryExpr {
  ExprPtr left;
  Token op;
  ExprPtr right;
};

struct GroupingExpr {
  ExprPtr expression;
};

struct LiteralExpr {
  Value value;
};

struct UnaryExpr {
  Token op;
  ExprPtr right;
};

struct VariableExpr {
  Token name;
};

struct Expr {
  using Node = std::variant<AssignExpr, BinaryExpr, GroupingExpr, LiteralExpr, UnaryExpr, VariableExpr>;

  template <typename T>
  explicit Expr(T&& value) : node(std::forward<T>(value)) {}

  Node node;
};

struct BlockStmt {
  std::vector<StmtPtr> statements;
};

struct ExprStmt {
  ExprPtr expression;
};

struct IfStmt {
  ExprPtr condition;
  StmtPtr thenBranch;
  StmtPtr elseBranch;
};

struct PrintStmt {
  ExprPtr expression;
};

struct VarStmt {
  Token name;
  ExprPtr initializer;
};

struct Stmt {
  using Node = std::variant<BlockStmt, ExprStmt, IfStmt, PrintStmt, VarStmt>;

  template <typename T>
  explicit Stmt(T&& value) : node(std::forward<T>(value)) {}

  Node node;
};

template <typename T, typename... Args>
inline ExprPtr MakeExpr(Args&&... args) {
  return std::make_unique<Expr>(T{std::forward<Args>(args)...});
}

template <typename T, typename... Args>
inline StmtPtr MakeStmt(Args&&... args) {
  return std::make_unique<Stmt>(T{std::forward<Args>(args)...});
}

}  // namespace idot
