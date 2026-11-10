using Fpas.Tests.Support;
using System.Diagnostics;
using System.Reflection;
using System.Runtime.CompilerServices;
using System.Runtime.Loader;

namespace Fpas.Compiler.Tests.Metadata;

public sealed class DebuggerAttributeTests
{
    [Theory]
    [InlineData(true, false, true)]
    [InlineData(true, true, false)]
    [InlineData(false, false, false)]
    public void Symbols_and_JIT_optimization_are_independent(bool symbols, bool optimize, bool disableOptimization)
    {
        var program = Compile("unit Model; public function Value(): int; begin return 42; end function; end unit;", symbols, optimize);
        WithAssembly(program, assembly =>
        {
            var attribute = Assert.IsType<DebuggableAttribute>(assembly.GetCustomAttribute<DebuggableAttribute>());
            Assert.Equal(disableOptimization, attribute.IsJITOptimizerDisabled);
            Assert.Equal(disableOptimization, attribute.IsJITTrackingEnabled);
            Assert.Equal(symbols, program.PortablePdb is not null);
            Assert.Equal(42, assembly.GetType("Model")!.GetMethod("Value")!.Invoke(null, null));
        });
    }

    [Fact]
    public void Typed_generic_and_variant_proxies_read_live_source_properties_and_preserve_display_overrides()
    {
        var program = Compile("""
            unit Demo.Views;
            uses System.Diagnostics as D;
            public type Box<T> = record public Item: T; end record;
            public type Constrained<T> = where T: struct; record public Item: T; end record;
            public type Privacy = record
              [D.DebuggerBrowsableAttribute(D.DebuggerBrowsableState.Never)] public Secret: int;
              public Visible: int;
            end record;
            public type Outcome<T> = variant Success(Value: T); Missing; end variant;
            [D.DebuggerDisplayAttribute('custom')]
            [D.DebuggerTypeProxyAttribute(typeof(string))]
            public type Custom = record public Value: int; end record;
            [D.DebuggerDisplayAttribute('choice')]
            [D.DebuggerDisplayAttribute('alternative')]
            public type CustomVariant = variant Present; end variant;
            public type Id = distinct long;
            public type Port = ushort range 100..200;
            end unit;
            """);
        WithAssembly(program, assembly =>
        {
            var box = assembly.GetType("Demo.Views+Box`1")!.MakeGenericType(typeof(int));
            var value = Activator.CreateInstance(box, 42)!;
            var proxy = Proxy(assembly, value);
            Assert.Equal(42, proxy.GetType().GetProperty("Item")!.GetValue(proxy));
            Assert.Equal("Box(Item = {Item})", box.GetCustomAttribute<DebuggerDisplayAttribute>()!.Value);
            Assert.True(proxy.GetType().IsDefined(typeof(CompilerGeneratedAttribute)));
            Assert.True(proxy.GetType().GetMethod("get_Item")!.IsDefined(typeof(DebuggerHiddenAttribute)));
            var field = Assert.Single(box.GetFields(BindingFlags.Instance | BindingFlags.NonPublic));
            Assert.Equal(DebuggerBrowsableState.Never, field.GetCustomAttribute<DebuggerBrowsableAttribute>()!.State);
            Assert.True(field.IsDefined(typeof(CompilerGeneratedAttribute)));
            field.SetValue(value, 43);
            Assert.Equal(43, proxy.GetType().GetProperty("Item")!.GetValue(proxy));
            var constrained = assembly.GetType("Demo.Views+Constrained`1")!.MakeGenericType(typeof(int));
            var constrainedView = Proxy(assembly, Activator.CreateInstance(constrained, 44)!);
            Assert.Equal(44, constrainedView.GetType().GetProperty("Item")!.GetValue(constrainedView));
            Assert.True(constrainedView.GetType().GetGenericTypeDefinition().GetGenericArguments()[0].GenericParameterAttributes
                .HasFlag(GenericParameterAttributes.NotNullableValueTypeConstraint));
            var privacy = Proxy(assembly, Activator.CreateInstance(assembly.GetType("Demo.Views+Privacy")!, 1, 2)!);
            Assert.Equal(DebuggerBrowsableState.Never, privacy.GetType().GetProperty("Secret")!.GetCustomAttribute<DebuggerBrowsableAttribute>()!.State);

            var success = assembly.GetType("Demo.Views+Outcome`1+SuccessCase")!.MakeGenericType(typeof(string));
            var result = Activator.CreateInstance(success, BindingFlags.Instance | BindingFlags.Public | BindingFlags.NonPublic,
                null, ["payload"], null)!;
            var caseView = Proxy(assembly, result);
            Assert.Equal("payload", caseView.GetType().GetProperty("Value")!.GetValue(caseView));
            Assert.Equal(0, caseView.GetType().GetProperty("Tag")!.GetValue(caseView));
            var missing = assembly.GetType("Demo.Views+Outcome`1+MissingCase")!.MakeGenericType(typeof(int));
            var empty = Activator.CreateInstance(missing, nonPublic: true)!;
            var emptyView = Proxy(assembly, empty);
            Assert.Equal(1, emptyView.GetType().GetProperty("Tag")!.GetValue(emptyView));
            var custom = assembly.GetType("Demo.Views+Custom")!;
            Assert.Equal("custom", Assert.Single(custom.GetCustomAttributes<DebuggerDisplayAttribute>()).Value);
            Assert.Equal(typeof(string).AssemblyQualifiedName, Assert.Single(custom.GetCustomAttributes<DebuggerTypeProxyAttribute>()).ProxyTypeName);
            Assert.Null(custom.GetNestedType("<DebugView>", BindingFlags.NonPublic));
            Assert.Equal(["choice", "alternative"], assembly.GetType("Demo.Views+CustomVariant+PresentCase")!
                .GetCustomAttributes<DebuggerDisplayAttribute>(inherit: false).Select(attribute => attribute.Value));
            Assert.Equal("Id({Value})", assembly.GetType("Demo.Views+Id")!.GetCustomAttribute<DebuggerDisplayAttribute>()!.Value);
            var port = assembly.GetType("Demo.Views+Port")!;
            Assert.Equal("Port({Value})", port.GetCustomAttribute<DebuggerDisplayAttribute>()!.Value);
            Assert.Equal((ushort)100, port.GetProperty("Value")!.GetValue(Activator.CreateInstance(port)));
        });
    }

    [Fact]
    public void Source_routines_and_async_MoveNext_remain_debuggable_while_plumbing_is_marked()
    {
        var program = Compile("""
            async program Methods;
            uses System.Threading.Tasks as Tasks, System.Diagnostics as D;
            type R = record public X: int := 1; end record;
            function Make(Value: int): function(): int;
            begin return function(): int => Value; end function;
            [D.DebuggerStepThroughAttribute]
            async function Work(): Tasks.Task<int>;
            begin await Tasks.Task.Delay(1); return 42; end function;
            begin discard await Work(); end;
            end program;
            """, target: CompilationTarget.Exe);
        WithAssembly(program, assembly =>
        {
            var source = assembly.GetType("Methods")!;
            var work = source.GetMethod("Work", BindingFlags.Static | BindingFlags.NonPublic)!;
            Assert.True(work.IsDefined(typeof(DebuggerStepThroughAttribute)));
            Assert.Single(work.GetCustomAttributes<DebuggerStepThroughAttribute>());
            Assert.False(work.IsDefined(typeof(DebuggerHiddenAttribute)));
            Assert.False(work.IsDefined(typeof(DebuggerNonUserCodeAttribute)));
            var machine = work.GetCustomAttribute<AsyncStateMachineAttribute>()!.StateMachineType;
            Assert.True(machine.IsDefined(typeof(CompilerGeneratedAttribute)));
            Assert.False(machine.GetMethod("MoveNext")!.IsDefined(typeof(DebuggerHiddenAttribute)));
            Assert.False(machine.GetMethod("MoveNext")!.IsDefined(typeof(DebuggerNonUserCodeAttribute)));
            Assert.True(machine.GetMethod("SetStateMachine")!.IsDefined(typeof(DebuggerHiddenAttribute)));
            var make = source.GetMethod("Make", BindingFlags.Static | BindingFlags.NonPublic)!;
            Assert.False(make.IsDefined(typeof(DebuggerNonUserCodeAttribute)));
            var read = (Delegate)make.Invoke(null, [42])!;
            Assert.Equal(42, read.DynamicInvoke());
            Assert.True(read.Target!.GetType().IsDefined(typeof(CompilerGeneratedAttribute)));
            Assert.False(read.Method.IsDefined(typeof(DebuggerHiddenAttribute)));
            Assert.Equal("function(Value = {Value})", read.Target.GetType().GetCustomAttribute<DebuggerDisplayAttribute>()!.Value);
            var initializer = source.GetNestedType("R", BindingFlags.NonPublic)!
                .GetMethod("<Default:X>", BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Static)!;
            Assert.False(initializer.IsDefined(typeof(DebuggerNonUserCodeAttribute)));
            Assert.False(initializer.IsDefined(typeof(DebuggerHiddenAttribute)));
        });
    }

    private static object Proxy(Assembly assembly, object value)
    {
        var type = value.GetType();
        var name = type.GetCustomAttribute<DebuggerTypeProxyAttribute>()!.ProxyTypeName.Split(',')[0];
        var proxy = assembly.GetType(name)!;
        if (proxy.IsGenericTypeDefinition) proxy = proxy.MakeGenericType(type.GetGenericArguments());
        return Activator.CreateInstance(proxy, [value])!;
    }

    private static CompiledProgram Compile(string source, bool symbols = true, bool optimize = false, CompilationTarget target = CompilationTarget.Library)
    {
        var result = FpasCompiler.Compile(new([new("views.fpas", source)], "DebugViews",
            [.. TestEnvironment.References, typeof(Core.RecordAttribute).Assembly.Location], target, EmitDebugInformation: symbols, Optimize: optimize));
        Assert.True(result.Success, string.Join(';', result.Diagnostics));
        return result.Program!;
    }

    private static void WithAssembly(CompiledProgram program, Action<Assembly> check)
    {
        var context = new AssemblyLoadContext("debug-attributes", isCollectible: true);
        try { check(context.LoadFromStream(new MemoryStream(program.AssemblyImage))); }
        finally { context.Unload(); }
    }
}
