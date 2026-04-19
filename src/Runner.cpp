#include "Idot/Runner.h"

#include <sstream>

#include "Idot/Lexer.h"
#include "Idot/Parser.h"

namespace idot {

void Session::Execute(const std::string& source, std::ostream& output) {
  Lexer lexer(source);
  std::vector<Token> tokens = lexer.ScanTokens();
  Parser parser(std::move(tokens));
  std::vector<StmtPtr> statements = parser.Parse();
  interpreter_.Execute(statements, output);
}

std::string ExecuteSourceToString(const std::string& source) {
  Session session;
  std::ostringstream output;
  session.Execute(source, output);
  return output.str();
}

}  // namespace idot
