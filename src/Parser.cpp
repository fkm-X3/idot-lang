#include "Idot/Parser.h"

#include <stdexcept>
#include <utility>

#include "Idot/Diagnostics.h"

namespace idot {

Parser::Parser(std::vector<Token> tokens) : tokens_(std::move(tokens)) {}

std::vector<StmtPtr> Parser::Parse() {
  std::vector<StmtPtr> statements;
  while (!IsAtEnd()) {
    statements.push_back(Declaration());
  }
  return statements;
}

StmtPtr Parser::Declaration() {
  if (Match({TokenType::KeywordLet})) {
    return VarDeclaration();
  }
  return Statement();
}

StmtPtr Parser::VarDeclaration() {
  const Token name = Consume(TokenType::Identifier, "Expected variable name after 'let'.");
  Consume(TokenType::Equal, "Expected '=' after variable name.");
  ExprPtr initializer = Expression();
  Consume(TokenType::Semicolon, "Expected ';' after variable declaration.");
  return MakeStmt<VarStmt>(name, std::move(initializer));
}

StmtPtr Parser::Statement() {
  if (Match({TokenType::KeywordIf})) {
    return IfStatement();
  }
  if (Match({TokenType::KeywordPrint})) {
    return PrintStatement();
  }
  if (Match({TokenType::LeftBrace})) {
    return MakeStmt<BlockStmt>(Block());
  }
  return ExpressionStatement();
}

StmtPtr Parser::IfStatement() {
  Consume(TokenType::LeftParen, "Expected '(' after 'if'.");
  ExprPtr condition = Expression();
  Consume(TokenType::RightParen, "Expected ')' after condition.");

  StmtPtr thenBranch = Statement();
  StmtPtr elseBranch = nullptr;
  if (Match({TokenType::KeywordElse})) {
    elseBranch = Statement();
  }
  return MakeStmt<IfStmt>(std::move(condition), std::move(thenBranch), std::move(elseBranch));
}

StmtPtr Parser::PrintStatement() {
  ExprPtr expression = Expression();
  Consume(TokenType::Semicolon, "Expected ';' after print expression.");
  return MakeStmt<PrintStmt>(std::move(expression));
}

StmtPtr Parser::ExpressionStatement() {
  ExprPtr expression = Expression();
  Consume(TokenType::Semicolon, "Expected ';' after expression.");
  return MakeStmt<ExprStmt>(std::move(expression));
}

std::vector<StmtPtr> Parser::Block() {
  std::vector<StmtPtr> statements;
  while (!Check(TokenType::RightBrace) && !IsAtEnd()) {
    statements.push_back(Declaration());
  }
  Consume(TokenType::RightBrace, "Expected '}' after block.");
  return statements;
}

ExprPtr Parser::Expression() { return Assignment(); }

ExprPtr Parser::Assignment() {
  ExprPtr expression = Equality();
  if (!Match({TokenType::Equal})) {
    return expression;
  }

  const Token equals = Previous();
  ExprPtr value = Assignment();
  if (auto* variable = std::get_if<VariableExpr>(&expression->node)) {
    return MakeExpr<AssignExpr>(variable->name, std::move(value));
  }

  throw DiagnosticError(ErrorPhase::Parse, equals.line, equals.column, "Invalid assignment target.");
}

ExprPtr Parser::Equality() {
  ExprPtr expression = Comparison();

  while (Match({TokenType::BangEqual, TokenType::EqualEqual})) {
    const Token op = Previous();
    ExprPtr right = Comparison();
    expression = MakeExpr<BinaryExpr>(std::move(expression), op, std::move(right));
  }

  return expression;
}

ExprPtr Parser::Comparison() {
  ExprPtr expression = Term();

  while (Match({TokenType::Greater, TokenType::GreaterEqual, TokenType::Less, TokenType::LessEqual})) {
    const Token op = Previous();
    ExprPtr right = Term();
    expression = MakeExpr<BinaryExpr>(std::move(expression), op, std::move(right));
  }

  return expression;
}

ExprPtr Parser::Term() {
  ExprPtr expression = Factor();

  while (Match({TokenType::Minus, TokenType::Plus})) {
    const Token op = Previous();
    ExprPtr right = Factor();
    expression = MakeExpr<BinaryExpr>(std::move(expression), op, std::move(right));
  }

  return expression;
}

ExprPtr Parser::Factor() {
  ExprPtr expression = Unary();

  while (Match({TokenType::Slash, TokenType::Star, TokenType::Percent})) {
    const Token op = Previous();
    ExprPtr right = Unary();
    expression = MakeExpr<BinaryExpr>(std::move(expression), op, std::move(right));
  }

  return expression;
}

ExprPtr Parser::Unary() {
  if (Match({TokenType::Bang, TokenType::Minus})) {
    const Token op = Previous();
    ExprPtr right = Unary();
    return MakeExpr<UnaryExpr>(op, std::move(right));
  }
  return Primary();
}

ExprPtr Parser::Primary() {
  if (Match({TokenType::KeywordFalse})) {
    return MakeExpr<LiteralExpr>(Value(false));
  }
  if (Match({TokenType::KeywordTrue})) {
    return MakeExpr<LiteralExpr>(Value(true));
  }
  if (Match({TokenType::KeywordNil})) {
    return MakeExpr<LiteralExpr>(Value::Nil());
  }
  if (Match({TokenType::Number})) {
    try {
      return MakeExpr<LiteralExpr>(Value(std::stod(Previous().lexeme)));
    } catch (const std::invalid_argument&) {
      throw DiagnosticError(ErrorPhase::Parse, Previous().line, Previous().column, "Invalid number literal.");
    } catch (const std::out_of_range&) {
      throw DiagnosticError(ErrorPhase::Parse, Previous().line, Previous().column, "Number literal out of range.");
    }
  }
  if (Match({TokenType::String})) {
    return MakeExpr<LiteralExpr>(Value(Previous().lexeme));
  }
  if (Match({TokenType::Identifier})) {
    return MakeExpr<VariableExpr>(Previous());
  }
  if (Match({TokenType::LeftParen})) {
    ExprPtr expression = Expression();
    Consume(TokenType::RightParen, "Expected ')' after expression.");
    return MakeExpr<GroupingExpr>(std::move(expression));
  }

  throw DiagnosticError(ErrorPhase::Parse, Peek().line, Peek().column, "Expected expression.");
}

bool Parser::Match(std::initializer_list<TokenType> types) {
  for (const TokenType type : types) {
    if (Check(type)) {
      Advance();
      return true;
    }
  }
  return false;
}

bool Parser::Check(TokenType type) const {
  if (IsAtEnd()) {
    return false;
  }
  return Peek().type == type;
}

const Token& Parser::Advance() {
  if (!IsAtEnd()) {
    ++current_;
  }
  return Previous();
}

bool Parser::IsAtEnd() const { return Peek().type == TokenType::EndOfFile; }

const Token& Parser::Peek() const { return tokens_[current_]; }

const Token& Parser::Previous() const { return tokens_[current_ - 1]; }

const Token& Parser::Consume(TokenType type, const char* message) {
  if (Check(type)) {
    return Advance();
  }
  throw DiagnosticError(ErrorPhase::Parse, Peek().line, Peek().column, message);
}

}  // namespace idot
