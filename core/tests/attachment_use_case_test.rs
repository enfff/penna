use penna_core::application::{
    AddAttachmentError, AddAttachmentUseCase, GetAttachmentUseCase, ListAttachmentsUseCase,
    RemoveAttachmentError, RemoveAttachmentUseCase, MAX_ATTACHMENT_BYTES,
};
use penna_core::domain::AttachmentMeta;
use penna_core::ports::{AttachmentStore, RepositoryError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type SharedFiles = Arc<Mutex<HashMap<(String, String), Vec<u8>>>>;

#[derive(Default, Clone)]
struct FakeStore {
    files: SharedFiles,
}

impl AttachmentStore for FakeStore {
    fn list_attachments(&self, id: &str) -> Result<Vec<AttachmentMeta>, RepositoryError> {
        Ok(self
            .files
            .lock()
            .unwrap()
            .iter()
            .filter(|((entry, _), _)| entry == id)
            .map(|((_, name), data)| AttachmentMeta {
                name: name.clone(),
                bytes: data.len() as u64,
            })
            .collect())
    }

    fn get_attachment(&self, id: &str, name: &str) -> Result<Option<Vec<u8>>, RepositoryError> {
        Ok(self.files.lock().unwrap().get(&(id.to_string(), name.to_string())).cloned())
    }

    fn add_attachment(
        &self,
        id: &str,
        name: &str,
        data: &[u8],
    ) -> Result<AttachmentMeta, RepositoryError> {
        self.files.lock().unwrap().insert(
            (id.to_string(), name.to_string()),
            data.to_vec(),
        );
        Ok(AttachmentMeta {
            name: name.to_string(),
            bytes: data.len() as u64,
        })
    }

    fn remove_attachment(
        &self,
        id: &str,
        name: &str,
    ) -> Result<Vec<AttachmentMeta>, RepositoryError> {
        let mut files = self.files.lock().unwrap();
        if files.remove(&(id.to_string(), name.to_string())).is_none() {
            return Err(RepositoryError::NotFound(name.to_string()));
        }
        Ok(files
            .iter()
            .filter(|((entry, _), _)| entry == id)
            .map(|((_, n), d)| AttachmentMeta {
                name: n.clone(),
                bytes: d.len() as u64,
            })
            .collect())
    }
}

#[test]
fn add_stores_bytes_and_reports_size() {
    let use_case = AddAttachmentUseCase::new(FakeStore::default());

    let meta = use_case
        .execute("202608241700", "photo.jpg", &[1, 2, 3, 4])
        .expect("add should succeed");

    assert_eq!(meta.name, "photo.jpg");
    assert_eq!(meta.bytes, 4);
}

#[test]
fn add_rejects_traversal_and_separator_names() {
    let use_case = AddAttachmentUseCase::new(FakeStore::default());

    for bad in ["../escape.jpg", "sub/dir.png", r"back\slash.gif", "..", "."] {
        let err = use_case
            .execute("202608241700", bad, b"x")
            .expect_err("must reject unsafe name");
        assert!(
            matches!(err, AddAttachmentError::InvalidName(_)),
            "name {} not rejected as invalid",
            bad
        );
    }
}

#[test]
fn add_rejects_oversized_payloads() {
    let use_case = AddAttachmentUseCase::new(FakeStore::default());

    let err = use_case
        .execute("202608241700", "huge.bin", &vec![0u8; MAX_ATTACHMENT_BYTES + 1])
        .expect_err("oversized payload must fail");

    assert!(matches!(err, AddAttachmentError::TooLarge { size, max }
        if size == MAX_ATTACHMENT_BYTES + 1 && max == MAX_ATTACHMENT_BYTES));
}

#[test]
fn remove_reports_missing_attachment_as_not_found() {
    let use_case = RemoveAttachmentUseCase::new(FakeStore::default());

    let err = use_case
        .execute("202608241700", "ghost.jpg")
        .expect_err("removing unknown attachment must fail");

    assert_eq!(
        err,
        RemoveAttachmentError::NotFound("ghost.jpg".to_string())
    );
}

#[test]
fn remove_returns_remaining_manifest() {
    let store = FakeStore::default();
    let add = AddAttachmentUseCase::new(store.clone());
    let remove = RemoveAttachmentUseCase::new(store);

    add.execute("202608241700", "a.jpg", b"one").unwrap();
    add.execute("202608241700", "b.jpg", b"two").unwrap();

    let remaining = remove.execute("202608241700", "a.jpg").unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].name, "b.jpg");
}

#[test]
fn list_and_get_round_trip_binary_content() {
    let store = FakeStore::default();
    let add = AddAttachmentUseCase::new(store.clone());
    let list = ListAttachmentsUseCase::new(store.clone());
    let get = GetAttachmentUseCase::new(store);

    let png = vec![0x89u8, b'P', b'N', b'G', 0xFF, 0x00];
    add.execute("202608241700", "shot.png", &png).unwrap();

    let listed = list.execute("202608241700").unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].bytes, png.len() as u64);

    let fetched = get.execute("202608241700", "shot.png").unwrap().unwrap();
    assert_eq!(fetched, png);
}
