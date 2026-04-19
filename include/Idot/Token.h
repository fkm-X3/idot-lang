#pragma once

#include <string>

namespace idot {

enum class TokenType {
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Slash,
  Star,
  Percent,
  Bang,
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,
  Identifier,
  String,
  Number,
  KeywordLet,
  KeywordIf,
  KeywordElse,
  KeywordTrue,
  KeywordFalse,
  KeywordNil,
  KeywordPrint,
  EndOfFile
};

inline const char* TokenTypeName(TokenType type) {
  switch (type) {
    case TokenType::LeftParen:
      return "(";
    case TokenType::RightParen:
      return ")";
    case TokenType::LeftBrace:
      return "{";
    case TokenType::RightBrace:
      return "}";
    case TokenType::Comma:
      return ",";
    case TokenType::Dot:
      return ".";
    case TokenType::Minus:
      return "-";
    case TokenType::Plus:
      return "+";
    case TokenType::Semicolon:
      return ";";
    case TokenType::Slash:
      return "/";
    case TokenType::Star:
      return "*";
    case TokenType::Percent:
      return "%";
    case TokenType::Bang:
      return "!";
    case TokenType::BangEqual:
      return "!=";
    case TokenType::Equal:
      return "=";
    case TokenType::EqualEqual:
      return "==";
    case TokenType::Greater:
      return ">";
    case TokenType::GreaterEqual:
      return ">=";
    case TokenType::Less:
      return "<";
    case TokenType::LessEqual:
      return "<=";
    case TokenType::Identifier:
      return "identifier";
    case TokenType::String:
      return "string";
    case TokenType::Number:
      return "number";
    case TokenType::KeywordLet:
      return "let";
    case TokenType::KeywordIf:
      return "if";
    case TokenType::KeywordElse:
      return "else";
    case TokenType::KeywordTrue:
      return "true";
    case TokenType::KeywordFalse:
      return "false";
    case TokenType::KeywordNil:
      return "nil";
    case TokenType::KeywordPrint:
      return "print";
    case TokenType::EndOfFile:
      return "EOF";
  }
  return "unknown";
}

struct Token {
  TokenType type;
  std::string lexeme;
  int line;
  int column;
};

}  // namespace idot
