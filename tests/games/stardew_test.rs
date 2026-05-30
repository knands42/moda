use mockito::Server;
use moda::error::ModManagerError;
use moda::games::Game;
use moda::games::StardewValley;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::tempdir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

const ENV_VAR: &str = "MODA_SMAPI_DOWNLOAD_URL";

/// Serializes tests that set the global MODA_SMAPI_DOWNLOAD_URL env var.
static SMAPI_LOCK: Mutex<()> = Mutex::new(());

fn create_smapi_zip() -> Vec<u8> {
    let dir = tempdir().unwrap();
    let path = dir.path().join("smapi.zip");
    let file = fs::File::create(&path).unwrap();
    let mut zip = ZipWriter::new(file);
    zip.start_file(
        "SMAPI 4.5.2 installer/internal/linux/StardewModdingAPI.dll",
        SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(b"fake smapi dll").unwrap();
    zip.start_file(
        "SMAPI 4.5.2 installer/internal/linux/smapi-internal/support.dll",
        SimpleFileOptions::default(),
    )
    .unwrap();
    zip.write_all(b"fake support dll").unwrap();
    zip.finish().unwrap();
    fs::read(&path).unwrap()
}

fn create_flat_zip() -> Vec<u8> {
    let dir = tempdir().unwrap();
    let path = dir.path().join("flat.zip");
    let file = fs::File::create(&path).unwrap();
    let mut zip = ZipWriter::new(file);
    zip.start_file("random.txt", SimpleFileOptions::default())
        .unwrap();
    zip.write_all(b"not smapi").unwrap();
    zip.finish().unwrap();
    fs::read(&path).unwrap()
}

fn mock_smapi_download(server: &mut Server, status: usize, body: Vec<u8>) -> String {
    let _m = server
        .mock("GET", "/smapi-download")
        .with_status(status)
        .with_body(body)
        .create();
    let url = format!("{}/smapi-download", server.url());
    std::env::set_var(ENV_VAR, &url);
    url
}

#[test]
fn test_new_creates_instance() {
    mock_smapi_download(&mut Server::new(), 200, create_smapi_zip());
    let sv = StardewValley::new(PathBuf::from("/games/stardew"));
    assert_eq!(sv.descriptor().name, "Stardew Valley");
}

#[test]
fn test_path_getters() {
    mock_smapi_download(&mut Server::new(), 200, create_smapi_zip());
    let sv = StardewValley::new(PathBuf::from("/games/stardew"));
    let game_path = sv.game_path();
    let mod_path = sv.game_mod_path();
    assert_eq!(game_path, PathBuf::from("/games/stardew"));
    assert_eq!(mod_path, PathBuf::from("/games/stardew/Mods"));
}

#[test]
fn test_registry_id() {
    let sv = StardewValley::new(PathBuf::from("/games/stardew"));
    assert_eq!(sv.descriptor().registry_id, "stardew_valley");
}

#[test]
fn test_pre_setup_smapi_already_installed() {
    let _lock = SMAPI_LOCK.lock().unwrap();
    mock_smapi_download(&mut Server::new(), 200, create_smapi_zip());
    let game_dir = tempdir().unwrap();
    fs::write(game_dir.path().join("SMAPI.ZipInstaller.dll"), b"").unwrap();

    let sv = StardewValley::new(game_dir.path().to_path_buf());
    let result = sv.pre_setup();

    assert!(result.is_ok());
}

#[test]
fn test_pre_setup_successful_install() {
    let _lock = SMAPI_LOCK.lock().unwrap();
    let mut server = Server::new();
    let zip_bytes = create_smapi_zip();
    mock_smapi_download(&mut server, 200, zip_bytes);

    let game_dir = tempdir().unwrap();
    let sv = StardewValley::new(game_dir.path().to_path_buf());
    let result = sv.pre_setup();

    assert!(result.is_ok());
    assert!(game_dir.path().join("StardewModdingAPI.dll").exists());
    assert!(game_dir.path().join("smapi-internal/support.dll").exists());
}

#[test]
fn test_pre_setup_download_failure() {
    let _lock = SMAPI_LOCK.lock().unwrap();
    let mut server = Server::new();
    mock_smapi_download(&mut server, 404, vec![]);

    let game_dir = tempdir().unwrap();
    let sv = StardewValley::new(game_dir.path().to_path_buf());
    let result = sv.pre_setup();

    assert!(result.is_err());
    assert!(matches!(result, Err(ModManagerError::GameSetupFailed(_))));
}

#[test]
fn test_pre_setup_invalid_zip_structure() {
    let _lock = SMAPI_LOCK.lock().unwrap();
    let mut server = Server::new();
    let zip_bytes = create_flat_zip();
    mock_smapi_download(&mut server, 200, zip_bytes);

    let game_dir = tempdir().unwrap();
    let sv = StardewValley::new(game_dir.path().to_path_buf());
    let result = sv.pre_setup();

    assert!(result.is_err());
    assert!(matches!(
        result,
        Err(ModManagerError::InvalidConfiguration(_))
    ));
}

#[test]
fn test_pre_setup_corrupt_zip() {
    let _lock = SMAPI_LOCK.lock().unwrap();
    let mut server = Server::new();
    mock_smapi_download(&mut server, 200, b"not a zip file".to_vec());

    let game_dir = tempdir().unwrap();
    let sv = StardewValley::new(game_dir.path().to_path_buf());
    let result = sv.pre_setup();

    assert!(result.is_err());
}
