use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use super::{
    announcement_create, announcement_data_source, announcement_delete, announcement_delete_many,
    announcement_get, announcement_list, announcement_update, api_token_clear, api_token_issue,
    api_token_list, auto_code_registry_append, auto_code_registry_clear, auto_code_registry_list,
    email_list, email_send, email_test, global_constraint_get, global_constraint_save,
    mcp_service_start, mcp_service_status, mcp_service_stop, mcp_tool_list, mcp_tool_test,
    mcp_tool_upsert, package_list, package_upsert, plugin_install_create, plugin_install_list,
    plugin_manifest_list, plugin_manifest_upsert, release_append, release_list, skill_delete,
    skill_detail, skill_list, skill_reference_create, skill_reference_get, skill_reference_save,
    skill_resource_create, skill_resource_get, skill_resource_save, skill_save,
    skill_script_create, skill_script_get, skill_script_save, skill_template_create,
    skill_template_get, skill_template_save, skill_tools,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tool/api-token/list", get(api_token_list))
        .route("/tool/api-token/issue", post(api_token_issue))
        .route("/tool/api-token/clear", post(api_token_clear))
        .route("/tool/package/list", get(package_list))
        .route("/tool/package/save", post(package_upsert))
        .route("/tool/plugin-manifest/list", get(plugin_manifest_list))
        .route("/tool/plugin-manifest/save", post(plugin_manifest_upsert))
        .route("/tool/plugin-install/list", get(plugin_install_list))
        .route("/tool/plugin-install/save", post(plugin_install_create))
        .route("/tool/auto-code/list", get(auto_code_registry_list))
        .route("/tool/auto-code/save", post(auto_code_registry_append))
        .route("/tool/auto-code/clear", post(auto_code_registry_clear))
        .route("/tool/release/list", get(release_list))
        .route("/tool/release/save", post(release_append))
        .route("/email/getEmailList", get(email_list))
        .route("/email/emailTest", post(email_test))
        .route("/email/sendEmail", post(email_send))
        .route("/info/getInfoDataSource", get(announcement_data_source))
        .route("/info/getInfoList", get(announcement_list))
        .route("/info/findInfo", get(announcement_get))
        .route("/info/createInfo", post(announcement_create))
        .route(
            "/info/updateInfo",
            post(announcement_update).put(announcement_update),
        )
        .route(
            "/info/deleteInfo",
            axum::routing::delete(announcement_delete),
        )
        .route(
            "/info/deleteInfoByIds",
            axum::routing::delete(announcement_delete_many),
        )
        .route("/autoCode/mcp", post(mcp_tool_upsert))
        .route("/autoCode/mcpStatus", post(mcp_service_status))
        .route("/autoCode/mcpStart", post(mcp_service_start))
        .route("/autoCode/mcpStop", post(mcp_service_stop))
        .route("/autoCode/mcpList", post(mcp_tool_list))
        .route("/autoCode/mcpTest", post(mcp_tool_test))
        .route("/skills/getTools", get(skill_tools))
        .route("/skills/getSkillList", post(skill_list))
        .route("/skills/getSkillDetail", post(skill_detail))
        .route("/skills/saveSkill", post(skill_save))
        .route("/skills/deleteSkill", post(skill_delete))
        .route("/skills/createScript", post(skill_script_create))
        .route("/skills/getScript", post(skill_script_get))
        .route("/skills/saveScript", post(skill_script_save))
        .route("/skills/createResource", post(skill_resource_create))
        .route("/skills/getResource", post(skill_resource_get))
        .route("/skills/saveResource", post(skill_resource_save))
        .route("/skills/createReference", post(skill_reference_create))
        .route("/skills/getReference", post(skill_reference_get))
        .route("/skills/saveReference", post(skill_reference_save))
        .route("/skills/createTemplate", post(skill_template_create))
        .route("/skills/getTemplate", post(skill_template_get))
        .route("/skills/saveTemplate", post(skill_template_save))
        .route("/skills/getGlobalConstraint", post(global_constraint_get))
        .route("/skills/saveGlobalConstraint", post(global_constraint_save))
}
