use regex::Regex;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_json::Value;

use std::collections::HashMap;

use crate::errors::LoginError;
use crate::types::Error;

const BASE_URL: &str = "http://jiofi.local.html/";
const LOGIN_URL: &str = "http://jiofi.local.html/cgi-bin/en-jio/login_check.html";
const MAC_CONFIG_PAGE: &str = "http://jiofi.local.html/cgi-bin/en-jio/mWMAC.html";
const POST_MAC_URL: &str = "http://jiofi.local.html/cgi-bin/en-jio/mWMAC_Apply.html";
const CLIENT_LIST_PAGE: &str = "http://jiofi.local.html/cgi-bin/en-jio/mConnected_Devices.html";
const LOGIN_CHECK: &str = "http://jiofi.local.html/cgi-bin/en-jio/login_Query.html";

pub struct JioPageNavigator {
    is_logged_in: bool,
    pub session_id: String,
    request_token: String,
    username: String,
    password: String,
    client: Client,
}

#[derive(Debug)]
pub enum MacFilterOptionType {
    MacRules(HashMap<String, String>),
    MacRuleTable(Vec<HashMap<String, String>>),
}

#[derive(Debug, Deserialize)]
pub struct ConnectedDevice {
    pub Host_name: String,
    pub IP_address: String,
    pub Lease_Time: String,
    pub MAC: String,
    pub Status: String,
}

impl JioPageNavigator {
    pub fn new(username: String, password: String) -> Result<JioPageNavigator, Error> {
        let headers = JioPageNavigator::get_headers();
        let client = Client::builder()
            .default_headers(headers)
            .cookie_store(true)
            .build()?;

        Ok(JioPageNavigator {
            is_logged_in: false,
            session_id: String::from(""),
            request_token: String::from(""),
            username,
            password,
            client,
        })
    }

    fn get_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers
            .insert(
                "accept",
                HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.9"));
        headers.insert(
            "accept-language",
            HeaderValue::from_static("en-GB,en-US;q=0.9,en;q=0.8"),
        );
        headers.insert(
            "content-type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers
    }

    pub async fn login(&mut self) -> Result<(), Error> {
        if !self.is_logged_in {
            let response = self.client.get(BASE_URL).send().await?;
            let respose_body = response.text().await?;

            let parsed_html = Html::parse_document(&respose_body);
            let request_token_selector = Selector::parse("#RequestVerifyToken").unwrap();

            let matches: Vec<_> = parsed_html.select(&request_token_selector).collect();
            let input_node = matches[0]
                .value()
                .attr("value")
                .expect("RequestVerifyToken input node missing the value attribute");
            self.request_token = input_node.to_string();

            let mut login_data = HashMap::new();
            login_data.insert("RequestVerifyToken", &self.request_token);
            login_data.insert("act", &self.username);
            login_data.insert("pwd", &self.password);

            self.client.post(LOGIN_URL).form(&login_data).send().await?;
            let is_logged_in = self.check_login_status().await?;
            if !is_logged_in {
                return Err(LoginError.into());
            }
            self.is_logged_in = is_logged_in;
        }

        Ok(())
    }

    async fn check_login_status(&self) -> Result<bool, Error> {
        let response = self.client.post(LOGIN_CHECK).send().await?;
        let data = response.text().await?;
        let parsed_data: Value = serde_json::from_str(&data).expect("Invalid JSON data!");
        return Ok(parsed_data["result"] == true && parsed_data["login"] == "1");
    }

    pub async fn get_mac_config_page_data(&mut self) -> Result<Vec<MacFilterOptionType>, Error> {
        self.login().await?;
        let mut res = vec![];
        let mut page_data: HashMap<String, String> = HashMap::new();

        let response = self.client.get(MAC_CONFIG_PAGE).send().await?;
        let respose_body = response.text().await?;
        let parsed_html = Html::parse_document(&respose_body);
        let request_token_selector = Selector::parse("input[name=RequestVerifyToken]").unwrap();

        let mut mac_rules: Vec<HashMap<String, String>> = vec![];

        let js_script = parsed_html
            .select(&Selector::parse("script").unwrap())
            .collect::<Vec<_>>()
            .last()
            .unwrap()
            .inner_html();
        let rule_address_regex = Regex::new(r"var wlan_paed_acl_address_\d+\s*=\s*(.+);").unwrap();
        let rule_name_regex = Regex::new(r"var acl_address_value_\d+_desc\s*=\s*(.+);").unwrap();

        for (rule_address, rule_name) in rule_address_regex
            .captures_iter(&js_script)
            .zip(rule_name_regex.captures_iter(&js_script))
        {
            if &rule_name[1] != "\"\"" {
                let rule_map = HashMap::from([
                    (
                        "rule_name".to_string(),
                        rule_name[1].to_owned().replace("\"", ""),
                    ),
                    (
                        "rule_address".to_string(),
                        rule_address[1].to_owned().replace("\"", ""),
                    ),
                ]);
                mac_rules.push(rule_map);
            }
        }
        let matches: Vec<_> = parsed_html.select(&request_token_selector).collect();
        let request_token = matches[0]
            .value()
            .attr("value")
            .expect("RequestVerifyToken input node missing the value attribute");

        page_data.insert("RequestVerifyToken".to_string(), request_token.to_owned());

        let options = ["MACFILTER_MODE", "MACFILTER_ENABLE"];

        for option in options {
            let selector_string = format!("input[name={}]", option);
            let option_selector = Selector::parse(&selector_string).unwrap();

            let mut option_state = "0";
            for elem in parsed_html.select(&option_selector) {
                if let Some(_) = elem.value().attr("checked") {
                    option_state = elem.value().attr("value").unwrap();
                }
            }
            page_data.insert(option.to_string(), option_state.to_owned());
        }
        res.push(MacFilterOptionType::MacRules(page_data));
        res.push(MacFilterOptionType::MacRuleTable(mac_rules));

        Ok(res)
    }

    pub async fn post_mac_config_page_data(
        &mut self,
        data: Vec<MacFilterOptionType>,
    ) -> Result<(), Error> {
        // self.login().await?;
        let mut form_data: HashMap<String, String> = HashMap::new();
        for op in data {
            let option = match op {
                MacFilterOptionType::MacRules(rule) => rule,
                MacFilterOptionType::MacRuleTable(table) => {
                    let mut mf_table: HashMap<String, String> = HashMap::new();
                    let mut encoded_table = "".to_string();
                    for rule in table {
                        let rule_name = rule
                            .get("rule_name")
                            .expect("rule_name not found, Invalid data format!");
                        let rule_address = rule
                            .get("rule_address")
                            .expect("rule_address not found, Invalid data format!");
                        let mac_rule = format!("{},{};", rule_name, rule_address);
                        encoded_table += &mac_rule;
                    }
                    mf_table.insert("MF_RULES_TABLES".to_string(), encoded_table);
                    mf_table
                }
            };
            form_data.extend(option);
        }
        self.client
            .post(POST_MAC_URL)
            .form(&form_data)
            .send()
            .await?;
        Ok(())
    }

    pub async fn get_connected_devices(&mut self) -> Result<Vec<ConnectedDevice>, Error> {
        self.login().await?;
        let res = self.client.get(CLIENT_LIST_PAGE).send().await?;
        let page_data = res.text().await?;
        let page_parser = Html::parse_document(&page_data);
        let script_tag = page_parser
            .select(&Selector::parse("script").unwrap())
            .collect::<Vec<_>>()[0];
        let client_list_regex = Regex::new(r"var client_list\s*=\s*Array\((.+)\);").unwrap();
        let script_str = script_tag.inner_html();
        let cl = client_list_regex
            .captures(&script_str)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str();
        let formatted_list = format!("[{}]", cl);
        let client_list: Vec<ConnectedDevice> =
            serde_json::from_str(&formatted_list).expect("Invalid JSON data!");
        Ok(client_list)
    }

    pub async fn update_setting(
        &mut self,
        options: HashMap<&str, MacFilterOptionType>,
        _override: bool,
    ) -> Result<(), Error> {
        // options is a hash map that takes in key of either rule_table, mac_rules
        self.login().await?;

        let current_settings = self.get_mac_config_page_data().await?;
        let mut payload: Vec<MacFilterOptionType> = vec![];

        for setting in current_settings {
            match setting {
                MacFilterOptionType::MacRuleTable(mut ops) => {
                    let setting = if let Some(rule_t) = options.get("rule_table") {
                        if let MacFilterOptionType::MacRuleTable(new_rule_t) = rule_t {
                            if _override {
                                MacFilterOptionType::MacRuleTable(new_rule_t.clone())
                            } else {
                                ops.extend(new_rule_t.clone());
                                MacFilterOptionType::MacRuleTable(ops.clone())
                            }
                        } else {
                            panic!("Invalid type variant assigned to the the key 'rule_table' please make sure the variant is 'MacFilterOptionType::MacRuleTable'");
                        }
                    } else {
                        MacFilterOptionType::MacRuleTable(ops)
                    };
                    payload.push(setting);
                }
                MacFilterOptionType::MacRules(mut ops) => {
                    let setting = if let Some(rule) = options.get("mac_rules") {
                        if let MacFilterOptionType::MacRules(new_rule) = rule {
                            if _override {
                                MacFilterOptionType::MacRules(new_rule.clone())
                            } else {
                                ops.extend(new_rule.clone());
                                MacFilterOptionType::MacRules(ops.clone())
                            }
                        } else {
                            panic!("Invalid type variant assigned to the the key 'mac_rules' please make sure the variant is 'MacFilterOptionType::MacRules'");
                        }
                    } else {
                        MacFilterOptionType::MacRules(ops)
                    };
                    payload.push(setting);
                }
            }
        }
        self.post_mac_config_page_data(payload).await?;

        Ok(())
    }
}
