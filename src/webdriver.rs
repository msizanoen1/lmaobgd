use failure::Error;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tokio::process::{Child, Command};

pub struct WebDriver {
    url: String,
    client: Client,
    session: WebDriverSession,
    child: Option<Child>,
}

impl WebDriver {
    pub async fn new_firefox<T>(command: Option<T>, headless: bool) -> Result<Self, Error>
    where
        T: Into<String>,
    {
        let command = command
            .map(|x| x.into())
            .unwrap_or_else(|| String::from("geckodriver"));
        let sa = std::net::SocketAddr::from(([0, 0, 0, 0], 0));
        let port = TcpListener::bind(sa).await?.local_addr()?.port();
        let url = format!("http://127.0.0.1:{}", port);
        let child = Command::new(command)
            .arg("-p")
            .arg(&port.to_string())
            .kill_on_drop(true)
            .spawn()?;
        let mut caps = HashMap::new();
        if headless {
            caps.insert(
                String::from("moz:firefoxOptions"),
                json!({
                    "args": ["-headless"],
                }),
            );
        }
        let mut this = Self::new(&url, HashMap::new(), vec![caps]).await?;
        this.child = Some(child);
        Ok(this)
    }

    pub async fn new<T: Into<String> + std::fmt::Display>(
        url: T,
        always_match: HashMap<String, serde_json::Value>,
        first_match: Vec<HashMap<String, serde_json::Value>>,
    ) -> Result<Self, Error> {
        let req = NewSessionRequest {
            capabilities: Capabilities {
                always_match,
                first_match,
            },
        };
        let client = Client::new();
        let new_url = format!("{}/session", url);
        let session: WdResponse<_> = client
            .post(&new_url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(Self {
            url: url.into(),
            session: session.value,
            client,
            child: None,
        })
    }

    pub async fn get_elements<T: Into<String>>(
        &self,
        using: Using,
        value: T,
    ) -> Result<Vec<WebElement>, Error> {
        let session_id = &self.session.session_id;
        let url = format!(
            "{base}/session/{session}/elements",
            base = self.url,
            session = session_id
        );
        let req = ElementRequest {
            using,
            value: value.into(),
        };
        Ok(self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn get_element<T: Into<String>>(
        &self,
        using: Using,
        value: T,
    ) -> Result<WebElement, Error> {
        let session_id = &self.session.session_id;
        let url = format!(
            "{base}/session/{session}/element",
            base = self.url,
            session = session_id
        );
        let req = ElementRequest {
            using,
            value: value.into(),
        };
        Ok(self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn get_elements_from_element<T: Into<String>>(
        &self,
        element: &WebElement,
        using: Using,
        value: T,
    ) -> Result<Vec<WebElement>, Error> {
        let session_id = &self.session.session_id;
        let url = format!(
            "{base}/session/{session}/element/{element}/elements",
            base = self.url,
            session = session_id,
            element = element.element_id,
        );
        let req = ElementRequest {
            using,
            value: value.into(),
        };
        Ok(self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn get_element_from_element<T: Into<String>>(
        &self,
        element: &WebElement,
        using: Using,
        value: T,
    ) -> Result<WebElement, Error> {
        let session_id = &self.session.session_id;
        let url = format!(
            "{base}/session/{session}/element/{element}/element",
            base = self.url,
            session = session_id,
            element = element.element_id,
        );
        let req = ElementRequest {
            using,
            value: value.into(),
        };
        Ok(self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn element_click(&self, element: &WebElement) -> Result<(), Error> {
        let url = format!(
            "{base}/session/{session}/element/{element}/click",
            base = self.url,
            session = self.session.session_id,
            element = element.element_id
        );
        self.client
            .post(&url)
            .json(&json!({}))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn get_element_attr(
        &self,
        element: &WebElement,
        attr: &str,
    ) -> Result<String, Error> {
        let url = format!(
            "{base}/session/{session}/element/{element}/attribute/{name}",
            base = self.url,
            session = self.session.session_id,
            element = element.element_id,
            name = attr
        );
        Ok(self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn get_element_prop<T: serde::de::DeserializeOwned>(
        &self,
        element: &WebElement,
        prop: &str,
    ) -> Result<T, Error> {
        let url = format!(
            "{base}/session/{session}/element/{element}/property/{name}",
            base = self.url,
            session = self.session.session_id,
            element = element.element_id,
            name = prop
        );
        Ok(self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn get_element_text(&self, element: &WebElement) -> Result<String, Error> {
        let url = format!(
            "{base}/session/{session}/element/{element}/text",
            base = self.url,
            session = self.session.session_id,
            element = element.element_id
        );
        Ok(self
            .client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn element_send_keys<T: Into<String>>(
        &self,
        element: &WebElement,
        keys: T,
    ) -> Result<(), Error> {
        let url = format!(
            "{base}/session/{session}/element/{element}/value",
            base = self.url,
            session = self.session.session_id,
            element = element.element_id
        );
        let req = SendKeyRequest { text: keys.into() };
        self.client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn run_script_unit<T>(&self, script: T) -> Result<(), Error>
    where
        T: Into<String> + Serialize,
    {
        let url = format!(
            "{base}/session/{session}/execute/sync",
            base = self.url,
            session = self.session.session_id
        );
        let data = json!({
            "args": [],
            "script": script,
        });
        self.client
            .post(&url)
            .json(&data)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn run_script_elem<T, V>(&self, script: T, element: &WebElement) -> Result<V, Error>
    where
        T: Into<String>,
        V: serde::de::DeserializeOwned,
    {
        let url = format!(
            "{base}/session/{session}/execute/sync",
            base = self.url,
            session = self.session.session_id
        );
        let req = ScriptInvokeElem {
            args: [element.clone()],
            script: script.into(),
        };
        Ok(self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json::<WdResponse<_>>()
            .await?
            .value)
    }

    pub async fn navigate<T: Into<String>>(&self, url: T) -> Result<(), Error> {
        let requrl = format!(
            "{base}/session/{session}/url",
            base = self.url,
            session = self.session.session_id
        );
        let req = NavigateRequest { url: url.into() };
        self.client
            .post(&requrl)
            .json(&req)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn close(self) -> Result<(), Error> {
        let url = format!(
            "{base}/session/{session}",
            base = self.url,
            session = self.session.session_id
        );
        self.client.delete(&url).send().await?.error_for_status()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub enum Using {
    #[serde(rename = "css selector")]
    CssSelector,
    #[serde(rename = "link text")]
    LinkText,
    #[serde(rename = "partial link text")]
    PartialLinkText,
    #[serde(rename = "tag name")]
    TagName,
    #[serde(rename = "xpath")]
    XPath,
}

#[derive(Serialize, Deserialize)]
pub struct NavigateRequest {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct SendKeyRequest {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct ElementRequest {
    pub using: Using,
    pub value: String,
}

#[derive(Serialize, Deserialize)]
pub struct ScriptInvokeElem {
    pub script: String,
    pub args: [WebElement; 1],
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebElement {
    #[serde(rename = "element-6066-11e4-a52e-4f735466cecf")]
    pub element_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WebDriverSession {
    pub session_id: String,
    pub capabilities: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct WdResponse<T> {
    value: T,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub always_match: HashMap<String, serde_json::Value>,
    pub first_match: Vec<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSessionRequest {
    pub capabilities: Capabilities,
}
