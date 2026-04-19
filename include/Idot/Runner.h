#pragma once

#include <ostream>
#include <string>

#include "Idot/Interpreter.h"

namespace idot {

class Session {
 public:
  void Execute(const std::string& source, std::ostream& output);

 private:
  Interpreter interpreter_;
};

std::string ExecuteSourceToString(const std::string& source);

}  // namespace idot
