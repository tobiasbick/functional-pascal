//! Metadata-only dependency closure for native errors in imported signatures.
use fpas_unit::interface::InterfaceType as Input;

impl crate::check::Checker {
    pub(super) fn install_native_error_dependencies(&mut self, root: &Input) {
        let mut pending = vec![root];
        while let Some(ty) = pending.pop() {
            match ty {
                Input::Named(name) => crate::std_registry::register_error_type(self, name),
                Input::Array(t) | Input::Option(t) | Input::Task(t) | Input::Channel(t) => {
                    pending.push(t)
                }
                Input::Dictionary(k, v) | Input::Result(k, v) => {
                    pending.extend([k.as_ref(), v.as_ref()])
                }
                Input::Enum(e) => {
                    crate::std_registry::register_error_type(self, &e.name);
                    pending.extend(e.type_arguments.iter());
                    pending.extend(
                        e.variants
                            .iter()
                            .flat_map(|v| v.fields.iter().map(|f| &f.ty)),
                    );
                }
                Input::Record(r) => {
                    pending.extend(r.type_arguments.iter());
                    pending.extend(r.fields.iter().map(|f| &f.ty));
                    for method in r.methods.iter().chain(&r.static_routines) {
                        pending.extend(method.callable.parameters.iter().map(|p| &p.ty));
                        pending.extend(method.callable.result.as_deref());
                    }
                }
                Input::Function(f) | Input::Procedure(f) => {
                    pending.extend(f.parameters.iter().map(|p| &p.ty));
                    pending.extend(f.result.as_deref());
                }
                Input::Application { arguments, .. } | Input::EnumApplication { arguments, .. } => {
                    pending.extend(arguments)
                }
                Input::Distinct(d) => pending.push(&d.underlying),
                _ => {}
            }
        }
    }
}
