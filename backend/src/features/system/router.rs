use axum::Router;
use axum::routing::{get, post};

use crate::state::AppState;

use super::{
    api_list, api_upsert, dictionary_detail_tree, dictionary_detail_upsert, dictionary_list,
    dictionary_upsert, error_logs, export_template_list, llm_config_delete, llm_config_list,
    llm_config_save, login_logs, operation_logs, params_list, params_upsert, system_config,
    system_runtime, update_system_config,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/getApiList", post(api_list))
        .route("/api/saveApi", post(api_upsert))
        .route("/sysDictionary/getSysDictionaryList", get(dictionary_list))
        .route("/sysDictionary/saveSysDictionary", post(dictionary_upsert))
        .route(
            "/sysDictionaryDetail/getDictionaryTreeList",
            get(dictionary_detail_tree),
        )
        .route(
            "/sysDictionaryDetail/saveSysDictionaryDetail",
            post(dictionary_detail_upsert),
        )
        .route("/sysParams/getSysParamsList", get(params_list))
        .route("/sysParams/saveSysParams", post(params_upsert))
        .route(
            "/sysOperationRecord/getSysOperationRecordList",
            get(operation_logs),
        )
        .route("/sysLoginLog/getLoginLogList", get(login_logs))
        .route("/sysError/getSysErrorList", get(error_logs))
        .route(
            "/sysExportTemplate/getSysExportTemplateList",
            get(export_template_list),
        )
        .route("/system/getServerInfo", post(system_runtime))
        .route("/system/getSystemConfig", post(system_config))
        .route("/system/setSystemConfig", post(update_system_config))
        .route("/tool/llm-config/list", get(llm_config_list))
        .route("/tool/llm-config/save", post(llm_config_save))
        .route("/tool/llm-config/delete", post(llm_config_delete))
}
