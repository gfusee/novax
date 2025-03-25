use novax::errors::NovaXError;
use novax_executor::ExecutorError;

pub(crate) fn map_novax_error_to_executor_error<F>(
    error: NovaXError,
    or_else: F
) -> ExecutorError
where
    F: FnOnce(NovaXError) -> ExecutorError
{
    match error {
        NovaXError::Executor(error) => error,
        _ => or_else(error).into()
    }
}