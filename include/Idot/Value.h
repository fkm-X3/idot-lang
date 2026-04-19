#pragma once

#include <string>
#include <variant>

namespace idot {

class Value {
 public:
  Value();
  explicit Value(double number);
  explicit Value(bool boolean);
  explicit Value(std::string text);

  static Value Nil();

  bool IsNil() const;
  bool IsNumber() const;
  bool IsBool() const;
  bool IsString() const;

  double AsNumber() const;
  bool AsBool() const;
  const std::string& AsString() const;

  std::string ToString() const;

  friend bool operator==(const Value& left, const Value& right);
  friend bool operator!=(const Value& left, const Value& right);

 private:
  using Data = std::variant<std::monostate, double, bool, std::string>;
  Data data_;
};

}  // namespace idot
