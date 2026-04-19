#pragma once

#include <cstddef>
#include <initializer_list>
#include <vector>

#include "Idot/Ast.h"
#include "Idot/Token.h"

namespace idot {

class Parser {
 public:
  explicit Parser(std::vector<Token> tokens);

  std::vector<StmtPtr> Parse();

 private:
  StmtPtr Declaration();
  StmtPtr VarDeclaration();
  StmtPtr Statement();
  StmtPtr IfStatement();
  StmtPtr PrintStatement();
  StmtPtr ExpressionStatement();
  std::vector<StmtPtr> Block();

  ExprPtr Expression();
  ExprPtr Assignment();
  ExprPtr Equality();
  ExprPtr Comparison();
  ExprPtr Term();
  ExprPtr Factor();
  ExprPtr Unary();
  ExprPtr Primary();

  bool Match(std::initializer_list<TokenType> types);
  bool Check(TokenType type) const;
  const Token& Advance();
  bool IsAtEnd() const;
  const Token& Peek() const;
  const Token& Previous() const;
  const Token& Consume(TokenType type, const char* message);

  std::vector<Token> tokens_;
  std::size_t current_ = 0;
};

}  // namespace idot
