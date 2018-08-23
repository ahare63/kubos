extern crate file_protocol;
extern crate file_service_rust;
extern crate kubos_system;
extern crate rand;
extern crate tempfile;
extern crate threadpool;

use kubos_system::Config as ServiceConfig;
use file_service_rust::recv_loop;
use std::thread;
use file_protocol::CborProtocol;
use file_protocol::FileProtocol;
use rand::{thread_rng, Rng};
use std::env;
use std::path::Path;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::time::Duration;
use tempfile::TempDir;
use threadpool::ThreadPool;

// NOTE: Each test's file contents must be unique. Otherwise the hash is the same, so
// the same storage directory is used across all of them, creating conflicts

macro_rules! service_new {
    ($port:expr) => {{
        thread::spawn(move || {
            recv_loop(ServiceConfig::new_from_str(
                "file-transfer-service",
                &format!(
                    r#"
                [file-transfer-service.addr]
                ip = "127.0.0.1"
                port = {}
                "#,
                    $port
                ),
            )).unwrap();
        });

        thread::sleep(Duration::new(0,500));
    }};
}

fn upload(port: u16, source_path: &str, target_path: &str) -> Result<String, String> {
    let f_protocol = FileProtocol::new_prefix("client".to_owned(), String::from("127.0.0.1"), port);

    println!(
        "Uploading local:{} to remote:{}",
        &source_path, &target_path
    );
    // Copy file to upload to temp storage. Calculate the hash and chunk info
    // Q: What's `mode` for? `local_import` always returns 0. Looks like it should be file permissions
    let (hash, num_chunks, mode) = f_protocol.local_import(&source_path)?;
    // Tell our destination the hash and number of chunks to expect
    f_protocol.send_sync(&hash, num_chunks)?;
    // Give it a moment to set everything up
    // Bug: Currently it's possible for the service to not be done setting up the temporary directory before
    // attempting to access it
    thread::sleep(Duration::new(1, 0));
    // Send the actual file
    f_protocol.send_export(&hash, &target_path, mode)?;
    Ok(hash.to_owned())
}

fn create_test_file(name: &str, contents: &[u8]) {
    let mut file = File::create(name).unwrap();
    file.write_all(contents).unwrap();
}

// upload single-chunk file from scratch
#[test]
fn upload_single() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7000;

    let contents = "test1".as_bytes();

    create_test_file(&source, &contents);

    service_new!(service_port);

    let result = upload(service_port, &source, &dest);

    if let Err(err) = result.clone() {
        println!("Error: {}", err);
    }

    assert!(result.is_ok());

    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// upload multi-chunk file from scratch
#[test]
fn upload_multi_clean() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7001;

    let contents = [1; 5000];

    create_test_file(&source, &contents);

    service_new!(service_port);

    let result = upload(service_port, &source, &dest);

    assert!(result.is_ok());

    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// upload multi-chunk file which we already have 1 chunk for
#[test]
fn upload_multi_resume() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7002;

    let contents = [2; 5000];

    create_test_file(&source, &contents);

    service_new!(service_port);

    // Go ahead and upload the whole file so we can manipulate the temporary directory
    let result = upload(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Remove a chunk so we can test the retry logic
    fs::remove_file(format!("fp/storage/{}/0", hash)).unwrap();

    // Upload the file again
    let result = upload(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// upload multi-chunk file which we already have all chunks for
#[test]
fn upload_multi_complete() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7005;

    let contents = [3; 5000];

    create_test_file(&source, &contents);

    service_new!(service_port);

    // Upload the file once (clean upload)
    let result = upload(service_port, &source, &dest);
    assert!(result.is_ok());

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Upload the file again
    let result = upload(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// upload. Create hash mismatch.
#[test]
fn upload_bad_hash() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7003;

    let contents = "test1".as_bytes();

    create_test_file(&source, &contents);

    service_new!(service_port);

    // Upload the file so we can mess with the temporary storage
    let result = upload(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Tweak the chunk contents so the future hash calculation will fail
    fs::write(format!("fp/storage/{}/0", hash), "bad data".as_bytes()).unwrap();

    // TODO: THIS SHOULD FAIL
    let result = upload(service_port, &source, &dest);
    // TODO: Verify exact error message
    assert!(result.is_ok());

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();
}

#[test]
fn upload_multi_client() {
    let service_port = 7004;

    // Spawn our single service
    service_new!(service_port);

    let pool = ThreadPool::new(5);

    // Spawn 5 simultaneous clients
    for num in 0..5 {
        pool.execute(move || {
            let test_dir = TempDir::new().expect("Failed to create test dir");
            let test_dir_str = test_dir.path().to_str().unwrap();
            let source = format!("{}/source", test_dir_str);
            let dest = format!("{}/dest", test_dir_str);
            //let contents = format!("test{}", num);
            let contents = [num; 5000];

            create_test_file(&source, &contents);

            let result = upload(service_port, &source, &dest);

            assert!(result.is_ok());

            let hash = result.unwrap();

            // TODO: Remove this sleep. We need it to let the service
            // finish its work. The upload logic needs to wait on
            // the final ACK message before returning
            thread::sleep(Duration::new(1, 0));

            // Cleanup the temporary files so that the test can be repeatable
            fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
            fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

            // Verify the final file's contents
            let dest_contents = fs::read(dest).unwrap();
            assert_eq!(&contents[..], dest_contents.as_slice());
        });
    }

    // Wait for all the threads to finish
    pool.join();
}

// Massive upload

#[test]
fn upload_large() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("temp/dest");
    let service_port = 7006;

    // Create a 10MB file filled with random data
    {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(source.clone())
            .unwrap();
        for _ in 0..100 {
            let mut contents = [0u8; 1_000_000];
            thread_rng().fill(&mut contents[..]);

            file.write(&contents).unwrap();
        }
    }

    service_new!(service_port);

    let result = upload(service_port, &source, &dest);

    assert!(result.is_ok());

    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The upload logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    //fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents

    let mut source_file = File::open(source).unwrap();
    let mut dest_file = File::open(dest).unwrap();
    for num in 0..2442 {
        let mut source_buf = [0u8; 4096];
        let mut dest_buf = [0u8; 4096];

        source_file.read(&mut source_buf).unwrap();
        dest_file.read(&mut dest_buf).unwrap();

        assert_eq!(&source_buf[..], &dest_buf[..], "Chunk mismatch: {}", num);
    }
}
