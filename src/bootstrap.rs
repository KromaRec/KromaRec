use axum::{
    http::{Method, StatusCode, Uri},
    response::Response,
};

pub fn supported_version(value: &str) -> bool {
    value.len() <= 32 && value.split('.').next() == Some("20230414")
}

pub fn serve(app: &crate::App, service: &str, method: &Method, uri: &Uri) -> Response {
    let path = uri.path();
    let body = match (service, method, path) {
        ("accounts", &Method::GET, "/emojiConfig/whitelistedEmojis") => {
            include_str!("../compat/whitelisted-emojis.json")
        }
        ("api", &Method::GET, "/api/config/v1/amplitude") => {
            r#"{"AmplitudeKey":"","UseRudderStack":false,"RudderStackKey":"","UseStatSig":false,"StatSigKey":"","StatSigEnvironment":0}"#
        }
        ("api", &Method::GET, "/api/config/v1/azurespeech") => {
            r#"{"Key":"dce8de5b297747d9b5bddcc7f19e8c5b","Region":"eastus","Enabled":false}"#
        }
        ("api", &Method::GET, "/api/config/v1/backtrace") => {
            r#"{"ReportBudget":0,"FilterType":0,"SampleRate":0,"LogLineCount":0,"CaptureNativeCrashes":0,"ANRThresholdMs":0,"MessageCount":0,"MessageRegex":"","VersionRegex":""}"#
        }
        ("api", &Method::GET, "/api/config/v2") => {
            return crate::json_body(StatusCode::OK, app.config.clone());
        }
        ("api", &Method::GET, "/api/gameconfigs/v1/all") => {
            include_str!("../compat/gameconfigs-v1-all.json")
        }
        ("api", &Method::GET, "/api/versioncheck/v4") => {
            let supported = form_urlencoded::parse(uri.query().unwrap_or("").as_bytes())
                .find(|(name, _)| name == "v")
                .is_some_and(|(_, value)| supported_version(&value));
            let body = serde_json::json!({
                "VersionStatus": u8::from(!supported),
                "UpdateNotificationStage": 0,
                "IsVersionIslanded": false,
                "IsCrossPlayDisabled": false
            });
            return crate::json_body(StatusCode::OK, body.to_string());
        }
        ("api", &Method::GET, "/api/versioncheck/islandedversions") => "[]",
        ("api", &Method::POST, "/statsigUserProperties") => r#"{"success":false}"#,
        ("api", &Method::GET, "/voice/config") => "{}",
        _ => return crate::not_found(),
    };
    crate::json_body(StatusCode::OK, body)
}

/// Trace only fixed route labels. Never include an arbitrary URL, ID or query.
pub fn trace_label(service: &str, path: &str) -> (&'static str, &'static str) {
    match (service, path) {
        ("discovery", "/") => ("discovery", "health"),
        ("" | "ns", "/") => ("ns", "discovery"),
        ("api", "/api/versioncheck/v4" | "/api/versioncheck/islandedversions") => {
            ("api", "versioncheck")
        }
        ("api", "/api/config/v2") => ("api", "config"),
        ("api", "/api/gameconfigs/v1/all") => ("api", "gameconfigs"),
        ("api", "/api/PlayerReporting/v1/deviceId" | "/PlayerReporting/v1/deviceId") => {
            ("api", "device-id")
        }
        ("auth", "/connect/token") => ("auth", "token"),
        ("auth", path) if path.starts_with("/cachedlogin/forplatformid/") => {
            ("auth", "platform-picker")
        }
        ("accounts", "/account/me") => ("accounts", "profile"),
        ("playersettings", "/playersettings") => ("playersettings", "settings"),
        ("match", "/player/connection-info") => ("match", "connection-info"),
        ("match", "/player/login") => ("match", "login"),
        ("match", "/player/heartbeat") => ("match", "heartbeat"),
        ("match", "/player/logout") => ("match", "logout"),
        ("match", "/player/exclusivelogin") => ("match", "exclusive-login"),
        ("match", "/player/verifylock") => ("match", "verify-lock"),
        ("match", "/invite") => ("match", "invite"),
        ("match", "/matchmake/none") => ("match", "startup-match"),
        ("match", "/matchmake/dorm") => ("match", "dorm-match"),
        ("match", path) if path.starts_with("/matchmake/") => ("match", "matchmake"),
        ("rooms", _) => ("rooms", "request"),
        ("cdn", _) => ("cdn", "asset"),
        ("notify", "/hub/v1/negotiate") => ("notify", "negotiate"),
        ("notify", "/hub/v1") => ("notify", "upgrade"),
        _ => ("other", "request"),
    }
}
