use std::borrow::Cow;
use std::error::Error;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use gix::bstr::ByteSlice;
use gix::object::tree;
use gix::objs::tree::EntryKind;
use gix::revision::walk::Sorting;
use gix::{Commit, ObjectId, Tree};

use crate::project::*;

pub fn open<P: AsRef<Path>>(p: P) -> Result<GitStore, Box<dyn Error>> {
    let r = gix::discover(&p)?;
    GitStore::new(r)
}

pub fn create<P: AsRef<Path>>(p: P) -> Result<GitStore, Box<dyn Error>> {
    let r = gix::init_bare(p)?;
    GitStore::new(r)
}

pub struct GitStore {
    r: gix::Repository,
}

impl GitStore {
    fn new(r: gix::Repository) -> Result<Self, Box<dyn Error>> {
        validate(&r)?;
        Ok(Self { r })
    }

    fn clone_with_cache(&self) -> Self {
        let mut r = self.r.clone();
        r.object_cache_size(Some(1024 * 1024));
        Self { r }
    }

    pub fn project_ids(&self) -> Result<Vec<ProjectID>, Box<dyn Error + 'static>> {
        if self.r.head()?.is_unborn() {
            return Ok(Vec::new());
        }
        let mut res = Vec::new();
        for e in self.r.head_commit()?.tree()?.iter() {
            let e = e?;
            match (e.mode().is_tree(), program_git(e.filename())) {
                (false, _) => (), // skip
                (true, p) => {
                    let p = p?;
                    for proj in e.object()?.try_into_tree()?.iter() {
                        res.push(ProjectID {
                            program: p,
                            name: proj?.filename().to_string(),
                        });
                    }
                }
            };
        }
        Ok(res)
    }

    pub(crate) fn read_project(
        &self,
        id: &ProjectID,
    ) -> Result<Option<RawProject>, Box<dyn Error>> {
        match self.r.head_commit() {
            Err(_) => Ok(None),
            Ok(c) => match c.tree()?.lookup_entry_by_path(Self::path_for(id))? {
                None => Ok(None),
                Some(e) => Ok(Some(RawProject {
                    archive: self.tree_to_archive(e.object()?.try_into_tree()?)?,
                })),
            },
        }
    }

    fn tree_to_archive(&self, tree: Tree) -> Result<RawArchive, Box<dyn Error>> {
        let entries = tree
            .iter()
            .map(|e| self.tree_entry_to_archive_entry(e))
            .collect::<Result<Vec<ArchiveEntry>, Box<dyn Error>>>()?;
        Ok(RawArchive { entries })
    }

    fn tree_entry_to_archive_entry(
        &self,
        e: Result<tree::EntryRef<'_, '_>, gix::objs::decode::Error>,
    ) -> Result<ArchiveEntry, Box<dyn Error>> {
        let e = e?;
        let name = e.filename().to_string();
        let contents = match e.mode().is_tree() {
            true => {
                ArchiveEntryContents::Archive(self.tree_to_archive(e.object()?.try_into_tree()?)?)
            }
            false => ArchiveEntryContents::Data(e.object()?.try_into_blob()?.data.clone()),
        };
        Ok(ArchiveEntry { name, contents })
    }

    pub(crate) fn commit(
        &self,
        projects: &[(ProjectID, RawProject)],
        commit_message: &str,
    ) -> Result<&'static str, Box<dyn Error>> {
        let head = self.r.head()?;
        let head_ref = head.referent_name().ok_or("invalid head ref")?;

        // Get the current tree (or empty tree if unborn)
        let (current_tree, parent_commit_ids) = if head.is_unborn() {
            (self.r.empty_tree(), Vec::new())
        } else {
            (
                self.r.head_commit()?.tree()?,
                vec![self.r.head_commit()?.id],
            )
        };

        // Create a new tree with the project changes
        let mut new_root_tree = tree::Editor::new(&current_tree)?;
        for (id, data) in projects {
            let proj_tree = self.create_proj_tree(data)?;
            new_root_tree.upsert(Self::path_for(id), EntryKind::Tree, proj_tree)?;
        }
        let new_root_tree_id = new_root_tree.write()?;

        // Create the commit
        if current_tree.id != new_root_tree_id {
            self.r.commit(
                head_ref,
                commit_message,
                new_root_tree_id,
                parent_commit_ids,
            )?;
            Ok("added")
        } else {
            Ok("already up to date")
        }
    }

    pub fn log(&self, since: Duration) -> Result<super::LogResult, Box<dyn Error>> {
        use super::LogResult;
        if self.r.head()?.is_unborn() {
            return Ok(LogResult::Unborn);
        }

        let cutoff_time = SystemTime::now() - since;

        let head_commit = self.r.head_commit()?;
        let head_commit_info = self.commit_info(&head_commit)?;
        if head_commit_info.date < cutoff_time {
            return Ok(LogResult::None(head_commit_info));
        }

        let with_cache = self.clone_with_cache();

        let revwalk = with_cache
            .r
            .rev_walk(Some(with_cache.r.head_commit()?.id()))
            .sorting(Sorting::ByCommitTime(Default::default()));
        let commit_infos = revwalk.all()?;

        let mut res = Vec::new();
        for info in commit_infos {
            let info = info?;
            let commit = info.object()?;
            let commit_info = with_cache.commit_info(&commit)?;
            if commit_info.date < cutoff_time {
                break;
            }
            res.push(commit_info);
        }

        Ok(LogResult::Some(res))
    }

    fn commit_info(&self, commit: &Commit) -> Result<super::CommitInfo, Box<dyn Error>> {
        let author_time = commit.author()?.time()?;
        let date = UNIX_EPOCH + Duration::from_secs(author_time.seconds as u64);
        let hash = format!("{}", commit.id().shorten_or_id());
        let message = match commit.message() {
            Ok(m) => match str::from_utf8(m.title) {
                Ok(s) => Cow::from(s),
                Err(e) => Cow::from(format!("error parsing commit message: {e}")),
            },
            Err(e) => Cow::from(format!("error getting commit message: {e}")),
        }
        .to_string();
        Ok(super::CommitInfo {
            hash,
            date,
            message,
        })
    }

    fn path_for(id: &ProjectID) -> String {
        let program = id.program;
        let name = &id.name;
        format!("{program}/{name}")
    }

    fn create_proj_tree(&self, proj: &RawProject) -> Result<ObjectId, Box<dyn Error>> {
        let mut new_tree = tree::Editor::new(&self.r.empty_tree())?;
        self.append_archive(&mut new_tree, &proj.archive, "")?;
        Ok(new_tree.write()?.detach())
    }

    fn append_archive(
        &self,
        tree: &mut tree::Editor,
        arch: &RawArchive,
        prefix: &str,
    ) -> Result<(), Box<dyn Error>> {
        for e in &arch.entries {
            self.append_archive_entry(tree, e, prefix)?;
        }
        Ok(())
    }

    fn append_archive_entry(
        &self,
        tree: &mut tree::Editor<'_>,
        e: &ArchiveEntry,
        prefix: &str,
    ) -> Result<(), Box<dyn Error>> {
        let name = &e.name;
        use ArchiveEntryContents::*;
        match &e.contents {
            Data(data) => {
                let blob_id = self.r.write_blob(data)?;
                tree.upsert(format!("{prefix}{name}"), EntryKind::Blob, blob_id)?;
            }
            Archive(arch) => {
                let subdir = format!("{prefix}{name}/");
                self.append_archive(tree, arch, &subdir)?;
            }
        };
        Ok(())
    }

    pub(crate) fn untrack(
        &self,
        id: &ProjectID,
        commit_message: &str,
    ) -> Result<&'static str, Box<dyn Error>> {
        let head = self.r.head()?;
        let head_ref = head.referent_name().ok_or("invalid head ref")?;

        // Get the current tree (or empty tree if unborn)
        let (current_tree, parent_commit_ids) = if head.is_unborn() {
            (self.r.empty_tree(), Vec::new())
        } else {
            (
                self.r.head_commit()?.tree()?,
                vec![self.r.head_commit()?.id],
            )
        };

        // Create a new tree without the project
        let mut new_root_tree = tree::Editor::new(&current_tree)?;
        new_root_tree.remove(Self::path_for(id))?;
        let new_root_tree_id = new_root_tree.write()?;

        // Create the commit
        if current_tree.id != new_root_tree_id {
            self.r.commit(
                head_ref,
                commit_message,
                new_root_tree_id,
                parent_commit_ids,
            )?;
            Ok("removed")
        } else {
            Ok("not tracked")
        }
    }
}

fn validate(r: &gix::Repository) -> Result<(), Box<dyn Error>> {
    if r.head()?.is_unborn() {
        return Ok(());
    }
    validate_tree(r.head_commit()?.tree()?)
}

fn validate_tree(t: gix::Tree) -> Result<(), Box<dyn Error>> {
    for e in t.iter() {
        let e = e?;
        if !e.mode().is_tree() {
            continue;
        }
        match (e.mode().is_tree(), program_git(e.filename())) {
            (true, Ok(_)) => {}
            (_, _) => {
                return Err(format!(
                    "repository can not be used for mind-meld, it has extra entries like {e}"
                )
                .into());
            }
        };
    }
    Ok(())
}
