use std::error::Error;
use std::process::exit;

use log::error;
use reqwest::blocking;
use serde_json::json;

use crate::clients::user_data::entity::B50Response;
use crate::config::consts::PROFILE;

/// 从远程服务器拿指定用户的 b50 数据
pub fn get_b50_data(username: &str) -> Result<B50Response, Box<dyn Error>> {
    let config = &PROFILE.remote_api.maimaidxprober;
    let payload = json!(
        {
            "username":username,
            "b50":true
        }
    );
    let request = blocking::Client::new()
        .post(&config.data_url)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(payload.to_string());
    let response = request.send()?;
    let status = response.status();
    let response = match status.as_u16() {
        200 => {
            let resp_text: B50Response = response.json().unwrap();
            resp_text
        }
        400 => {
            error!("未找到此玩家，请确保此玩家的用户名和查分器中的用户名相同");
            exit(exitcode::NOUSER);
        }
        403 => {
            error!("该用户禁止了其他人获取数据");
            exit(exitcode::UNAVAILABLE);
        }
        _ => {
            error!("[{}] <-- http 请求错误", status);
            exit(exitcode::NOHOST);
        }
    };
    Ok(response)
}

pub mod entity {
    use std::cmp::Ordering;

    use serde::{Deserialize, Serialize};

    /// 查分器返回的数据
    #[derive(Debug, Serialize, Deserialize)]
    pub struct B50Response {
        /// 查分器用户名
        pub username: String,
        /// 谱面列表
        pub charts: Charts,
        /// 用户名( Maimai 机台上显示的)
        pub nickname: String,
        /// 底分
        pub rating: i32,
        /// 不知道干啥的,先放着
        pub additional_rating: i32,
        /// 又一个不知道干啥的,先放着
        pub plate: String,
        /// 又又一个不知道干啥的,先放着
        pub user_general_data: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Charts {
        dx: Vec<ChartInfoResponse>,
        sd: Vec<ChartInfoResponse>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ChartInfoResponse {
        /// 达成率
        pub achievements: f32,
        /// 谱面定数
        pub ds: f32,
        /// DX 分数
        #[serde(rename = "dxScore")]
        pub dx_score: i32,
        /// FULL COMBO
        pub fc: String,
        /// FULL SYNC
        pub fs: String,
        /// 等级
        pub level: String,
        /// 标记是第几个难度的谱面(感觉跟下面的重复了)
        ///
        /// - `0`: Basic
        /// - `1`: Advanced
        /// - `2`: Expert
        /// - `3`: Master
        /// - `4`: Re:Master
        pub level_index: i32,
        /// 难度标签
        pub level_label: LevelLabel,
        /// 难度分
        pub ra: i32,
        /// 等级
        pub rate: ChartRate,
        /// 这里的 ID 跟 db 内的 ID 相关联的
        pub song_id: i32,
        /// 歌曲标题
        pub title: String,
        /// 歌曲类型
        #[serde(rename = "type")]
        pub song_type: String,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum LevelLabel {
        Basic,
        Advanced,
        Expert,
        Master,
        #[serde(rename = "Re:MASTER")]
        ReMaster,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    #[serde(rename_all = "lowercase")]
    #[derive(PartialEq, PartialOrd)]
    pub enum ChartRate {
        D,
        C,
        B,
        BB,
        BBB,
        A,
        AA,
        AAA,
        S,
        SP,
        SS,
        SSP,
        SSS,
        SSSP,
    }

    impl PartialEq for ChartInfoResponse {
        fn eq(&self, other: &Self) -> bool {
            self.ra == other.ra
        }
    }

    impl PartialOrd for ChartInfoResponse {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Eq for ChartInfoResponse {}

    impl Ord for ChartInfoResponse {
        fn cmp(&self, other: &Self) -> Ordering {
            self.ra.cmp(&other.ra)
        }
    }
}
