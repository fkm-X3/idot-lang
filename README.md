# idot
idot is a coding language that currently compiles to c

usage: idotc.py [options] <filename>
```
# Compile and produce a binary
python idotc.py examples/hello.id

# Emit C only (don't invoke gcc)
python idotc.py examples/hello.id --emit-c

# Debug: dump tokens
python idotc.py examples/hello.id --tokens

# Debug: dump AST
python idotc.py examples/hello.id --ast

# Inspect the loaded language spec
python idotc.py --spec

# Run all tests
python tests/run_tests.py
```
