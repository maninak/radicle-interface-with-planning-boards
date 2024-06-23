use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use axum::body::{Body, Bytes};
use axum::http::{Method, Request};
use axum::Router;
use serde_json::Value;
use time::OffsetDateTime;
use tower::ServiceExt;

use radicle::cob::patch::MergeTarget;
use radicle::crypto::ssh::keystore::MemorySigner;
use radicle::crypto::ssh::Keystore;
use radicle::crypto::{KeyPair, Seed, Signer};
use radicle::git::{raw as git2, RefString};
use radicle::identity::Visibility;
use radicle::profile::{env, Home};
use radicle::storage::ReadStorage;
use radicle::Storage;
use radicle::{node, profile};
use radicle_crypto::test::signer::MockSigner;

use crate::api::{auth, Context};

pub const RID: &str = "rad:z4FucBZHZMCsxTyQE1dfE2YR59Qbp";
pub const RID_PRIVATE: &str = "rad:zLuTzcmoWMcdK37xqArS8eckp9vK";
pub const HEAD: &str = "e8c676b9e3b42308dc9d218b70faa5408f8e58ca";
pub const PARENT: &str = "ee8d6a29304623a78ebfa5eeed5af674d0e58f83";
pub const INITIAL_COMMIT: &str = "f604ce9fd5b7cc77b7609beda45ea8760bee78f7";
pub const DID: &str = "did:key:z6MknSLrJoTcukLrE435hVNQT4JUhbvWLX4kUzqkEStBU8Vi";
pub const ISSUE_ID: &str = "ca67d195c0b308b51810dedd93157a20764d5db5";
pub const ISSUE_DISCUSSION_ID: &str = "41e2823caa54f1d53e375035ed4aabd0a89fa855";
pub const ISSUE_COMMENT_ID: &str = "e9f963fab82ad875e46b29a327c5d3d51f825cdc";
pub const SESSION_ID: &str = "u9MGAkkfkMOv0uDDB2WeUHBT7HbsO2Dy";
pub const TIMESTAMP: u64 = 1671125284;
pub const CONTRIBUTOR_RID: &str = "rad:z4XaCmN3jLSeiMvW15YTDpNbDHFhG";
pub const CONTRIBUTOR_DID: &str = "did:key:z6Mkk7oqY4pPxhMmGEotDYsFo97vhCj85BLY1H256HrJmjN8";
pub const CONTRIBUTOR_ALIAS: &str = "seed";
pub const CONTRIBUTOR_PATCH_ID: &str = "3e3f0dc34b3eeb64cfbc7218fbd52b97246e0564";

/// Create a new profile.
pub fn profile(home: &Path, seed: [u8; 32]) -> radicle::Profile {
    let home = Home::new(home).unwrap();
    let keystore = Keystore::new(&home.keys());
    let keypair = KeyPair::from_seed(Seed::from(seed));
    let alias = node::Alias::new("seed");
    let storage = Storage::open(
        home.storage(),
        radicle::git::UserInfo {
            alias: alias.clone(),
            key: keypair.pk.into(),
        },
    )
    .unwrap();

    let mut db = home.policies_mut().unwrap();
    db.follow(&keypair.pk.into(), Some(&alias)).unwrap();

    radicle::storage::git::transport::local::register(storage.clone());
    keystore.store(keypair.clone(), "radicle", None).unwrap();

    radicle::Profile {
        home,
        storage,
        keystore,
        public_key: keypair.pk.into(),
        config: profile::Config::new(alias),
    }
}

pub fn seed(dir: &Path) -> Context {
    let home = dir.join("radicle");
    let profile = profile(home.as_path(), [0xff; 32]);
    let signer = Box::new(MockSigner::from_seed([0xff; 32]));

    crate::logger::init().ok();

    seed_with_signer(dir, profile, &signer)
}

pub fn contributor(dir: &Path) -> Context {
    let mut seed = [0xff; 32];
    *seed.last_mut().unwrap() = 0xee;

    let home = dir.join("radicle");
    let profile = profile(home.as_path(), seed);
    let signer = MemorySigner::load(&profile.keystore, None).unwrap();

    seed_with_signer(dir, profile, &signer)
}

fn seed_with_signer<G: Signer>(dir: &Path, profile: radicle::Profile, signer: &G) -> Context {
    const DEFAULT_BRANCH: &str = "master";

    crate::logger::init().ok();

    profile.policies_mut().unwrap();
    profile.database_mut().unwrap(); // Create the database.

    let mut policies = profile.policies_mut().unwrap();
    let workdir = dir.join("hello-world-private");
    fs::create_dir_all(&workdir).unwrap();

    // add commits to workdir (repo)
    let mut opts = git2::RepositoryInitOptions::new();
    opts.initial_head(DEFAULT_BRANCH);
    let repo = git2::Repository::init_opts(&workdir, &opts).unwrap();
    let tree = radicle::git::write_tree(
        Path::new("README"),
        "Hello Private World!\n".as_bytes(),
        &repo,
    )
    .unwrap();

    let sig_time = git2::Time::new(1673001014, 0);
    let sig = git2::Signature::new("Alice Liddell", "alice@radicle.xyz", &sig_time).unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit\n", &tree, &[])
        .unwrap();

    // rad init
    let repo = git2::Repository::open(&workdir).unwrap();
    let name = "hello-world-private".to_string();
    let description = "Private Rad repository for tests".to_string();
    let branch = RefString::try_from(DEFAULT_BRANCH).unwrap();
    let visibility = Visibility::Private {
        allow: BTreeSet::default(),
    };
    let (rid, _, _) = radicle::rad::init(
        &repo,
        &name,
        &description,
        branch,
        visibility,
        signer,
        &profile.storage,
    )
    .unwrap();

    policies.seed(&rid, node::policy::Scope::All).unwrap();

    let workdir = dir.join("hello-world");

    env::set_var(env::GIT_COMMITTER_DATE, TIMESTAMP.to_string());

    fs::create_dir_all(&workdir).unwrap();

    // add commits to workdir (repo)
    let mut opts = git2::RepositoryInitOptions::new();
    opts.initial_head(DEFAULT_BRANCH);
    let repo = git2::Repository::init_opts(&workdir, &opts).unwrap();
    let tree =
        radicle::git::write_tree(Path::new("README"), "Hello World!\n".as_bytes(), &repo).unwrap();

    let sig_time = git2::Time::new(1673001014, 0);
    let sig = git2::Signature::new("Alice Liddell", "alice@radicle.xyz", &sig_time).unwrap();

    let oid = repo
        .commit(Some("HEAD"), &sig, &sig, "Initial commit\n", &tree, &[])
        .unwrap();
    let commit = repo.find_commit(oid).unwrap();

    repo.checkout_tree(tree.as_object(), None).unwrap();

    let tree = radicle::git::write_tree(
        Path::new("CONTRIBUTING"),
        "Thank you very much!\n".as_bytes(),
        &repo,
    )
    .unwrap();
    let sig_time = git2::Time::new(1673002014, 0);
    let sig = git2::Signature::new("Alice Liddell", "alice@radicle.xyz", &sig_time).unwrap();

    let oid2 = repo
        .commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Add contributing file\n",
            &tree,
            &[&commit],
        )
        .unwrap();
    let commit2 = repo.find_commit(oid2).unwrap();

    repo.checkout_tree(tree.as_object(), None).unwrap();

    fs::create_dir(workdir.join("dir1")).unwrap();
    fs::write(
        workdir.join("dir1").join("README"),
        "Hello World from dir1!\n",
    )
    .unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_all(["."], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();

    let oid = index.write_tree().unwrap();
    let tree = repo.find_tree(oid).unwrap();

    let sig_time = git2::Time::new(1673003014, 0);
    let sig = git2::Signature::new("Alice Liddell", "alice@radicle.xyz", &sig_time).unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "Add another folder\n",
        &tree,
        &[&commit2],
    )
    .unwrap();

    // rad init
    let repo = git2::Repository::open(&workdir).unwrap();
    let name = "hello-world".to_string();
    let description = "Rad repository for tests".to_string();
    let branch = RefString::try_from(DEFAULT_BRANCH).unwrap();
    let visibility = Visibility::default();
    let (rid, _, _) = radicle::rad::init(
        &repo,
        &name,
        &description,
        branch,
        visibility,
        signer,
        &profile.storage,
    )
    .unwrap();
    policies.seed(&rid, node::policy::Scope::All).unwrap();

    let storage = &profile.storage;
    let repo = storage.repository(rid).unwrap();
    let mut issues = profile.issues_mut(&repo).unwrap();
    let issue = issues
        .create(
            "Issue #1".to_string(),
            "Change 'hello world' to 'hello everyone'".to_string(),
            &[],
            &[],
            [],
            signer,
        )
        .unwrap();
    tracing::debug!(target: "test", "Contributor issue: {}", issue.id());

    // eq. rad patch open
    let mut patches = profile.patches_mut(&repo).unwrap();
    let oid = radicle::git::Oid::from_str(HEAD).unwrap();
    let base = radicle::git::Oid::from_str(PARENT).unwrap();
    let patch = patches
        .create(
            "A new `hello world`",
            "change `hello world` in README to something else",
            MergeTarget::Delegates,
            base,
            oid,
            &[],
            signer,
        )
        .unwrap();
    tracing::debug!(target: "test", "Contributor patch: {}", patch.id());

    let workdir = dir.join("again-hello-world");

    env::set_var(env::GIT_COMMITTER_DATE, TIMESTAMP.to_string());

    fs::create_dir_all(&workdir).unwrap();

    // add commits to workdir (repo)
    let mut opts = git2::RepositoryInitOptions::new();
    opts.initial_head(DEFAULT_BRANCH);
    let repo = git2::Repository::init_opts(&workdir, &opts).unwrap();
    let tree = radicle::git::write_tree(
        Path::new("README"),
        "Hello World Again!\n".as_bytes(),
        &repo,
    )
    .unwrap();

    let sig_time = git2::Time::new(1673001014, 0);
    let sig = git2::Signature::new("Alice Liddell", "alice@radicle.xyz", &sig_time).unwrap();

    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit\n", &tree, &[])
        .unwrap();

    repo.checkout_tree(tree.as_object(), None).unwrap();

    // rad init
    let repo = git2::Repository::open(&workdir).unwrap();
    let name = "again-hello-world".to_string();
    let description = "Rad repository for sorting".to_string();
    let branch = RefString::try_from(DEFAULT_BRANCH).unwrap();
    let visibility = Visibility::default();
    let (rid, _, _) = radicle::rad::init(
        &repo,
        &name,
        &description,
        branch,
        visibility,
        signer,
        &profile.storage,
    )
    .unwrap();
    policies.seed(&rid, node::policy::Scope::All).unwrap();

    let options = crate::Options {
        aliases: std::collections::HashMap::new(),
        listen: std::net::SocketAddr::from(([0, 0, 0, 0], 8080)),
        cache: Some(crate::DEFAULT_CACHE_SIZE),
    };

    Context::new(Arc::new(profile), &options)
}

/// Adds an authorized session to the Context::sessions HashMap.
pub async fn create_session(ctx: Context) {
    let issued_at = OffsetDateTime::now_utc();
    let mut sessions = ctx.sessions().write().await;
    sessions.insert(
        String::from(SESSION_ID),
        auth::Session {
            status: auth::AuthState::Authorized,
            public_key: ctx.profile().public_key,
            alias: ctx.profile().config.node.alias.clone(),
            issued_at,
            expires_at: issued_at
                .checked_add(auth::AUTHORIZED_SESSIONS_EXPIRATION)
                .unwrap(),
        },
    );
}

pub async fn get(app: &Router, path: impl ToString) -> Response {
    Response(
        app.clone()
            .oneshot(request(path, Method::GET, None, None))
            .await
            .unwrap(),
    )
}

pub async fn post(
    app: &Router,
    path: impl ToString,
    body: Option<Body>,
    auth: Option<String>,
) -> Response {
    Response(
        app.clone()
            .oneshot(request(path, Method::POST, body, auth))
            .await
            .unwrap(),
    )
}

pub async fn patch(
    app: &Router,
    path: impl ToString,
    body: Option<Body>,
    auth: Option<String>,
) -> Response {
    Response(
        app.clone()
            .oneshot(request(path, Method::PATCH, body, auth))
            .await
            .unwrap(),
    )
}

pub async fn put(
    app: &Router,
    path: impl ToString,
    body: Option<Body>,
    auth: Option<String>,
) -> Response {
    Response(
        app.clone()
            .oneshot(request(path, Method::PUT, body, auth))
            .await
            .unwrap(),
    )
}

fn request(
    path: impl ToString,
    method: Method,
    body: Option<Body>,
    auth: Option<String>,
) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(path.to_string())
        .header("Content-Type", "application/json");
    if let Some(token) = auth {
        request = request.header("Authorization", format!("Bearer {token}"));
    }

    request.body(body.unwrap_or_else(Body::empty)).unwrap()
}

#[derive(Debug)]
pub struct Response(axum::response::Response);

impl Response {
    pub async fn json(self) -> Value {
        let body = self.body().await;
        serde_json::from_slice(&body).unwrap()
    }

    pub async fn id(self) -> radicle::git::Oid {
        let json = self.json().await;
        let string = json["id"].as_str().unwrap();

        radicle::git::Oid::from_str(string).unwrap()
    }

    pub async fn success(self) -> bool {
        let json = self.json().await;
        let success = json["success"].as_bool();

        success.unwrap_or(false)
    }

    pub fn status(&self) -> axum::http::StatusCode {
        self.0.status()
    }

    pub async fn body(self) -> Bytes {
        axum::body::to_bytes(self.0.into_body(), usize::MAX)
            .await
            .unwrap()
    }
}
