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
use std::fs::File;
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

        thread::sleep(Duration::new(1, 0));
    }};
}

fn download(port: u16, source_path: &str, target_path: &str) -> Result<String, String> {
    let f_protocol = FileProtocol::new_prefix("client".to_owned(), String::from("127.0.0.1"), port);

    println!(
        "Downloading remote:{} to local:{}",
        &source_path, &target_path
    );

    // Send our file request to the remote addr and get the returned data
    let (hash, num_chunks, mode) = f_protocol.send_import(source_path)?;

    // Check the number of chunks we need to receive and then receive them
    f_protocol.sync_and_send(&hash, Some(num_chunks))?;

    // Save received data to the requested path
    f_protocol.local_export(&hash, target_path, mode)?;

    Ok(hash.to_owned())
}

fn create_test_file(name: &str, contents: &[u8]) {
    let mut file = File::create(name).unwrap();
    file.write_all(contents).unwrap();
}

// upload single-chunk file from scratch
#[test]
fn download_single() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7000;

    let contents = "test1".as_bytes();

    create_test_file(&source, &contents);

    service_new!(service_port);

    let result = download(service_port, &source, &dest);

    if let Err(err) = result.clone() {
        println!("Error: {}", err);
    }

    assert!(result.is_ok());

    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// download multi-chunk file from scratch
#[test]
fn download_multi_clean() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7001;

    let contents = [1; 5000];

    create_test_file(&source, &contents);

    service_new!(service_port);

    let result = download(service_port, &source, &dest);

    assert!(result.is_ok());

    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// download multi-chunk file which we already have 1 chunk for
#[test]
fn download_multi_resume() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7002;

    let contents = [2; 5000];

    create_test_file(&source, &contents);

    service_new!(service_port);

    // Go ahead and download the whole file so we can manipulate the temporary directory
    let result = download(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Remove a chunk so we can test the retry logic
    fs::remove_file(format!("fp/storage/{}/0", hash)).unwrap();

    // download the file again
    let result = download(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// download multi-chunk file which we already have all chunks for
#[test]
fn download_multi_complete() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7005;

    let contents = [3; 5000];

    create_test_file(&source, &contents);

    service_new!(service_port);

    // download the file once (clean download)
    let result = download(service_port, &source, &dest);
    assert!(result.is_ok());

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // download the file again
    let result = download(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}

// download. Create hash mismatch.
#[test]
fn download_bad_hash() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7003;

    let contents = "test1".as_bytes();

    create_test_file(&source, &contents);

    service_new!(service_port);

    // download the file so we can mess with the temporary storage
    let result = download(service_port, &source, &dest);
    assert!(result.is_ok());
    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Tweak the chunk contents so the future hash calculation will fail
    fs::write(format!("client/storage/{}/0", hash), "bad data".as_bytes()).unwrap();

    // TODO: THIS SHOULD FAIL
    let result = download(service_port, &source, &dest);
    // TODO: Verify exact error message
    assert!(result.is_ok());

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();
}

#[test]
fn download_multi_client() {
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

            let result = download(service_port, &source, &dest);

            assert!(result.is_ok());

            let hash = result.unwrap();

            // TODO: Remove this sleep. We need it to let the service
            // finish its work. The download logic needs to wait on
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

// Massive download
/*
#[test]
fn download_large() {
    let test_dir = TempDir::new().expect("Failed to create test dir");
    let test_dir_str = test_dir.path().to_str().unwrap();
    let source = format!("{}/source", test_dir_str);
    let dest = format!("{}/dest", test_dir_str);
    let service_port = 7006;

    // Create a 1GB file filled with random data
    let mut contents = [0u8; 1_000_000];
    thread_rng().fill(&mut contents[..]);

    create_test_file(&source, &contents);

    service_new!(service_port);

    let result = download(service_port, &source, &dest);

    assert!(result.is_ok());

    let hash = result.unwrap();

    // TODO: Remove this sleep. We need it to let the service
    // finish its work. The download logic needs to wait on
    // the final ACK message before returning
    thread::sleep(Duration::new(1, 0));

    // Cleanup the temporary files so that the test can be repeatable
    fs::remove_dir_all(format!("client/storage/{}", hash)).unwrap();
    fs::remove_dir_all(format!("fp/storage/{}", hash)).unwrap();

    // Verify the final file's contents
    let dest_contents = fs::read(dest).unwrap();
    assert_eq!(&contents[..], dest_contents.as_slice());
}
*/
