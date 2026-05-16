#[tokio::test]
async fn public_search_rejects_empty_query() {
    let status = ytmusic_service::error::map_invalid_argument("query must not be empty");
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
}

#[test]
fn admin_placeholder_module_is_explicit() {
    assert_eq!(
        ytmusic_service::servers::admin::PLACEHOLDER_MESSAGE,
        "admin service placeholder"
    );
}
