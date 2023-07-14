# Narra
Tour of the language : 
http://dkour.tech/post/declarative-narrative-scripting/

I used Peggy for the grammar, but I made it so only the compiler executable is dependent on some Javascript engine to reduce memory footprint/overhead. 

## Usage :
Run `build.bat` to generate the compiler executable.
You'll then get a `compile.exe` if you're on Windows, which you can use to compile .narra files into .nb files: 
```
compile.exe test.narra test.nb
```
Then include a dependency to this crate in some other project, implement the trait in narra_front to handle events, then read the .nb file to commence execution.
TODO ..

Generate compiler executable from grammar using build.bat
