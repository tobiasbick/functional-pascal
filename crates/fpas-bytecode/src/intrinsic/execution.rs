//! Runtime ownership and mutable-dispatch metadata for intrinsic instructions.

use super::{
    ArrayIntrinsic, DictIntrinsic, Intrinsic, OptionIntrinsic, ResultIntrinsic, TestIntrinsic,
    TimeIntrinsic,
};

/// Runtime module that owns execution of an intrinsic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntrinsicOwner {
    /// Pure or host-library implementation in `fpas-std`.
    Standard,
    /// VM implementation backed by process, console, network, or test state.
    Hosted,
    /// VM implementation that invokes an FPAS callback.
    Callback,
    /// VM task scheduler implementation.
    Task,
}

impl Intrinsic {
    /// Return the single runtime module that owns this intrinsic.
    #[must_use]
    pub const fn owner(self) -> IntrinsicOwner {
        match self {
            Self::Args(_) | Self::Console(_) | Self::Net(_) | Self::Http(_) => {
                IntrinsicOwner::Hosted
            }
            Self::Test(
                TestIntrinsic::AssertScreenLine
                | TestIntrinsic::AssertScreenCell
                | TestIntrinsic::PushReadLn
                | TestIntrinsic::ScratchDir,
            ) => IntrinsicOwner::Hosted,
            Self::Task(_) => IntrinsicOwner::Task,
            Self::Array(
                ArrayIntrinsic::Map
                | ArrayIntrinsic::Filter
                | ArrayIntrinsic::Reduce
                | ArrayIntrinsic::Find
                | ArrayIntrinsic::FindIndex
                | ArrayIntrinsic::Any
                | ArrayIntrinsic::All
                | ArrayIntrinsic::FlatMap
                | ArrayIntrinsic::ForEach,
            )
            | Self::Dict(DictIntrinsic::Map | DictIntrinsic::Filter)
            | Self::Result(
                ResultIntrinsic::Map | ResultIntrinsic::AndThen | ResultIntrinsic::OrElse,
            )
            | Self::Option(
                OptionIntrinsic::Map | OptionIntrinsic::AndThen | OptionIntrinsic::OrElse,
            ) => IntrinsicOwner::Callback,
            _ => IntrinsicOwner::Standard,
        }
    }

    /// Return whether execution may need mutable scheduler state or owned arguments.
    #[must_use]
    pub const fn requires_mutable_dispatch(self) -> bool {
        matches!(self, Self::Task(_) | Self::Time(TimeIntrinsic::Sleep))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ArgsIntrinsic, ConsoleIntrinsic, NetIntrinsic, TaskIntrinsic};

    #[test]
    fn complete_intrinsic_families_have_one_non_standard_owner() {
        assert!(Intrinsic::all().all(|intrinsic| match intrinsic {
            Intrinsic::Args(_) | Intrinsic::Console(_) | Intrinsic::Net(_) | Intrinsic::Http(_) =>
                intrinsic.owner() == IntrinsicOwner::Hosted,
            Intrinsic::Task(_) => intrinsic.owner() == IntrinsicOwner::Task,
            _ => true,
        }));
    }

    #[test]
    fn split_families_classify_hosted_and_callback_members() {
        let expected = [
            (
                Intrinsic::Array(ArrayIntrinsic::Map),
                IntrinsicOwner::Callback,
            ),
            (
                Intrinsic::Dict(DictIntrinsic::Filter),
                IntrinsicOwner::Callback,
            ),
            (
                Intrinsic::Result(ResultIntrinsic::AndThen),
                IntrinsicOwner::Callback,
            ),
            (
                Intrinsic::Option(OptionIntrinsic::OrElse),
                IntrinsicOwner::Callback,
            ),
            (
                Intrinsic::Test(TestIntrinsic::AssertScreenLine),
                IntrinsicOwner::Hosted,
            ),
            (
                Intrinsic::Test(TestIntrinsic::ScratchDir),
                IntrinsicOwner::Hosted,
            ),
            (
                Intrinsic::Test(TestIntrinsic::AssertTrue),
                IntrinsicOwner::Standard,
            ),
            (
                Intrinsic::Array(ArrayIntrinsic::Length),
                IntrinsicOwner::Standard,
            ),
        ];
        assert!(
            expected
                .into_iter()
                .all(|(intrinsic, owner)| intrinsic.owner() == owner)
        );
    }

    #[test]
    fn only_task_wait_and_sleep_require_mutable_dispatch() {
        let actual = Intrinsic::all()
            .filter(|intrinsic| intrinsic.requires_mutable_dispatch())
            .collect::<Vec<_>>();
        assert_eq!(
            actual,
            vec![
                Intrinsic::Task(TaskIntrinsic::Wait),
                Intrinsic::Task(TaskIntrinsic::WaitAll),
                Intrinsic::Time(TimeIntrinsic::Sleep),
            ]
        );
    }

    #[test]
    fn representative_complete_family_members_keep_expected_owner() {
        assert_eq!(
            [
                Intrinsic::Args(ArgsIntrinsic::ParamCount),
                Intrinsic::Console(ConsoleIntrinsic::WriteLn),
                Intrinsic::Net(NetIntrinsic::Close),
            ]
            .map(Intrinsic::owner),
            [IntrinsicOwner::Hosted; 3]
        );
    }
}
