use anyhow::Result;
use futures::future::join_all;
use std::fmt::Debug;
use tokio::task::JoinHandle;

pub async fn run_all<T, E>(tasks: Vec<JoinHandle<Result<Option<T>, E>>>) -> Result<Vec<T>, E>
where
    E: Debug,
{
    let results = join_all(tasks)
        .await
        .into_iter()
        .filter_map(|r| match r {
            Ok(inner_result) => inner_result.unwrap_or_else(|e| {
                eprintln!("Task execution error: {:?}", e);
                None
            }),
            Err(join_err) => {
                eprintln!("Task join error: {:?}", join_err);
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(results)
}
