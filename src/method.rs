use std::time::Duration;

use crate::{
    error,
    options::TaskOptions,
    response,
    utils::{value_into_vec, PushExt},
    Client, Error, InnerClient, TaskHooks,
};
use serde::Serialize;
use serde_json::{json, to_value, Map, Value};
use snafu::prelude::*;

type Result<T> = std::result::Result<T, Error>;

impl InnerClient {
    async fn custom_tell_multi(
        &self,
        method: &str,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        let mut params = value_into_vec(json!([offset, num]));
        params.push_some(keys)?;
        self.call_and_subscribe(method, params, None).await
    }

    pub async fn custom_tell_stopped(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        self.custom_tell_multi("tellStopped", offset, num, keys)
            .await
    }
}

/// The parameter `how` in `changePosition`.
///
/// <https://aria2.github.io/manual/en/html/aria2c.html#aria2.changePosition>
#[allow(non_camel_case_types)]
#[derive(Serialize, Debug, Clone, PartialEq)]
pub enum PositionHow {
    POS_SET,
    POS_CUR,
    POS_END,
}

impl Client {
    pub async fn get_version(&self) -> Result<response::Version> {
        self.call_and_subscribe("getVersion", vec![], None).await
    }

    pub async fn add_uri(
        &self,
        uris: Vec<String>,
        options: Option<TaskOptions>,
        position: Option<u32>,
        hooks: Option<TaskHooks>,
    ) -> Result<String> {
        let mut params = vec![to_value(uris).context(error::JsonSnafu)?];
        params.push_else(options, json!({}))?;
        params.push_some(position)?;

        let gid: String = self.call_and_subscribe("addUri", params, None).await?;
        self.set_hooks(&gid, hooks).await;
        Ok(gid)
    }

    pub async fn add_torrent(
        &self,
        torrent: impl AsRef<[u8]>,
        uris: Option<Vec<String>>,
        options: Option<TaskOptions>,
        position: Option<u32>,
        hooks: Option<TaskHooks>,
    ) -> Result<String> {
        let mut params = vec![Value::String(base64::encode(torrent))];
        params.push_else(uris, json!([]))?;
        params.push_else(options, json!({}))?;
        params.push_some(position)?;

        let gid: String = self.call_and_subscribe("addTorrent", params, None).await?;
        self.set_hooks(&gid, hooks).await;
        Ok(gid)
    }

    pub async fn add_metalink(
        &self,
        metalink: impl AsRef<[u8]>,
        options: Option<TaskOptions>,
        position: Option<u32>,
        hooks: Option<TaskHooks>,
    ) -> Result<String> {
        let mut params = vec![Value::String(base64::encode(metalink))];
        params.push_else(options, json!({}))?;
        params.push_some(position)?;

        let gid: String = self.call_and_subscribe("addMetalink", params, None).await?;
        self.set_hooks(&gid, hooks).await;
        Ok(gid)
    }

    async fn do_gid(&self, method: &str, gid: &str, timeout: Option<Duration>) -> Result<()> {
        self.call_and_subscribe::<String>(method, vec![Value::String(gid.to_string())], timeout)
            .await?;
        Ok(())
    }

    pub async fn remove(&self, gid: &str) -> Result<()> {
        self.do_gid("remove", gid, Some(self.0.extended_timeout))
            .await
    }

    pub async fn force_remove(&self, gid: &str) -> Result<()> {
        self.do_gid("forceRemove", gid, None).await
    }

    pub async fn pause(&self, gid: &str) -> Result<()> {
        self.do_gid("pause", gid, Some(self.0.extended_timeout))
            .await
    }

    pub async fn pause_all(&self) -> Result<()> {
        self.call_and_subscribe::<String>("pauseAll", vec![], Some(self.0.extended_timeout))
            .await?;
        Ok(())
    }

    pub async fn force_pause(&self, gid: &str) -> Result<()> {
        self.do_gid("forcePause", gid, None).await
    }

    pub async fn force_pause_all(&self) -> Result<()> {
        self.call_and_subscribe::<String>("forcePauseAll", vec![], None)
            .await?;
        Ok(())
    }

    pub async fn unpause(&self, gid: &str) -> Result<()> {
        self.do_gid("unpause", gid, None).await
    }

    pub async fn unpause_all(&self) -> Result<()> {
        self.call_and_subscribe::<String>("unpauseAll", vec![], None)
            .await?;
        Ok(())
    }

    pub async fn custom_tell_status(
        &self,
        gid: &str,
        keys: Option<Vec<String>>,
    ) -> Result<Map<String, Value>> {
        let mut params = vec![Value::String(gid.to_string())];
        params.push_some(keys)?;
        self.call_and_subscribe("tellStatus", params, None).await
    }

    pub async fn tell_status(&self, gid: &str) -> Result<response::Status> {
        self.call_and_subscribe("tellStatus", vec![Value::String(gid.to_string())], None)
            .await
    }

    pub async fn get_uris(&self, gid: &str) -> Result<Vec<response::Uri>> {
        self.call_and_subscribe("getUris", vec![Value::String(gid.to_string())], None)
            .await
    }

    pub async fn get_files(&self, gid: &str) -> Result<Vec<response::File>> {
        self.call_and_subscribe("getFiles", vec![Value::String(gid.to_string())], None)
            .await
    }

    pub async fn get_peers(&self, gid: &str) -> Result<Vec<response::Peer>> {
        self.call_and_subscribe("getPeers", vec![Value::String(gid.to_string())], None)
            .await
    }

    pub async fn get_servers(&self, gid: &str) -> Result<Vec<response::GetServersResult>> {
        self.call_and_subscribe("getServers", vec![Value::String(gid.to_string())], None)
            .await
    }

    pub async fn tell_active(&self) -> Result<Vec<response::Status>> {
        self.call_and_subscribe("tellActive", vec![], None).await
    }

    pub async fn tell_waiting(&self, offset: i32, num: i32) -> Result<Vec<response::Status>> {
        self.call_and_subscribe("tellWaiting", value_into_vec(json!([offset, num])), None)
            .await
    }

    pub async fn tell_stopped(&self, offset: i32, num: i32) -> Result<Vec<response::Status>> {
        self.call_and_subscribe("tellStopped", value_into_vec(json!([offset, num])), None)
            .await
    }

    pub async fn custom_tell_active(
        &self,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        let mut params = Vec::new();
        params.push_some(keys)?;
        self.call_and_subscribe("tellActive", params, None).await
    }

    pub async fn custom_tell_waiting(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        self.0
            .custom_tell_multi("tellWaiting", offset, num, keys)
            .await
    }

    pub async fn custom_tell_stopped(
        &self,
        offset: i32,
        num: i32,
        keys: Option<Vec<String>>,
    ) -> Result<Vec<Map<String, Value>>> {
        self.0.custom_tell_stopped(offset, num, keys).await
    }

    pub async fn change_position(&self, gid: &str, pos: i32, how: PositionHow) -> Result<i32> {
        let params = value_into_vec(json!([gid, pos, how]));
        self.call_and_subscribe("changePosition", params, None)
            .await
    }

    /// # Returns
    /// This method returns a list which contains two integers.
    ///
    /// The first integer is the number of URIs deleted.
    /// The second integer is the number of URIs added.
    pub async fn change_uri(
        &self,
        gid: &str,
        file_index: i32,
        del_uris: Vec<String>,
        add_uris: Vec<String>,
        position: Option<i32>,
    ) -> Result<(i32, i32)> {
        let mut params = value_into_vec(json!([gid, file_index, del_uris, add_uris]));
        params.push_some(position)?;
        self.call_and_subscribe("changeUri", params, None).await
    }

    pub async fn get_option(&self, gid: &str) -> Result<TaskOptions> {
        self.call_and_subscribe("getOption", vec![Value::String(gid.to_string())], None)
            .await
    }

    pub async fn change_option(&self, gid: &str, options: TaskOptions) -> Result<()> {
        self.call_and_subscribe(
            "changeOption",
            vec![
                Value::String(gid.to_string()),
                to_value(options).context(error::JsonSnafu)?,
            ],
            None,
        )
        .await
    }

    pub async fn get_global_option(&self) -> Result<TaskOptions> {
        self.call_and_subscribe("getGlobalOption", vec![], None)
            .await
    }

    pub async fn change_global_option(&self, options: TaskOptions) -> Result<()> {
        self.call_and_subscribe(
            "changeGlobalOption",
            vec![to_value(options).context(error::JsonSnafu)?],
            None,
        )
        .await
    }

    pub async fn get_global_stat(&self) -> Result<response::GlobalStat> {
        self.call_and_subscribe("getGlobalStat", vec![], None).await
    }

    pub async fn purge_download_result(&self) -> Result<()> {
        self.call_and_subscribe::<String>("purgeDownloadResult", vec![], None)
            .await?;
        Ok(())
    }

    pub async fn remove_download_result(&self, gid: &str) -> Result<()> {
        self.call_and_subscribe::<String>(
            "removeDownloadResult",
            vec![Value::String(gid.to_string())],
            None,
        )
        .await?;
        Ok(())
    }

    pub async fn get_session_info(&self) -> Result<response::SessionInfo> {
        self.call_and_subscribe("getSessionInfo", vec![], None)
            .await
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.call_and_subscribe::<String>("shutdown", vec![], Some(self.0.extended_timeout))
            .await?;
        Ok(())
    }

    pub async fn force_shutdown(&self) -> Result<()> {
        self.call_and_subscribe::<String>("forceShutdown", vec![], None)
            .await?;
        Ok(())
    }

    pub async fn save_session(&self) -> Result<()> {
        self.call_and_subscribe::<String>("saveSession", vec![], None)
            .await?;
        Ok(())
    }
}
