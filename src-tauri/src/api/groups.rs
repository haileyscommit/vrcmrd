use tauri::Wry;
use vrchatapi::models::LimitedUserGroups;

#[tauri::command]
pub async fn get_all_groups(
    app: tauri::AppHandle<Wry>,
    user_id: String,
) -> Result<Vec<LimitedUserGroups>, String> {
    let response = try_request!(app.clone(), |config| {
        vrchatapi::apis::users_api::get_user_groups(config, &user_id)
    }, { wait_for_api_ready: true })
    .await;
    match response {
        Ok(Some(groups)) => {
            // TODO: apply group-related advisories
            Ok(groups)
        }
        Ok(None) => Ok(vec![]),
        Err(e) => {
            eprintln!("Failed to get user groups: {}", e);
            Err(e.to_string())
        }
    }
}
