# Hello from FPAS

main.fpas is the first executable example of the CLR compiler. It declares an
explicitly typed string constant and calls the actual Console.WriteLine method.
The entry block ends with `end;`; the enclosing program ends with `end program;`.

Hello.fpasproj uses the locally packaged FPAS SDK. Follow the
[SDK bootstrap instructions](../../docs/implementation/msbuild.md#local-sdk-bootstrap),
then build the project and run bin/Debug/net10.0/Hello.dll with dotnet.
The [direct CLI instructions](../../docs/implementation/first-program.md#run-the-example)
remain available when supplying reference assemblies explicitly.
