#include "Idot/Lexer.h"

#include <cctype>
#include <unordered_map>

#include "Idot/Diagnostics.h"

namespace idot {
namespace {

const std::unordered_map<std::string, TokenType> kKeywords = {
    {"let", TokenType::KeywordLet},   {"if", TokenType::KeywordIf},
    {"else", TokenType::KeywordElse}, {"true", TokenType::KeywordTrue},
    {"false", TokenType::KeywordFalse}, {"nil", TokenType::KeywordNil},
    {"print", TokenType::KeywordPrint},
};

bool IsAlpha(char value) { return std::isalpha(static_cast<unsigned char>(value)) != 0 || value == '_'; }

bool IsAlphaNumeric(char value) {
  return std::isalnum(static_cast<unsigned char>(value)) != 0 || value == '_';
}

}  // namespace

Lexer::Lexer(std::string source) : source_(std::move(source)) {}

std::vector<Token> Lexer::ScanTokens() {
  while (!IsAtEnd()) {
    start_ = current_;
    startLine_ = line_;
    startColumn_ = column_;
    ScanToken();
  }

  tokens_.push_back(Token{TokenType::EndOfFile, "", line_, column_});
  return tokens_;
}

bool Lexer::IsAtEnd() const { return current_ >= source_.size(); }

char Lexer::Advance() {
  char value = source_[current_++];
  if (value == '\n') {
    ++line_;
    column_ = 1;
  } else {
    ++column_;
  }
  return value;
}

bool Lexer::Match(char expected) {
  if (IsAtEnd()) {
    return false;
  }
  if (source_[current_] != expected) {
    return false;
  }
  ++current_;
  ++column_;
  return true;
}

char Lexer::Peek() const { return IsAtEnd() ? '\0' : source_[current_]; }

char Lexer::PeekNext() const {
  if (current_ + 1 >= source_.size()) {
    return '\0';
  }
  return source_[current_ + 1];
}

void Lexer::ScanToken() {
  const char token = Advance();
  switch (token) {
    case '(':
      AddToken(TokenType::LeftParen);
      break;
    case ')':
      AddToken(TokenType::RightParen);
      break;
    case '{':
      AddToken(TokenType::LeftBrace);
      break;
    case '}':
      AddToken(TokenType::RightBrace);
      break;
    case ',':
      AddToken(TokenType::Comma);
      break;
    case '.':
      AddToken(TokenType::Dot);
      break;
    case '-':
      AddToken(TokenType::Minus);
      break;
    case '+':
      AddToken(TokenType::Plus);
      break;
    case ';':
      AddToken(TokenType::Semicolon);
      break;
    case '*':
      AddToken(TokenType::Star);
      break;
    case '%':
      AddToken(TokenType::Percent);
      break;
    case '!':
      AddToken(Match('=') ? TokenType::BangEqual : TokenType::Bang);
      break;
    case '=':
      AddToken(Match('=') ? TokenType::EqualEqual : TokenType::Equal);
      break;
    case '<':
      AddToken(Match('=') ? TokenType::LessEqual : TokenType::Less);
      break;
    case '>':
      AddToken(Match('=') ? TokenType::GreaterEqual : TokenType::Greater);
      break;
    case '/':
      if (Match('/')) {
        while (Peek() != '\n' && !IsAtEnd()) {
          Advance();
        }
      } else {
        AddToken(TokenType::Slash);
      }
      break;
    case '"':
      String();
      break;
    case ' ':
    case '\r':
    case '\t':
    case '\n':
      break;
    default:
      if (std::isdigit(static_cast<unsigned char>(token)) != 0) {
        Number();
      } else if (IsAlpha(token)) {
        Identifier();
      } else {
        throw DiagnosticError(ErrorPhase::Lex, startLine_, startColumn_,
                              std::string("Unexpected character '") + token + "'.");
      }
      break;
  }
}

void Lexer::AddToken(TokenType type) { AddToken(type, source_.substr(start_, current_ - start_)); }

void Lexer::AddToken(TokenType type, const std::string& lexeme) {
  tokens_.push_back(Token{type, lexeme, startLine_, startColumn_});
}

void Lexer::String() {
  while (Peek() != '"' && !IsAtEnd()) {
    Advance();
  }

  if (IsAtEnd()) {
    throw DiagnosticError(ErrorPhase::Lex, startLine_, startColumn_, "Unterminated string.");
  }

  Advance();
  AddToken(TokenType::String, source_.substr(start_ + 1, current_ - start_ - 2));
}

void Lexer::Number() {
  while (std::isdigit(static_cast<unsigned char>(Peek())) != 0) {
    Advance();
  }

  if (Peek() == '.' && std::isdigit(static_cast<unsigned char>(PeekNext())) != 0) {
    Advance();
    while (std::isdigit(static_cast<unsigned char>(Peek())) != 0) {
      Advance();
    }
  }

  AddToken(TokenType::Number);
}

void Lexer::Identifier() {
  while (IsAlphaNumeric(Peek())) {
    Advance();
  }

  const std::string text = source_.substr(start_, current_ - start_);
  auto keyword = kKeywords.find(text);
  if (keyword != kKeywords.end()) {
    AddToken(keyword->second, text);
    return;
  }
  AddToken(TokenType::Identifier, text);
}

}  // namespace idot
