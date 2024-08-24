//! This crate exports some GitHub API bindings through [`GitHub`].

use std::collections::HashSet;

use futures::TryFutureExt;
use reqwest::{header, Client, Response, Result};
use serde::Deserialize;
use tracing::{debug, info, instrument, warn, Level};

/// Asynchronous GitHub API bindings that wraps a [`reqwest::Client`] internally,
/// so it's safe and cheap to clone this struct and send it to different threads.
#[derive(Debug, Clone)]
pub struct GitHub {
    client: Client
}

impl GitHub {
    /// Creates a new [`GitHub`] interface with personal access token.
    ///
    /// # Panics
    ///
    /// Panics if the argument contains invalid header value characters.
    ///
    /// # Errors
    ///
    /// Fails if a TLS backend cannot be initialized, or the resolver
    /// cannot load the system configuration.
    pub fn with_token(token: &str) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("User-Agent", header::HeaderValue::from_static("gfas"));
        headers.insert("Authorization", format!("token {token}").parse().unwrap());

        let client = Client::builder().default_headers(headers).build()?;

        Ok(Self { client })
    }

    /// Paginates through the given user profile link and returns
    /// discovered users collected in [`HashSet`].
    ///
    /// `role` should be either `"following"` or `"followers"`.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during sending requests.
    #[instrument(skip(self), ret(level = Level::TRACE), err)]
    pub async fn explore(&self, user: &str, role: &str) -> Result<HashSet<String>> {
        let mut res = HashSet::new();

        let url = format!("https://api.github.com/users/{user}/{role}");

        #[derive(Deserialize)]
        struct User {
            login: String
        }

        const PER_PAGE: usize = 100;

        for page in 1.. {
            debug!("page {page}");

            let users: Vec<_> = self
                .client
                .get(&url)
                .query(&[("page", page), ("per_page", PER_PAGE)])
                .send()
                .and_then(|r| r.json::<Vec<User>>())
                .await?
                .into_iter()
                .map(|u| u.login)
                .collect();

            let len = users.len();

            res.extend(users);

            info!("{}(+{len})", res.len());

            if len < PER_PAGE {
                break;
            }
        }

        Ok(res)
    }

    /// Follows a user.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during sending the request.
    #[instrument(skip(self), ret(level = Level::TRACE), err)]
    pub async fn follow(&self, user: &str) -> Result<Response> {
        warn!("");

        self.client.put(format!("https://api.github.com/user/following/{user}")).send().await
    }

    /// Unfollows a user.
    ///
    /// # Errors
    ///
    /// Fails if an error occurs during sending the request.
    #[instrument(skip(self), ret(level = Level::TRACE), err)]
    pub async fn unfollow(&self, user: &str) -> Result<Response> {
        warn!("");

        self.client.delete(format!("https://api.github.com/user/following/{user}")).send().await
    }
}
