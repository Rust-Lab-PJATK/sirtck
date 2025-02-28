use std::sync::Arc;
use futures::stream::{self, StreamExt};
use std::env;

use crate::check_branch_manager::check_branch_via_task;
use crate::file_task_manager::{FileTaskRepository, TaskRepository};
use crate::pr_task_manager::PrTaskManager;

mod file_task_manager;
mod entity;
mod pr_task_manager;
mod check_branch_manager;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let tasks_path = std::path::PathBuf::from("tasks/tasks.json");

    let mut repo = FileTaskRepository::new(tasks_path);
    let all_tasks = repo.get_all();
    println!("Loaded tasks: {:?}", all_tasks);

    let token = env::var("GITHUB_TOKEN").ok();
    let owner = &env::var("GITHUB_OWNER").ok().unwrap();
    let github_repo_name = &env::var("GITHUB_REPO_NAME").ok().unwrap();

    let pr_manager = Arc::new(
        PrTaskManager::new(owner, github_repo_name, token.as_deref())
            .await,
    );

    stream::iter(all_tasks)
        .for_each_concurrent(None, |task| {
            let pr_manager = Arc::clone(&pr_manager);
            async move {
                match pr_manager.get_prs_for_task(&task.id).await {
                    Ok(pr_list) => {
                        let task_clone = task.clone();
                        stream::iter(pr_list)
                            .for_each_concurrent(None, |pr| {
                                let pr_manager = Arc::clone(&pr_manager);
                                let task_clone = task_clone.clone();
                                async move {
                                    if !pr_manager
                                        .pr_has_comment_with_text(&pr, "Score")
                                        .await
                                        .expect("Error checking PR comments")
                                    {
                                        let score = check_branch_via_task(&task_clone, &pr);
                                        let comment = format!(
                                            "## 🦀 Test Results 🦀\n\n**Score:** {}/{}\n\n### Test Output\n```\n{}\n```\n\n### Clippy Output\n```\n{}\n```",
                                            score.received_points,
                                            score.max_points,
                                            score.test_output,
                                            score.clippy_output
                                        );
                                        pr_manager
                                            .add_comment_to_pr(&pr, &comment)
                                            .await
                                            .unwrap();
                                    }
                                }
                            })
                            .await;
                    }
                    Err(e) => {
                        eprintln!("Error fetching PRs for task {}: {:?}", task.id, e);
                    }
                }
            }
        })
        .await;
}
