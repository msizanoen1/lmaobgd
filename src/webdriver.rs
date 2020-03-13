use failure::Error;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

pub struct WebDriver {
    url: String,
    client: Client,
    session: WebDriverSession,
}

impl WebDriver {
    pub fn new<T: Into<String> + std::fmt::Display>(
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
        let url = format!("{}/session", url);
        let session: WdResponse<_> = client
            .post(&url)
            .json(&req)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(Self {
            url: url.into(),
            session: session.value,
            client,
        })
    }

    pub fn get_elements<T: Into<String>>(
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
            .send()?
            .error_for_status()?
            .json::<WdResponse<_>>()?
            .value)
    }

    pub fn get_element<T: Into<String>>(
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
            .send()?
            .error_for_status()?
            .json::<WdResponse<_>>()?
            .value)
    }

    pub fn get_elements_from_element<T: Into<String>>(
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
            .send()?
            .error_for_status()?
            .json::<WdResponse<_>>()?
            .value)
    }

    pub fn get_element_from_element<T: Into<String>>(
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
            .send()?
            .error_for_status()?
            .json::<WdResponse<_>>()?
            .value)
    }

    pub fn element_click(&self, element: &WebElement) -> Result<(), Error> {
        let url = format!(
            "{base}/session/{session}/element/{element}/click",
            base = self.url,
            session = self.session.session_id,
            element = element.element_id
        );
        self.client
            .post(&url)
            .json(&json!({}))
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn get_element_attr(&self, element: &WebElement, attr: &str) -> Result<String, Error> {
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
            .send()?
            .error_for_status()?
            .json::<WdResponse<_>>()?
            .value)
    }

    pub fn get_element_text(&self, element: &WebElement) -> Result<String, Error> {
        let url = format!(
            "{base}/session/{session}/element/{element}/text",
            base = self.url,
            session = self.session.session_id,
            element = element.element_id
        );
        Ok(self
            .client
            .get(&url)
            .send()?
            .error_for_status()?
            .json::<WdResponse<_>>()?
            .value)
    }

    pub fn element_send_keys<T: Into<String>>(
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
            .send()?
            .error_for_status()?;
        Ok(())
    }

    pub fn navigate<T: Into<String>>(&self, url: T) -> Result<(), Error> {
        let requrl = format!(
            "{base}/session/{session}/url",
            base = self.url,
            session = self.session.session_id
        );
        let req = NavigateRequest { url: url.into() };
        self.client
            .post(&requrl)
            .json(&req)
            .send()?
            .error_for_status()?;
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
pub struct WebElement {
    #[serde(rename = "element-6066-11e4-a52e-4f735466cecf")]
    pub element_id: String,
}

#[derive(Serialize, Deserialize)]
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
