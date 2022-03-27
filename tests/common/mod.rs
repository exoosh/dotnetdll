use dotnetdll::prelude::*;
use std::process::Command;
use tempfile::TempDir;

pub struct WriteContext<'a> {
    pub resolution: Resolution<'a>,
    pub mscorlib: AssemblyRefIndex,
    pub console: TypeRefIndex,
    pub object_ctor: MethodRefIndex,
    pub class: TypeIndex,
    pub default_ctor: MethodIndex,
}

#[allow(unused_macros)]
macro_rules! asm {
    ($ins:ident) => {
        Instruction::$ins
    };
    ($ins:ident $($param:expr),+) => {
        Instruction::$ins($($param),+)
    };
    ($($ins:ident $($param:expr),*;)*) => {
        vec![
            $(
                $crate::common::asm! { $ins $($param),* }
            ),*
        ]
    }
}
#[allow(unused_imports)]
pub(crate) use asm;

pub fn write_fixture(
    name: &str,
    test: impl FnOnce(&mut WriteContext) -> (Vec<LocalVariable>, Vec<Instruction>),
    expect: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let dll_name = format!("{}.dll", name);

    let mut res = Resolution::new(Module::new(&dll_name));
    res.assembly = Some(Assembly::new(name));
    res.push_global_module_type();

    let mscorlib = res.push_assembly_reference(ExternalAssemblyReference::new("mscorlib"));

    let console = res.push_type_reference(type_ref! { System.Console in #mscorlib });

    let object = res.push_type_reference(type_ref! { System.Object in #mscorlib });

    let class = res.push_type_definition(TypeDefinition::new(None, "Program"));
    res[class].extends = Some(object.into());

    let object_type = BaseType::class(object).into();
    let object_ctor = res.push_method_reference(method_ref! { void #object_type::.ctor() });

    let default_ctor = res.push_method(
        class,
        Method::new(
            Accessibility::Public,
            msig! { void () },
            ".ctor",
            Some(body::Method::new(asm! {
                LoadArgument 0;
                call object_ctor;
                Return;
            })),
        ),
    );
    res[default_ctor].special_name = true;
    res[default_ctor].runtime_special_name = true;

    let mut ctx = WriteContext {
        resolution: res,
        mscorlib,
        console,
        class,
        default_ctor,
        object_ctor,
    };

    let (vars, ins) = test(&mut ctx);

    let main = ctx.resolution.push_method(
        class,
        Method::new(
            Accessibility::Public,
            msig! { static void (string[]) },
            "Main",
            Some(body::Method::with_locals(vars, ins)),
        ),
    );

    ctx.resolution.entry_point = Some(main.into());

    let written = DLL::write(&ctx.resolution, false, true)?;

    let dir = TempDir::new()?;

    let dll_path = dir.path().join(&dll_name);
    std::fs::write(&dll_path, written)?;

    std::fs::copy(
        "tests/common/test.runtimeconfig.json",
        dir.path().join(format!("{}.runtimeconfig.json", name)),
    )?;

    let output = Command::new("dotnet").arg(&dll_path).output()?;

    let stderr = String::from_utf8(output.stderr)?;

    println!("{}", stderr);
    if stderr.contains("Unhandled exception") {
        if let Ok(i) = std::env::var("ILDASM") {
            let ildasm = Command::new(i).arg(&dll_path).output()?;
            println!("{}", String::from_utf8(ildasm.stdout)?);
        }

        // Command::new("gdb")
        //     .arg("-ex")
        //     .arg("set substitute-path /runtime /home/nick/Desktop/runtime")
        //     .arg("--args")
        //     .arg("/home/nick/Desktop/runtime/artifacts/bin/testhost/net7.0-Linux-Debug-x64/shared/Microsoft.NETCore.App/7.0.0/corerun")
        //     .arg(&dll_path)
        //     .spawn()
        //     .unwrap()
        //     .wait()
        //     .unwrap();

        let ilverify = Command::new("ilverify")
            .arg(dll_path)
            .arg("-r")
            .arg("/usr/share/dotnet/shared/Microsoft.NETCore.App/6.0.2/*.dll")
            .output()?;
        println!("{}", String::from_utf8(ilverify.stdout)?);
    }

    assert_eq!(output.stdout, expect);

    Ok(())
}
