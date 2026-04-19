#pragma once

#include <memory>
#include <string>
#include <unordered_map>

#include "Idot/Token.h"
#include "Idot/Value.h"

namespace idot {

class Environment {
 public:
  explicit Environment(std::shared_ptr<Environment> enclosing = nullptr);

  void Define(const std::string& name, const Value& value);
  void Assign(const Token& name, const Value& value);
  Value Get(const Token& name) const;

 private:
  std::unordered_map<std::string, Value> values_;
  std::shared_ptr<Environment> enclosing_;
};

}  // namespace idot
