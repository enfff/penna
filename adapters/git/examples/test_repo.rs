use penna_adapters_git::GitEntryRepository;
use penna_core::domain::{Entry, EntryId};
use penna_core::ports::EntryRepository;
use std::path::PathBuf;

fn main() {
    let journal_path = PathBuf::from("/home/enf/Projects/penna-myjournal");
    
    println!("Opening git repository at: {journal_path:?}");
    
    match GitEntryRepository::new(journal_path.clone()) {
        Ok(repo) => {
            println!("✓ Successfully opened git repository");
            
            // Test list
            println!("\n--- Listing entries ---");
            match repo.list() {
                Ok(entries) => {
                    println!("Found {} entries:", entries.len());
                    for entry in &entries {
                        println!("  - {} (ID: {})", entry.title, entry.id.0);
                    }
                }
                Err(e) => println!("✗ Failed to list entries: {e:?}"),
            }
            
            // Test create
            println!("\n--- Creating new entry ---");
            let new_entry = Entry {
                id: EntryId("test-entry-1".to_string()),
                title: "Test Entry".to_string(),
                body: "This is a test entry body.\n\nWith multiple paragraphs.".to_string(),
                tags: vec!["test".to_string(), "demo".to_string()],
                created_at: "1234567890".to_string(),
                updated_at: "1234567890".to_string(),
            };
            
            match repo.save(&new_entry) {
                Ok(_) => println!("✓ Successfully created entry"),
                Err(e) => println!("✗ Failed to save entry: {e:?}"),
            }
            
            // Test get
            println!("\n--- Getting entry ---");
            match repo.get("test-entry-1") {
                Ok(Some(entry)) => {
                    println!("✓ Found entry:");
                    println!("  Title: {}", entry.title);
                    println!("  Body: {}", entry.body);
                    println!("  Tags: {:?}", entry.tags);
                }
                Ok(None) => println!("✗ Entry not found"),
                Err(e) => println!("✗ Failed to get entry: {e:?}"),
            }
            
            // Test update
            println!("\n--- Updating entry ---");
            let mut updated_entry = new_entry.clone();
            updated_entry.body = "Updated body content".to_string();
            updated_entry.updated_at = "9999999999".to_string();
            
            match repo.save(&updated_entry) {
                Ok(_) => println!("✓ Successfully updated entry"),
                Err(e) => println!("✗ Failed to update entry: {e:?}"),
            }
            
            // Test delete
            println!("\n--- Deleting entry ---");
            match repo.delete("test-entry-1") {
                Ok(_) => println!("✓ Successfully deleted entry"),
                Err(e) => println!("✗ Failed to delete entry: {e:?}"),
            }
            
            // Verify deletion
            println!("\n--- Verifying deletion ---");
            match repo.get("test-entry-1") {
                Ok(None) => println!("✓ Entry successfully deleted (not found)"),
                Ok(Some(_)) => println!("✗ Entry still exists"),
                Err(e) => println!("✗ Error checking: {e:?}"),
            }
        }
        Err(e) => {
            println!("✗ Failed to open repository: {e:?}");
            std::process::exit(1);
        }
    }
}
