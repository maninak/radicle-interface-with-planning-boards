//! Utilities for building JSON responses of our API.

use std::collections::BTreeMap;
use std::path::Path;
use std::str;

use base64::prelude::{Engine, BASE64_STANDARD};
use radicle::cob::{CodeLocation, Reaction};
use radicle::patch::ReviewId;
use serde_json::{json, Value};

use radicle::cob::issue::{Issue, IssueId};
use radicle::cob::patch::{Merge, Patch, PatchId, Review};
use radicle::cob::thread::{Comment, CommentId, Edit};
use radicle::cob::{ActorId, Author};
use radicle::git::RefString;
use radicle::node::{Alias, AliasStore};
use radicle::prelude::NodeId;
use radicle::storage::{git, refs, RemoteRepository};
use radicle_surf::blob::Blob;
use radicle_surf::tree::{EntryKind, Tree};
use radicle_surf::{Commit, Oid};

use crate::api::auth::Session;

/// Returns JSON of a commit.
pub(crate) fn commit(commit: &Commit) -> Value {
    json!({
      "id": commit.id,
      "author": {
        "name": commit.author.name,
        "email": commit.author.email
      },
      "summary": commit.summary,
      "description": commit.description(),
      "parents": commit.parents,
      "committer": {
        "name": commit.committer.name,
        "email": commit.committer.email,
        "time": commit.committer.time.seconds()
      }
    })
}

/// Returns JSON of a session.
pub(crate) fn session(session_id: String, session: &Session) -> Value {
    json!({
      "sessionId": session_id,
      "status": session.status,
      "publicKey": session.public_key,
      "alias": session.alias,
      "issuedAt": session.issued_at.unix_timestamp(),
      "expiresAt": session.expires_at.unix_timestamp()
    })
}

/// Returns JSON for a blob with a given `path`.
pub(crate) fn blob<T: AsRef<[u8]>>(blob: &Blob<T>, path: &str) -> Value {
    json!({
        "binary": blob.is_binary(),
        "name": name_in_path(path),
        "content": blob_content(blob),
        "path": path,
        "lastCommit": commit(blob.commit())
    })
}

/// Returns a string for the blob content, encoded in base64 if binary.
pub fn blob_content<T: AsRef<[u8]>>(blob: &Blob<T>) -> String {
    match str::from_utf8(blob.content()) {
        Ok(s) => s.to_owned(),
        Err(_) => BASE64_STANDARD.encode(blob.content()),
    }
}

/// Returns JSON for a tree with a given `path` and `stats`.
pub(crate) fn tree(tree: &Tree, path: &str) -> Value {
    let prefix = Path::new(path);
    let entries = tree
        .entries()
        .iter()
        .map(|entry| {
            json!({
                "path": prefix.join(entry.name()),
                "oid": entry.object_id(),
                "name": entry.name(),
                "kind": match entry.entry() {
                    EntryKind::Tree(_) => "tree",
                    EntryKind::Blob(_) => "blob",
                    EntryKind::Submodule { .. } => "submodule"
                },
            })
        })
        .collect::<Vec<_>>();

    json!({
        "entries": &entries,
        "lastCommit": commit(tree.commit()),
        "name": name_in_path(path),
        "path": path,
    })
}

/// Returns JSON for an `issue`.
pub(crate) fn issue(id: IssueId, issue: Issue, aliases: &impl AliasStore) -> Value {
    json!({
        "id": id.to_string(),
        "author": author(&issue.author(), aliases.alias(issue.author().id())),
        "title": issue.title(),
        "state": issue.state(),
        "assignees": issue.assignees().map(|assignee|
            author(&Author::from(*assignee.as_key()), aliases.alias(assignee))
        ).collect::<Vec<_>>(),
        "discussion": issue.comments().map(|(id, c)| issue_comment(id, c, aliases)).collect::<Vec<_>>(),
        "labels": issue.labels().collect::<Vec<_>>(),
    })
}

/// Returns JSON for a `patch`.
pub(crate) fn patch(
    id: PatchId,
    patch: Patch,
    repo: &git::Repository,
    aliases: &impl AliasStore,
) -> Value {
    json!({
        "id": id.to_string(),
        "author": author(patch.author(), aliases.alias(patch.author().id())),
        "title": patch.title(),
        "state": patch.state(),
        "target": patch.target(),
        "labels": patch.labels().collect::<Vec<_>>(),
        "merges": patch.merges().map(|(nid, m)| merge(nid, m, aliases)).collect::<Vec<_>>(),
        "assignees": patch.assignees().map(|assignee|
            author(&Author::from(*assignee), aliases.alias(&assignee))
        ).collect::<Vec<_>>(),
        "revisions": patch.revisions().map(|(id, rev)| {
            json!({
                "id": id,
                "author": author(rev.author(), aliases.alias(rev.author().id())),
                "description": rev.description(),
                "edits": rev.edits().map(|e| edit(e, aliases)).collect::<Vec<_>>(),
                "reactions": rev.reactions().iter().flat_map(|(location, reaction)| {
                    reactions(reaction.iter().fold(BTreeMap::new(), |mut acc: BTreeMap<&Reaction, Vec<_>>, (author, emoji)| {
                        acc.entry(emoji).or_default().push(author);
                        acc
                    }), location.as_ref(), aliases)
                }).collect::<Vec<_>>(),
                "base": rev.base(),
                "oid": rev.head(),
                "refs": get_refs(repo, patch.author().id(), &rev.head()).unwrap_or_default(),
                "discussions": rev.discussion().comments().map(|(id, c)| {
                    patch_comment(id, c, aliases)
                }).collect::<Vec<_>>(),
                "timestamp": rev.timestamp().as_secs(),
                "reviews": patch.reviews_of(id).map(move |(id, r)| {
                    review(id, r, aliases)
                }).collect::<Vec<_>>(),
            })
        }).collect::<Vec<_>>(),
    })
}

/// Returns JSON for a `reaction`.
fn reactions(
    reactions: BTreeMap<&Reaction, Vec<&ActorId>>,
    location: Option<&CodeLocation>,
    aliases: &impl AliasStore,
) -> Vec<Value> {
    reactions
        .into_iter()
        .map(|(emoji, authors)| {
            if let Some(l) = location {
                json!({ "location": l, "emoji": emoji, "authors": authors.into_iter().map(|a|
                    author(&Author::from(*a), aliases.alias(a))
                ).collect::<Vec<_>>()})
            } else {
                json!({ "emoji": emoji, "authors": authors.into_iter().map(|a|
                    author(&Author::from(*a), aliases.alias(a))
                ).collect::<Vec<_>>()})
            }
        })
        .collect::<Vec<_>>()
}

/// Returns JSON for an `author` and fills in `alias` when present.
pub(crate) fn author(author: &Author, alias: Option<Alias>) -> Value {
    match alias {
        Some(alias) => json!({
            "id": author.id,
            "alias": alias,
        }),
        None => json!(author),
    }
}

/// Returns JSON for a patch `Merge` and fills in `alias` when present.
fn merge(nid: &NodeId, merge: &Merge, aliases: &impl AliasStore) -> Value {
    json!({
        "author": author(&Author::from(*nid), aliases.alias(nid)),
        "commit": merge.commit,
        "timestamp": merge.timestamp.as_secs(),
        "revision": merge.revision,
    })
}

/// Returns JSON for a patch `Review` and fills in `alias` when present.
fn review(id: &ReviewId, review: &Review, aliases: &impl AliasStore) -> Value {
    let a = review.author();
    json!({
        "id": id,
        "author": author(a, aliases.alias(a.id())),
        "verdict": review.verdict(),
        "summary": review.summary(),
        "comments": review.comments().map(|(id, c)| review_comment(id, c, aliases)).collect::<Vec<_>>(),
        "timestamp": review.timestamp().as_secs(),
    })
}

/// Returns JSON for an `Edit`.
fn edit(edit: &Edit, aliases: &impl AliasStore) -> Value {
    json!({
      "author": author(&Author::from(edit.author), aliases.alias(&edit.author)),
      "body": edit.body,
      "timestamp": edit.timestamp.as_secs(),
      "embeds": edit.embeds,
    })
}

/// Returns JSON for a Issue `Comment`.
fn issue_comment(id: &CommentId, comment: &Comment, aliases: &impl AliasStore) -> Value {
    json!({
        "id": *id,
        "author": author(&Author::from(comment.author()), aliases.alias(&comment.author())),
        "body": comment.body(),
        "edits": comment.edits().map(|e| edit(e, aliases)).collect::<Vec<_>>(),
        "embeds": comment.embeds().to_vec(),
        "reactions": reactions(comment.reactions(), None, aliases),
        "timestamp": comment.timestamp().as_secs(),
        "replyTo": comment.reply_to(),
        "resolved": comment.is_resolved(),
    })
}

/// Returns JSON for a Patch `Comment`.
fn patch_comment(
    id: &CommentId,
    comment: &Comment<CodeLocation>,
    aliases: &impl AliasStore,
) -> Value {
    json!({
        "id": *id,
        "author": author(&Author::from(comment.author()), aliases.alias(&comment.author())),
        "body": comment.body(),
        "edits": comment.edits().map(|e| edit(e, aliases)).collect::<Vec<_>>(),
        "embeds": comment.embeds().to_vec(),
        "reactions": reactions(comment.reactions(), None, aliases),
        "timestamp": comment.timestamp().as_secs(),
        "replyTo": comment.reply_to(),
        "location": comment.location(),
        "resolved": comment.is_resolved(),
    })
}

/// Returns JSON for a `Review`.
fn review_comment(
    id: &CommentId,
    comment: &Comment<CodeLocation>,
    aliases: &impl AliasStore,
) -> Value {
    json!({
        "id": *id,
        "author": author(&Author::from(comment.author()), aliases.alias(&comment.author())),
        "body": comment.body(),
        "edits": comment.edits().map(|e| edit(e, aliases)).collect::<Vec<_>>(),
        "embeds": comment.embeds().to_vec(),
        "reactions": reactions(comment.reactions(), None, aliases),
        "timestamp": comment.timestamp().as_secs(),
        "replyTo": comment.reply_to(),
        "location": comment.location(),
        "resolved": comment.is_resolved(),
    })
}

/// Returns the name part of a path string.
fn name_in_path(path: &str) -> &str {
    match path.rsplit('/').next() {
        Some(name) => name,
        None => path,
    }
}

fn get_refs(
    repo: &git::Repository,
    id: &ActorId,
    head: &Oid,
) -> Result<Vec<RefString>, refs::Error> {
    let remote = repo.remote(id)?;
    let refs = remote
        .refs
        .iter()
        .filter_map(|(name, o)| {
            if o == head {
                Some(name.to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(refs)
}
