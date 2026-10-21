//! Ownership evidence for native values and writable collection receivers.

use super::super::super::Checker;
use crate::TransferCapability;
use crate::std_registry::{NativeLowering, NativeOperation};
use crate::types::Ty;
use fpas_parser::Expr;

struct RetainedInputs {
    receiver: bool,
    arguments: &'static [(&'static str, usize)],
    unknown_result: bool,
    callback: Option<(&'static str, usize)>,
}

impl Checker {
    /// Synthesized designator receivers retain the capability of their storage root.
    pub(in crate::check) fn native_receiver_capability(
        &self,
        receiver: &Expr,
        ty: &Ty,
    ) -> TransferCapability {
        if !self.type_can_contain_callable(ty) {
            return TransferCapability::Transferable;
        }
        if let Expr::Designator(designator) = receiver {
            return self
                .designator_root_symbol(&designator.parts)
                .map_or(TransferCapability::Unknown, |(symbol, _)| symbol.capability);
        }
        self.expr_capability(Self::expr_lookup_key(receiver))
    }

    /// Only retained inputs contribute local evidence; invoking a callback does not retain it.
    pub(in crate::check) fn native_result_capability(
        &self,
        operation: &NativeOperation,
        receiver: TransferCapability,
        args: &[Expr],
        result: &Ty,
    ) -> TransferCapability {
        use TransferCapability::{Transferable, Unknown};
        if !self.type_can_contain_callable(result) {
            return Transferable;
        }
        let retained = self.native_value_inputs(operation, result);
        let argument = |name: &str, position: usize| {
            if args.iter().any(|arg| arg.argument_name().is_some()) {
                args.iter().find(|arg| {
                    arg.argument_name()
                        .is_some_and(|actual| actual.eq_ignore_ascii_case(name))
                })
            } else {
                args.get(position)
            }
        };
        let mut capability = if let Some((name, position)) = retained.callback {
            argument(name, position).map_or(Unknown, |value| {
                self.callable_result_capability(value.argument_value())
            })
        } else if retained.unknown_result {
            Unknown
        } else {
            Transferable
        };
        if retained.receiver {
            capability = capability.join(receiver);
        }
        for &(name, position) in retained.arguments {
            capability = capability.join(argument(name, position).map_or(Unknown, |value| {
                self.expr_capability(Self::expr_lookup_key(value.argument_value()))
            }));
        }
        capability
    }

    /// Deferred recursive capture provenance follows the same retained input edges.
    pub(in crate::check) fn native_retained_inputs(
        &self,
        operation: &NativeOperation,
        result: &Ty,
    ) -> (bool, &'static [(&'static str, usize)]) {
        let retained = self.native_value_inputs(operation, result);
        (retained.receiver, retained.arguments)
    }

    fn native_value_inputs(&self, operation: &NativeOperation, result: &Ty) -> RetainedInputs {
        if !self.type_can_contain_callable(result) {
            return RetainedInputs {
                receiver: false,
                arguments: &[],
                unknown_result: false,
                callback: None,
            };
        }
        let implementation = match operation.lowering {
            NativeLowering::InPlace(base) => base,
            _ => operation.implementation,
        };
        let callback = match implementation {
            "Std.Arrays.Map"
            | "Std.Arrays.FlatMap"
            | "Std.Dictionaries.Map"
            | "Std.Options.Map"
            | "Std.Options.AndThen"
            | "Std.Options.OrElse"
            | "Std.Results.Map"
            | "Std.Results.MapError"
            | "Std.Results.AndThen"
            | "Std.Results.OrElse" => Some(("F", 0)),
            "Std.Arrays.Reduce" | "Std.Dictionaries.Reduce" | "Std.Str.Reduce" => Some(("F", 1)),
            _ => None,
        };
        let (receiver, arguments, unknown_result): (bool, &'static [(&'static str, usize)], bool) =
            match implementation {
                "Std.Arrays.Fill" => (false, &[("Value", 0)], false),
                "Std.Arrays.Push" => (true, &[("Value", 0)], false),
                "Std.Arrays.Concat" => (true, &[("B", 0)], false),
                "Std.Dictionaries.Merge" => (true, &[("D2", 0)], false),
                "Std.Arrays.Sort"
                | "Std.Arrays.SortBy"
                | "Std.Arrays.Reverse"
                | "Std.Arrays.Filter"
                | "Std.Arrays.Slice"
                | "Std.Arrays.Pop"
                | "Std.Arrays.Find"
                | "Std.Dictionaries.Remove"
                | "Std.Dictionaries.Filter"
                | "Std.Dictionaries.Get"
                | "Std.Dictionaries.Keys"
                | "Std.Dictionaries.Values"
                | "Std.Options.Unwrap"
                | "Std.Results.Unwrap" => (true, &[], false),
                "Std.Arrays.Reduce" | "Std.Dictionaries.Reduce" | "Std.Str.Reduce" => {
                    // An empty input returns Init; a callback can produce a different graph.
                    (false, &[("Init", 0)], true)
                }
                "Std.Options.OrElse" => (true, &[], true),
                "Std.Results.Map" | "Std.Results.AndThen" => match result {
                    Ty::Result(_, error) if self.type_can_contain_callable(error) => {
                        (true, &[], true)
                    }
                    _ => (false, &[], true),
                },
                "Std.Results.MapError" | "Std.Results.OrElse" => match result {
                    Ty::Result(ok, _) if self.type_can_contain_callable(ok) => (true, &[], true),
                    _ => (false, &[], true),
                },
                // Mapping callbacks may return transferable values even with local captures,
                // and transferable callbacks may create new local closures when invoked.
                _ => (false, &[], true),
            };
        RetainedInputs {
            receiver,
            arguments,
            unknown_result,
            callback,
        }
    }
}
