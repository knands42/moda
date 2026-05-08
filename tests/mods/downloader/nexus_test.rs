use mockito::Server;
use moda::mods::downloader::NexusClient;

#[test]
fn test_nexus_client_new() {
    let mut server = Server::new();
    let mock_response = r#"{"data": {"id": "45720", "game_scoped_id": "45720", "game_id": "stardewvalley", "name": "Test Mod"}}"#;

    let _mock = server
        .mock("GET", "/games/stardewvalley/mods/45720")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(mock_response)
        .create();

    let url = server.url();
    std::env::set_var("NEXUS_API_BASE", url);

    let client = NexusClient::new("test_key".to_string());
    let mod_info = client.get_mod_info("stardewvalley", 45720);

    assert!(mod_info.is_ok());
    assert_eq!(mod_info.unwrap().name, Some("Test Mod".to_string()));
}

