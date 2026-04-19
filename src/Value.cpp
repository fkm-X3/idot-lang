#include "Idot/Value.h"

#include <iomanip>
#include <sstream>
#include <stdexcept>

namespace idot {

Value::Value() : data_(std::monostate{}) {}

Value::Value(double number) : data_(number) {}

Value::Value(bool boolean) : data_(boolean) {}

Value::Value(std::string text) : data_(std::move(text)) {}

Value Value::Nil() { return Value(); }

bool Value::IsNil() const { return std::holds_alternative<std::monostate>(data_); }

bool Value::IsNumber() const { return std::holds_alternative<double>(data_); }

bool Value::IsBool() const { return std::holds_alternative<bool>(data_); }

bool Value::IsString() const { return std::holds_alternative<std::string>(data_); }

double Value::AsNumber() const {
  if (!IsNumber()) {
    throw std::logic_error("Value is not a number.");
  }
  return std::get<double>(data_);
}

bool Value::AsBool() const {
  if (!IsBool()) {
    throw std::logic_error("Value is not a bool.");
  }
  return std::get<bool>(data_);
}

const std::string& Value::AsString() const {
  if (!IsString()) {
    throw std::logic_error("Value is not a string.");
  }
  return std::get<std::string>(data_);
}

std::string Value::ToString() const {
  if (IsNil()) {
    return "nil";
  }
  if (IsBool()) {
    return AsBool() ? "true" : "false";
  }
  if (IsNumber()) {
    std::ostringstream stream;
    stream << std::setprecision(15) << AsNumber();
    std::string value = stream.str();
    if (value.find('.') != std::string::npos) {
      while (!value.empty() && value.back() == '0') {
        value.pop_back();
      }
      if (!value.empty() && value.back() == '.') {
        value.pop_back();
      }
    }
    return value;
  }
  return AsString();
}

bool operator==(const Value& left, const Value& right) { return left.data_ == right.data_; }

bool operator!=(const Value& left, const Value& right) { return !(left == right); }

}  // namespace idot
