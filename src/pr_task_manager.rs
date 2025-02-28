use octocrab::{models::pulls::PullRequest, Octocrab};
use anyhow::Result;
use octocrab::params::State;

/// Manages PR-related tasks for a given repository.
pub struct PrTaskManager {
    octocrab: Octocrab,
    owner: String,
    repo: String,
}

impl PrTaskManager {
    /// Creates a new `PrTaskManager` instance.
    ///
    /// If a personal access token is provided, it will be used for authentication.
    pub async fn new(owner: &str, repo: &str, token: Option<&str>) -> Self {
        let builder = Octocrab::builder();
        let octocrab = if let Some(token) = token {
            builder.personal_token(token.to_string()).build().unwrap()
        } else {
            builder.build().unwrap()
        };

        PrTaskManager {
            octocrab,
            owner: owner.to_string(),
            repo: repo.to_string(),
        }
    }

    /// Retrieves all pull requests for the repository that match the given task ID.
    ///
    /// A pull request is considered to be related to the task if its head branch name contains the `task_id`.
    pub async fn get_prs_for_task(&self, task_id: &str) -> Result<Vec<PullRequest>> {
        let mut all_prs = Vec::new();

        let mut page = self.octocrab
            .pulls(&self.owner, &self.repo)
            .list()
            .state(State::All)
            .per_page(100)
            .send()
            .await?;

        loop {
            let mut filtered: Vec<PullRequest> = page
                .items
                .into_iter()
                .filter(|pr| pr.head.ref_field.contains(task_id))
                .collect();

            all_prs.append(&mut filtered);

            if let Some(next_page) = self.octocrab.get_page(&page.next).await? {
                page = next_page;
            } else {
                break;
            }
        }

        Ok(all_prs)
    }

    /// Checks if the specified pull request already has a comment containing the given search text.
    pub async fn pr_has_comment_with_text(&self, pr: &PullRequest, search_text: &str) -> Result<bool> {
        let pr_number = pr.number;
        let mut page = self.octocrab
            .issues(&self.owner, &self.repo)
            .list_comments(pr_number)
            .per_page(100)
            .send()
            .await?;

        loop {
            if page.items.iter().any(|comment| {
                comment.body.as_ref().map_or(false, |body| body.contains(search_text))
            }) {
                return Ok(true);
            }

            if let Some(next_page) = self.octocrab.get_page(&page.next).await? {
                page = next_page;
            } else {
                break;
            }
        }

        Ok(false)
    }

    /// Adds a comment to the specified pull request.
    pub async fn add_comment_to_pr(&self, pr: &PullRequest, comment: &str) -> Result<()> {
        let pr_number = pr.number;
        self.octocrab
            .issues(&self.owner, &self.repo)
            .create_comment(pr_number, comment)
            .await?;
        Ok(())
    }
}
