use async_trait::async_trait;
use reqwest::Client;
use serde_json::Value;

use crate::IAnalyze;
#[derive(Default)]
pub struct Bilibili {
    http_client: Client,
}

impl Bilibili {
    async fn get_real_url(
        &self,
        room_id: &str,
        qn: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let room_url = "https://api.live.bilibili.com/xlive/web-room/v2/index/getRoomPlayInfo";
        let mut param = [
            ("room_id", room_id),
            ("protocol", "0,1"),
            ("format", "0,1,2"),
            ("codec", "0,1"),
            ("qn", qn.unwrap_or("1000")),
            ("platform", "web"),
            ("ptype", "8"),
        ];
        let resp = self
            .http_client
            .get(room_url)
            .query(&param)
            .send()
            .await?
            .json::<Value>()
            .await?;
        let mut stream_info = resp["data"]["playurl_info"]["playurl"]["stream"].clone();
        // dbg!(&stream_info);
        let qn_max = stream_info
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|f| {
                let q = &f["format"][0]["codec"][0]["accept_qn"];
                return q.as_array().unwrap();
            })
            .map(|i| i.as_i64().unwrap())
            .max();
        let qn_max = qn_max.unwrap();
        if qn_max != 1000 {
            let sdf = qn_max.to_string();
            param[4].1 = sdf.as_str();
            stream_info = self
                .http_client
                .get(room_url)
                .query(&param)
                .send()
                .await?
                .json::<Value>()
                .await?["data"]["playurl_info"]["playurl"]["stream"]
                .clone();
        }
        let stream_urls = stream_info
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| i["format"][0]["format_name"].as_str().unwrap() == "ts")
            .flat_map(|i| {
                let array = i["format"].as_array().unwrap();
                let base_url = &array.last().unwrap()["codec"][0]["base_url"];
                let url_info = &array.last().unwrap()["codec"][0]["url_info"];
                return url_info
                    .as_array()
                    .unwrap()
                    .iter()
                    .enumerate()
                    .map(|(index, info)| {
                        let host = info["host"].as_str().unwrap();
                        let extra = info["extra"].as_str().unwrap();
                        return (
                            format!("线路{}", index + 1),
                            format!("{}{}{}", host, base_url.as_str().unwrap(), extra),
                        );
                    });
            })
            .collect::<Vec<_>>();
        dbg!(stream_urls);
        Ok(())
    }
}
#[async_trait]
impl IAnalyze for Bilibili {
    async fn get_real_url(&self, room_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let resp = self
            .http_client
            .get("https://api.live.bilibili.com/room/v1/Room/room_init")
            .query(&[("id", room_id)])
            .send()
            .await?
            .json::<Value>()
            .await?;
        if resp["msg"].as_str().unwrap() == "直播间不存在" {
            panic!("直播间不存在");
        }
        if resp["data"]["live_status"].as_i64().unwrap() != 1 {
            panic!("未开播")
        }

        let real_room_id = resp["data"]["room_id"].as_i64().unwrap().to_string();
        self.get_real_url(&real_room_id, None).await?;
        Ok(())
    }
}
