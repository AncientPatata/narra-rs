@echo off

REM Run the npx command
call npx peggy ./examples/narra.peg -o ./examples/parser.js --format globals -e "parser"

echo "Parsed Narra grammar file"

REM Run the cargo command
call cargo run --example compile ./examples/test.narra ./examples/test.nb

REM Remove the file
del /F .\examples\test.nb

REM Copy the file
copy .\target\debug\examples\compile.exe .\compile.exe

echo "Generated Compiler executable"