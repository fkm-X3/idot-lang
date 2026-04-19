#pragma once

#include <cstddef>
#include <string>
#include <vector>

#include "Idot/Token.h"

namespace idot {

class Lexer {
 public:
  explicit Lexer(std::string source);

  std::vector<Token> ScanTokens();

 private:
  bool IsAtEnd() const;
  char Advance();
  bool Match(char expected);
  char Peek() const;
  char PeekNext() const;

  void ScanToken();
  void AddToken(TokenType type);
  void AddToken(TokenType type, const std::string& lexeme);

  void String();
  void Number();
  void Identifier();

  std::string source_;
  std::vector<Token> tokens_;
  std::size_t start_ = 0;
  std::size_t current_ = 0;
  int line_ = 1;
  int column_ = 1;
  int startLine_ = 1;
  int startColumn_ = 1;
};

}  // namespace idot
