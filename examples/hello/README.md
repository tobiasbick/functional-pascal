# Hello from FPAS

main.fpas is the first executable example of the CLR compiler. It declares an
explicitly typed string constant and calls the actual Console.WriteLine method.
The entry block ends with `end;`; the enclosing program ends with `end program;`.

Hello.fpasproj uses the locally packaged FPAS SDK. Follow the
[SDK bootstrap instructions](../../docs/build/development.md#build-an-fpas-application),
then build the project and run bin/Debug/net10.0/Hello.dll with dotnet.
The [direct CLI instructions](../../docs/build/development.md#direct-compiler-invocation)
remain available when supplying reference assemblies explicitly.
