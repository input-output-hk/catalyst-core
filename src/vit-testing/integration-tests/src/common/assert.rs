use std::path::Path;

pub fn wait_until_folder_contains_all_qrs<P: AsRef<Path>>(qrs_count: usize, folder: P) {
    println!("waiting for qr code in: {:?}", folder.as_ref());

    loop {
        let qrs = std::fs::read_dir(folder.as_ref()).unwrap();
        let actual = qrs.into_iter().count();
        println!("waiting for qr code in: {}/{}", actual, qrs_count);
        if actual >= qrs_count {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
